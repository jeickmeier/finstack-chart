//! FIX-GG06 shared native/publication fixture authors; hosts author these independently.
use chart_core::{
    grammar::{DodgePreserve, Profile},
    prelude::*,
};
pub fn author(mode: usize) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 1., 2.])
        .column("y", [2., 3., -1.])
        .column("g", ["a", "b", "a"])
        .column("lo", [0.6, 0.8, 1.6])
        .column("hi", [1.4, 1.2, 2.4])
        .column("panel", ["one", "one", "two"])
        .build()?;
    let position = match mode {
        0 => ggplot_stack().vjust(0.5),
        1 => ggplot_fill().reverse(true),
        2 => ggplot_dodge().width(0.8).preserve(DodgePreserve::Single),
        3 => dodge2().width(0.8).padding(0.2),
        4 => nudge(0.2, -0.1),
        5 => jitter_dodge(42).displacement(0.2, 0.1),
        6 => ggplot_stack().reverse(true),
        _ => dodge2()
            .width(0.8)
            .reverse(true)
            .preserve(DodgePreserve::Single),
    };
    let interval = matches!(mode, 1 | 3 | 6 | 7);
    let mapping = if matches!(mode, 1 | 6) {
        aes().x(0.6).x2(1.4).y("y").y2(0.).group("g")
    } else if interval {
        aes().x("lo").x2("hi").y("y").y2(0.).group("g")
    } else {
        aes().x("x").y("y").group("g")
    };
    let mut builder = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(mapping)
        .layer(if interval {
            rectangle().position(position)
        } else {
            points().position(position)
        });
    if mode == 7 {
        builder = builder.facet(facet_wrap("panel"));
    }
    builder.build()
}
pub fn name(mode: usize) -> &'static str {
    match mode {
        0 => "Stack anchors",
        1 => "Signed fill",
        2 => "Dodge single",
        3 => "Variable-width dodge2",
        4 => "Nudge",
        5 => "Keyed jitter-dodge",
        6 => "Signed stack",
        _ => "Dodge2 unequal facets",
    }
}
