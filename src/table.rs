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
    pub(crate) num_sources: usize,
    pub(crate) num_destinations: usize,
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

    /// Number of sources (rows) in the table.
    pub fn num_sources(&self) -> usize {
        self.num_sources
    }

    /// Number of destinations (columns) in the table.
    pub fn num_destinations(&self) -> usize {
        self.num_destinations
    }

    /// All durations in one pass, in row-major order: the duration from source `from` to
    /// destination `to` is at index `from * num_destinations() + to`.
    ///
    /// Unreachable cells are `f32::INFINITY` (where [`Response::get_duration`] returns
    /// `ErrorKind::NoRoute`), so callers should check `is_finite()`.
    pub fn durations(&self) -> Result<Vec<f32>> {
        let len = self.num_cells();
        let mut out = Vec::with_capacity(len);
        let written = call_with_error!(osrmc_table_response_durations(
            self.handle,
            out.as_mut_ptr(),
            len as _
        ))?;
        self.finish(out, written as usize)
    }

    /// All distances in one pass, with the same layout and unreachable semantics as
    /// [`Response::durations`].
    pub fn distances(&self) -> Result<Vec<f32>> {
        if !self.include_distance {
            return Err("distances() called on table response without distance".into());
        }

        let len = self.num_cells();
        let mut out = Vec::with_capacity(len);
        let written = call_with_error!(osrmc_table_response_distances(
            self.handle,
            out.as_mut_ptr(),
            len as _
        ))?;
        self.finish(out, written as usize)
    }

    fn num_cells(&self) -> usize {
        self.num_sources * self.num_destinations
    }

    fn finish(&self, mut out: Vec<f32>, written: usize) -> Result<Vec<f32>> {
        let expected = self.num_cells();
        if written != expected {
            return Err(format!(
                "table response has {} cells, expected {}x{}",
                written, self.num_sources, self.num_destinations
            )
            .into());
        }
        // SAFETY: the osrmc_bulk shim initialised the first `written` elements, and `written`
        // equals the `len` the buffer was allocated with, so it is within capacity.
        unsafe { out.set_len(written) };
        Ok(out)
    }
}
