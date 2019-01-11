use crate::errors::*;
use crate::Point;

pub struct RouteParameters {
    pub handle: osrmc_sys::osrmc_route_params_t,
}

impl_drop!(RouteParameters, osrmc_sys::osrmc_route_params_destruct);

impl RouteParameters {
    pub fn new() -> Result<RouteParameters> {
        let handle = call_with_error!(osrmc_route_params_construct())?;
        Ok(RouteParameters { handle })
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

pub struct RouteResponse {
    handle: osrmc_sys::osrmc_route_response_t,
}

impl_drop!(RouteResponse, osrmc_sys::osrmc_route_response_destruct);

impl From<osrmc_sys::osrmc_route_response_t> for RouteResponse {
    fn from(handle: osrmc_sys::osrmc_route_response_t) -> Self {
        RouteResponse { handle }
    }
}

impl RouteResponse {
    pub fn distance(&self) -> Result<f32> {
        let result = call_with_error!(osrmc_route_response_distance(self.handle))?;
        Ok(result)
    }

    pub fn duration(&self) -> Result<f32> {
        let result = call_with_error!(osrmc_route_response_duration(self.handle))?;
        Ok(result)
    }
}
