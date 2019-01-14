# osrm-rs

Rust bindings for Open Source Routing Machine (OSRM).

## Developing

```sh
# Install OSRM and its dependencies.
brew install osrm-backend

# Update/initialise the libosrmc submodule:
git submodule update --init

cargo test
cargo build
```

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


## Testing

Requires: `wget`, `osrm-bckend`, `docker`

```
# First download/process the required maps:
./prepare-test-data

# Run tests:
cargo test
```
