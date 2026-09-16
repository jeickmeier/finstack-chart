//! GG07 independent cap/join authors with translucent solid and dashed strokes.
use chart_core::{
    grammar::{AestheticUnits, LineType},
    prelude::*,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 2.1, 3.])
        .column("y", [1., 3., 1., 2.])
        .build()?;
    let ends = [LineEnd::Butt, LineEnd::Round, LineEnd::Square];
    let joins = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];
    let layer = line()
        .linewidth(8.)
        .aesthetic_units(AestheticUnits::Points)
        .alpha(0.5)
        .lineend(if mode < 3 { ends[mode] } else { LineEnd::Butt })
        .linejoin(if mode < 3 {
            LineJoin::Miter
        } else {
            joins[mode - 3]
        })
        .line_type(if mode < 3 {
            LineType::Dashed
        } else {
            LineType::Solid
        });
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .x_axis(x_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .build()
}
