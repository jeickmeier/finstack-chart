//! Primary components for the compiled example extension. Applications author no protocol IDs.
use chart_core::{
    ChartResult, Revision,
    grammar::{StatField, StatNumeric},
    plot::*,
};

/// Registered density statistic over an owner-checked field or source name.
pub fn density_histogram(field: impl Into<Mapping>, edges: Vec<f64>) -> StatBuilder {
    custom_stat(
        super::HISTOGRAM,
        Revision::new(1),
        serde_json::json!({"edges":edges}),
    )
    .field_parameter("input", field)
}
/// Chamfered density rectangles with explicit generated mappings and a portable/native capability.
pub fn chamfered_bars(native: bool) -> LayerBuilder {
    rectangle()
        .geometry(
            if native {
                super::NATIVE_BARS
            } else {
                super::BARS
            },
            Revision::new(1),
            serde_json::Value::Null,
        )
        .after_stat(
            stat_aes()
                .x(StatField::Custom("left".into()))
                .y(StatNumeric::Literal(0.))
                .x2(StatField::Custom("right".into()))
                .y2(StatField::Custom("density".into())),
        )
}
/// One shared registered transform and two consumers through the primary API.
pub fn density_plot(data: Data, field: &str, edges: Vec<f64>, native: bool) -> ChartResult<Plot> {
    let density = transform("density", density_histogram(data.field(field)?, edges));
    let bars = chamfered_bars(native)
        .name("density-bars")
        .from_transform(density.handle()?);
    let markers = points()
        .name("density-points")
        .from_transform(density.handle()?)
        .after_stat(
            stat_aes()
                .x(StatField::Custom("left".into()))
                .y(StatField::Custom("density".into())),
        );
    plot(data)
        .extensions(super::registry()?)
        .transform(density)
        .layer(bars)
        .layer(markers)
        .build()
}
