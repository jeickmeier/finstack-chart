//! AXIS-05 independent domain, tick and label styles.
use chart_core::{
    ChartResult, Revision,
    grammar::OperationRef,
    layout::{AxisSide, GuideComponents, GuideFormatter, GuideGeometry, GuideProfile},
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
                .guide_components(Some(serde_json::from_value::<GuideComponents>(serde_json::json!({
                    "domain":{"color":"#173f66","width":2.,"dashes":[8.,3.]},
                    "ticks":{"color":"#77889999","width":1.5},
                    "labels":{"color":"#174c75","font_size":12.},
                    "per_tick":[{"index":2,"label":{"color":"#cc4400","font_size":18.}}, {"index":3,"line":{"visible":false}}, {"index":4,"line":{"color":"#cc4400","dashes":[2.,3.]}}]
                })).expect("static component fixture")))
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
                .guide_components(Some(serde_json::from_value::<GuideComponents>(serde_json::json!({"domain":{"visible":false},"ticks":{"color":"#228844"},"labels":{"color":"#225533","font_size":11.}})).expect("static component fixture")))
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
                .guide_components(Some(serde_json::from_value::<GuideComponents>(serde_json::json!({"domain":{"color":"#228844","width":2.},"labels":{"color":"#225533","font_size":12.},"per_tick":[{"index":2,"label":{"visible":false}}]})).expect("static component fixture")))
                .tick_size(0.)
                .tick_padding(3.)
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some([0., 0.5, 1.].map(Into::into).to_vec()))
                .tick_format(Some(formatter("Same"))),
        )
        .title(title("Independent domain, tick and label styles"))
        .build()
}

/// Independently sized and rotated labels through the existing supplied-font shaper.
pub fn typography() -> ChartResult<Plot> {
    let components:GuideComponents=serde_json::from_value(serde_json::json!({"labels":{"font_size":12.,"typography":{"text":"ignored","size":1.25}},"per_tick":[{"index":1,"label":{"font_size":18.,"rotation":30.,"typography":{"text":"ignored","size":1.,"weight":400,"tabular":true}}}]})).expect("static typography fixture");
    plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [0., 1.])
            .build()?,
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .guide_profile(GuideProfile::D3_3_0_0)
            .tick_values(Some(vec![0f64.into(), 0.5.into(), 1f64.into()]))
            .guide_components(Some(components)),
    )
    .y_axis(y_axis().visible(false))
    .build()
}
