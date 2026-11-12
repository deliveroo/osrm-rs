#![deny(warnings)]

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[macro_use]
mod macros;

mod errors;
mod route;
mod table;

pub use self::errors::{Error, ErrorKind, Result};
pub use self::table::RawTable;
pub use self::table::Response as TableResponse;

#[derive(Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug)]
pub struct RouteResponse {
    pub duration: f32,
    pub distance: f32,
}

/// Whether `table_raw`, `table_rows` and `route_raw` work: this crate was built
/// against an OSRM that provides them. Against stock OSRM they fail with
/// `NotImplemented` and the snap cache counters are zero.
pub fn raw_results_available() -> bool {
    unsafe { osrmc_sys::osrmc_ext_raw_available() != 0 }
}

/// How the engine holds its dataset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Map the dataset files (`true`, OSRM's default) or read them into process memory.
    pub use_mmap: bool,
}

/// Totals of the engine's snap cache since it was created, all zero when it is
/// off. Lookups are plain snaps by the table and route services; a missed
/// coordinate is stored (admitted) on its second sighting, and storing into a
/// full set evicts an entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnapCacheCounters {
    pub hits: u64,
    pub misses: u64,
    pub admissions: u64,
    pub evictions: u64,
    pub entries: u64,
    pub capacity: u64,
}

impl From<osrmc_sys::osrmc_snap_cache_counters> for SnapCacheCounters {
    fn from(counters: osrmc_sys::osrmc_snap_cache_counters) -> Self {
        SnapCacheCounters {
            hits: counters.hits,
            misses: counters.misses,
            admissions: counters.admissions,
            evictions: counters.evictions,
            entries: counters.entries,
            capacity: counters.capacity,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options { use_mmap: true }
    }
}

struct Config {
    handle: osrmc_sys::osrmc_config_t,
}

impl_drop!(Config, osrmc_sys::osrmc_config_destruct);

impl Config {
    fn new<P: AsRef<Path>>(path: P, options: Options) -> Result<Config> {
        let path = path.as_ref().as_os_str().as_bytes();
        let cstring = CString::new(path)?;
        let handle = call_with_error!(osrmc_config_construct(cstring.as_ptr()))?;
        let config = Config { handle };
        if !options.use_mmap {
            unsafe { osrmc_sys::osrmc_config_set_use_mmap(config.handle, 0) };
        }
        Ok(config)
    }
}

pub struct Osrm {
    handle: osrmc_sys::osrmc_osrm_t,
}

impl_drop!(Osrm, osrmc_sys::osrmc_osrm_destruct);

// This is just a thin wrapper around the OSRM C++ class, which is thread-safe.
unsafe impl Send for Osrm {}
unsafe impl Sync for Osrm {}

impl Osrm {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Osrm> {
        Osrm::with_options(path, Options::default())
    }

    pub fn with_options<P: AsRef<Path>>(path: P, options: Options) -> Result<Osrm> {
        let config = Config::new(path, options)?;
        let handle = call_with_error!(osrmc_osrm_construct(config.handle))?;
        Ok(Osrm { handle })
    }

    pub fn table(
        &self,
        sources: &[Coordinate],
        destinations: &[Coordinate],
    ) -> Result<TableResponse> {
        if sources.is_empty() || destinations.is_empty() {
            return Err("sources/destinations can not be empty".into());
        }

        let include_distance = true;
        let mut params = table::Parameters::new(include_distance)?;
        for source in sources {
            params.add_source(source)?;
        }
        for destination in destinations {
            params.add_destination(destination)?;
        }

        let handle = call_with_error!(osrmc_table(self.handle, params.handle))?;
        Ok(TableResponse {
            include_distance,
            handle,
        })
    }

    /// Computes a table and returns the search's own result vectors, without
    /// building a response object. Needs an OSRM build with `OSRM::TableRaw`.
    pub fn table_raw(
        &self,
        sources: &[Coordinate],
        destinations: &[Coordinate],
    ) -> Result<RawTable> {
        let params = table::Parameters::with_coordinates(sources, destinations)?;
        let handle = call_with_error!(osrmc_table_raw(self.handle, params.handle))?;
        Ok(RawTable { handle })
    }

