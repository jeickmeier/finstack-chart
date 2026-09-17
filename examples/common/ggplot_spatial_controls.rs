//! GG11 source and statistic-driven spatial publication controls.
use chart_core::{grammar::*, prelude::*};
pub const CASES: usize = 16;
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    if (6..=9).contains(&mode) {
        let n = if mode == 9 { 2 } else { 5 };
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        for row in 0..n {
            for col in 0..n {
                let px = col as f64 - if n == 5 { 2. } else { 0. };
                let py = row as f64 - if n == 5 { 2. } else { 0. };
                x.push(px);
                y.push(py);
                z.push(if mode == 8 && row == 2 && col == 2 {
                    None
                } else {
                    Some(if mode == 9 {
                        if row == col { 0. } else { 2. }
                    } else {
                        px * px + py * py
                    })
                });
            }
        }
        let data = Data::columns()
            .column("x", x)
            .column("y", y)
            .column("z", z)
            .build()?;
        let filled = mode == 7 || mode == 8;
        let levels = ContourLevels {
            breaks: Some(if mode == 9 {
                vec![0.5, 1., 1.5]
            } else if filled {
                vec![0.5, 2., 4.]
            } else {
                vec![1., 2., 4.]
            }),
            ..Default::default()
        };
        let stat = spatial_stat(SpatialKind::Contour { levels, filled }).input("z");
        let layer = if filled {
            contour_filled().stat(stat)
        } else {
            contour().stat(stat)
        };
        return plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build();
    }
    let data = Data::columns()
        .column("x", [0., 0.25, 0.5, 1., 1.5, 2., 2.])
        .column("y", [0., 0.75, 1., 0.5, 1.5, 2., 0.])
        .column("z", [1., 2., 4., 8., 16., 32., 64.])
        .column("w", [1., 2., 0., 3., 1., 2., 1.])
        .build()?;
    let axis = SummaryBins {
        options: GgplotBinOptions {
            binwidth: Some(1.),
            boundary: Some(0.),
            ..Default::default()
        },
        ..Default::default()
    };
    let layer = match mode {
        0 => bin2d().stat(
            spatial_stat(SpatialKind::Rectangular {
                axes: [axis.clone(), axis],
                summary: None,
                drop: false,
            })
            .weight("w"),
        ),
        1 => bin2d().stat(
            spatial_stat(SpatialKind::Rectangular {
                axes: [axis.clone(), axis],
                summary: Some(SummaryFunction::Mean),
                drop: false,
            })
            .input("z"),
        ),
        2 | 3 => hex().stat(
            spatial_stat(SpatialKind::Hexagonal {
                binwidth: Some([1., 1.]),
                bins: [30, 30],
                summary: if mode == 3 {
                    Some(SummaryFunction::Median)
                } else {
                    None
                },
                drop: true,
            })
            .input("z")
            .weight("w"),
        ),
        4 | 5 | 14 | 15 => {
            let filled = mode == 5 || mode == 15;
            let stat = spatial_stat(SpatialKind::Density {
                bandwidth: Some([1., 2.]),
                adjust: [1., 1.],
                n: [25, 25],
                contour: if mode == 14 {
                    None
                } else {
                    Some(ContourLevels::default())
                },
                contour_var: if mode == 15 {
                    DensityContour::Normalized
                } else {
                    DensityContour::Density
                },
                filled,
            })
            .weight("w");
            if mode == 14 {
                bin2d().stat(stat)
            } else if filled {
                contour_filled().stat(stat)
            } else {
                density2d().stat(stat)
            }
        }
        10..=12 => ellipse().stat(
            spatial_stat(SpatialKind::Ellipse {
                kind: match mode {
                    10 => EllipseKind::Normal,
                    11 => EllipseKind::T,
                    _ => EllipseKind::Euclidean,
                },
                level: 0.8,
                segments: 51,
            })
            .weight("w"),
        ),
        _ => hex()
            .stat(identity_stat())
            .recipe(BuiltinRecipe::Hexagon(TileRecipe {
                width: Some(0.4),
                height: Some(0.3),
            }))
            .fill(chart_core::color::Paint::from_css("#D4A843")?),
    };
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .build()
}
