//! Primary recipes for native interaction and linked-view examples.
use chart_core::prelude::*;

pub fn data(case: usize) -> ChartResult<Data> {
    let (x, y) = match case {
        3 => (categorical(["Gamma", "Alpha", "Beta"]), vec![3., 1., 2.]),
        4 => (
            timestamps(
                vec![1709078400000, 1709164800000, 1709251200000, 1709337600000],
                TimeUnit::Milliseconds,
                "UTC",
            ),
            vec![1., 2., 3., 2.],
        ),
        _ => (column(vec![0., 1., 2., 3., 4.]), vec![2.; 5]),
    };
    let keys = 9007199254743001..(9007199254743001 + y.len() as u64);
    Data::columns()
        .column("x", x)
        .column(
            "y",
            column(y.clone()).validity((0..y.len()).map(|i| case != 2 || i != 2).collect()),
        )
        .keys(keys)
        .build()
}
pub fn plot_for(case: usize, data: Data) -> ChartResult<Plot> {
    let layer = match case {
        1 => bars().width(20.),
        2 | 4 => line(),
        _ => points(),
    };
    let mut p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(layer.name("observations").color(chart_core::scene::Color {
            red: 30,
            green: 125,
            blue: 180,
            alpha: 210,
        }));
    p = match case {
        3 => p.x_axis(
            x_axis().scale(
                scale_point()
                    .categories(["Alpha", "Beta", "Gamma"])
                    .point_padding(0.5),
            ),
        ),
        4 => p
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "x".into(),
                        origin: 1709164800000,
                    })
                    .y("y"),
            )
            .x_axis(x_axis().scale(scale_utc().interval(UtcInterval::Days(1)))),
        _ => p
            .x_axis(x_axis().scale(scale_linear().domain(0., 4.)))
            .y_axis(y_axis().scale(scale_linear().domain(0., 4.))),
    };
    p.build()
}
