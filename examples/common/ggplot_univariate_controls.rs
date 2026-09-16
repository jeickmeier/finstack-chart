//! GG09 independently authored empirical, theoretical and function publications.
use chart_core::{grammar::*, prelude::*};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 2., 4., 5., 6.])
        .column("y", [1., 3., 3., 2., 5., 4.])
        .column("w", [1., 2., 0., 1., 3., 1.])
        .build()?;
    let layer = match mode {
        0 => density().stat(density_stat().input("x").weight("w")),
        1 => ecdf().stat(ecdf_stat().input("x").weight("w")),
        2 => qq().stat(qq_stat().input("y")),
        3 => qq_line().stat(qq_line_stat().input("y")),
        4 | 5 => line().stat(
            univariate_stat(UnivariateKind::Function {
                function: AnalyticFunction::Expression(
                    Expression::read(FunctionArgument::Value).binary(
                        ExpressionBinary::Multiply,
                        Expression::read(FunctionArgument::Value),
                    ),
                ),
                n: 21,
                range: None,
            })
            .input("x"),
        ),
        6 => points().stat(unique_stat().x("x").y("y")),
        _ => line().stat(
            connect_stat(Connection::Matrix(vec![[0., 0.], [0.25, 0.75], [1., 1.]]))
                .x("x")
                .y("y"),
        ),
    };
    let mut builder = plot(data).profile(Profile::Ggplot2_4_0_3).layer(layer);
    if mode == 5 {
        builder = builder.x_axis(x_axis().scale(scale_log(10.)));
    }
    builder.build()
}
