//! Cross-host registered protocol gallery over one explicit versioned registry.
use chart_core::plot::{
    shape_arc, shape_area, shape_area_radial, shape_line, shape_line_radial, shape_link_horizontal,
    shape_pie, shape_stack, shape_symbol,
};
use chart_core::{
    ChartResult, Revision,
    grammar::{GroupValue, NumericAesthetic as A, OperationRef, ShapeFamily as F, ShapeOperation},
    prelude::*,
    theme::NamedTheme,
};
fn op(name: &str, parameters: serde_json::Value) -> ShapeOperation {
    ShapeOperation {
        operation: OperationRef::new(format!("example.{name}"), Revision::new(1)),
        parameters,
    }
}
pub fn figure(preset: NamedTheme) -> ChartResult<Plot> {
    let labels = [
        "Shifted line",
        "Gapped area",
        "Custom links",
        "Radial curve",
        "Radial area",
        "Rectangle symbols",
        "Ordered pie",
        "Custom stack bars",
        "Custom stack area",
        "Shared arc engine",
    ];
    let (mut x, mut y, mut x2, mut y2) = (vec![], vec![], vec![], vec![]);
    let (mut angle, mut radius, mut area, mut value) = (vec![], vec![], vec![], vec![]);
    let (mut groups, mut panels, mut slots, mut keys) = (vec![], vec![], vec![], vec![]);
    for (slot, label) in labels.iter().enumerate() {
        let n = match slot {
            2 | 5 | 6 => 3,
            3 | 4 => 9,
            7 | 8 => 6,
            9 => 1,
            _ => 7,
        };
        for i in 0..n {
            let stack = slot == 7 || slot == 8;
            x.push(if stack {
                0.75 + (i / 2) as f64 * 1.2
            } else if slot == 0 || slot == 1 {
                0.5 + i as f64 * 0.5
            } else if slot == 2 {
                0.5
            } else if slot == 5 {
                0.8 + i as f64 * 1.2
            } else {
                2.
            });
            let height = if stack {
                if i % 2 == 0 { 1. } else { 0.6 }
            } else if slot == 0 || slot == 1 {
                1. + (i % 3) as f64 * 0.5
            } else if slot == 2 {
                0.75 + i as f64 * 0.75
            } else {
                2.
            };
            y.push(if slot == 1 && i == 3 {
                None
            } else {
                Some(height + if stack { (i / 2) as f64 * 0.1 } else { 0. })
            });
            x2.push(if slot == 2 {
                3.5
            } else {
                *x.last().expect("x")
            });
            y2.push(if stack {
                0.
            } else if slot == 2 {
                3.25 - i as f64 * 0.75
            } else {
                0.5
            });
            angle.push(i as f64 * std::f64::consts::PI / 4.);
            radius.push(24. + (i % 3) as f64 * 6.);
            area.push([64., 144., 256.][i % 3]);
            value.push((i + 1) as f64);
            groups.push(
                if stack {
                    if i % 2 == 0 { "A" } else { "B" }
                } else if slot == 2 || slot == 5 || slot == 6 {
                    ["A", "B", "C"][i]
                } else {
                    "A"
                }
                .to_string(),
            );
            panels.push((*label).to_string());
            slots.push(slot as i64);
            keys.push(
                9007199254741001
                    + slot as u64 * 100
                    + if slot == 6 {
                        (n - 1 - i) as u64
                    } else {
                        i as u64
                    },
            );
        }
    }
    let data = Data::columns()
        .name("custom-shapes")
        .column("x", x)
        .column("y", y)
        .column("x2", x2)
        .column("y2", y2)
        .column("angle", angle)
        .column("radius", radius)
        .column("area", area)
        .column("value", value)
        .column("group", categorical(groups))
        .column("panel", categorical(panels))
        .column("slot", slots)
        .keys(keys)
        .build()?;
    let mut draft = plot(data.clone())
        .extensions(chart_extension_example::registry()?)
        .aes(
            aes()
                .x("x")
                .y("y")
                .x2("x2")
                .y2("y2")
                .group("group")
                .color("group"),
        )
        .theme(theme().preset(preset))
        .title(title(format!("Registered shapes / {preset:?}")))
        .subtitle(subtitle(
            "Custom curves, symbols, comparisons and stack layouts",
        ))
        .facet(facet_wrap("panel").columns(2).gap(12.))
        .x_axis(x_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .scale(color_discrete("group").domain(["A", "B", "C"]))
        .legend(legend().scale("group").title("Source"));
    for (slot, label) in labels.into_iter().enumerate() {
        let mut layer = match slot {
            0 => shape_line(),
            1 => shape_area(),
            2 => shape_link_horizontal(),
            3 => shape_line_radial()
                .shape_value(A::Angle, data.field("angle")?)
                .shape_value(A::Radius, data.field("radius")?),
            4 => shape_area_radial()
                .shape_value(A::Angle, data.field("angle")?)
                .shape_value(A::InnerRadius, 18.)
                .shape_value(A::OuterRadius, 40.),
            5 => shape_symbol()
                .shape_value(A::AreaSize, data.field("area")?)
                .symbol_size_guide("Area", vec![64., 144., 256.])
                .shape_protocol(
                    F::Symbol,
                    op("rectangle_symbol", serde_json::json!({"amount":4})),
                ),
            6 => shape_pie()
                .pie_grouped(false)
                .shape_value(A::PieValue, data.field("value")?)
                .shape_value(A::OuterRadius, 40.)
                .shape_protocol(
                    F::PieComparator,
                    op("field_comparator", serde_json::json!({"field":"key"})),
                ),
            7 | 8 => (if slot == 7 { bars() } else { shape_area() })
                .position(shape_stack(vec![
                    GroupValue::Text("A".into()),
                    GroupValue::Text("B".into()),
                ]))
                .shape_protocol(
                    F::StackOrder,
                    op("first_value_order", serde_json::json!({})),
                )
                .shape_protocol(
                    F::StackOffset,
                    op("shift_offset", serde_json::json!({"amount":0.25})),
                ),
            _ => shape_arc()
                .shape_value(A::InnerRadius, 20.)
                .shape_value(A::OuterRadius, 40.)
                .shape_value(A::EndAngle, std::f64::consts::PI * 1.5),
        };
        if slot < 5 {
            layer =
                layer.shape_protocol(F::Curve, op("shift_curve", serde_json::json!({"amount":5})));
        }
        draft = draft.layer(
            layer
                .name(label)
                .filter(filter("slot").minimum(slot as f64).maximum(slot as f64)),
        );
    }
    draft.build()
}
