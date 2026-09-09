//! FIX-S05/09: every pinned radial/link path through chart projection and exact provenance.
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{Compiler, NumericAesthetic as A, PreparedChart, RadialParameters},
    inspection::{InputOrigin, InspectionAction, InspectionMode, Inspector, SelectionRegion},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    path::{Affine, Command, PathRequest},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::{Coordinate, CurveSpec},
    state::ChartState,
};
use serde_json::{Value, json};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn checked(result: ChartResult<Plot>) -> ChartResult<PreparedChart> {
    let p = result?;
    Compiler::new().prepare(
        p.definition(),
        &p.source(),
        &ChartState::default(),
        p.compile_limits(),
    )
}
fn prepared(p: &Plot) -> PreparedChart {
    Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap()
}
fn frame(p: &Plot, radial: bool) -> Arc<LaidOutChart> {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.padding = 0.;
    for a in &mut r.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(if radial {
            Bounds::new(0., 4.).unwrap()
        } else {
            Bounds::new(-50., 50.).unwrap()
        }));
    }
    if radial {
        r.axes[1].scale = AxisScale::Nonlinear {
            domain: ContinuousDomain::explicit(Bounds::new(1., 10000.).unwrap()),
            transform: chart_core::scales::ScaleTransform::Log { base: 10. },
        };
    }
    Arc::new(layout(Arc::new(prepared(p)), &r, &Metrics).unwrap())
}
fn near(actual: f64, expected: f64, id: &str) {
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs().max(1.),
        "{id}: {actual} != {expected}"
    );
}
fn compare(actual: &Value, expected: &Value, id: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => near(a.as_f64().unwrap(), b.as_f64().unwrap(), id),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{id}");
            for (a, b) in a.iter().zip(b) {
                compare(a, b, id);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{id}"
            );
            for (k, a) in a {
                compare(a, &b[k], id);
            }
        }
        _ => assert_eq!(actual, expected, "{id}"),
    }
}

