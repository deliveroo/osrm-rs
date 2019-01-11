use crate::errors::*;
use crate::Point;

pub struct Parameters {
    pub handle: osrmc_sys::osrmc_route_params_t,
}

impl_drop!(Parameters, osrmc_sys::osrmc_route_params_destruct);

impl Parameters {
    pub fn new() -> Result<Parameters> {
        let handle = call_with_error!(osrmc_route_params_construct())?;
        Ok(Parameters { handle })
    }

    pub fn add_coordinate(&mut self, point: &Point) -> Result<()> {
        call_with_error!(osrmc_params_add_coordinate(
            self.handle as osrmc_sys::osrmc_params_t,
            point.longitude,
            point.latitude
        ))?;
        Ok(())
    }
}

pub struct Response {
    handle: osrmc_sys::osrmc_route_response_t,
}

impl_drop!(Response, osrmc_sys::osrmc_route_response_destruct);

impl From<osrmc_sys::osrmc_route_response_t> for Response {
    fn from(handle: osrmc_sys::osrmc_route_response_t) -> Self {
        Response { handle }
    }
}

impl Response {
    pub fn distance(&self) -> Result<f32> {
        let result = call_with_error!(osrmc_route_response_distance(self.handle))?;
        Ok(result)
    }

    pub fn duration(&self) -> Result<f32> {
        let result = call_with_error!(osrmc_route_response_duration(self.handle))?;
        Ok(result)
    }
}
