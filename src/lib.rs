use std::ffi::CString;

mod errors;
mod table;
#[macro_use]
mod macros;

pub use self::errors::*;
use self::table::{TableParameters, TableResponse};

pub struct Point {
    latitude: f32,
    longitude: f32,
}

struct Config {
    handle: osrmc_sys::osrmc_config_t,
}

impl_drop!(Config, osrmc_sys::osrmc_config_destruct);

impl Config {
    fn new<S: Into<Vec<u8>>>(path: S) -> Result<Config> {
        let cstring = CString::new(path)?;
        let handle = call_with_error!(osrmc_config_construct(cstring.as_ptr()))?;
        Ok(Config { handle })
    }
}

pub struct OSRM {
    handle: osrmc_sys::osrmc_osrm_t,
}

impl_drop!(OSRM, osrmc_sys::osrmc_osrm_destruct);

impl OSRM {
    pub fn new<S: Into<Vec<u8>>>(path: S) -> Result<OSRM> {
        let config = Config::new(path)?;
        let handle = call_with_error!(osrmc_osrm_construct(config.handle))?;
        Ok(OSRM { handle })
    }

    pub fn table(&self, sources: &[Point], destinations: &[Point]) -> Result<f32> {
        let mut params = TableParameters::new()?;
        for source in sources {
            params.add_source(source)?;
        }
        for destination in destinations {
            params.add_destination(destination)?;
        }

        let handle = call_with_error!(osrmc_table(self.handle, params.handle))?;
        let response = TableResponse::from(handle);
        Ok(response.to_one()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let osrm = OSRM::new("./data/1.osrm").expect("uh oh");
        let result = osrm
            .table(
                &[Point {
                    latitude: 51.5062628,
                    longitude: -0.0996648,
                }],
                &[Point {
                    latitude: 51.5062628,
                    longitude: -0.124899,
                }],
            )
            .expect("uh oh");
        assert_eq!(result, 0.0);
    }
}
