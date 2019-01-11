use crate::{errors::*, Coordinate};

pub struct Parameters {
    pub handle: osrmc_sys::osrmc_table_params_t,
    num_coords: usize,
}

impl_drop!(Parameters, osrmc_sys::osrmc_table_params_destruct);

impl Parameters {
    pub fn new() -> Result<Parameters> {
        let handle = call_with_error!(osrmc_table_params_construct())?;
        Ok(Parameters {
            handle,
            num_coords: 0,
        })
    }

    fn add_coordinate(&mut self, coordinate: &Coordinate) -> Result<usize> {
        call_with_error!(osrmc_params_add_coordinate(
            self.handle as osrmc_sys::osrmc_params_t,
            coordinate.longitude,
            coordinate.latitude
        ))?;
        let index = self.num_coords;
        self.num_coords += 1;
        Ok(index)
    }

    pub fn add_source(&mut self, coordinate: &Coordinate) -> Result<()> {
        let index = self.add_coordinate(coordinate)?;
        call_with_error!(osrmc_table_params_add_source(self.handle, index))?;
        Ok(())
    }

    pub fn add_destination(&mut self, coordinate: &Coordinate) -> Result<()> {
        let index = self.add_coordinate(coordinate)?;
        call_with_error!(osrmc_table_params_add_destination(self.handle, index))?;
        Ok(())
    }
}

pub struct Response {
    handle: osrmc_sys::osrmc_table_response_t,
}

impl_drop!(Response, osrmc_sys::osrmc_table_response_destruct);

impl Response {
    pub fn get_duration(&self, from: usize, to: usize) -> Result<f32> {
        let result = call_with_error!(osrmc_table_response_duration(
            self.handle,
            from as u64,
            to as u64
        ))?;
        Ok(result)
    }
}

impl From<osrmc_sys::osrmc_table_response_t> for Response {
    fn from(handle: osrmc_sys::osrmc_table_response_t) -> Response {
        Response { handle }
    }
}
