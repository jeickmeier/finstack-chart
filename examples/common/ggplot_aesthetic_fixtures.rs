//! FIX-GG03: independent Rust author of the R glyph publication fixture.
use chart_core::plot::shape_symbol;
use chart_core::{ChartResult, prelude::*, shape::SymbolKind};

pub fn figure() -> ChartResult<Plot> {
    let data = Data::columns()
        .keys([9007199254741001, 9007199254741003])
        .column("x", [1., 3.])
        .column("y", [2., 2.])
        .column("inside", categorical(["A", "B"]))
        .column("outside", categorical(["B", "A"]))
        .column("area", [1., 4.])
        .column("alpha", [0.5, 1.])
        .build()?;
    let mut gallery = plot(data)
        .aes(aes().x("x").y("y"))
        .x_axis(x_axis().scale(scale_linear().domain(-0.5, 6.5)))
        .y_axis(y_axis().scale(scale_linear().domain(-0.5, 3.5)));
    for code in 0..26_u8 {
        let data = Data::columns()
            .name(format!("glyph-{code}"))
            .keys([9007199254741001 + u64::from(code)])
            .column("x", [f64::from(code % 7)])
            .column("y", [f64::from(3 - code / 7)])
            .build()?;
        gallery = gallery.layer(
            shape_symbol()
                .data(data)
                .symbol_kind(SymbolKind::Ggplot(code))
                .symbol_size(100.)
                .fill(rgb(255, 0, 0))
                .stroke(rgb(0, 0, 255))
                .linewidth(1.),
        );
    }
    gallery
        .title(title(
            "R point glyphs 0–25: independent red fill and blue outline",
        ))
        .build()
}
