use std::ffi::CString;

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
    fn new<S: Into<Vec<u8>>>(path: S) -> Result<Config> {
        let cstring = CString::new(path)?;
        let handle = call_with_error!(osrmc_config_construct(cstring.as_ptr()))?;
        Ok(Config { handle })
    }
}

pub struct Osrm {
    handle: osrmc_sys::osrmc_osrm_t,
}

impl_drop!(Osrm, osrmc_sys::osrmc_osrm_destruct);

impl Osrm {
    pub fn new<S: Into<Vec<u8>>>(path: S) -> Result<Osrm> {
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
    use super::*;

    #[test]
    fn it_works() {
        let osrm = Osrm::new("./data/1.osrm").expect("uh oh");
        let result = osrm
            .table(
                &[Coordinate {
                    latitude: 51.5062628,
                    longitude: -0.0996648,
                }],
                &[Coordinate {
                    latitude: 51.5062628,
                    longitude: -0.124899,
                }],
            )
            .expect("uh oh");
        assert_eq!(result.get_duration(0, 0).unwrap(), 0.0);
    }
}
