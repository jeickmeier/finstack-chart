//! GG09 precomputed source geometry and control contracts; estimator authors are separate.
use chart_core::plot::{boxplot, boxplot_stat, dotplot, dotplot_stat, violin, violin_stat};
use chart_core::{RowKey, grammar::*, prelude::*, scene::Color};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    if mode >= 11 {
        let data = Data::columns()
            .column("x", [0., 0., 0.2, 1., 1.2, 2., 2., 2.])
            .build()?;
        let outline = [
            AreaOutline::Upper,
            AreaOutline::Lower,
            AreaOutline::Both,
            AreaOutline::Full,
        ][mode - 11];
        return plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(
                chart_core::plot::density()
                    .stat(chart_core::plot::density_stat().input("x"))
                    .recipe(BuiltinRecipe::Density(DensityRecipe { outline }))
                    .fill(Color {
                        red: 255,
                        green: 215,
                        blue: 0,
                        alpha: 255,
                    })
                    .alpha(0.2),
            )
            .build();
    }
    if mode >= 8 {
        let data = Data::columns()
            .column("x", [1.; 8])
            .column("y", [0., 1., 1., 2., 3., 4., 5., 20.])
            .build()?;
        let layer = match mode {
            8 => boxplot().stat(boxplot_stat().input(data.field("y")?).x("x")),
            9 => violin().stat(violin_stat().input(source_expr(data.field("y")?)).x("x")),
            _ => dotplot().stat(dotplot_stat().input("y")),
        };
        return plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(layer)
            .build();
    }
    if mode < 4 {
        let data = Data::columns()
            .keys([10, 20])
            .column("x", [1., 2.])
            .column("middle", [2.5, 3.])
            .column("lo", [0.75, 2.])
            .column("hi", [3.25, 4.])
            .column("min", [0., 1.])
            .column("max", [4., 5.])
            .column("nl", [1.1034641071565687, 1.5868050382201329])
            .column("nu", [3.8965358928434313, 4.413194961779867])
            .column("relative", [8_f64.sqrt(), 5_f64.sqrt()])
            .build()?;
        let mut spec = BoxplotRecipe {
            width: Some(0.75),
            notch: mode == 1,
            notch_width: 0.3,
            staple_width: if mode == 0 { 0. } else { 0.7 },
            variable_width: mode == 2,
            source_outliers: vec![SourceBoxOutliers {
                row: RowKey::new(10),
                values: vec![20.],
            }],
            ..Default::default()
        };
        if mode == 2 {
            spec.median.color = Some(
                Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                }
                .into(),
            );
            spec.median.linewidth = Some(1.);
            spec.whisker.line_type = Some(LineType::Dashed);
            spec.outlier.shape = Some(21);
            spec.outlier.fill = Some(
                Color {
                    red: 255,
                    green: 215,
                    blue: 0,
                    alpha: 255,
                }
                .into(),
            );
            spec.outlier.size = Some(3.);
        }
        let mut layer = rule()
            .recipe(BuiltinRecipe::Boxplot(Box::new(spec)))
            .recipe_value(RecipeAesthetic::Lower, "lo")
            .recipe_value(RecipeAesthetic::Upper, "hi")
            .recipe_value(RecipeAesthetic::Middle, "middle")
            .recipe_value(RecipeAesthetic::WhiskerLower, "min")
            .recipe_value(RecipeAesthetic::WhiskerUpper, "max")
            .recipe_value(RecipeAesthetic::NotchLower, "nl")
            .recipe_value(RecipeAesthetic::NotchUpper, "nu")
            .recipe_value(RecipeAesthetic::RelativeWidth, "relative");
        let mut aes = aes().x("x").y("middle").group("x");
        if mode == 3 {
            aes = aes.x("middle").y("x");
            layer = layer.orientation(Orientation::Horizontal);
        }
        return plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes)
            .layer(layer)
            .build();
    }
    if mode == 4 {
        let data = Data::columns()
            .keys([1, 2, 3, 4, 5])
            .column("x", [1.; 5])
            .column("y", [0., 1., 2., 3., 4.])
            .column("width", [0.1, 0.7, 1., 0.5, 0.1])
            .column("q", [None, Some(0.25), Some(0.5), Some(0.75), None])
            .build()?;
        return plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(
                rule()
                    .recipe(BuiltinRecipe::Violin(ViolinRecipe {
                        width: Some(0.8),
                        quantile: IntervalStroke {
                            line_type: Some(LineType::Dashed),
                            color: Some(
                                Color {
                                    red: 255,
                                    green: 0,
                                    blue: 0,
                                    alpha: 255,
                                }
                                .into(),
                            ),
                            ..Default::default()
                        },
                    }))
                    .recipe_value(RecipeAesthetic::ViolinWidth, "width")
                    .recipe_value(RecipeAesthetic::QuantileFlag, "q"),
            )
            .build();
    }
    let data = Data::columns()
        .keys([1, 2, 3])
        .column("bin", [0., 1., 2.])
        .column("count", [3., 1., 4.])
        .build()?;
    let axis = if mode == 6 { DotAxis::Y } else { DotAxis::X };
    let stack = if mode == 7 {
        DotStack::CenterWhole
    } else {
        DotStack::Up
    };
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(if axis == DotAxis::X {
            aes().x("bin").y(0.)
        } else {
            aes().x(1.).y("bin")
        })
        .layer(
            rule()
                .recipe(BuiltinRecipe::Dotplot(DotplotRecipe {
                    bin_axis: axis,
                    stack,
                    stack_ratio: 0.8,
                    dot_size: 0.7,
                    ..Default::default()
                }))
                .recipe_value(RecipeAesthetic::Count, "count")
                .recipe_value(RecipeAesthetic::BinWidth, 0.5),
        )
        .build()
}
