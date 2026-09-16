//! GG07 independent source/statistic polygon, tile and row-grid raster authors.
use chart_core::{
    grammar::{BuiltinRecipe, PolygonRecipe, RasterRecipe, RecipeAesthetic, StatField, TileRecipe},
    prelude::*,
    scene::FillRule,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let mut b = if mode < 4 {
        let reverse = mode % 2 == 1;
        let data = Data::columns()
            .column("x", vec![0., 4., 4., 0., 1., 3., 3., 1.])
            .column(
                "y",
                if reverse {
                    vec![0., 0., 4., 4., 3., 3., 1., 1.]
                } else {
                    vec![0., 0., 4., 4., 1., 1., 3., 3.]
                },
            )
            .column(
                "sub",
                vec![
                    "outer", "outer", "outer", "outer", "inner", "inner", "inner", "inner",
                ],
            )
            .build()?;
        plot(data).aes(aes().x("x").y("y")).layer(
            points()
                .recipe(BuiltinRecipe::Polygon(PolygonRecipe {
                    rule: if mode < 2 {
                        FillRule::EvenOdd
                    } else {
                        FillRule::NonZero
                    },
                }))
                .recipe_value(RecipeAesthetic::Subgroup, "sub"),
        )
    } else if mode == 8 {
        let data = Data::columns().column("g", ["A", "A", "B"]).build()?;
        plot(data).layer(
            points()
                .stat(count().group("g"))
                .after_stat(stat_aes().x(StatField::Group).y(StatField::Count))
                .recipe(BuiltinRecipe::Tile(TileRecipe {
                    width: Some(0.8),
                    height: Some(0.8),
                })),
        )
    } else {
        let data = Data::columns()
            .column("x", [1., 3., 1., 2., 3.])
            .column("y", [1., 1., 2., 2., 2.])
            .column("v", [1., 3., 4., 5., 6.])
            .build()?;
        let recipe = if mode < 6 {
            BuiltinRecipe::Tile(TileRecipe {
                width: if mode == 5 { Some(0.8) } else { None },
                height: if mode == 5 { Some(0.6) } else { None },
            })
        } else {
            BuiltinRecipe::Raster(RasterRecipe {
                interpolate: mode == 7,
                ..Default::default()
            })
        };
        plot(data)
            .aes(aes().x("x").y("y"))
            .layer(points().recipe(recipe))
    };
    b = b
        .x_axis(x_axis().visible(false))
        .y_axis(y_axis().visible(false));
    b.build()
}
