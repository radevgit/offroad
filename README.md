# Offroad
[![Crates.io](https://img.shields.io/crates/v/offroad.svg?color=blue)](https://crates.io/crates/offroad)
[![Documentation](https://docs.rs/offroad/badge.svg)](https://docs.rs/offroad)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Adding the library to your project

To use the Offroad library in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
offroad = "0.4"
```

## 2D offsetting for arc polylines/polygons
![](https://raw.githubusercontent.com/radevgit/offroad/refs/heads/main/img/offsets.svg "arc-line polygon offsets")


## Examples

### Offsetting Arc Lines

```rust
use togo::prelude::*;
use offroad::prelude::*;

fn main() {
    // Configuration for offsetting
    let mut cfg = OffsetCfg::default();
    let mut svg = SVG::new(300.0, 300.0, Some("/tmp/arcline.svg"));
    cfg.svg = Some(&mut svg);
    // Show original arcline in SVG output
    cfg.svg_orig = true;
    // Show final offset arclines in SVG output
    cfg.svg_final = true;

    let arc0 = arc_circle_parametrization(point(40.0, 100.0), point(140.0, 200.0), 0.0);
    let arc1 = arc_circle_parametrization(point(140.0, 200.0), point(240.0, 100.0), 0.5);
    let arc2 = arc_circle_parametrization(point(240.0, 100.0), point(40.0, 100.0), 1.3);
    let arcs_orig = vec![arc0, arc1, arc2];

    // External offsetting
    let offset_arclines = offset_arcline_to_arcline(&arcs_orig, 15.0, &mut cfg);

    println!("Input arcline has {} vertices", arcs_orig.len());
    println!("Output has {} arclines", offset_arclines.len());
    for (i, arcline) in offset_arclines.iter().enumerate() {
        println!("Arcline {}: {} vertices", i, arcline.len());
    }

    if let Some(svg) = cfg.svg.as_mut() {
        // Write svg to file
        svg.write_stroke_width(0.1);
    }
}
```



