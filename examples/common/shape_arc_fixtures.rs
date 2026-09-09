//! Arc corners/padding and pie/donut layout through ordinary shared chart geometry.
use chart_core::{
    ChartResult,
    grammar::NumericAesthetic as A,
    prelude::*,
    shape::{ArcDatum, ArcParameters, PieAngles, PieOrder},
};
use std::f64::consts::{FRAC_PI_2, TAU};
pub fn figure() -> ChartResult<Plot> {
    let base = Data::columns()
        .column("x", [0., 8.])
        .column("y", [0., 8.])
        .build()?;
    let mut draft = plot(base)
        .aes(aes().x("x").y("y"))
        .x_axis(x_axis().scale(scale_linear().domain(0., 8.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 8.)).visible(false))
        .title(title(
            "Arcs and pies / corners, padding and source identity",
        ));
    let cases = [
        ("Quarter", 0., 48., 0., FRAC_PI_2, 0., 0., None),
        ("Annulus", 27., 48., 0., TAU, 0., 0., None),
        ("Rounded", 20., 48., 0.1, 2.3, 9., 0., None),
        ("Padded", 20., 48., 0., 1.2, 4., 0.18, Some(60.)),
        ("Reverse", 20., 48., 2.8, -1.2, 6., 0.1, None),
        ("Swapped radii", 48., 20., 0., 4.9, 5., 0., None),
        ("Tiny wedge", 20., 48., 0., 0.08, 20., 0., None),
        ("Signed inner", -10., 48., 0., 1.6, 0., 0., None),
    ];
    for (
        i,
        (
            name,
            inner_radius,
            outer_radius,
            start_angle,
            end_angle,
            corner_radius,
            pad_angle,
            pad_radius,
        ),
    ) in cases.into_iter().enumerate()
    {
        let x = (i % 4) as f64 * 2. + 1.;
        let y = 6.5 - (i / 4) as f64 * 2.3;
        let data = Data::columns()
            .name(format!("arc-{i}"))
            .column("x", [x])
            .column("y", [y])
            .build()?;
        draft = draft
            .layer(
                shape_arc()
                    .name(format!("arc-{i}"))
                    .data(data)
                    .arc_parameters(ArcParameters {
                        datum: ArcDatum {
                            inner_radius,
                            outer_radius,
                            start_angle,
                            end_angle,
                            pad_angle,
                        },
                        corner_radius,
                        pad_radius,
                    })
                    .color(chart_core::color::Paint::from_css("#2162a8")?),
            )
            .layer(
                labels()
                    .id(format!("arc-label-{i}"))
                    .at(x - 0.8, y + 1.1)
                    .text(name)
                    .style(text_style().size(0.95)),
            );
    }
    for (i, (name, inner, pad, corner, start, end, order)) in [
        ("Pie / source order", 0., 0., 0., 0., TAU, PieOrder::Input),
        (
            "Donut / descending",
            32.,
            0.06,
            7.,
            0.4,
            0.4 + TAU,
            PieOrder::ValuesDescending,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 2. + i as f64 * 4.;
        let data = Data::columns()
            .name(format!("pie-{i}"))
            .column("x", [x; 4])
            .column("y", [1.45; 4])
            .column("weight", [1., 2., 3., 2.])
            .column("slice", ["A", "B", "C", "D"])
            .keys([
                9007199254741001,
                9007199254741003,
                9007199254741005,
                9007199254741007,
            ])
            .build()?;
        draft = draft
            .layer(
                shape_pie()
                    .name(format!("pie-{i}"))
                    .data(data)
                    .aes(aes().color("slice").color_scale("slices"))
                    .pie_order(order)
                    .pie_angles(PieAngles {
                        start_angle: start,
                        end_angle: end,
                        pad_angle: pad,
                    })
                    .shape_value(A::PieValue, "weight")
                    .shape_value(A::InnerRadius, inner)
                    .shape_value(A::OuterRadius, 65.)
                    .shape_value(A::CornerRadius, corner),
            )
            .layer(
                labels()
                    .id(format!("pie-label-{i}"))
                    .at(x - 1., 2.75)
                    .text(name)
                    .style(text_style().size(0.95)),
            );
    }
    draft
        .scale(
            color_discrete("slices").palette(
                ["#2162a8", "#5b9cce", "#e4ad4b", "#b55037"]
                    .into_iter()
                    .map(chart_core::color::Paint::from_css)
                    .collect::<ChartResult<Vec<_>>>()?,
            ),
        )
        .legend(legend().scale("slices").title("Source slice"))
        .build()
}
