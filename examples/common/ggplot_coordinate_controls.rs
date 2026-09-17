//! GG13 coordinate guides authored through the common typed chart pipeline.
use chart_core::{
    grammar::*,
    layout::{AxisSide, GuideComponents, GuideTextStyle},
    prelude::*,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    if mode >= 6 {
        return marks(mode);
    }
    let data = Data::columns()
        .column("x", vec![0., 1., 2., 3., 4.])
        .column("y", vec![1., 2., 3., 2., 1.])
        .build()?;
    let mut radial = RadialCoordinate {
        expand: false,
        ..Default::default()
    };
    match mode {
        0 => radial.mode = RadialMode::Polar,
        1 => {
            radial.start = std::f64::consts::FRAC_PI_4;
            radial.end = Some(1.5 * std::f64::consts::PI);
            radial.inner_radius = 0.3;
        }
        2 => {
            radial.reverse = RadialReverse::Both;
            radial.radial_axis = Some(RadialAxisPlacement::Inside);
            radial.inner_radius = 0.3;
        }
        4 => radial.inner_radius = 0.4,
        _ => {}
    }
    let coordinate = if mode == 5 {
        CoordinateSpec::Cartesian(CartesianCoordinate {
            ratio: Some(1.),
            ..Default::default()
        })
    } else {
        CoordinateSpec::Radial(radial)
    };
    let mut x = x_axis()
        .scale(scale_linear().domain(0., 4.))
        .ticks((0..=4).map(|i| {
            (
                chart_core::composition::ScaleValue::Number(i as f64),
                i.to_string(),
            )
        }));
    if mode == 3 {
        x = x.guide_components(Some(GuideComponents {
            labels: GuideTextStyle {
                rotation: Some(0.),
                ..Default::default()
            },
            ..Default::default()
        }));
    }
    let mut p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points())
        .x_axis(x)
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)))
        .coordinate(coordinate);
    if mode == 4 {
        p = p
            .guide(axis_guide("theta-inner", "x").side(AxisSide::Top))
            .guide(axis_guide("r-secondary", "y").side(AxisSide::Right));
    }
    p.build()
}

fn marks(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = if mode == 10 {
        Data::columns()
            .column("x", [1., 3., 1., 2., 3.])
            .column("y", [1., 1., 2., 2., 2.])
            .build()?
    } else if mode == 13 {
        Data::columns()
            .column("x", categorical(["A", "B", "C", "D", "E"]))
            .column("y", [1., 2., 3., 2., 1.])
            .build()?
    } else {
        Data::columns()
            .column("x", [0., 1., 2., 3., 4.])
            .column("y", [1., 2., 3., 2., 1.])
            .column("lo", [0.5, 1., 2., 1., 0.5])
            .column("hi", [1.5, 3., 4., 3., 1.5])
            .build()?
    };
    let mut p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"));
    let coordinate = match mode {
        6 => {
            p = p.layer(
                errorbar()
                    .recipe_value(RecipeAesthetic::Lower, "lo")
                    .recipe_value(RecipeAesthetic::Upper, "hi"),
            );
            CoordinateSpec::Cartesian(CartesianCoordinate {
                flip: true,
                ..Default::default()
            })
        }
        7 => {
            p = p
                .layer(line())
                .layer(segment().aes(aes().x("x").y("y").x2(3.8).y2(3.5)).recipe(
                    BuiltinRecipe::Segment {
                        arrow: Some(ArrowSpec {
                            closed: true,
                            length_mm: 3.,
                            ..Default::default()
                        }),
                    },
                ));
            CoordinateSpec::Transformed(TransformedCoordinate {
                y: chart_core::scales::GgplotTransform::Sqrt,
                ..Default::default()
            })
        }
        8 => {
            p = p.layer(rectangle().recipe(BuiltinRecipe::Column(ColumnRecipe::default())));
            CoordinateSpec::Radial(RadialCoordinate {
                mode: RadialMode::Polar,
                expand: false,
                ..Default::default()
            })
        }
        9 => {
            p = p.layer(line()).layer(points().radius(5.));
            CoordinateSpec::Radial(RadialCoordinate {
                inner_radius: 0.45,
                clip: Some(CoordinateClip::On),
                expand: false,
                ..Default::default()
            })
        }
        10 => {
            p = p.layer(points().recipe(BuiltinRecipe::Raster(RasterRecipe {
                interpolate: true,
                ..Default::default()
            })));
            CoordinateSpec::Radial(RadialCoordinate {
                inner_radius: 0.5,
                clip: Some(CoordinateClip::On),
                expand: false,
                ..Default::default()
            })
        }
        11 => {
            p = p.layer(points()).theme(theme().style(style().gradient(
                chart_core::scene::LinearGradient {
                    direction: chart_core::scene::GradientDirection::Horizontal,
                    start: rgb(255, 220, 100),
                    end: rgb(40, 90, 180),
                },
            )));
            CoordinateSpec::Radial(RadialCoordinate {
                start: std::f64::consts::FRAC_PI_4,
                end: Some(1.5 * std::f64::consts::PI),
                inner_radius: 0.35,
                clip: Some(CoordinateClip::On),
                expand: false,
                ..Default::default()
            })
        }
        12 => {
            p = p.layer(
                rectangle()
                    .recipe(BuiltinRecipe::Column(ColumnRecipe::default()))
                    .stat(count().ggplot_count().x("x")),
            );
            CoordinateSpec::Cartesian(CartesianCoordinate {
                xlim: Some([Some(ScaleValue::Number(1.)), Some(ScaleValue::Number(3.))]),
                expand: [false; 4],
                ..Default::default()
            })
        }
        _ => {
            p = p.layer(points());
            CoordinateSpec::Cartesian(CartesianCoordinate {
                flip: true,
                reverse: CoordinateReverse::Y,
                ..Default::default()
            })
        }
    };
    p.coordinate(coordinate).build()
}