    /// Computes a table and passes each finished source row to
    /// `on_row(row, durations, distances)` instead of storing it. `on_row` is
    /// called once per row, possibly from several threads at the same time.
    /// Units and no-route markers are those of [`RawTable`]. Needs an OSRM
    /// build with `OSRM::TableRows`.
    pub fn table_rows<F>(
        &self,
        sources: &[Coordinate],
        destinations: &[Coordinate],
        on_row: F,
    ) -> Result<()>
    where
        F: Fn(usize, &[i32], &[f32]) + Sync,
    {
        let params = table::Parameters::with_coordinates(sources, destinations)?;
        let context = table::RowContext {
            on_row: &on_row,
            columns: destinations.len(),
        };
        call_with_error!(osrmc_table_rows(
            self.handle,
            params.handle,
            Some(table::row_trampoline::<F>),
            &context as *const table::RowContext<F> as *mut std::os::raw::c_void
        ))
    }

    pub fn route(&self, from: &Coordinate, to: &Coordinate) -> Result<RouteResponse> {
        let mut params = route::Parameters::new()?;
        params.add_coordinate(from)?;
        params.add_coordinate(to)?;

        let handle = call_with_error!(osrmc_route(self.handle, params.handle))?;
        let response = route::Response::from(handle);

        Ok(RouteResponse {
            duration: response.duration()?,
            distance: response.distance()?,
        })
    }

    /// Same values as [`Osrm::route`], read without building a response
    /// object. Needs an OSRM build with `OSRM::RouteRaw`.
    pub fn route_raw(&self, from: &Coordinate, to: &Coordinate) -> Result<RouteResponse> {
        let mut params = route::Parameters::new()?;
        params.add_coordinate(from)?;
        params.add_coordinate(to)?;

        let mut duration = 0f32;
        let mut distance = 0f32;
        call_with_error!(osrmc_route_raw(
            self.handle,
            params.handle,
            &mut duration,
            &mut distance
        ))?;
        Ok(RouteResponse { duration, distance })
    }

