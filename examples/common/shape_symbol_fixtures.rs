//! All built-in symbol geometries at multiple sizes and mapped guide glyphs.
use chart_core::{
    ChartResult,
    grammar::NumericAesthetic as A,
    prelude::*,
    shape::{SymbolKind as S, SymbolPaint},
    theme::NamedTheme,
};
pub fn figure(preset: NamedTheme) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [0., 5.])
        .column("y", [0., 15.])
        .build()?;
    let mut draft = plot(data)
        .aes(aes().x("x").y("y"))
        .theme(theme().preset(preset))
        .x_axis(x_axis().scale(scale_linear().domain(0., 5.)).visible(false))
        .y_axis(
            y_axis()
                .scale(scale_linear().domain(0., 15.))
                .visible(false),
        )
        .title(title(format!(
            "Symbols / area and stroke size / {preset:?}"
        )));
    let kinds = [
        S::Circle,
        S::Cross,
        S::Diamond,
        S::Square,
        S::Star,
        S::Triangle,
        S::Wye,
        S::Plus,
        S::Times,
        S::Asterisk,
        S::Diamond2,
        S::Square2,
        S::Triangle2,
    ];
    for (i, kind) in kinds.into_iter().enumerate() {
        let y = 13.5 - i as f64;
        let d = Data::columns()
            .name(format!("symbol-{i}"))
            .column("x", [2., 3., 4.])
            .column("y", [y, y, y])
            .column("size", [16., 64., 256.])
            .keys([
                9007199254741001 + i as u64 * 10,
                9007199254741003 + i as u64 * 10,
                9007199254741005 + i as u64 * 10,
            ])
            .build()?;
        draft = draft
            .layer(
                shape_symbol()
                    .name(format!("symbol-{i}"))
                    .data(d)
                    .symbol_kind(kind)
                    .symbol_paint(if i < 7 {
                        SymbolPaint::Fill
                    } else {
                        SymbolPaint::Stroke
                    })
                    .shape_value(A::AreaSize, "size")
                    .color(chart_core::color::Paint::from_css(if i < 7 {
                        "#2162a8"
                    } else {
                        "#b55037"
                    })?),
            )
            .layer(
                labels()
                    .id(format!("symbol-label-{i}"))
                    .at(0.1, y)
                    .text(format!("{kind:?}"))
                    .style(text_style().size(0.85)),
            );
    }
    for (i, label) in ["16", "64", "256"].into_iter().enumerate() {
        draft = draft.layer(
            labels()
                .id(format!("size-{i}"))
                .at(2. + i as f64, 14.4)
                .text(label)
                .style(text_style().size(0.85)),
        );
    }
    let d = Data::columns()
        .name("mapped")
        .column("x", [2., 3., 4.])
        .column("y", [0.4, 0.4, 0.4])
        .column("size", [16., 64., 256.])
        .column("kind", ["A", "B", "C"])
        .build()?;
    draft = draft
        .layer(
            shape_symbol()
                .name("mapped")
                .data(d)
                .symbol_types(
                    "kind",
                    vec!["A".into(), "B".into(), "C".into()],
                    vec![S::Circle, S::Square, S::Plus],
                )
                .symbol_title("Type")
                .shape_value(A::AreaSize, "size")
                .symbol_size_guide("Area", vec![16., 64., 256.])
                .color(chart_core::color::Paint::from_css("#2162a8")?),
        )
        .layer(
            labels()
                .id("mapped-label")
                .at(0.1, 0.4)
                .text("Mapped")
                .style(text_style().size(0.85)),
        );
    draft.build()
}
