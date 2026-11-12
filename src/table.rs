use std::collections::hash_map::{Entry, HashMap};

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

    /// Parameters for a table of `sources` x `destinations`, with distances.
    /// Each distinct coordinate is passed (and snapped) once, with sources and
    /// destinations referring to it by index.
    pub fn with_coordinates(
        sources: &[Coordinate],
        destinations: &[Coordinate],
    ) -> Result<Parameters> {
        if sources.is_empty() || destinations.is_empty() {
            return Err("sources/destinations can not be empty".into());
        }
        let mut params = Parameters::new(true)?;
        let mut indices = HashMap::with_capacity(sources.len() + destinations.len());
        let mut index_of = |params: &mut Parameters, coordinate: &Coordinate| -> Result<u64> {
            let key = (
                coordinate.latitude.to_bits(),
                coordinate.longitude.to_bits(),
            );
            let index = match indices.entry(key) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => *entry.insert(params.add_coordinate(coordinate)?),
            };
            Ok(index as u64)
        };
        for source in sources {
            let index = index_of(&mut params, source)?;
            call_with_error!(osrmc_table_params_add_source(params.handle, index))?;
        }
        for destination in destinations {
            let index = index_of(&mut params, destination)?;
            call_with_error!(osrmc_table_params_add_destination(params.handle, index))?;
        }
        // OSRM requires at least two coordinates even when every source and
        // destination is the same point. Keep the request valid without
        // changing its source/destination indexes or result dimensions.
        if params.num_coords == 1 {
            params.add_coordinate(&sources[0])?;
        }
        Ok(params)
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

/// A table as the search computed it, owned by OSRM and borrowed without copying.
pub struct RawTable {
    pub(crate) handle: osrmc_sys::osrmc_table_raw_t,
}

impl_drop!(RawTable, osrmc_sys::osrmc_table_raw_destruct);

// The handle owns two vectors that are only read after construction.
unsafe impl Send for RawTable {}
unsafe impl Sync for RawTable {}

impl RawTable {
    /// Marks a cell without a route in [`RawTable::durations`].
    pub const NO_ROUTE_DURATION: i32 = i32::MAX;
    /// Marks a cell without a route in [`RawTable::distances`].
    pub const NO_ROUTE_DISTANCE: f32 = f32::MAX;

    /// Durations in deciseconds, row-major sources x destinations.
    pub fn durations(&self) -> &[i32] {
        let mut len = 0;
        let data = unsafe { osrmc_sys::osrmc_table_raw_durations(self.handle, &mut len) };
        if data.is_null() || len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(data, len as usize) }
    }

    /// Distances in metres, row-major sources x destinations.
    pub fn distances(&self) -> &[f32] {
        let mut len = 0;
        let data = unsafe { osrmc_sys::osrmc_table_raw_distances(self.handle, &mut len) };
        if data.is_null() || len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(data, len as usize) }
    }
}

pub(crate) struct RowContext<'a, F> {
    pub(crate) on_row: &'a F,
    pub(crate) columns: usize,
}

/// Called by OSRM for each finished row, from its worker threads.
pub(crate) unsafe extern "C" fn row_trampoline<F>(
    context: *mut std::os::raw::c_void,
    row: std::os::raw::c_ulong,
    durations: *const std::os::raw::c_int,
    distances: *const f32,
) where
    F: Fn(usize, &[i32], &[f32]) + Sync,
{
    let context = &*(context as *const RowContext<F>);
    let durations = std::slice::from_raw_parts(durations, context.columns);
    let distances = std::slice::from_raw_parts(distances, context.columns);
    // A panic must not unwind into OSRM.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (context.on_row)(row as usize, durations, distances)
    }));
    if outcome.is_err() {
        std::process::abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_coordinates_preserve_native_minimum() -> Result<()> {
        let point = Coordinate {
            latitude: 24.447_618,
            longitude: 54.358_475,
        };
        for (rows, columns) in [(1, 1), (1, 4), (3, 1), (3, 4)] {
            let params = Parameters::with_coordinates(
                &vec![point.clone(); rows],
                &vec![point.clone(); columns],
            )?;
            // OSRM TableParameters::IsValid requires at least two coordinates,
            // including for nonempty tables whose points are all identical.
            assert_eq!(params.num_coords, 2, "{}x{} identical table", rows, columns);
        }
        Ok(())
    }
}
