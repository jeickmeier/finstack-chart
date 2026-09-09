//! SHP-09 / FIX-S09: retained curved dashes, source identity and bounded ink containment.
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::Compiler,
    inspection::{InspectionMode, Inspector},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    path::{Command, PathGeometry},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::{PathCommand, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::CurveSpec,
    state::ChartState,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(curve: CurveSpec, y: [f64; 2], dashes: Vec<f64>) -> Arc<LaidOutChart> {
    let p = plot(
        Data::columns()
            .identity(1)
            .column("x", [0., 1.])
            .column("y", y)
            .keys([9007199254740993, 9007199254740997])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(shape_line().curve(curve))
    .build()
    .unwrap();
    let pre = Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap();
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 100., 100.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.padding = 0.;
    r.host_theme.dashes = Some(dashes);
    for a in &mut r.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 1.).unwrap()));
    }
    Arc::new(layout(Arc::new(pre), &r, &Metrics).unwrap())
}
#[test]
fn theme_retains_curve_geometry_and_sources_and_uses_scene_version_four() {
    let solid = frame(CurveSpec::BumpX, [0., 1.], vec![]);
    let dashed = frame(CurveSpec::BumpX, [0., 1.], vec![10., 10.]);
    assert_eq!(solid.scene().wire_version(), 3);
    assert_eq!(dashed.scene().wire_version(), 4);
    let shapes = |chart: &LaidOutChart| {
        chart
            .scene()
            .items()
            .iter()
            .enumerate()
            .find_map(|(i, item)| {
                if let Primitive::ShapePath {
                    geometry,
                    anchors,
                    dashes,
                    ..
                } = &item.primitive
                {
                    Some((
                        geometry.clone(),
                        anchors.clone(),
                        dashes.clone(),
                        chart.targets()[i].clone(),
                    ))
                } else {
                    None
                }
            })
            .unwrap()
    };
    let a = shapes(&solid);
    let b = shapes(&dashed);
    assert_eq!(a.0, b.0);
    assert_eq!(a.1, b.1);
    assert_eq!(a.3, b.3);
    assert_eq!(b.2, vec![10., 10.]);
    assert!(matches!(b.0.commands()[1], Command::CubicTo(_)));
    let ink = b.0.dashed(&b.2, 0.01, 10000).unwrap();
    assert!(
        ink.iter()
            .all(|c| matches!(c, PathCommand::MoveTo(_) | PathCommand::LineTo(_)))
    );
    assert!(
        ink.iter()
            .filter(|c| matches!(c, PathCommand::MoveTo(_)))
            .count()
            > 1
    );
    assert_eq!(
        serde_json::to_value(solid.scene().items())
            .unwrap()
            .to_string()
            .matches("dashes")
            .count(),
        0
    );
}
#[test]
fn containment_rejects_dash_gaps_without_inventing_source_targets() {
    let f = frame(CurveSpec::BumpX, [0.5, 0.5], vec![10., 10.]);
    let inspector = Inspector::new(f.clone(), 1., 128).unwrap();
    for x in [5., 25., 45., 65., 85.] {
        assert_eq!(
            inspector
                .query(Point::new(x, 50.).unwrap(), InspectionMode::Containment)
                .hits
                .len(),
            1
        );
    }
    for x in [15., 35., 55., 75., 95.] {
        assert!(
            inspector
                .query(Point::new(x, 50.).unwrap(), InspectionMode::Containment)
                .hits
                .is_empty()
        );
    }
    assert_eq!(f.targets().iter().map(Vec::len).sum::<usize>(), 2);
}
#[test]
fn dash_subpaths_reset_phase_and_validate_empty_paths_and_limits() {
    let p = PathGeometry::from_commands(
        vec![
            Command::MoveTo([0., 0.]),
            Command::LineTo([30., 0.]),
            Command::MoveTo([0., 5.]),
            Command::LineTo([30., 5.]),
        ],
        10,
    )
    .unwrap();
    let actual = p.dashed(&[10., 10.], 0.01, 100).unwrap();
    let pt = |x, y| Point::new(x, y).unwrap();
    assert_eq!(
        actual,
        vec![
            PathCommand::MoveTo(pt(0., 0.)),
            PathCommand::LineTo(pt(10., 0.)),
            PathCommand::MoveTo(pt(20., 0.)),
            PathCommand::LineTo(pt(30., 0.)),
            PathCommand::MoveTo(pt(0., 5.)),
            PathCommand::LineTo(pt(10., 5.)),
            PathCommand::MoveTo(pt(20., 5.)),
            PathCommand::LineTo(pt(30., 5.))
        ]
    );
    assert_eq!(
        p.dashed(&[1e-12, 1e-12], 0.01, 100).unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
    let empty = PathGeometry::from_commands(vec![], 10).unwrap();
    for pattern in [vec![], vec![1.], vec![1., 0.], vec![1., f64::NAN]] {
        assert!(empty.dashed(&pattern, 0.01, 100).is_err());
    }
}

#[test]
fn closed_dash_seams_retain_the_join_between_last_and_first_ink() {
    let p = PathGeometry::from_commands(
        vec![
            Command::MoveTo([0., 0.]),
            Command::LineTo([15., 0.]),
            Command::LineTo([15., 15.]),
            Command::Close,
        ],
        10,
    )
    .unwrap();
    let all = p.dashed(&[100., 1.], 0.01, 100).unwrap();
    assert!(matches!(all.last(), Some(PathCommand::Close)));
    let split = p.dashed(&[9., 3.], 0.01, 100).unwrap();
    // Its perimeter is 30 + 15*sqrt(2), so the closing seam lies inside ink.
    // No MoveTo at that seam may break the joined incoming and outgoing segments.
    assert!(
        !split
            .iter()
            .any(|c| matches!(c,PathCommand::MoveTo(p) if p.x()==0. && p.y()==0.))
    );
    let origin = split
        .iter()
        .position(|c| matches!(c,PathCommand::LineTo(p) if p.x()==0. && p.y()==0.))
        .unwrap();
    assert!(matches!(split[origin + 1], PathCommand::LineTo(_)));
}
