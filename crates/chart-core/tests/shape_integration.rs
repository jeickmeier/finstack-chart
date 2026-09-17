//! FIX-S02/03/09: projected geometry, provenance, actual curved containment and legacy contracts.
use chart_core::plot::shape_area;
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::Compiler,
    inspection::{InputOrigin, InspectionAction, InspectionMode, Inspector, SelectionRegion},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    path::{Command, Path},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::{CurveSpec, Line},
    state::ChartState,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(plot: &Plot, log: bool) -> Arc<LaidOutChart> {
    let prepared = Compiler::new()
        .prepare(
            plot.definition(),
            &plot.source(),
            &ChartState::default(),
            plot.compile_limits(),
        )
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.padding = 0.;
    for a in &mut request.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 4.).unwrap()));
    }
    if log {
        request.axes[1].scale = AxisScale::Nonlinear {
            domain: ContinuousDomain::explicit(Bounds::new(1., 10000.).unwrap()),
            transform: chart_core::scales::ScaleTransform::Log { base: 10. },
        };
    }
    Arc::new(layout(Arc::new(prepared), &request, &Metrics).unwrap())
}
fn base() -> Data {
    Data::columns()
        .column("x", [0., 4.])
        .column("y", [1., 10000.])
        .build()
        .unwrap()
}
#[test]
fn curves_project_before_generation_and_controls_are_never_source_targets() {
    let p = plot(base())
        .aes(aes().x("x").y("y"))
        .layer(chart_core::plot::shape_line().curve(CurveSpec::BumpX))
        .build()
        .unwrap();
    let f = frame(&p, true);
    let index = f
        .scene()
        .items()
        .iter()
        .position(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
        .unwrap();
    let Primitive::ShapePath {
        geometry, anchors, ..
    } = &f.scene().items()[index].primitive
    else {
        unreachable!()
    };
    assert_eq!(
        anchors,
        &[Point::new(0., 200.).unwrap(), Point::new(400., 0.).unwrap()]
    );
    assert_eq!(
        geometry.commands(),
        &[
            Command::MoveTo([0., 200.]),
            Command::CubicTo([200., 200., 200., 0., 400., 0.])
        ]
    );
    assert_eq!(f.targets()[index].len(), 2);
    assert_eq!(f.scene().wire_version(), 3);
    let mut inspector = Inspector::new(f.clone(), 5., 128).unwrap();
    let hit = inspector.query(Point::new(200., 100.).unwrap(), InspectionMode::Containment);
    assert_eq!(hit.hits.len(), 1);
    assert!(f.targets()[index].contains(&hit.hits[0].target));
    assert!(
        inspector
            .query(Point::new(100., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    let controls = inspector
        .select(
            f.scene().stamp(),
            &SelectionRegion::Rectangle(Rect::new(190., 190., 20., 10.).unwrap()),
            128,
        )
        .unwrap();
    assert!(controls.is_empty());
    let mut focused = vec![];
    for _ in 0..3 {
        inspector
            .dispatch(
                f.scene().stamp(),
                InspectionAction::StepFocus { forward: true },
                InputOrigin::Keyboard,
            )
            .unwrap();
        focused.push(inspector.hits()[0].target.clone());
    }
    assert_ne!(focused[0], focused[1]);
    assert_eq!(focused[0], focused[2]);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 7,
        definition: p.definition().clone(),
    };
    assert!(envelope.validate().is_ok());
    envelope.version = 6;
    assert!(envelope.validate().is_err());
    let wire = p.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(value["version"], 7);
    assert!(Plot::from_json(&wire).is_ok());
    value["version"] = 6.into();
    assert!(Plot::from_json(&value.to_string()).is_err());
}
#[test]
fn general_area_has_independent_boundaries_and_no_lower_upper_order_constraint() {
    let data = Data::columns()
        .column("x", [0., 1., 2., 3.])
        .column("y", [3., 3., 3., 3.])
        .column("x2", [1., 2., 3., 4.])
        .column("y2", [1., 1., 1., 1.])
        .build()
        .unwrap();
    let p = plot(data.clone())
        .aes(aes().x("x").y("y").x2("x2").y2("y2"))
        .layer(chart_core::plot::shape_area())
        .build()
        .unwrap();
    let f = frame(&p, false);
    let index = f
        .scene()
        .items()
        .iter()
        .position(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
        .unwrap();
    let Primitive::ShapePath {
        geometry, anchors, ..
    } = &f.scene().items()[index].primitive
    else {
        unreachable!()
    };
    assert_eq!(geometry.commands()[0], Command::MoveTo([100., 150.]));
    assert_eq!(anchors.len(), 4);
    assert_eq!(f.targets()[index].len(), 4);
    let inspector = Inspector::new(f.clone(), 5., 128).unwrap();
    assert_eq!(
        inspector
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    assert!(
        inspector
            .query(Point::new(10., 190.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    assert_eq!(
        inspector
            .select(
                f.scene().stamp(),
                &SelectionRegion::Series {
                    layer: p.definition().layers[0].id,
                    panel: None
                },
                128
            )
            .unwrap()
            .len(),
        4
    );
    let old = plot(data)
        .aes(aes().x("x").y("y").y2("y2"))
        .layer(ribbon())
        .build()
        .unwrap();
    let prepared = Compiler::new()
        .prepare(
            old.definition(),
            &old.source(),
            &ChartState::default(),
            old.compile_limits(),
        )
        .unwrap();
    assert!(prepared.layers()[0].marks().is_empty());
}
#[test]
fn singleton_curve_policy_remains_distinct_from_legacy_line_points() {
    for (curve, empty) in [
        (CurveSpec::Linear, false),
        (CurveSpec::BasisOpen, true),
        (CurveSpec::Natural, false),
    ] {
        let data = Data::columns()
            .column("x", [1.])
            .column("y", [2.])
            .build()
            .unwrap();
        let p = plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(chart_core::plot::shape_line().curve(curve))
            .build()
            .unwrap();
        let f = frame(&p, false);
        let shapes: Vec<_> = f
            .scene()
            .items()
            .iter()
            .filter(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
            .collect();
        assert_eq!(shapes.is_empty(), empty);
        assert!(
            !f.scene()
                .items()
                .iter()
                .any(|i| i.layer.is_some() && matches!(i.primitive, Primitive::Point { .. }))
        );
        let legacy = plot(data)
            .aes(aes().x("x").y("y"))
            .layer(line())
            .build()
            .unwrap();
        assert!(
            frame(&legacy, false)
                .scene()
                .items()
                .iter()
                .any(|i| matches!(i.primitive, Primitive::Point { .. }))
        );
    }
}
#[test]
fn nonzero_holes_curved_strokes_caps_and_subdivision_limits_are_explicit() {
    let mut path = Path::new();
    path.arc([0., 0.], 10., 0., std::f64::consts::TAU, false)
        .unwrap();
    path.move_to(5., 0.).unwrap();
    path.arc([0., 0.], 5., 0., std::f64::consts::TAU, true)
        .unwrap();
    let flat = path.geometry().flatten(0.001, 10000).unwrap();
    assert!(!flat.contains(Point::new(0., 0.).unwrap(), true, None));
    assert!(flat.contains(Point::new(7., 0.).unwrap(), true, None));
    assert!(!flat.contains(Point::new(11., 0.).unwrap(), true, None));
    assert!(path.geometry().flatten(0.000001, 3).is_err());
    let curve = Line::new()
        .curve(CurveSpec::BumpX)
        .unwrap()
        .generate(&[[0., 0.], [100., 100.]])
        .unwrap();
    let flat = curve.geometry().flatten(0.001, 10000).unwrap();
    assert!(flat.contains(Point::new(50., 50.).unwrap(), false, Some(2.)));
    assert!(!flat.contains(Point::new(-0.5, 0.).unwrap(), false, Some(2.)));
    assert!(!flat.contains(Point::new(25., 50.).unwrap(), false, Some(2.)));
    let mut join = Path::new();
    join.move_to(0., 10.).unwrap();
    join.line_to(10., 10.).unwrap();
    join.line_to(10., 0.).unwrap();
    let flat = join.geometry().flatten(0.001, 100).unwrap();
    assert!(flat.contains(Point::new(10.9, 10.9).unwrap(), false, Some(2.)));
}

#[test]
fn missing_secondary_coordinate_splits_the_paired_run_and_retains_source_keys() {
    let data = Data::columns()
        .column("x", vec![0., 1., 2., 3.])
        .column("y", vec![0., 1., 2., 3.])
        .column("x2", vec![Some(1.), None, Some(3.), Some(4.)])
        .column("y2", vec![1., 2., 3., 4.])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").y("y").x2("x2").y2("y2"))
        .layer(shape_area())
        .build()
        .unwrap();
    let f = frame(&p, false);
    let shapes: Vec<_> = f
        .scene()
        .items()
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i.primitive, Primitive::ShapePath { .. }))
        .collect();
    assert_eq!(shapes.len(), 2);
    assert_eq!(f.targets()[shapes[0].0].len(), 1);
    assert_eq!(f.targets()[shapes[1].0].len(), 2);
    assert_eq!(f.prepared().layers()[0].invalid_geometry(), 1);
}

#[test]
fn clipped_area_interiors_keep_source_focus_without_inventing_brush_vertices() {
    let data = Data::columns()
        .column("x", [0., 4.])
        .column("y", [-10., -10.])
        .column("x2", [0., 4.])
        .column("y2", [10., 10.])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").y("y").x2("x2").y2("y2"))
        .layer(shape_area())
        .build()
        .unwrap();
    let f = frame(&p, false);
    let mut inspector = Inspector::new(f.clone(), 3., 128).unwrap();
    let hit = inspector.query(Point::new(200., 100.).unwrap(), InspectionMode::Containment);
    assert_eq!(hit.hits.len(), 1);
    assert_eq!(hit.hits[0].position.y(), 0.);
    assert_eq!(
        inspector
            .select(
                f.scene().stamp(),
                &SelectionRegion::Series {
                    layer: p.definition().layers[0].id,
                    panel: None
                },
                128
            )
            .unwrap()
            .len(),
        2
    );
    assert!(
        inspector
            .select(
                f.scene().stamp(),
                &SelectionRegion::Rectangle(Rect::new(0., 0., 5., 5.).unwrap()),
                128
            )
            .unwrap()
            .is_empty()
    );
    inspector
        .dispatch(
            f.scene().stamp(),
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard,
        )
        .unwrap();
    assert_eq!(inspector.hits().len(), 1);
    assert!(
        inspector
            .query(Point::new(200., -1.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
}
