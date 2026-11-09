//! CSIG-3488: OSRM's many-to-many search creates a `tbb::task_scheduler_init` on every call. Without a
//! process-lifetime scheduler, TBB tears its worker pool down between calls and detaches every worker,
//! which can SIGSEGV on glibc < 2.43 (glibc bug 19951). `Osrm::new` therefore keeps a scheduler alive;
//! this test checks that the TBB worker pool survives between table calls. Without the keepalive no TBB
//! worker threads exist while the process is idle, because each call tears the pool down on exit.
//!
//! Needs the OSRM test data, like the unit tests (`./prepare-test-data.sh`). Override with
//! `OSRM_TEST_FILE` and `OSRM_TEST_COORDS="lat,lon;lat,lon;lat,lon"` (three points on that map).
//!
//! This is the only test in this binary so no other test threads come and go while it counts threads.
#![cfg(target_os = "linux")]

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use osrm::{Coordinate, Osrm};

const DEFAULT_FILE: &str = "./test-data/gcc-states-latest.osrm";
// Abu Dhabi, inside the default test map.
const DEFAULT_COORDS: &str = "24.447618,54.371037;24.45487,54.391076;24.454979,54.37652";

fn coordinates() -> Vec<Coordinate> {
    let raw = env::var("OSRM_TEST_COORDS").unwrap_or_else(|_| DEFAULT_COORDS.to_string());
    raw.split(';')
        .map(|pair| {
            let mut parts = pair.split(',').map(|v| v.trim().parse::<f32>().unwrap());
            Coordinate {
                latitude: parts.next().unwrap(),
                longitude: parts.next().unwrap(),
            }
        })
        .collect()
}

/// `n` points scattered (deterministically) around `center`, so the search has enough targets for TBB to
/// actually fan work out to its workers.
fn scatter(center: &Coordinate, n: usize) -> Vec<Coordinate> {
    let mut state: u32 = 0x9e37_79b9;
    let mut next = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5
    };
    (0..n)
        .map(|_| Coordinate {
            latitude: center.latitude + next() * 0.02,
            longitude: center.longitude + next() * 0.02,
        })
        .collect()
}

fn thread_ids() -> BTreeSet<u64> {
    fs::read_dir("/proc/self/task")
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().parse().unwrap())
        .collect()
}

#[test]
fn tbb_workers_are_not_recreated_between_table_calls() {
    let file = env::var("OSRM_TEST_FILE").unwrap_or_else(|_| DEFAULT_FILE.to_string());
    assert!(
        Path::new(&format!("{}.hsgr", file)).exists() || Path::new(&file).exists(),
        "Couldn't load {}. Has `./prepare-test-data.sh` been run?",
        file
    );

    let osrm = Osrm::new(&file).unwrap();
    let center = coordinates().remove(0);
    let sources = scatter(&center, 40);
    let destinations = scatter(&center, 40);
    let table = || {
        osrm.table(&sources, &destinations)
            .expect("table call failed");
    };

    // Warm up: TBB starts its workers lazily on the first parallel calls.
    for _ in 0..200 {
        table();
    }
    thread::sleep(Duration::from_millis(300));
    let before = thread_ids();
    assert!(
        before.len() > 2,
        "expected TBB worker threads to persist while idle after warm-up (pool torn down after each call?), saw {:?}",
        before
    );

    // Sequential calls with idle gaps: each call's scheduler is the only active user, so without the
    // keepalive the pool would be destroyed and recreated every time.
    for _ in 0..300 {
        table();
        thread::sleep(Duration::from_millis(2));
    }
    thread::sleep(Duration::from_millis(300));
    let after = thread_ids();

    assert_eq!(
        before, after,
        "thread set changed across table calls: TBB workers were torn down and recreated"
    );
}