    /// Counters of the snap cache shared by the table and route calls. Needs an
    /// OSRM build with `OSRM::GetSnapCacheCounters`.
    pub fn snap_cache_counters(&self) -> SnapCacheCounters {
        let mut counters = std::mem::MaybeUninit::uninit();
        unsafe {
            osrmc_sys::osrmc_snap_cache_counters(self.handle, counters.as_mut_ptr());
            counters.assume_init()
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::errors::ErrorKind;

    const OSRM_FILE: &str = "./test-data/gcc-states-latest.osrm";

    const COORDINATE_A: Coordinate = Coordinate {
        latitude: 24.447_618,
        longitude: 54.371_037,
    };
    const COORDINATE_B: Coordinate = Coordinate {
        latitude: 24.454_87,
        longitude: 54.391_076,
    };
    const COORDINATE_C: Coordinate = Coordinate {
        latitude: 24.454_979,
        longitude: 54.376_52,
    };

    const COORDINATE_BROKEN_A: Coordinate = Coordinate {
        latitude: 25.071_65,
        longitude: 55.402_115,
    };
    const COORDINATE_BROKEN_B: Coordinate = Coordinate {
        latitude: 25.086_226,
        longitude: 55.385_334,
    };

    const COORDINATE_INVALID: Coordinate = Coordinate {
        latitude: -190.0,
        longitude: -190.0,
    };

    fn load_osrm() -> Result<Osrm> {
        if !Path::new(OSRM_FILE).exists() {
            return Err(format!(
                "Couldn't load {}. Has `./prepare-test-data.sh` been run?",
                OSRM_FILE
            )
            .into());
        }

        let osrm = Osrm::new(OSRM_FILE)?;
        Ok(osrm)
    }

    #[test]
    fn test_table() -> Result<()> {
        let osrm = load_osrm()?;
        let result = osrm.table(&[COORDINATE_A, COORDINATE_B], &[COORDINATE_C])?;

        assert_ne!(result.get_duration(0, 0)?, 0.0);
        assert_ne!(result.get_duration(1, 0)?, 0.0);
        assert_ne!(result.get_duration(0, 0)?, result.get_duration(1, 0)?);

        assert_ne!(result.get_distance(0, 0)?, 0.0);
        assert_ne!(result.get_distance(1, 0)?, 0.0);
        assert_ne!(result.get_distance(0, 0)?, result.get_distance(1, 0)?);

        Ok(())
    }

    #[test]
    fn test_table_no_parameters() -> Result<()> {
        let osrm = load_osrm()?;
        let result = osrm.table(&[], &[]);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_route() -> Result<()> {
        let osrm = load_osrm()?;

        let result1 = osrm.route(&COORDINATE_A, &COORDINATE_B)?;

        assert_ne!(result1.duration, 0.0);
        assert_ne!(result1.distance, 0.0);

        let result2 = osrm.route(&COORDINATE_A, &COORDINATE_C)?;

        assert_ne!(result2.duration, 0.0);
        assert_ne!(result2.distance, 0.0);

        assert_ne!(result1.duration, result2.duration);
        assert_ne!(result1.distance, result2.distance);

        Ok(())
    }

    #[test]
    fn test_unroutable() -> Result<()> {
        let osrm = load_osrm()?;

        let result = osrm.route(&COORDINATE_BROKEN_A, &COORDINATE_BROKEN_B);
        assert_eq!(result.err().unwrap().kind(), ErrorKind::NoRoute);

        Ok(())
    }

    #[test]
    fn test_unroutable_table() -> Result<()> {
        let osrm = load_osrm()?;

        let resp = osrm.table(&[COORDINATE_BROKEN_A], &[COORDINATE_BROKEN_B])?;

        let result = resp.get_duration(0, 0);
        assert_eq!(result.err().unwrap().kind(), ErrorKind::NoRoute);

        let result = resp.get_distance(0, 0);
        assert_eq!(result.err().unwrap().kind(), ErrorKind::NoRoute);

        Ok(())
    }

    fn grid() -> Vec<Coordinate> {
        (0..12)
            .flat_map(|i| {
                (0..12).map(move |j| Coordinate {
                    latitude: 24.44 + 0.002 * i as f32,
                    longitude: 54.36 + 0.0025 * j as f32,
                })
            })
            .collect()
    }

    #[test]
    fn test_table_raw_matches_table() -> Result<()> {
        let osrm = load_osrm()?;
        let sources = grid();
        let mut destinations = grid();
        destinations.truncate(50);
        destinations.push(COORDINATE_BROKEN_B);

        let table = osrm.table(&sources, &destinations)?;
        let raw = osrm.table_raw(&sources, &destinations)?;
        assert_eq!(raw.durations().len(), sources.len() * destinations.len());
        assert_eq!(raw.distances().len(), sources.len() * destinations.len());

        for from in 0..sources.len() {
            for to in 0..destinations.len() {
                let cell = from * destinations.len() + to;
                match table.get_duration(from, to) {
                    // The JSON response divides deciseconds by 10 as a double.
                    Ok(duration) => {
                        assert_eq!(duration, (f64::from(raw.durations()[cell]) / 10.0) as f32)
                    }
                    Err(err) => {
                        assert_eq!(err.kind(), ErrorKind::NoRoute);
                        assert_eq!(raw.durations()[cell], RawTable::NO_ROUTE_DURATION);
                    }
                }
                match table.get_distance(from, to) {
                    // The JSON response rounds to decimetres in float, then divides as a double.
                    Ok(distance) => assert_eq!(
                        distance,
                        (f64::from((raw.distances()[cell] * 10.0).round()) / 10.0) as f32
                    ),
                    Err(err) => {
                        assert_eq!(err.kind(), ErrorKind::NoRoute);
                        assert_eq!(raw.distances()[cell], RawTable::NO_ROUTE_DISTANCE);
                    }
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_table_rows_match_table_raw() -> Result<()> {
        let osrm = load_osrm()?;
        let sources = grid();
        let destinations = grid();

        let raw = osrm.table_raw(&sources, &destinations)?;
        let rows = std::sync::Mutex::new(vec![None; sources.len()]);
        osrm.table_rows(&sources, &destinations, |row, durations, distances| {
            let mut rows = rows.lock().unwrap();
            assert!(rows[row].is_none(), "row {} handed over twice", row);
            rows[row] = Some((durations.to_vec(), distances.to_vec()));
        })?;

        let columns = destinations.len();
        for (row, values) in rows.into_inner().unwrap().into_iter().enumerate() {
            let (durations, distances) = values.expect("every row is handed over");
            assert_eq!(
                &durations[..],
                &raw.durations()[row * columns..(row + 1) * columns]
            );
            assert_eq!(
                &distances[..],
                &raw.distances()[row * columns..(row + 1) * columns]
            );
        }

        Ok(())
    }

    #[test]
    fn test_table_raw_no_parameters() -> Result<()> {
        let osrm = load_osrm()?;
        assert!(osrm.table_raw(&[], &[]).is_err());
        assert!(osrm.table_rows(&[], &[], |_, _, _| {}).is_err());
        Ok(())
    }

    #[test]
    fn test_route_raw_matches_route() -> Result<()> {
        let osrm = load_osrm()?;
        let points = grid();
        for (i, from) in points.iter().enumerate().step_by(5) {
            for to in points.iter().skip(i % 7).step_by(9) {
                match (osrm.route(from, to), osrm.route_raw(from, to)) {
                    (Ok(route), Ok(raw)) => {
                        assert_eq!(route.duration, raw.duration);
                        assert_eq!(route.distance, raw.distance);
                    }
                    (Err(route), Err(raw)) => assert_eq!(route.kind(), raw.kind()),
                    (route, raw) => panic!("route {:?} but route_raw {:?}", route, raw),
                }
            }
        }

        let raw = osrm.route_raw(&COORDINATE_BROKEN_A, &COORDINATE_BROKEN_B);
        assert_eq!(raw.err().unwrap().kind(), ErrorKind::NoRoute);
        let raw = osrm.route_raw(&COORDINATE_A, &COORDINATE_INVALID);
        assert_eq!(raw.err().unwrap().kind(), ErrorKind::InvalidCoordinate);

        Ok(())
    }

    #[test]
    fn test_in_memory_dataset_gives_the_same_results() -> Result<()> {
        let mapped = load_osrm()?;
        let in_memory = Osrm::with_options(OSRM_FILE, Options { use_mmap: false })?;
        let points = grid();

        let a = mapped.table_raw(&points, &points)?;
        let b = in_memory.table_raw(&points, &points)?;
        assert_eq!(a.durations(), b.durations());
        assert_eq!(a.distances(), b.distances());

        Ok(())
    }

    #[test]
    fn test_raw_results_available() {
        assert!(raw_results_available());
    }

    #[test]
    fn test_snap_cache_counts_and_keeps_results() -> Result<()> {
        let osrm = load_osrm()?;
        let points = grid();
        // One lookup per distinct coordinate and request.
        let lookups = points.len() as u64;

        let first = osrm.table_raw(&points, &points)?;
        let counters = osrm.snap_cache_counters();
        assert!(counters.capacity > 0);
        assert_eq!(counters.misses, lookups);
        assert_eq!(counters.hits, 0);
        assert_eq!(counters.admissions, 0);

        // Stored on the second sighting, read from the cache on the third.
        for round in 2..=3 {
            let again = osrm.table_raw(&points, &points)?;
            assert_eq!(first.durations(), again.durations());
            assert_eq!(first.distances(), again.distances());
            let counters = osrm.snap_cache_counters();
            assert_eq!(counters.hits + counters.misses, round * lookups);
            assert_eq!(counters.entries, counters.admissions - counters.evictions);
        }
        let counters = osrm.snap_cache_counters();
        assert!(counters.admissions > 0);
        assert!(counters.hits > 0);

        Ok(())
    }

    #[test]
    fn test_invalid_coordinate() -> Result<()> {
        let osrm = load_osrm()?;

        let result = osrm.route(&COORDINATE_A, &COORDINATE_INVALID);
        assert_eq!(result.err().unwrap().kind(), ErrorKind::InvalidCoordinate);

        Ok(())
    }
}
