//! AXIS-04 signed geometry and independently translated guides.
use chart_core::{
    ChartResult, Revision,
    grammar::OperationRef,
    layout::{AxisSide, GuideFormatter, GuideGeometry, GuideProfile},
    prelude::*,
};
fn formatter(mode: &str) -> GuideFormatter {
    GuideFormatter::Registered {
        operation: OperationRef::new("example.guide_format", Revision::new(1)),
        parameters: serde_json::json!({"mode":mode}),
    }
}
pub fn figure() -> ChartResult<Plot> {
    let data = Data::columns()
        .name("ticks")
        .keys((0..5).map(|i| 9007199254741001 + i))
        .column("x", [0., 0.25, 0.5, 0.75, 1.])
        .column("y", [1., 2., 3., 2., 1.])
        .build()?;
    plot(data)
        .extensions(chart_extension_example::registry()?)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 1.))
                .range(500., 100.)
                .guide_profile(GuideProfile::D3_3_0_0)
                .guide_geometry(Some(GuideGeometry {
                    inner: Some(-180.),
                    outer: Some(12.),
                    padding: Some(8.),
                    clip_ticks: true,
                    ..Default::default()
                }))
                .tick_values(Some(
                    [0., 0.25, 0.5, 0.5, 0.75, 1.].map(Into::into).to_vec(),
                ))
                .tick_format(Some(GuideFormatter::Labels(
                    ["zero", "", "mid", "", "", "one"].map(Into::into).to_vec(),
                ))),
        )
        .y_axis(y_axis().range(280., 100.).visible(false))
        .guide(
            axis_guide("top", "x")
                .side(AxisSide::Top)
                .tick_size_inner(-18.)
                .tick_size_outer(-12.)
                .tick_padding(-8.)
                .tick_offset(Some(0.))
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some([0., 0.5, 1.].map(Into::into).to_vec()))
                .tick_format(Some(formatter("Indexed"))),
        )
        .guide(
            axis_guide("lower", "x")
                .side(AxisSide::Bottom)
                .translate(15., 40.)
                .tick_size(0.)
                .tick_padding(3.)
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some([0., 0.5, 1.].map(Into::into).to_vec()))
                .tick_format(Some(formatter("Same"))),
        )
        .title(title("Signed ticks and reversed ranges"))
        .build()
}
