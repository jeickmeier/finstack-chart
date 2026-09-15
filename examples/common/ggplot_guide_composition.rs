//! FIX-GG05 shared native/publication authors for composed guides.
use chart_core::{
    grammar::{KeyOverrides, LayerLegend, LegendAesthetic, LegendOptions, LegendPosition, Profile},
    layout::{AxisCap, AxisSide, GgplotAxisOptions, LogTickOptions},
    prelude::*,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", vec![1., 4., 10., 100.])
        .column("g", vec!["A", "B", "C", "D"])
        .column("f", vec!["one", "one", "two", "two"])
        .build()?;
    let mut b = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.));
    if mode < 6 {
        let position = match mode {
            0 => LegendPosition::Right,
            1 => LegendPosition::Left,
            2 => LegendPosition::Top,
            3 => LegendPosition::Bottom,
            _ => LegendPosition::Inside { x: 0.5, y: 0.5 },
        };
        let o = LegendOptions {
            position: Some(position),
            ncol: Some(2),
            reverse: Some(mode % 2 == 1),
            override_aes: KeyOverrides {
                size: Some(4.),
                ..Default::default()
            },
            ..Default::default()
        };
        b = b
            .aes(aes().x("x").y(1.).color("g").shape("g"))
            .layer(points().legend(LayerLegend::default()))
            .legend(
                legend()
                    .aesthetic(LegendAesthetic::Color)
                    .options(o.clone()),
            )
            .legend(legend().aesthetic(LegendAesthetic::Shape).options(o));
        if mode == 5 {
            b = b.facet(facet_wrap("f").collect_guides(false));
        }
    } else if mode >= 8 {
        use chart_core::{grammar::NumericAesthetic, interpolate::Number, scales::*};
        let scale = ggplot_numeric_default(GgplotNumericPalette::Size)?
            .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                limits: Some([Some(Number(0.)), Some(Number(20.))]),
                breaks: GgplotBreaks::Explicit(vec![Number(5.), Number(10.)]),
                ..Default::default()
            })))?
            .with_guide(GgplotScaleGuide::BinnedBins(GgplotGuideLabels::Automatic))?
            .with_colorbar_options(GgplotColorbarOptions {
                show_limits: mode == 8,
                ..Default::default()
            });
        b = b
            .layer(points().numeric_scale(NumericAesthetic::Size, "x", scale))
            .legend(
                legend()
                    .aesthetic(LegendAesthetic::Size)
                    .options(LegendOptions {
                        position: Some(if mode == 8 {
                            LegendPosition::Right
                        } else {
                            LegendPosition::Bottom
                        }),
                        reverse: Some(mode == 9),
                        ..Default::default()
                    }),
            );
    } else if mode == 7 {
        let mut path = chart_core::path::Path::new();
        path.move_to(0., 0.)?;
        path.line_to(40., 0.)?;
        path.line_to(20., 20.)?;
        path.close_path()?;
        b = b
            .layer(points())
            .legend(legend().custom(chart_core::grammar::CustomLegend {
                id: chart_core::ScaleId::new(999),
                bounds: [0., 0., 40., 20.],
                paths: vec![chart_core::grammar::CustomGuidePath {
                    geometry: path.geometry(),
                    fill: Some(chart_core::color::parse_r("#123456")?),
                    stroke: None,
                }],
                options: LegendOptions {
                    title: Some("Custom vector".into()),
                    position: Some(LegendPosition::Left),
                    ..Default::default()
                },
            }));
    } else {
        b = b
            .layer(points())
            .axis(
                x_axis()
                    .scale(scale_log(10.).domain(1., 100.))
                    .label("Primary logarithmic axis")
                    .ggplot_axis(Some(GgplotAxisOptions {
                        n_dodge: 2,
                        check_overlap: true,
                        cap: AxisCap::Both,
                        stack_order: Some(0),
                        stack_spacing: 6.,
                        ..Default::default()
                    })),
            )
            .guide(
                axis_guide("outer", "x")
                    .side(AxisSide::Bottom)
                    .label("Stacked log ticks")
                    .ggplot_axis(Some(GgplotAxisOptions {
                        stack_order: Some(1),
                        logticks: Some(LogTickOptions {
                            expanded: false,
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
            );
    }
    b = b.axis(y_axis().visible(false));
    b.build()
}
