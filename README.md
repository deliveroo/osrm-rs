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
