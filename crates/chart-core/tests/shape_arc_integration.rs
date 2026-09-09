//! FIX-S02/03/09: projected geometry, provenance, actual curved containment and legacy contracts.
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{Compiler, NumericAesthetic as A, PreparedGeometry, Profile},
    inspection::{InputOrigin, InspectionAction, InspectionMode, Inspector},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    path::{Affine, Command},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::{Arc as ArcGenerator, ArcDatum, Pie, PieOrder},
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

fn prepared(p: &Plot) -> chart_core::grammar::PreparedChart {
    Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap()
}
#[test]
fn arc_radii_are_display_units_and_centers_use_named_projection() {
    let d = Data::columns()
        .column("x", [2.])
        .column("y", [100.])
        .build()
        .unwrap();
    let p = plot(d)
        .aes(aes().x("x").y("y"))
        .layer(
            shape_arc()
                .shape_value(A::InnerRadius, 20.)
                .shape_value(A::OuterRadius, 40.)
                .shape_value(A::EndAngle, std::f64::consts::FRAC_PI_2),
        )
        .build()
        .unwrap();
    let prepared = prepared(&p);
    let domains = prepared.layers()[0].domains();
    assert_eq!(domains.x.unwrap().minimum, 2.);
    assert_eq!(domains.x.unwrap().maximum, 2.);
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
    let reference = ArcGenerator::new()
        .generate(ArcDatum {
            inner_radius: 20.,
            outer_radius: 40.,
            start_angle: 0.,
            end_angle: std::f64::consts::FRAC_PI_2,
            pad_angle: 0.,
        })
        .unwrap()
        .geometry()
        .transformed(
            Affine::new([1., 0., 0., 1., 200., 100.]).unwrap(),
            0.01,
            100,
        )
        .unwrap();
    assert_eq!(*geometry, reference);
    assert!(
        geometry
            .commands()
            .iter()
            .any(|c| matches!(c, Command::Arc { radius: 40., .. }))
    );
    assert_eq!(anchors.len(), 1);
    assert_eq!(f.targets()[index].len(), 1);
    let inspector = Inspector::new(f.clone(), 5., 128).unwrap();
    assert!(
        inspector
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    assert_eq!(
        inspector
            .query(Point::new(222., 78.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    assert_eq!(p.definition().wire_version(), 7);
    let wire = p.to_json().unwrap();
    assert!(Plot::from_json(&wire).is_ok());
    let mut v: serde_json::Value = serde_json::from_str(&wire).unwrap();
    v["version"] = 6.into();
    assert!(Plot::from_json(&v.to_string()).is_err());
}
#[test]
fn pie_layout_preserves_input_targets_and_open_donut_holes() {
    let d = Data::columns()
        .column("weight", [1., 2., 1.])
        .keys([9007199254741001, 9007199254741003, 9007199254741005])
        .build()
        .unwrap();
    let p = plot(d)
        .aes(aes().x(2.).y(2.))
        .layer(
            shape_pie()
                .pie_order(PieOrder::Input)
                .shape_value(A::PieValue, "weight")
                .shape_value(A::InnerRadius, 20.),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let marks = pre.layers()[0].marks();
    assert_eq!(marks.len(), 3);
    let expected = Pie::new()
        .order(PieOrder::Input)
        .layout(&[1., 2., 1.])
        .unwrap();
    for (mark, slice) in marks.iter().zip(expected) {
        let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
            unreachable!()
        };
        assert_eq!(
            *geometry,
            ArcGenerator::new()
                .generate(slice.arc_datum(20., 40.))
                .unwrap()
                .geometry()
        );
        assert_eq!(mark.targets.len(), 1);
    }
    let f = frame(&p, false);
    let mut inspector = Inspector::new(f.clone(), 5., 128).unwrap();
    assert!(
        inspector
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    for (i, (x, y)) in [(222., 78.), (222., 122.), (178., 78.)]
        .into_iter()
        .enumerate()
    {
        let hit = inspector.query(Point::new(x, y).unwrap(), InspectionMode::Containment);
        assert_eq!(hit.hits.len(), 1);
        assert_eq!(hit.hits[0].target, marks[i].targets[0]);
    }
    let mut focused = vec![];
    for _ in 0..4 {
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
    assert_ne!(focused[1], focused[2]);
    assert_eq!(focused[0], focused[3]);
}
#[test]
fn grouped_pies_ignore_slice_color_for_automatic_population_and_validate_channels() {
    let d = Data::columns()
        .column("weight", [1., 2., 3., 1.])
        .column("slice", ["a", "b", "c", "a"])
        .column("group", ["p", "p", "q", "q"])
        .build()
        .unwrap();
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().color("slice"))
        .layer(shape_pie().shape_value(A::PieValue, "weight"))
        .build()
        .unwrap();
    let pre = prepared(&p);
    assert_eq!(pre.layers()[0].marks().len(), 4);
    assert!(
        pre.layers()[0]
            .marks()
            .iter()
            .all(|m| m.group == chart_core::grammar::GroupValue::All)
    );
    let p = plot(d.clone())
        .aes(aes().group("group"))
        .layer(
            shape_pie()
                .pie_order(PieOrder::Input)
                .shape_value(A::PieValue, "weight"),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let mut groups = std::collections::BTreeMap::new();
    for m in pre.layers()[0].marks() {
        groups
            .entry(m.group.clone())
            .or_insert_with(Vec::new)
            .push(m);
    }
    assert_eq!(groups.len(), 2);
    for marks in groups.values() {
        assert_eq!(marks.len(), 2);
        let PreparedGeometry::ShapePath { geometry, .. } = &marks[0].geometry else {
            unreachable!()
        };
        assert!(geometry.commands().len() > 2);
    }
    for layer in [
        shape_pie(),
        shape_pie()
            .shape_value(A::PieValue, "weight")
            .shape_value(A::StartAngle, 0.),
        points().shape_value(A::OuterRadius, 20.),
    ] {
        let p = plot(d.clone()).aes(aes().x(0.).y(0.)).layer(layer).build();
        if let Ok(p) = p {
            assert!(
                Compiler::new()
                    .prepare(
                        p.definition(),
                        &p.source(),
                        &ChartState::default(),
                        p.compile_limits()
                    )
                    .is_err()
            );
        }
    }
}

#[test]
fn generated_counts_feed_one_pie_without_losing_aggregate_membership() {
    let d = Data::columns()
        .column("category", ["a", "b", "b", "c"])
        .build()
        .unwrap();
    let p = plot(d)
        .layer(
            shape_pie()
                .stat(count().group("category"))
                .after_stat(stat_aes().x(2.).y(2.))
                .pie_grouped(false)
                .pie_order(PieOrder::Input)
                .shape_value(A::PieValue, chart_core::grammar::StatField::Count),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let marks = pre.layers()[0].marks();
    assert_eq!(marks.len(), 3);
    let expected = Pie::new()
        .order(PieOrder::Input)
        .layout(&[1., 2., 1.])
        .unwrap();
    for (mark, slice) in marks.iter().zip(expected) {
        let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
            unreachable!()
        };
        assert_eq!(
            *geometry,
            ArcGenerator::new()
                .generate(slice.arc_datum(0., 40.))
                .unwrap()
                .geometry()
        );
        let chart_core::provenance::Target::Aggregate { members, .. } = &mark.targets[0] else {
            panic!("count requires aggregate provenance");
        };
        assert_eq!(members.len() as f64, slice.value);
    }
}

#[test]
fn named_shape_statistical_inputs_reject_unexpected_fields() {
    let layer = chart_core::plot::host::Component::new("shape_pie", "[]").unwrap();
    assert!(
        layer
            .set(
                "shape_value",
                r#"["PieValue",{"Statistical":"Count","unexpected":1}]"#
            )
            .is_err()
    );
    assert!(
        layer
            .set("shape_value", r#"["PieValue",{"Statistical":"Count"}]"#)
            .is_ok()
    );
}
