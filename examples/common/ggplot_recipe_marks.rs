//! Independent GG07 count, column, rug, grid curve and signed spoke authors.
use chart_core::{grammar::*, prelude::*};
pub fn author(mode: usize) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 1., 2., 3.])
        .column("y", [1., 1., -1., 2.])
        .column("end_x", [3., 3., 4., 4.])
        .column("end_y", [2., 2., 2., 0.])
        .column(
            "angle",
            [0., 0., std::f64::consts::FRAC_PI_2, std::f64::consts::PI],
        )
        .column("radius", [1., 1., -1., 2.])
        .column("w", [1., 2., 1., 4.])
        .build()?;
    let layer = match mode {
        0 => points()
            .recipe(BuiltinRecipe::Count(CountRecipe::default()))
            .stat(count().sum_count().x("x").y("y").count_weight("w")),
        1 => rectangle().recipe(BuiltinRecipe::Column(ColumnRecipe {
            width: Some(0.6),
            just: 0.5,
        })),
        2 => points().recipe(BuiltinRecipe::Rug(RugRecipe {
            sides: "bltr".into(),
            length: 0.04,
            outside: false,
        })),
        3 => rule()
            .aes(aes().x("x").y("y").x2("end_x").y2("end_y"))
            .recipe(BuiltinRecipe::Curve(CurveRecipe {
                curvature: 0.4,
                angle: 60.,
                ncp: 5,
                arrow: Some(ArrowSpec {
                    length_mm: 3.,
                    closed: true,
                    ..Default::default()
                }),
            })),
        _ => rule()
            .recipe(BuiltinRecipe::Spoke(SpokeRecipe::default()))
            .recipe_value(RecipeAesthetic::Angle, "angle")
            .recipe_value(RecipeAesthetic::Radius, "radius"),
    };
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .x_axis(x_axis().scale(scale_linear().domain(0., 5.)))
        .y_axis(y_axis().scale(scale_linear().domain(-2., 4.)))
        .build()
}