fn select(config: &Value, name: &str, default: Coordinate) -> Coordinate {
    config
        .get(name)
        .filter(|v| !v.is_null())
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .unwrap_or(default)
}
fn optional(config: &Value, name: &str, default: Option<Coordinate>) -> Option<Coordinate> {
    config
        .get(name)
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .unwrap_or(default)
}
fn read(c: Coordinate, row: &[f64]) -> f64 {
    match c {
        Coordinate::Constant(v) => v,
        Coordinate::Column(i) => row[i],
    }
}
fn endpoint<'a>(config: &'a Value, name: &str, input: &'a Value) -> Vec<f64> {
    let selected = match config.get(name) {
        None => &input[name],
        Some(Value::String(s)) => &input[s.to_lowercase()],
        Some(c) => &c["Constant"],
    };
    serde_json::from_value(selected.clone()).unwrap()
}
#[test]
fn pinned_radial_and_link_paths_flow_through_named_chart_channels() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/radial.json")).unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let config = &case["config"];
        let family = case["family"].as_str().unwrap();
        let radial = family != "link";
        let curve: CurveSpec = config
            .get("curve")
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .unwrap_or(if family == "link" {
                CurveSpec::BumpX
            } else {
                CurveSpec::Linear
            });
        let (data, layer) = if family.starts_with("link") {
            let source = endpoint(config, "source", &case["input"]);
            let target = endpoint(config, "target", &case["input"]);
            let x = select(
                config,
                if radial { "angle" } else { "x" },
                Coordinate::Column(0),
            );
            let y = select(
                config,
                if radial { "radius" } else { "y" },
                Coordinate::Column(1),
            );
            let (x, y, x2, y2) = (
                read(x, &source),
                read(y, &source),
                read(x, &target),
                read(y, &target),
            );
            let data = Data::columns()
                .keys([9007199254741001])
                .column("value", [0.])
                .build()
                .unwrap();
            let layer = if radial {
                shape_link_radial()
                    .shape_value(A::StartAngle, x)
                    .shape_value(A::InnerRadius, y)
                    .shape_value(A::EndAngle, x2)
                    .shape_value(A::OuterRadius, y2)
                    .aes(aes().x(2.).y(100.))
            } else {
                shape_link(curve).aes(aes().x(x).y(y).x2(x2).y2(y2))
            };
            (data, layer)
        } else {
            let input: Vec<Vec<f64>> = serde_json::from_value(case["input"].clone()).unwrap();
            let (mut x0, mut y0, mut x1, mut y1) = (
                Coordinate::Column(0),
                Coordinate::Constant(0.),
                None,
                Some(Coordinate::Column(1)),
            );
            let area = family == "areaRadial" && case["helper"].is_null();
            if family == "lineRadial" {
                x0 = select(config, "angle", Coordinate::Column(0));
                y0 = select(config, "radius", Coordinate::Column(1));
            } else {
                x0 = select(config, "start_angle", x0);
                y0 = select(config, "inner_radius", y0);
                x1 = optional(config, "end_angle", x1);
                y1 = optional(config, "outer_radius", y1);
                if config.get("angle").is_some() {
                    x0 = select(config, "angle", x0);
                    x1 = None;
                }
                if config.get("radius").is_some() {
                    y0 = select(config, "radius", y0);
                    y1 = None;
                }
                match case["helper"].as_str() {
                    Some("EndAngle") => x0 = x1.unwrap_or(Coordinate::Constant(0.)),
                    Some("OuterRadius") => y0 = y1.unwrap_or(Coordinate::Constant(0.)),
                    _ => {}
                }
            }
            let defined = |i: usize| match config.get("defined") {
                Some(Value::Bool(v)) => *v,
                Some(Value::Array(v)) => v[i].as_bool().unwrap(),
                _ => true,
            };
            let values = |c: Coordinate| {
                input
                    .iter()
                    .enumerate()
                    .map(|(i, row)| defined(i).then(|| read(c, row)))
                    .collect::<Vec<_>>()
            };
            let data = Data::columns()
                .keys((0..input.len()).map(|i| 9007199254741001 + i as u64))
                .column("a", values(x0))
                .column("r", values(y0))
                .column("a2", values(x1.unwrap_or(x0)))
                .column("r2", values(y1.unwrap_or(y0)))
                .build()
                .unwrap();
            let field = |name: &str| NumericScaleInput::from(data.field(name).unwrap());
            let layer = if area {
                shape_area_radial()
                    .shape_value(A::StartAngle, field("a"))
                    .shape_value(A::InnerRadius, field("r"))
                    .shape_value(A::EndAngle, field("a2"))
                    .shape_value(A::OuterRadius, field("r2"))
            } else {
                shape_line_radial()
                    .shape_value(A::Angle, field("a"))
                    .shape_value(A::Radius, field("r"))
            };
            (data, layer.curve(curve).aes(aes().x(2.).y(100.)))
        };
        let p = plot(data)
            .layer(layer)
            .build()
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let f = frame(&p, radial);
        let mut actual = vec![];
        for (i, item) in f.scene().items().iter().enumerate() {
            if let Primitive::ShapePath {
                geometry, anchors, ..
            } = &item.primitive
            {
                actual.extend_from_slice(geometry.commands());
                assert_eq!(anchors.len(), f.targets()[i].len(), "{id}");
                for target in &f.targets()[i] {
                    let value = serde_json::to_value(target).unwrap();
                    assert!(
                        value["Source"]["key"]
                            .as_str()
                            .unwrap()
                            .parse::<u64>()
                            .unwrap()
                            >= 9007199254741001,
                        "{id}"
                    );
                }
            }
        }
        let reference = PathRequest {
            version: 1,
            digits: None,
            limits: Default::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        }
        .build()
        .unwrap()
        .geometry()
        .transformed(
            Affine::new(if radial {
                [1., 0., 0., 1., 200., 100.]
            } else {
                [4., 0., 0., -2., 200., 100.]
            })
            .unwrap(),
            0.01,
            1_000_000,
        )
        .unwrap();
        compare(&json!(actual), &json!(reference.commands()), id);
        if radial {
            let d = prepared(&p);
            let domains = d.layers()[0].domains();
            if !actual.is_empty() {
                assert_eq!(domains.x.unwrap().minimum, 2.);
                assert_eq!(domains.x.unwrap().maximum, 2.);
                assert_eq!(domains.y.unwrap().minimum, 100.);
                assert_eq!(domains.y.unwrap().maximum, 100.);
            }
        }
        let wire = p.to_json().unwrap();
        assert_eq!(serde_json::from_str::<Value>(&wire).unwrap()["version"], 7);
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    }
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 697);
}

