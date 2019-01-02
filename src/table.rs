use crate::{call_with_error, errors::*, impl_drop, Point};

pub struct TableParameters {
    pub handle: osrmc_sys::osrmc_table_params_t,
    num_coords: usize,
}

impl_drop!(TableParameters, osrmc_sys::osrmc_table_params_destruct);

impl TableParameters {
    pub fn new() -> Result<TableParameters> {
        let handle = call_with_error!(osrmc_table_params_construct())?;
        Ok(TableParameters {
            handle,
            num_coords: 0,
        })
    }

    fn add_coordinate(&mut self, point: &Point) -> Result<usize> {
        call_with_error!(osrmc_params_add_coordinate(
            self.handle as osrmc_sys::osrmc_params_t,
            point.longitude,
            point.latitude
        ))?;
        let index = self.num_coords;
        self.num_coords += 1;
        Ok(index)
    }

    pub fn add_source(&mut self, point: &Point) -> Result<()> {
        let index = self.add_coordinate(point)?;
        call_with_error!(osrmc_table_params_add_source(self.handle, index))?;
        Ok(())
    }

    pub fn add_destination(&mut self, point: &Point) -> Result<()> {
        let index = self.add_coordinate(point)?;
        call_with_error!(osrmc_table_params_add_destination(self.handle, index))?;
        Ok(())
    }
}

pub struct TableResponse {
    handle: osrmc_sys::osrmc_table_response_t,
}

impl_drop!(TableResponse, osrmc_sys::osrmc_table_response_destruct);

impl TableResponse {
    pub fn get_duration(&self, from: usize, to: usize) -> Result<f32> {
        let result = call_with_error!(osrmc_table_response_duration(
            self.handle,
            from as u64,
            to as u64
        ))?;
        Ok(result)
    }
}

impl From<osrmc_sys::osrmc_table_response_t> for TableResponse {
    fn from(handle: osrmc_sys::osrmc_table_response_t) -> TableResponse {
        TableResponse { handle }
    }
}
