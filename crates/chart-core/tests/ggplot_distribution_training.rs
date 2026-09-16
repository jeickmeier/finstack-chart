//! GG09: analytical grids use the existing filtered physical-axis scale training.
use chart_core::{grammar::*, prelude::*};

#[test]
fn density_shares_other_layers_range_in_both_orientations() {
    for orientation in [Orientation::Vertical, Orientation::Horizontal] {
        let d = Data::columns().column("v", [1., 2.]).build().unwrap();
        let overlay = Data::columns()
            .name("overlay")
            .column("x", [0., 10.])
            .column("y", [0., 10.])
            .build()
            .unwrap();
        let mapping = if orientation == Orientation::Vertical {
            aes().x("v")
        } else {
            aes().y("v")
        };
        let chart = plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(mapping)
            .layer(
                density().orientation(orientation).stat(
                    density_stat()
                        .x("v")
                        .distribution_options(DistributionKind::Density {
                            trim: false,
                            controls: DensityControls {
                                bandwidth: Bandwidth::Fixed(0.5),
                                n: 16,
                                ..Default::default()
                            },
                        }),
                ),
            )
            .layer(points().data(overlay).aes(aes().x("x").y("y")))
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        let PreparedRows::Statistical(rows) = chart.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(rows.first().unwrap().value(&StatField::X), Some(0.));
        assert_eq!(rows.last().unwrap().value(&StatField::X), Some(10.));
    }
}

#[test]
fn horizontal_density_facets_share_only_the_physical_sample_axis() {
    for free_y in [false, true] {
        let data = Data::columns()
            .column("v", [0., 1., 100., 101.])
            .column("g", ["A", "A", "B", "B"])
            .build()
            .unwrap();
        let chart = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().y("v"))
            .layer(
                density().orientation(Orientation::Horizontal).stat(
                    density_stat()
                        .x("v")
                        .distribution_options(DistributionKind::Density {
                            trim: false,
                            controls: DensityControls {
                                bandwidth: Bandwidth::Fixed(0.5),
                                n: 16,
                                ..Default::default()
                            },
                        }),
                ),
            )
            .facet(facet_wrap("g").free_x(!free_y).free_y(free_y))
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        for (i, panel) in chart.panels().iter().enumerate() {
            let PreparedRows::Statistical(rows) = panel.chart.layers()[0].table().rows() else {
                panic!()
            };
            assert_eq!(
                rows.first().unwrap().value(&StatField::X),
                Some(if free_y { i as f64 * 100. } else { 0. })
            );
            assert_eq!(
                rows.last().unwrap().value(&StatField::X),
                Some(if free_y { i as f64 * 100. + 1. } else { 101. })
            );
        }
    }
}
