# osrm-rs

Rust bindings for Open Source Routing Machine (OSRM).

## Example

```rust
let osrm = OSRM::new("./data/1.osrm")?;
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
    )?;
assert_eq!(result.get_duration(0, 0)?, 0.0);
```

## Raw results

`Osrm::table_raw`, `Osrm::table_rows` and `Osrm::route_raw` return the same numbers as `table` and `route`
without building an OSRM response object: `table_raw` borrows the search's own result vectors (durations in
deciseconds, distances in metres), `table_rows` hands over each finished row from OSRM's worker threads, and
`route_raw` reads a route's duration and distance directly. `Osrm::with_options` can read the dataset into
process memory instead of mapping it.

They are implemented in `osrmc-sys/src/osrmc_ext.cc` next to libosrmc and need an OSRM build with
`OSRM::TableRaw`, `OSRM::TableRows` and `OSRM::RouteRaw`, which are not in stock OSRM 5.26. Against stock OSRM
the crate still builds (`build.rs` checks the installed headers) and `osrm::raw_results_available()` is false: the raw
calls then fail with `NotImplemented`. Build with `OSRMC_EXT_REQUIRE_RAW=1` to make the build fail instead when
the patched headers are not found. `Osrm::snap_cache_counters` reports OSRM's snap cache (zero when it is off or
unavailable).

## Developing

```sh
# Install OSRM and its dependencies
brew install osrm-backend
# OR `brew install --HEAD deliveroo/osrm/osrm-backend` for Deliveroo's patched version

# Update/initialise the libosrmc submodule
git submodule update --init

# Build library
cargo build
```

## Testing

```sh
# Follow steps in `Developing` section

# Install additional dependencies
brew install wget docker

# Download/process the required maps
./prepare-test-data.sh

# Run tests
cargo test
```

The suite supports stock and patched OSRM builds. With stock OSRM it checks that the legacy table/route
APIs still work, raw calls return `NotImplemented` without invoking row callbacks, and cache counters
are zero. Tests that require raw results print a skip message; use `cargo test -- --nocapture` to see it.

For a patched build, run `OSRM_TEST_REQUIRE_RAW=1 cargo test`. This fails if raw-result support is missing,
so a patched test job cannot silently skip the parity checks. Raw parity tests include all-identical
coordinates in 1×1, 1×N, N×1 and N×M tables. The cache test expects the cache to be enabled.
