//! FIX-GG02: primary stage authors shared by native display and immutable publication.
use chart_core::{
    grammar::{AfterScaleAesthetic, Expression, ExpressionReduce, ScaleOob, ThemeRead},
    prelude::*,
    theme::{GeometryTheme, rgb},
};
fn data() -> ChartResult<Data> {
    Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [1., 10., 100.])
        .build()
}
fn figure(data: Data) -> PlotBuilder {
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
}
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let mut result = vec![];
    for (name, axis) in [
        ("log_mean", y_axis().scale(scale_log(10.))),
        (
            "coordinate_log_mean",
            y_axis().coordinate_scale(scale_log(10.)),
        ),
        (
            "scale_limit_mean",
            y_axis().scale(scale_linear().domain(1., 10.)),
        ),
        ("coordinate_zoom_mean", y_axis().viewport(1., 10.)),
        (
            "squish_mean",
            y_axis()
                .scale(scale_linear().domain(1., 10.))
                .oob(ScaleOob::Squish),
        ),
        (
            "keep_mean",
            y_axis()
                .scale(scale_linear().domain(1., 10.))
                .oob(ScaleOob::Keep),
        ),
    ] {
        result.push((
            name,
            figure(data()?)
                .layer(
                    points()
                        .stat(summary().x("y"))
                        .after_stat(stat_aes().x(1.).y(StatField::Mean)),
                )
                .y_axis(axis)
                .build()?,
        ));
    }
    let hist = Data::columns()
        .column("value", [1., 2., 5., 20., 50., 200., 500.])
        .build()?;
    result.push((
        "horizontal_log_histogram",
        plot(hist.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().y("value"))
            .layer(histogram().breaks(vec![1., 10., 100., 1000.]))
            .y_axis(y_axis().scale(scale_log(10.)))
            .build()?,
    ));
    let count = Expression::read(BinField::Count);
    result.push((
        "after_stat_expression",
        plot(hist)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("value"))
            .layer(
                histogram().breaks(vec![1., 10., 100., 1000.]).after_bin(
                    bin_aes().y(count.clone() / count.reduce(ExpressionReduce::Sum, false)),
                ),
            )
            .build()?,
    ));
    let categories = Data::columns()
        .column("category", categorical(["A", "A", "B"]))
        .build()?;
    result.push((
        "horizontal_count",
        plot(categories)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().y("category"))
            .layer(bars().orientation(Orientation::Horizontal))
            .y_axis(y_axis().scale(scale_band()))
            .build()?,
    ));
    let data = data()?;
    result.push((
        "source_expression",
        figure(data.clone())
            .aes(
                aes()
                    .x(source_expr(data.field("x")?) + 1.)
                    .y(source_expr("y") * 2.),
            )
            .layer(points())
            .build()?,
    ));
    result.push((
        "after_scale_expression",
        figure(data.clone())
            .layer(
                points().after_scale(
                    scale_aes().size(after_scale_expr(AfterScaleAesthetic::Size) * 2.),
                ),
            )
            .build()?,
    ));
    result.push((
        "theme_expression",
        figure(data.clone())
            .theme(theme().geometry(GeometryTheme {
                accent: rgb(18, 86, 171),
                ..Default::default()
            }))
            .layer(points().after_scale(scale_aes().color(from_theme(ThemeRead::Accent))))
            .build()?,
    ));
    result.push((
        "log_after_stat_expression",
        figure(data)
            .layer(
                points()
                    .size(2.)
                    .color(rgb(18, 86, 171))
                    .stat(summary().x("y"))
                    .after_stat(stat_aes().x(1.).y(Expression::read(StatField::Mean) * 2.)),
            )
            .y_axis(y_axis().scale(scale_log(10.).domain(1., 100.)))
            .build()?,
    ));
    Ok(result)
}
