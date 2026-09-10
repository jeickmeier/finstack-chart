//! AXIS-01 noninjective custom projection, shared marks and independently placed guides.
use chart_core::{ChartResult, Revision, composition::ScaleValue, layout::AxisSide, prelude::*};
pub fn figure() -> ChartResult<Plot> {
    let data = Data::columns()
        .name("source")
        .keys([
            9007199254741001,
            9007199254741002,
            9007199254741003,
            9007199254741004,
        ])
        .column("x", [-10., -2., 2., 10.])
        .column("y", [0., 1., 2., 3.])
        .build()?;
    let guide = |name: &str, side, dx, dy| {
        axis_guide(name, "x")
            .side(side)
            .translate(dx, dy)
            .ticks([0., 2., 10.].map(|v| (ScaleValue::Number(v), format!("{name} {v}"))))
    };
    plot(data)
        .extensions(chart_extension_example::registry()?)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .coordinate_scale(scale_registered(
                    "example.fold",
                    Revision::new(1),
                    serde_json::json!({"limit":10.}),
                ))
                .range(100., 500.)
                .visible(false),
        )
        .y_axis(y_axis().range(300., 100.).visible(false))
        .guide(guide("top", AxisSide::Top, 0., 0.))
        .guide(guide("lower", AxisSide::Bottom, 10., 30.))
        .title(title("Folded values / one scale / two guides"))
        .build()
}
