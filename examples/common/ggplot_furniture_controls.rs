//! GG14 figure title/caption alignment and theme-driven tags through primary authors.
use chart_core::{ChartResult, prelude::*, theme::*};
pub const CASES: usize = 8;
pub fn author(mode: usize) -> ChartResult<Plot> {
    let mut elements = ElementTheme::preset(ThemePreset::Grey)?;
    let position = if mode == 1 { "plot" } else { "panel" };
    for name in ["plot.title.position", "plot.caption.position"] {
        elements = elements.element(name, ThemeEntry::Value(ThemeValue::Text(position.into())))?;
    }
    let (location, tag) = match mode {
        2 => ("margin", "topleft"),
        3 => ("margin", "bottomright"),
        4 => ("panel", "bottomleft"),
        5 => ("panel", "top"),
        6 => ("plot", "topright"),
        _ => ("plot", "topright"),
    };
    elements = elements
        .element(
            "plot.tag.location",
            ThemeEntry::Value(ThemeValue::Text(location.into())),
        )?
        .element(
            "plot.tag.position",
            ThemeEntry::Value(if mode == 5 {
                ThemeValue::Vector(vec![ThemeValue::Number(0.5), ThemeValue::Number(0.5)])
            } else {
                ThemeValue::Text(tag.into())
            }),
        )?;
    if mode == 7 {
        elements = elements.element("plot.tag", ThemeEntry::Blank)?;
    }
    let data = Data::columns()
        .column("x", [0., 1., 2., 3.])
        .column("y", [1., 3., 2., 4.])
        .column("group", ["a", "a", "b", "b"])
        .build()?;
    let mut builder = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .theme(theme().elements(elements)?)
        .title(title("Figure alignment"))
        .subtitle(subtitle("Explicit panel or plot span"))
        .caption(caption("Caption aligned to the same selected span"))
        .tag(rich_text("A"));
    if mode == 6 {
        builder = builder.facet(facet_wrap("group"));
    }
    builder.build()
}
