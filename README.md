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
assert_eq!(result, 0.0);
```
