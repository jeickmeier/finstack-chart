//! Independent FIX-GG06 native/publication authors matching the five host cases.
use chart_core::{
    grammar::{BinClosure, GgplotBinOptions, SummaryBins, SummaryHelper},
    prelude::*,
};
pub fn author(mode: usize) -> ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 1., 2., 2., 4., 4.])
        .column("y", [1., 3., 4., 8., 2., 6.])
        .column("w", [2., -1., 0., 3., 0., -2.])
        .column("g", ["A", "A", "A", "A", "B", "B"])
        .build()?;
    let mut stat = if mode == 0 {
        count()
            .ggplot_count()
            .x("x")
            .group("g")
            .count_weight(data.field("w")?)
    } else {
        summary()
            .x("x")
            .y("y")
            .group("g")
            .summary_helper(SummaryHelper::default())
    };
    if mode == 2 || mode == 3 {
        stat = stat.summary_bins(SummaryBins {
            breaks: Some(vec![0., 2., 5.]),
            options: GgplotBinOptions {
                closed: if mode == 2 {
                    BinClosure::Right
                } else {
                    BinClosure::Left
                },
                ..Default::default()
            },
            ..Default::default()
        });
    }
    if mode == 4 {
        stat = stat.summary_helper(SummaryHelper::mean_cl_boot());
    }
    plot(data)
        .layer(points().stat(stat))
        .x_axis(x_axis().scale(scale_linear().domain(-1., 6.)))
        .y_axis(y_axis().scale(scale_linear().domain(-4., 12.)))
        .build()
}
