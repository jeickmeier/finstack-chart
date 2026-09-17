//! Shared native/publication gallery: five offsets, sparse tidy cells, bars and areas.
use chart_core::plot::{shape_area, shape_stack};
use chart_core::{
    ChartResult,
    grammar::GroupValue,
    prelude::*,
    shape::{StackMissing, StackOffset, StackOrder},
    theme::NamedTheme,
};
pub fn figure(preset: NamedTheme) -> ChartResult<Plot> {
    let offsets = [
        StackOffset::None,
        StackOffset::Expand,
        StackOffset::Diverging,
        StackOffset::Silhouette,
        StackOffset::Wiggle,
    ];
    let mut x = vec![];
    let mut y = vec![];
    let mut groups = vec![];
    let mut names = vec![];
    let mut forms = vec![];
    let mut slots = vec![];
    let mut keys = vec![];
    for (oi, offset) in offsets.iter().enumerate() {
        for (kind, form) in ["Area", "Bars"].into_iter().enumerate() {
            for sample in 0..6 {
                for (g, name) in ["A", "B", "C"].into_iter().enumerate() {
                    if sample == 2 && g == 1 {
                        continue;
                    }
                    x.push(sample as f64);
                    y.push(if sample == 3 && g == 2 {
                        -2.
                    } else {
                        ((sample + 2 * g) % 4 + 1) as f64
                    });
                    groups.push(name.to_string());
                    names.push(format!("{offset:?}"));
                    forms.push(form.to_string());
                    slots.push((2 * oi + kind) as i64);
                    keys.push(
                        9_007_199_254_741_001 + (oi * 100 + kind * 30 + sample * 3 + g) as u64,
                    );
                }
            }
        }
    }
    let data = Data::columns()
        .name("stacks")
        .column("x", x)
        .column("y", y)
        .column("group", categorical(groups))
        .column("offset", categorical(names))
        .column("form", categorical(forms))
        .column("slot", slots)
        .keys(keys)
        .build()?;
    let mut draft = plot(data)
        .aes(
            aes()
                .x("x")
                .x2("x")
                .y("y")
                .y2(0.)
                .group("group")
                .color("group"),
        )
        .theme(theme().preset(preset))
        .title(title(format!("Stack offsets / InsideOut / {preset:?}")))
        .subtitle(subtitle(
            "Signed heights, a missing sample, and explicit zero filling",
        ))
        .facet(facet_grid("offset", "form").free_y(true).gap(12.))
        .x_axis(x_axis().scale(scale_linear().domain(-0.5, 5.5)))
        .scale(color_discrete("group").domain(["A", "B", "C"]))
        .legend(legend().scale("group").title("Series"));
    for (oi, offset) in offsets.into_iter().enumerate() {
        for kind in 0..2 {
            let layer = if kind == 0 {
                shape_area()
            } else {
                bars().width(15.)
            };
            let slot = (2 * oi + kind) as f64;
            draft = draft.layer(
                layer
                    .name(format!("{offset:?}-{kind}"))
                    .filter(filter("slot").minimum(slot).maximum(slot))
                    .position(
                        shape_stack(["A", "B", "C"].map(|v| GroupValue::Text(v.into())).to_vec())
                            .stack_order(StackOrder::InsideOut)
                            .stack_offset(offset)
                            .stack_missing(StackMissing::Zero),
                    ),
            );
        }
    }
    draft.build()
}
