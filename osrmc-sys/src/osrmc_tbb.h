#ifndef OSRMC_TBB_H
#define OSRMC_TBB_H

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Keep a TBB scheduler (and therefore TBB's global worker pool) alive for the life of the process.
 *
 * Deliveroo's OSRM fork (v5.26.0-parallel) constructs a tbb::task_scheduler_init on every many-to-many
 * (table) call. Whenever no other call is in flight the last scheduler is destroyed, TBB tears the worker
 * pool down and calls pthread_detach on each worker. On glibc < 2.43 that races with the exiting worker
 * (glibc bug 19951, use-after-free in pthread_detach) and can SIGSEGV the process (CSIG-3488, CSIG-3725).
 * Holding one scheduler on a thread that never exits keeps the pool alive, so the per-call schedulers only
 * ever bump a reference count.
 *
 * Idempotent and thread-safe: the first call starts the keepalive thread and returns once its scheduler is
 * live, later calls return immediately. Uses OSRM_NUM_THREADS exactly like the fork, falling back to
 * tbb::task_scheduler_init::default_num_threads().
 */
void osrmc_tbb_keepalive(void);

#ifdef __cplusplus
}
#endif

#endif /* OSRMC_TBB_H */