#[test]
fn radial_holes_gaps_centers_and_real_source_anchors_are_preserved() {
    let angles: Vec<_> = (0..=8)
        .map(|i| i as f64 * std::f64::consts::TAU / 8.)
        .collect();
    let d = Data::columns()
        .column("angle", angles)
        .keys(9007199254741001..9007199254741010)
        .build()
        .unwrap();
    let p = plot(d)
        .aes(aes().x(2.).y(100.))
        .layer(
            shape_area_radial()
                .shape_value(A::Angle, "angle")
                .shape_value(A::InnerRadius, 10.)
                .shape_value(A::OuterRadius, 30.),
        )
        .build()
        .unwrap();
    let f = frame(&p, true);
    let mut index = Inspector::new(f.clone(), 5., 128).unwrap();
    assert!(
        index
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    assert_eq!(
        index
            .query(Point::new(200., 80.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    let all = index
        .select(
            f.scene().stamp(),
            &SelectionRegion::Series {
                layer: p.definition().layers[0].id,
                panel: None,
            },
            128,
        )
        .unwrap();
    assert_eq!(all.len(), 9);
    let mut focus = vec![];
    for _ in 0..10 {
        index
            .dispatch(
                f.scene().stamp(),
                InspectionAction::StepFocus { forward: true },
                InputOrigin::Keyboard,
            )
            .unwrap();
        focus.push(index.hits()[0].target.clone());
    }
    assert_eq!(focus[0], focus[9]);
    assert_ne!(focus[0], focus[1]);
    let d = Data::columns()
        .column("angle", [Some(0.), Some(1.), None, Some(2.), Some(3.)])
        .build()
        .unwrap();
    for (connect, count) in [(false, 2), (true, 1)] {
        let p = plot(d.clone())
            .aes(aes().x(2.).y(100.))
            .layer(
                shape_line_radial()
                    .shape_value(A::Angle, "angle")
                    .shape_value(A::Radius, -20.)
                    .connect_gaps(connect),
            )
            .build()
            .unwrap();
        let f = frame(&p, true);
        let paths: Vec<_> = f
            .scene()
            .items()
            .iter()
            .filter_map(|i| {
                if let Primitive::ShapePath { anchors, .. } = &i.primitive {
                    Some(anchors)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(paths.len(), count);
        assert_eq!(paths.iter().map(|p| p.len()).sum::<usize>(), 4);
        assert_eq!(paths[0][0], Point::new(200., 120.).unwrap());
    }
    let different_centers = Data::columns()
        .column("x", [1., 2.])
        .column("a", [0., 1.])
        .build()
        .unwrap();
    assert!(
        checked(
            plot(different_centers)
                .aes(aes().x("x").y(100.))
                .layer(shape_line_radial().shape_value(A::Angle, "a"))
                .build()
        )
        .is_err()
    );
    let d = Data::columns().column("value", [1.]).build().unwrap();
    assert!(
        checked(
            plot(d.clone())
                .layer(shape_line_radial().shape_value(A::EndAngle, 1.))
                .build()
        )
        .is_err()
    );
    assert!(
        checked(
            plot(d.clone())
                .layer(
                    shape_area_radial()
                        .shape_value(A::Angle, 1.)
                        .shape_value(A::StartAngle, 2.)
                )
                .build()
        )
        .is_err()
    );
    assert!(
        checked(
            plot(d)
                .layer(shape_link_radial().radial_parameters(RadialParameters {
                    outer_radius: Some(f64::NAN),
                    ..Default::default()
                }))
                .build()
        )
        .is_err()
    );
}

#[test]
fn link_endpoint_anchors_retain_one_edge_identity_and_never_create_control_targets() {
    let data = Data::columns()
        .column("value", [0.])
        .keys([9007199254741001])
        .build()
        .unwrap();
    let p = plot(data)
        .layer(shape_link_horizontal().aes(aes().x(-50.).y(-50.).x2(50.).y2(50.)))
        .build()
        .unwrap();
    let f = frame(&p, false);
    let i = f
        .scene()
        .items()
        .iter()
        .position(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
        .unwrap();
    let Primitive::ShapePath {
        geometry, anchors, ..
    } = &f.scene().items()[i].primitive
    else {
        unreachable!()
    };
    assert_eq!(
        geometry.commands(),
        &[
            Command::MoveTo([0., 200.]),
            Command::CubicTo([200., 200., 200., 0., 400., 0.])
        ]
    );
    assert_eq!(
        anchors,
        &[Point::new(0., 200.).unwrap(), Point::new(400., 0.).unwrap()]
    );
    assert_eq!(f.targets()[i].len(), 2);
    assert_eq!(f.targets()[i][0], f.targets()[i][1]);
    let mut inspector = Inspector::new(f.clone(), 8., 128).unwrap();
    let mut identities = vec![];
    for bounds in [
        Rect::new(0., 190., 10., 10.).unwrap(),
        Rect::new(390., 0., 10., 10.).unwrap(),
    ] {
        let selected = inspector
            .select(f.scene().stamp(), &SelectionRegion::Rectangle(bounds), 128)
            .unwrap();
        assert_eq!(selected.len(), 1);
        identities.push(selected[0].identity.clone());
    }
    assert_eq!(identities[0], identities[1]);
    assert!(
        inspector
            .select(
                f.scene().stamp(),
                &SelectionRegion::Rectangle(Rect::new(190., 190., 20., 10.).unwrap()),
                128
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        inspector
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    for _ in 0..3 {
        inspector
            .dispatch(
                f.scene().stamp(),
                InspectionAction::StepFocus { forward: true },
                InputOrigin::Keyboard,
            )
            .unwrap();
        assert_eq!(inspector.hits().len(), 1);
        assert_eq!(
            chart_core::state::TargetIdentity::from(&inspector.hits()[0].target),
            identities[0]
        );
    }
}

#[test]
fn facet_vertex_budget_counts_shape_commands_and_source_anchors_across_panels() {
    use chart_core::{DiagnosticCode, grammar::CompileLimits};
    let data = Data::columns()
        .column("facet", categorical(["A", "B"]))
        .build()
        .unwrap();
    // Each quarter sector contains four commands and one source anchor: total ten.
    let make = |maximum| {
        plot(data.clone())
            .facet(facet_wrap("facet"))
            .layer(shape_arc().shape_value(A::EndAngle, std::f64::consts::FRAC_PI_2))
            .compile_limits(CompileLimits {
                max_vertices: maximum,
                ..Default::default()
            })
            .build()
    };
    assert_eq!(
        checked(make(7)).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(checked(make(10)).unwrap().panels().len(), 2);
}

#[test]
fn generated_radial_edges_accept_literals_and_train_literal_scales_per_prepared_row() {
    use chart_core::{
        DiagnosticCode,
        grammar::{PreparedGeometry, StatField},
        interpolate::Value,
        scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    };
    let data = Data::columns()
        .column("category", ["a", "b", "b"])
        .column("angle", [0., 1., 2.])
        .build()
        .unwrap();
    let layer = || {
        shape_link_radial()
            .stat(count().group("category"))
            .after_stat(stat_aes().x(2.).y(100.))
            .shape_value(A::StartAngle, 0.)
            .shape_value(A::EndAngle, 1.)
            .shape_value(A::InnerRadius, 10.)
            .shape_value(A::OuterRadius, StatField::Count)
    };
    let p = plot(data.clone()).layer(layer()).build().unwrap();
    let prepared = prepared(&p);
    let marks = prepared.layers()[0].marks();
    assert_eq!(marks.len(), 2);
    for (mark, outer) in marks.iter().zip([1., 2.]) {
        assert_eq!(mark.targets.len(), 2);
        assert_eq!(mark.targets[0], mark.targets[1]);
        let PreparedGeometry::ShapePathRun { anchors, .. } = &mark.geometry else {
            panic!("radial edge")
        };
        near(anchors[1].x(), outer * 1f64.sin(), "aggregate radius");
        near(anchors[1].y(), -outer * 1f64.cos(), "aggregate radius");
    }
    let scale = ScaleConstructor::Quantile
        .create(ScaleOptions {
            range: Some([10., 20.].map(Value::number).to_vec()),
            ..Default::default()
        })
        .unwrap()
        .mapped(ScaleTraining::Eligible)
        .unwrap();
    let p = plot(data.clone())
        .layer(layer().numeric_scale(A::OuterRadius, 7., scale))
        .build();
    let trained = checked(p).unwrap();
    for mark in trained.layers()[0].marks() {
        let PreparedGeometry::ShapePathRun { anchors, .. } = &mark.geometry else {
            panic!("trained radial edge")
        };
        near(anchors[1].x(), 20. * 1f64.sin(), "literal quantile radius");
    }
    assert_eq!(
        checked(
            plot(data.clone())
                .layer(layer().shape_value(A::StartAngle, "angle"))
                .build()
        )
        .unwrap_err()
        .code,
        DiagnosticCode::SchemaConflict
    );
    assert!(
        checked(
            plot(data)
                .layer(layer().shape_value(A::StartAngle, f64::NAN))
                .build()
        )
        .is_err()
    );
}

#[test]
fn projected_shape_budget_is_shared_by_panels_and_insets_before_callbacks() {
    use std::cell::Cell;
    struct Counting(Cell<usize>);
    impl TextMeasurer for Counting {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            self.0.set(self.0.get() + 1);
            Metrics.measure(r)
        }
    }
    for facets in [false, true] {
        for inset in [false, true] {
            let data = Data::columns()
                .column(
                    "facet",
                    categorical(if facets { vec!["A", "B"] } else { vec!["A"] }),
                )
                .build()
                .unwrap();
            let mut draft =
                plot(data).layer(shape_arc().shape_value(A::EndAngle, std::f64::consts::FRAC_PI_2));
            if facets {
                draft = draft.facet(facet_wrap("facet"));
            }
            let p = draft.build().unwrap();
            let mut definition = p.definition().clone();
            if inset {
                let pre = prepared(&p);
                definition
                    .figure
                    .get_or_insert_with(Default::default)
                    .insets
                    .push(chart_core::composition::Inset {
                        panel: facets.then(|| pre.panels()[0].key.clone()),
                        layers: vec![definition.layers[0].id],
                        id: "repeat".into(),
                        rectangle: [0.6, 0.1, 0.3, 0.3],
                        x_view: None,
                        y_view: None,
                        guides: false,
                    });
            }
            let pre = Arc::new(
                Compiler::new()
                    .prepare(
                        &definition,
                        &p.source(),
                        &ChartState::default(),
                        p.compile_limits(),
                    )
                    .unwrap(),
            );
            // Each quarter sector is four commands and one source anchor. An inset repeats one.
            let required = 5 * (if facets { 2 } else { 1 } + usize::from(inset));
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
            request.max_vertices = required - 1;
            let metrics = Counting(Cell::new(0));
            assert_eq!(
                layout(pre.clone(), &request, &metrics).unwrap_err().code,
                chart_core::DiagnosticCode::ResourceLimit
            );
            assert_eq!(metrics.0.get(), 0, "budget must fail before text callbacks");
            request.max_vertices = required;
            assert!(layout(pre, &request, &metrics).is_ok());
        }
    }
}
