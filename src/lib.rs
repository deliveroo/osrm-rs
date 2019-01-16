#![deny(warnings)]

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[macro_use]
mod macros;

mod errors;
mod route;
mod table;

pub use self::errors::{Error, Result};
pub use self::table::Response as TableResponse;

#[derive(Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub latitude: f32,
    pub longitude: f32,
}

pub struct RouteResponse {
    pub duration: f32,
    pub distance: f32,
}

struct Config {
    handle: osrmc_sys::osrmc_config_t,
}

impl_drop!(Config, osrmc_sys::osrmc_config_destruct);

impl Config {
    fn new<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref().as_os_str().as_bytes();
        let cstring = CString::new(path)?;
        let handle = call_with_error!(osrmc_config_construct(cstring.as_ptr()))?;
        Ok(Config { handle })
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
        let config = Config::new(path)?;
        let handle = call_with_error!(osrmc_osrm_construct(config.handle))?;
        Ok(Osrm { handle })
    }

    pub fn table(
        &self,
        sources: &[Coordinate],
        destinations: &[Coordinate],
    ) -> Result<TableResponse> {
        let mut params = table::Parameters::new()?;
        for source in sources {
            params.add_source(source)?;
        }
        for destination in destinations {
            params.add_destination(destination)?;
        }

        let handle = call_with_error!(osrmc_table(self.handle, params.handle))?;
        Ok(TableResponse::from(handle))
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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const OSRM_FILE: &str = "./test-data/berlin-latest.osrm";

    fn load_osrm() -> Result<Osrm> {
        if !Path::new(OSRM_FILE).exists() {
            return Err(format!(
                "Couldn't load {}. Has `./prepare-test-data.sh` been run?",
                OSRM_FILE
            ))?;
        }

        let osrm = Osrm::new(OSRM_FILE)?;
        Ok(osrm)
    }

    #[test]
    fn test_table() -> Result<()> {
        let osrm = load_osrm()?;
        let result = osrm.table(
            &[
                Coordinate {
                    latitude: 52.519930,
                    longitude: 13.438640,
                },
                Coordinate {
                    latitude: 52.525081,
                    longitude: 13.430388,
                },
            ],
            &[Coordinate {
                latitude: 52.513191,
                longitude: 13.415852,
            }],
        )?;

        assert_ne!(result.get_duration(0, 0)?, 0.0);
        assert_ne!(result.get_duration(1, 0)?, 0.0);
        assert_ne!(result.get_duration(0, 0)?, result.get_duration(1, 0)?);

        Ok(())
    }

    #[test]
    fn test_route() -> Result<()> {
        let osrm = load_osrm()?;

        let result1 = osrm.route(
            &Coordinate {
                latitude: 52.519930,
                longitude: 13.438640,
            },
            &Coordinate {
                latitude: 52.525081,
                longitude: 13.430388,
            },
        )?;

        assert_ne!(result1.duration, 0.0);
        assert_ne!(result1.distance, 0.0);

        let result2 = osrm.route(
            &Coordinate {
                latitude: 52.519930,
                longitude: 13.438640,
            },
            &Coordinate {
                latitude: 52.513191,
                longitude: 13.415852,
            },
        )?;

        assert_ne!(result2.duration, 0.0);
        assert_ne!(result2.distance, 0.0);

        assert_ne!(result1.duration, result2.duration);
        assert_ne!(result1.distance, result2.distance);

        Ok(())
    }
}
