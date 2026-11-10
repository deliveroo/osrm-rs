#include <condition_variable>
#include <cstdlib>
#include <memory>
#include <mutex>
#include <thread>

#include <tbb/task_scheduler_init.h>

#include "osrmc_tbb.h"

namespace {

int requested_threads() {
  /* Same rule as many_to_many_ch.cpp in deliveroo/osrm-backend v5.26.0-parallel. */
  const char* env = std::getenv("OSRM_NUM_THREADS");
  const int threads = env != nullptr ? std::atoi(env) : tbb::task_scheduler_init::default_num_threads();
  return threads > 0 ? threads : tbb::task_scheduler_init::default_num_threads();
}

}  // namespace

extern "C" void osrmc_tbb_keepalive(void) {
  static std::once_flag once;
  std::call_once(once, [] {
    /*
     * The detached thread outlives this lambda, so the handshake state is shared and owned by both sides:
     * if the waiter returns as soon as it sees `ready`, the worker's trailing notify_one() must not touch
     * a stack-allocated condition variable that has already been destroyed.
     */
    struct Handshake {
      std::mutex mutex;
      std::condition_variable cv;
      bool ready = false;
    };
    const auto handshake = std::make_shared<Handshake>();

    /*
     * A dedicated thread owns the scheduler: a task_scheduler_init lives and dies with the thread that
     * created it, and callers of osrmc_osrm_construct (map loaders) may exit. This thread parks forever,
     * so it never races its own pthread_detach and the scheduler is never destroyed.
     */
    std::thread([handshake] {
      tbb::task_scheduler_init scheduler{requested_threads()};
      {
        std::lock_guard<std::mutex> lock(handshake->mutex);
        handshake->ready = true;
      }
      handshake->cv.notify_one();

      std::mutex park_mutex;
      std::condition_variable park_cv;
      std::unique_lock<std::mutex> park_lock(park_mutex);
      park_cv.wait(park_lock, [] { return false; });
    }).detach();

    /* Block until the scheduler exists so the first table call already sees a live pool. */
    std::unique_lock<std::mutex> lock(handshake->mutex);
    handshake->cv.wait(lock, [&handshake] { return handshake->ready; });
  });
}
