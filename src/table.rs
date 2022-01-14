use crate::{errors::*, Coordinate};

struct Annotations {
    handle: osrmc_sys::osrmc_table_annotations_t,
}

impl_drop!(Annotations, osrmc_sys::osrmc_table_annotations_destruct);

impl Annotations {
    fn new(include_distance: bool) -> Result<Annotations> {
        let handle = call_with_error!(osrmc_table_annotations_construct())?;
        let mut annotations = Annotations { handle };
        if include_distance {
            annotations.set_distance()?;
        }
        Ok(annotations)
    }

    fn set_distance(&mut self) -> Result<()> {
        call_with_error!(osrmc_table_annotations_enable_distance(self.handle, true))?;
        Ok(())
    }
}

pub struct Parameters {
    pub handle: osrmc_sys::osrmc_table_params_t,
    num_coords: usize,
}

impl_drop!(Parameters, osrmc_sys::osrmc_table_params_destruct);

impl Parameters {
    pub fn new(include_distance: bool) -> Result<Parameters> {
        let handle = call_with_error!(osrmc_table_params_construct())?;

        let annotations = Annotations::new(include_distance)?;
        call_with_error!(osrmc_table_params_set_annotations(
            handle,
            annotations.handle
        ))?;

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
        call_with_error!(osrmc_table_params_add_source(self.handle, index as u64))?;
        Ok(())
    }

    pub fn add_destination(&mut self, coordinate: &Coordinate) -> Result<()> {
        let index = self.add_coordinate(coordinate)?;
        call_with_error!(osrmc_table_params_add_destination(
            self.handle,
            index as u64
        ))?;
        Ok(())
    }
}

pub struct Response {
    pub(crate) include_distance: bool,
    pub(crate) handle: osrmc_sys::osrmc_table_response_t,
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

    pub fn get_distance(&self, from: usize, to: usize) -> Result<f32> {
        if !self.include_distance {
            return Err("get_distance() called on table response without distance".into());
        }

        let result = call_with_error!(osrmc_table_response_distance(
            self.handle,
            from as u64,
            to as u64
        ))?;
        Ok(result)
    }
}
