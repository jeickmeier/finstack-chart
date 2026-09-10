//! AXIS-02 primary authoring: independent values, labels and registered formatters.
use chart_core::{
    ChartResult, Revision,
    grammar::OperationRef,
    layout::{AxisSide, GuideFormatter, GuideProfile},
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
                .range(100., 500.)
                .guide_profile(GuideProfile::D3_3_0_0)
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
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some([0., 0.5, 1.].map(Into::into).to_vec()))
                .tick_format(Some(formatter("Indexed"))),
        )
        .guide(
            axis_guide("lower", "x")
                .side(AxisSide::Bottom)
                .translate(0., 32.)
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some([0., 0.5, 1.].map(Into::into).to_vec()))
                .tick_format(Some(formatter("Same"))),
        )
        .title(title("Independent values and labels"))
        .build()
}
