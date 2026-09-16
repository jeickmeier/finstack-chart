//! GG07 interval, step, reference and arrow authoring publications.
use chart_core::{
    grammar::{
        ArrowEnds, ArrowSpec, BuiltinRecipe, IntervalKind, IntervalPoint, IntervalRecipe,
        IntervalStroke, LineType, RecipeAesthetic, StepDirection,
    },
    prelude::*,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., -1., 1.])
        .column("lo", [1., -2., 0.])
        .column("hi", [3., 0., 2.])
        .build()?;
    let mut b = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"));
    if mode < 8 {
        let kind = [
            IntervalKind::LineRange,
            IntervalKind::PointRange,
            IntervalKind::ErrorBar,
            IntervalKind::Crossbar,
        ][mode % 4];
        let mut layer = rule()
            .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                kind,
                width: Some(0.5),
                ..Default::default()
            }))
            .recipe_value(RecipeAesthetic::Lower, "lo")
            .recipe_value(RecipeAesthetic::Upper, "hi");
        if mode >= 4 {
            b = b.aes(aes().x("y").y("x"));
            layer = layer.orientation(Orientation::Horizontal);
        }
        b = b.layer(layer);
    } else if mode < 11 {
        b = b.layer(step(
            [StepDirection::Hv, StepDirection::Vh, StepDirection::Mid][mode - 8],
        ));
    } else if mode == 11 {
        b = b.layer(segment().aes(aes().x("x").y("y").x2(3.5).y2(3.)).recipe(
            BuiltinRecipe::Segment {
                arrow: Some(ArrowSpec {
                    ends: ArrowEnds::Both,
                    length_mm: 3.,
                    closed: true,
                    ..Default::default()
                }),
            },
        ));
    } else if mode == 15 {
        b = b.layer(
            crossbar()
                .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                    kind: IntervalKind::Crossbar,
                    middle: IntervalStroke {
                        color: Some(rgb(255, 0, 0).into()),
                        linewidth: Some(2.),
                        line_type: Some(LineType::Dashed),
                    },
                    box_style: IntervalStroke {
                        color: Some(rgb(0, 0, 255).into()),
                        linewidth: Some(0.25),
                        line_type: Some(LineType::Dotted),
                    },
                    ..Default::default()
                }))
                .recipe_value(RecipeAesthetic::Lower, "lo")
                .recipe_value(RecipeAesthetic::Upper, "hi")
                .fill(rgb(255, 215, 0))
                .alpha(0.2),
        );
    } else if mode == 16 {
        b = b.layer(
            pointrange()
                .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                    kind: IntervalKind::PointRange,
                    fatten: Some(2.),
                    point: IntervalPoint {
                        size: Some(1.),
                        stroke: Some(2.),
                        shape: Some(21),
                        fill: Some(rgb(255, 215, 0).into()),
                    },
                    ..Default::default()
                }))
                .recipe_value(RecipeAesthetic::Lower, "lo")
                .recipe_value(RecipeAesthetic::Upper, "hi")
                .linewidth(0.25),
        );
    } else {
        b = b.layer(points()).layer(match mode {
            12 => abline(0.5, 0.),
            13 => hline(0.5),
            _ => vline(2.),
        });
    }
    b.x_axis(x_axis().visible(false))
        .y_axis(y_axis().visible(false))
        .build()
}
