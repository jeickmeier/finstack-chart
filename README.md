# finstack-chart

Compose immutable plots from ordinary data, retain a `Chart` for live updates, and use
one shared engine for native windows and headless SVG/PDF/PNG publication.

```rust
use chart_core::prelude::*;

let data = Data::columns()
    .column("time", [1.0, 2.0, 3.0])
    .column("value", [10.0, 12.0, 11.0])
    .build()?;
let plot = plot(data)
    .aes(aes().x("time").y("value"))
    .layer(line().name("prices"))
    .layer(points().size(3.0))
    .title(title("Prices"))
    .x_axis(x_axis().label("Time"))
    .y_axis(y_axis().label("USD"))
    .build()?;
```

`labels()` adds an x/y annotation. Titles, subtitles, axes, legends, captions and notes
have separate builders. Statistics, positions, scales, facets, rich text and registered
extensions compose through this same API. Start with the
[authoring guide](docs/authoring-guide.md) and
[capability register](docs/primary-authoring-api.md).

For publication, create a reusable `chart_export::Output` from supplied font bytes and
call `output.request(&plot, export_options(PageSize::millimeters(180.0, 120.0)?))?.prepare()?.export(Format::Svg)?.bytes`. For native use, load a
`gpui_charts::NativeFont` and retain a `ChartView` created from
`ChartInput::from_plot(&plot, font)`. Kit is optional. Core never scans system fonts,
starts a GPUI event loop or requires Python/browser objects.

## Run the examples

```sh
mise trust
mise install
mise run fmt
mise run check
mise run test
mise exec -- cargo run -p chart-gallery --locked
mise exec -- cargo run -p chart-export --example headless_authoring --locked
```

The [native gallery](examples/chart-gallery/src/main.rs) demonstrates named recipes,
nullable rows, exact timestamps, immutable edits, inspection and remounting. The
[publication example](crates/chart-export/examples/headless_authoring.rs) writes actual
SVG/PDF/PNG files. Native macOS development needs the selected full Xcode installation
in [ADR-001](docs/adr/001-host-dependency-and-toolchain.md).

Python and Node WASM provide ordinary columns/rows and component builders over Rust.
See [host examples](docs/authoring-guide.md#python-and-javascript) and run
`mise run primary-authoring-proof` with installed mypy, TypeScript, Node and matching
wasm-bindgen CLI 0.2.128. `mise run bindings-proof` retains the version-1 compatibility
fixtures. These are executable proof adapters; no wheel/npm/browser distribution is
implied. [Binding instructions](fixtures/bindings/README.md) describe tool setup.

## Status and contracts

The original WP-01–23 scope is complete in the local evidence ledger. Primary authoring
implementation and qualification are in progress; expanded D3/ggplot2 parity and G4
remain open. Packages are unpublished 0.1.0 with `publish = false`. The planned additive
migration release is 0.2.0; supported low-level Rust paths and version-1 wire envelopes
remain available. No package version bump or publication is part of the refactor.

- [Specification](docs/spec/gpui-charts-specification.md) owns behavior and acceptance.
- [Implementation status](docs/implementation-status.md) records current evidence and next actions.
- [Authoring plan](docs/impl_plans/primary-authoring-api-plan.md) and [ADR-013](docs/adr/013-primary-authoring-api.md) own the public API migration.
- [Release guide](docs/release-guide.md), [evidence index](docs/release-evidence.md) and [support matrix](docs/support-matrix.md) distinguish tested support from open gates.
- [Changelog](CHANGELOG.md) and [migration guide](docs/authoring-guide.md#migration-from-low-level-entry-points) describe compatibility.

`chart-core` owns shared semantics; `chart-export` owns publication; `gpui-charts` owns
native integration. Optional GPUI Kit examples live in the gallery. Standalone scales, colors, geometry,
inspection and resource protocols remain in their specialist modules. Rust is canonical
for all hosts. Read [AGENTS.md](AGENTS.md) and [AI development](docs/ai-development.md)
before contributing. Every new capability must extend a primary component or add a new
builder, with applicable bindings and acceptance evidence in the same work package.
