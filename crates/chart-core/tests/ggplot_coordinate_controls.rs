//! GG13 coordinate population, geometry, inverse and destination contracts.
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{
        CartesianCoordinate, CoordinateClip, CoordinateSpec, RadialCoordinate, RadialMode,
        TransformedCoordinate,
    },
    layout::{CoordinateInverse, Coordinates, LaidOutChart, LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(spec: CoordinateSpec, layer: LayerBuilder) -> LaidOutChart {
    let d = Data::columns()
        .column("x", [0., 0.25, 0.5, 0.75, 1.])
        .column("y", [0., 0.25, 0.5, 0.75, 1.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .coordinate(spec)
        .build()
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for a in &mut request.axes {
        a.visible = false;
    }
    layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap()
}
#[test]
fn fixed_aspect_and_flip_use_shared_point_geometry_and_inverse() {
    for flip in [false, true] {
        let spec = CoordinateSpec::Cartesian(CartesianCoordinate {
            ratio: Some(2.),
            flip,
            expand: [false; 4],
            ..Default::default()
        });
        let f = frame(spec.clone(), points());
        let bounds = f.plot().unwrap();
        assert!((bounds.height() / bounds.width() - if flip { 0.5 } else { 2. }).abs() < 1e-12);
        let axes = f.axes();
        let x = axes.values().find(|a| a.spec.side.horizontal()).unwrap();
        let y = axes.values().find(|a| !a.spec.side.horizontal()).unwrap();
        let map = Coordinates::new(&spec, x, y, bounds).unwrap();
        let q = map
            .project(&ScaleValue::Number(0.25), &ScaleValue::Number(0.75))
            .unwrap()
            .unwrap();
        let CoordinateInverse::Values(values) = map.inverse(q, 8).unwrap() else {
            panic!("inverse")
        };
        let (ScaleValue::Number(a), ScaleValue::Number(b)) = &values[0] else {
            panic!("numeric")
        };
        assert!((a - 0.25).abs() < 1e-12 && (b - 0.75).abs() < 1e-12);
        assert_eq!(
            f.scene()
                .items()
                .iter()
                .filter(|i| i.layer.is_some() && matches!(i.primitive, Primitive::Point { .. }))
                .count(),
            5
        );
    }
}
#[test]
fn radial_paths_subdivide_and_annulus_clips_source_point_paint() {
    let spec = CoordinateSpec::Radial(RadialCoordinate {
        mode: RadialMode::Radial,
        inner_radius: 0.4,
        clip: Some(CoordinateClip::On),
        expand: false,
        ..Default::default()
    });
    let f = frame(spec, points());
    assert!(
        f.scene()
            .items()
            .iter()
            .any(|i| i.layer.is_some() && matches!(i.primitive, Primitive::ShapePath { .. }))
    );
    // Every retained fill vertex is outside the source's 0.4 inner-radius ratio,
    // up to the common 0.1-pixel clipping error used for every renderer.
    let p = f.plot().unwrap();
    let c = Point::new(
        p.origin().x() + p.width() / 2.,
        p.origin().y() + p.height() / 2.,
    )
    .unwrap();
    for i in f.scene().items().iter().filter(|i| i.layer.is_some()) {
        if let Primitive::ShapePath { geometry, .. } = &i.primitive {
            for sub in geometry.flatten(0.01, 100000).unwrap().subpaths {
                for q in sub.points {
                    assert!(
                        libm::hypot(q.x() - c.x(), q.y() - c.y()) >= p.width() * 0.4 * 0.4 - 0.11
                    );
                }
            }
        }
    }
}
#[test]
fn coordinate_transform_keeps_stat_population_and_curves_lines() {
    let f = frame(
        CoordinateSpec::Transformed(TransformedCoordinate {
            x: chart_core::scales::GgplotTransform::Identity,
            y: chart_core::scales::GgplotTransform::Sqrt,
            ..Default::default()
        }),
        line(),
    );
    assert!(f.scene().items().iter().filter(|i| i.layer.is_some()).any(
        |i| matches!(&i.primitive,Primitive::ShapePath{geometry,..} if geometry.commands().len()>5)
    ));
}
#[test]
fn transformed_arrowheads_keep_physical_length_after_stem_subdivision() {
    use chart_core::grammar::{ArrowEnds, ArrowSpec, BuiltinRecipe};
    let length = 3.;
    let f = frame(
        CoordinateSpec::Radial(RadialCoordinate {
            expand: false,
            ..Default::default()
        }),
        segment()
            .aes(aes().x("x").y("y").x2(0.8).y2(0.75))
            .recipe(BuiltinRecipe::Segment {
                arrow: Some(ArrowSpec {
                    length_mm: length,
                    ends: ArrowEnds::Last,
                    closed: true,
                    ..Default::default()
                }),
            }),
    );
    let mut count = 0;
    for item in f.scene().items().iter().filter(|i| i.layer.is_some()) {
        if let Primitive::ShapePath {
            geometry,
            fill: Some(_),
            ..
        } = &item.primitive
        {
            let commands = geometry.lower(0.01, 10000).unwrap();
            let points: Vec<_> = commands
                .iter()
                .filter_map(|c| match c {
                    chart_core::scene::PathCommand::MoveTo(p)
                    | chart_core::scene::PathCommand::LineTo(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if points.len() == 3 {
                count += 1;
                assert!(
                    (libm::hypot(points[0].x() - points[1].x(), points[0].y() - points[1].y())
                        - length * 96. / 25.4)
                        .abs()
                        < 1e-10
                );
            }
        }
    }
    assert!(count > 0);
}
#[test]
fn coordinate_navigation_uses_unique_post_transform_inverse_and_visible_view() {
    use chart_core::{
        navigation::{Navigation, NavigationBoundary, Navigator},
        state::AxisWindow,
    };
    use std::sync::Arc;
    let f = Arc::new(frame(
        CoordinateSpec::Transformed(TransformedCoordinate {
            x: chart_core::scales::GgplotTransform::Identity,
            y: chart_core::scales::GgplotTransform::Sqrt,
            view: CartesianCoordinate {
                expand: [false; 4],
                ..Default::default()
            },
        }),
        points(),
    ));
    let p = f.plot().unwrap();
    let anchor = Point::new(
        p.origin().x() + p.width() / 2.,
        p.origin().y() + p.height() / 2.,
    )
    .unwrap();
    let ids: Vec<_> = f.axes().keys().copied().collect();
    let navigator = Navigator::new(f.clone());
    let windows = navigator
        .navigate(
            f.scene().stamp(),
            &ids,
            None,
            Navigation::Zoom { anchor, factor: 2. },
            NavigationBoundary::Extend,
        )
        .unwrap();
    for axis in f.axes().values() {
        let AxisWindow::Numeric(a, b) = windows[&axis.spec.id] else {
            panic!("numeric")
        };
        let expected = if axis.spec.side.horizontal() {
            [0.25, 0.75]
        } else {
            [0.0625, 0.5625]
        };
        assert!(
            (a - expected[0]).abs() < 1e-12 && (b - expected[1]).abs() < 1e-12,
            "{a} {b}"
        );
    }
    let f = Arc::new(frame(
        CoordinateSpec::Radial(RadialCoordinate {
            expand: false,
            ..Default::default()
        }),
        points(),
    ));
    let p = f.plot().unwrap();
    let center = Point::new(
        p.origin().x() + p.width() / 2.,
        p.origin().y() + p.height() / 2.,
    )
    .unwrap();
    let ids: Vec<_> = f.axes().keys().copied().collect();
    let navigator = Navigator::new(f.clone());
    assert!(
        navigator
            .navigate(
                f.scene().stamp(),
                &ids,
                None,
                Navigation::Zoom {
                    anchor: center,
                    factor: 2.
                },
                NavigationBoundary::Extend
            )
            .is_err()
    );
    assert!(
        navigator
            .navigate(
                f.scene().stamp(),
                &ids,
                None,
                Navigation::Pan { dx: 1., dy: 0. },
                NavigationBoundary::Extend
            )
            .is_err()
    );
}
