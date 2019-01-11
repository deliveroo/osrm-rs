use std::ffi::CString;

mod errors;
mod table;
#[macro_use]
mod macros;
mod route;

pub use self::errors::*;
pub use self::table::Response as TableResponse;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
}

pub struct Response {
    pub duration: f32,
    pub distance: f32,
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

    pub fn table(&self, sources: &[Point], destinations: &[Point]) -> Result<TableResponse> {
        let mut params = TableParameters::new()?;
        for source in sources {
            params.add_source(source)?;
        }
        for destination in destinations {
            params.add_destination(destination)?;
        }

        let handle = call_with_error!(osrmc_table(self.handle, params.handle))?;
        Ok(TableResponse::from(handle))
    }

    pub fn route(&self, from: &Point, to: &Point) -> Result<Response> {
        let mut params = route::Parameters::new()?;
        params.add_coordinate(from);
        params.add_coordinate(to);

        let handle = call_with_error!(osrmc_route(self.handle, params.handle))?;
        let response = route::Response::from(handle);

        Ok(Response {
            duration: response.duration()?,
            distance: response.distance()?,
        })
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
        assert_eq!(result.get_duration(0, 0).unwrap(), 0.0);
    }
}
