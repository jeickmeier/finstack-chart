//! AXIS-06 shared immutable authoring inputs for native and publication proofs.
use chart_core::{
    ChartResult,
    layout::{GuideFormatter, GuideProfile},
    plot::{AxisBuilder, Plot},
    prelude::*,
};
fn axis(domain: f64, values: &[f64], labels: &[&str]) -> AxisBuilder {
    x_axis()
        .scale(scale_linear().domain(0., domain))
        .range(50., 450.)
        .guide_profile(GuideProfile::D3_3_0_0)
        .tick_values(Some(values.iter().copied().map(Into::into).collect()))
        .tick_format(Some(GuideFormatter::Labels(
            labels.iter().map(|s| (*s).into()).collect(),
        )))
}
pub fn sequence() -> ChartResult<[Plot; 3]> {
    let a = plot(
        Data::columns()
            .column("x", [0., 0.5, 1.])
            .column("y", [0., 1., 0.])
            .build()?,
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(axis(1., &[0., 0.5, 1.], &["zero", "half", "one"]))
    .y_axis(y_axis().visible(false))
    .build()?;
    let b = a
        .edit()
        .x_axis(axis(2., &[0., 1., 2.], &["ZERO", "ONE", "TWO"]))
        .build()?;
    let c = b
        .edit()
        .x_axis(axis(
            4.,
            &[0., 2., 4.],
            &["zero again", "two again", "four"],
        ))
        .build()?;
    Ok([a, b, c])
}
