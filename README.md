# waterfall - graphical rendering of a heatmap

waterfall is a library to render a PNG image from a Heatmap. It is expected to be useful in visualising latency measurements over time

## Usage

To use `waterfall`, first add this to your `Cargo.toml`:

```toml
[dependencies]
waterfall = "0.2.0"
```

Then, add this to your crate root:

```rust
extern crate waterfall;
```

## Features

* renders a PNG of a heatmap
* uses `rusttype` to render annotations

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
