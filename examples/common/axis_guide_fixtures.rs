//! AXIS-01 shared positional mapping with independent guide names and placement.
use chart_core::{ChartResult, composition::ScaleValue, layout::AxisSide, prelude::*};

pub fn figure() -> ChartResult<Plot> {
    let data = Data::columns()
        .name("observations")
        .keys([9007199254741001, 9007199254741002, 9007199254741003])
        .column("x", [2., 5., 8.])
        .column("y", [1., 3., 2.])
        .build()?;
    let guide = |name: &str, side, dx, dy| {
        axis_guide(name, "x")
            .side(side)
            .translate(dx, dy)
            .ticks([2., 5., 8.].map(|value| (ScaleValue::Number(value), format!("{name} {value}"))))
    };
    plot(data)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points().size(4.))
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 10.))
                .visible(false),
        )
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)))
        .guide(guide("top", AxisSide::Top, 0., 0.))
        .guide(guide("lower", AxisSide::Bottom, 0., 0.))
        .guide(guide("shifted", AxisSide::Bottom, 15., 32.))
        .title(title("One positional scale / three independent guides"))
        .build()
}
