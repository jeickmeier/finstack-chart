//! WP-07 inspection uses immutable presented geometry, clips and original provenance.
use chart_core::data::*;
use chart_core::grammar::*;
use chart_core::inspection::*;
use chart_core::layout::*;
use chart_core::provenance::{ResolvedTarget, Target};
use chart_core::scales::*;
use chart_core::services::*;
use chart_core::state::ChartState;
use chart_core::transaction::DataStore;
use chart_core::*;
use std::sync::Arc;
const D: DatasetId = DatasetId::new(1);
const X: FieldId = FieldId::new(1);
const Y: FieldId = FieldId::new(2);
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn scene(geom: Geom, ys: &[Option<f64>], extra: bool) -> Arc<LaidOutChart> {
    let rows = TypedRows::snapshot(
        D,
        Revision::INITIAL,
        (1..=ys.len() as u64).map(RowKey::new).collect(),
        ys.iter().copied().enumerate().collect(),
        100,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(X, "x", |r| Some(r.0 as f64))
        .float(Y, "y", |r| r.1)
        .finish(DataLimits::default())
        .unwrap();
    let source =
        DataStore::new(SourceEpoch::new(1), vec![(D, batch)], DataLimits::default()).unwrap();
    let mut d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        D,
        geom,
        if matches!(geom, Geom::Bar { .. }) {
            SourceAes::new().x(X).y(Y).y2(Numeric::Literal(0.))
        } else {
            SourceAes::new().x(X).y(Y)
        },
    ));
    if extra {
        d.layers.push(Layer::new(
            LayerId::new(2),
            D,
            geom,
            SourceAes::new().x(X).y(Y),
        ));
    }
    let p = Compiler::new()
        .prepare(
            &d,
            &source.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
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
    Arc::new(layout(Arc::new(p), &request, &Metrics).unwrap())
}
fn hover(i: &mut Inspector, x: f64, y: f64) -> InspectionOutcome {
    i.dispatch(
        i.presented().scene().stamp(),
        InspectionAction::Hover(Some(Point::new(x, y).unwrap())),
        InputOrigin::Pointer,
    )
    .unwrap()
}
fn key(t: &Target) -> u64 {
    let Target::Source(s) = t else {
        panic!("source")
    };
    s.key.get()
}
#[test]
fn scatter_nearest_radius_clips_and_equal_distance_z_order() {
    let mut i = Inspector::new(
        scene(
            Geom::Point,
            &[Some(2.), Some(2.), Some(2.), Some(2.), Some(2.), Some(2.)],
            true,
        ),
        20.,
        8,
    )
    .unwrap();
    assert_eq!(i.candidate_count(), 10);
    assert!(hover(&mut i, 102., 100.).changed);
    assert_eq!(key(&i.hits()[0].target), 2);
    assert_eq!(i.hits()[0].layer, LayerId::new(2));
    assert!(!hover(&mut i, 103., 100.).changed);
    hover(&mut i, 150., 100.);
    assert!(i.hits().is_empty());
    hover(&mut i, 401., 100.);
    assert!(i.hits().is_empty());
}
#[test]
fn nearest_x_groups_preserve_gap_targets_and_never_invent_interpolation() {
    let mut i = Inspector::new(
        scene(Geom::line(), &[Some(1.), None, Some(3.)], true),
        10.,
        8,
    )
    .unwrap();
    hover(&mut i, 175., 75.);
    assert_eq!(i.hits().len(), 2);
    assert!(i.hits().iter().all(|h| key(&h.target) == 3));
    assert!(
        matches!(i.hits()[0].target.resolve(i.presented().prepared().source().get().unwrap()).unwrap(),ResolvedTarget::Source(row)if row.value(Y)==Some(ValueRef::Float64(3.)))
    );
    // Equidistant x positions choose one x group, never merge two different abscissas.
    hover(&mut i, 100., 75.);
    assert_eq!(i.hits().len(), 2);
    assert!(i.hits().iter().all(|h| key(&h.target) == 3));
    let mut limited = Inspector::new(i.presented().clone(), 10., 1).unwrap();
    hover(&mut limited, 175., 75.);
    assert_eq!(limited.hits().len(), 1);
}
#[test]
fn keyboard_wrap_clear_idempotency_and_stale_stamp_are_atomic() {
    let s = scene(Geom::Point, &[Some(1.), None, Some(3.)], false);
    let mut i = Inspector::new(s.clone(), 10., 8).unwrap();
    let stamp = s.scene().stamp();
    for expected in [1, 3, 1] {
        let result = i
            .dispatch(
                stamp,
                InspectionAction::StepFocus { forward: true },
                InputOrigin::Keyboard,
            )
            .unwrap();
        assert!(result.changed);
        assert_eq!(result.origin, InputOrigin::Keyboard);
        assert_eq!(key(&i.hits()[0].target), expected);
        assert!(i.has_keyboard_focus());
    }
    let prior = i.hits().to_vec();
    let mut wrong = stamp;
    wrong.layout = Revision::new(99);
    assert_eq!(
        i.dispatch(wrong, InspectionAction::Clear, InputOrigin::Programmatic)
            .unwrap_err()
            .code,
        DiagnosticCode::Superseded
    );
    assert_eq!(i.hits(), prior);
    assert!(
        i.dispatch(stamp, InspectionAction::Clear, InputOrigin::Programmatic)
            .unwrap()
            .changed
    );
    assert!(
        !i.dispatch(stamp, InspectionAction::Clear, InputOrigin::Pointer)
            .unwrap()
            .changed
    );
    drop(s);
    i.dispatch(
        stamp,
        InspectionAction::StepFocus { forward: false },
        InputOrigin::Keyboard,
    )
    .unwrap();
    assert_eq!(key(&i.hits()[0].target), 3);
}
#[test]
fn empty_inspection_and_invalid_options_are_recoverable() {
    let s = scene(Geom::Point, &[], false);
    assert!(Inspector::new(s.clone(), f64::NAN, 8).is_err());
    assert!(Inspector::new(s.clone(), 10., 0).is_err());
    let mut i = Inspector::new(s, 10., 8).unwrap();
    assert!(
        !i.dispatch(
            i.presented().scene().stamp(),
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard
        )
        .unwrap()
        .changed
    );
    assert!(!hover(&mut i, 100., 100.).changed);
}
#[test]
fn histogram_hit_retains_aggregate_members_and_respects_containment() {
    let rows = TypedRows::snapshot(
        D,
        Revision::INITIAL,
        (1..=4).map(RowKey::new).collect(),
        vec![0., 0.5, 1., 2.],
        4,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(X, "x", |v| Some(*v))
        .finish(DataLimits::default())
        .unwrap();
    let source =
        DataStore::new(SourceEpoch::new(1), vec![(D, batch)], DataLimits::default()).unwrap();
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        LayerId::new(1),
        D,
        BinSpec::new(X, vec![0., 1., 2.]),
    ));
    let prepared = Compiler::new()
        .prepare(
            &d,
            &source.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
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
    }
    let mut i = Inspector::new(
        Arc::new(layout(Arc::new(prepared), &r, &Metrics).unwrap()),
        10.,
        8,
    )
    .unwrap();
    hover(&mut i, 100., 100.);
    assert_eq!(i.hits().len(), 1);
    let ResolvedTarget::Aggregate { members, .. } = i.hits()[0]
        .target
        .resolve(i.presented().prepared().source().get().unwrap())
        .unwrap()
    else {
        panic!("aggregate")
    };
    assert_eq!(
        members.iter().map(|r| r.key().get()).collect::<Vec<_>>(),
        vec![1, 2]
    );
    hover(&mut i, -1., 100.);
    assert!(i.hits().is_empty());
}

fn dense_scene(n: usize, geom: Geom) -> Arc<LaidOutChart> {
    let rows = TypedRows::snapshot(
        D,
        Revision::INITIAL,
        (1..=n as u64).map(RowKey::new).collect(),
        (0..n).collect(),
        n,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(X, "x", |i| Some((*i % 100) as f64))
        .float(Y, "y", |i| Some((*i / 100) as f64))
        .finish(DataLimits::default())
        .unwrap();
    let source =
        DataStore::new(SourceEpoch::new(1), vec![(D, batch)], DataLimits::default()).unwrap();
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        D,
        geom,
        SourceAes::new().x(X).y(Y),
    ));
    let p = Compiler::new()
        .prepare(
            &d,
            &source.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
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
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(
            Bounds::new(
                0.,
                if a.side.horizontal() {
                    100.
                } else {
                    (n / 100) as f64
                },
            )
            .unwrap(),
        ));
    }
    Arc::new(layout(Arc::new(p), &request, &Metrics).unwrap())
}
#[test]
fn dense_spatial_and_sorted_x_match_scan_without_source_work() {
    for geom in [Geom::Point, Geom::line()] {
        let chart = dense_scene(20_000, geom);
        let prepared = chart.prepared().clone();
        let i = Inspector::new(chart, 3., 16).unwrap();
        let mut examined = 0;
        for seed in 0..100 {
            let p =
                Point::new(f64::from((seed * 127) % 400), f64::from((seed * 47) % 200)).unwrap();
            let indexed = i.query(p, InspectionMode::Auto);
            let scan = i.query_scan(p, InspectionMode::Auto);
            assert_eq!(indexed.hits, scan.hits);
            assert!(scan.examined >= 20_000);
            examined += indexed.examined;
        }
        assert!(examined < 100 * 400, "index examined {examined}");
        assert!(Arc::ptr_eq(&prepared, i.presented().prepared()));
        let copy = i.clone();
        assert!(Arc::ptr_eq(i.presented(), copy.presented()));
    }
}
#[test]
fn brushes_lasso_series_and_limits_preserve_vertex_provenance() {
    let chart = scene(Geom::line(), &[Some(1.), None, Some(3.)], false);
    let i = Inspector::new(chart.clone(), 10., 8).unwrap();
    let stamp = chart.scene().stamp();
    // The line gap has no invented vertex or source identity.
    assert!(
        i.select(
            stamp,
            &SelectionRegion::Rectangle(Rect::new(90., 90., 20., 20.).unwrap()),
            16
        )
        .unwrap()
        .is_empty()
    );
    let selected = i
        .select(stamp, &SelectionRegion::XRange(190., 210.), 16)
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].identity,
        chart_core::state::TargetIdentity::Source {
            dataset: D,
            key: RowKey::new(3)
        }
    );
    let lasso = SelectionRegion::Lasso(vec![
        Point::new(190., 40.).unwrap(),
        Point::new(210., 40.).unwrap(),
        Point::new(200., 60.).unwrap(),
    ]);
    assert_eq!(i.select(stamp, &lasso, 16).unwrap(), selected);
    assert!(
        i.select(stamp, &SelectionRegion::Lasso(vec![]), 16)
            .is_err()
    );
    assert!(
        i.select(
            stamp,
            &SelectionRegion::Series {
                layer: LayerId::new(1),
                panel: None
            },
            1
        )
        .is_err()
    );
    assert_eq!(
        i.select(
            stamp,
            &SelectionRegion::Series {
                layer: LayerId::new(1),
                panel: None
            },
            16
        )
        .unwrap()
        .len(),
        2
    );
    let mut wrong = stamp;
    wrong.layout = Revision::new(99);
    assert_eq!(
        i.select(wrong, &lasso, 16).unwrap_err().code,
        DiagnosticCode::Superseded
    );
    assert!(Arc::ptr_eq(chart.prepared(), i.presented().prepared()));
}
#[test]
fn rectangle_selection_intersects_bars_instead_of_testing_centers() {
    let i = Inspector::new(
        scene(
            Geom::Bar {
                width: 20.,
                nonnegative: false,
            },
            &[Some(2.), Some(2.)],
            false,
        ),
        10.,
        8,
    )
    .unwrap();
    let stamp = i.presented().scene().stamp();
    // Bar at x=100 spans x=90..110 and y=100..200; its center is outside this brush.
    let result = i
        .select(
            stamp,
            &SelectionRegion::Rectangle(Rect::new(109., 105., 2., 2.).unwrap()),
            16,
        )
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].identity,
        chart_core::state::TargetIdentity::Source {
            dataset: D,
            key: RowKey::new(2)
        }
    );
    assert!(
        i.select(
            stamp,
            &SelectionRegion::Rectangle(Rect::new(111., 105., 2., 2.).unwrap()),
            16
        )
        .unwrap()
        .is_empty()
    );
    let point = Point::new(109., 105.).unwrap();
    assert_eq!(i.query(point, InspectionMode::Containment).hits.len(), 1);
    assert!(i.query(point, InspectionMode::NearestPoint).hits.is_empty());
}
#[test]
fn stable_keyboard_identity_survives_rebuild_and_removal_clears_it() {
    let first = scene(Geom::Point, &[Some(1.), Some(2.), Some(3.)], false);
    let mut i = Inspector::new(first, 10., 8).unwrap();
    i.dispatch(
        i.presented().scene().stamp(),
        InspectionAction::StepFocus { forward: true },
        InputOrigin::Keyboard,
    )
    .unwrap();
    let identity = chart_core::state::MarkTarget::from_inspected(&i.hits()[0], SourceEpoch::new(1));
    let mut next =
        Inspector::new(scene(Geom::Point, &[Some(2.), Some(3.)], false), 10., 8).unwrap();
    assert!(next.restore_focus(&identity).unwrap());
    assert_eq!(key(&next.hits()[0].target), 1);
    assert_ne!(next.hits()[0].position, i.hits()[0].position);
    let mut removed = Inspector::new(scene(Geom::Point, &[None, Some(3.)], false), 10., 8).unwrap();
    assert!(!removed.restore_focus(&identity).unwrap());
    assert!(removed.hits().is_empty());
    assert!(!removed.has_keyboard_focus());
}

#[test]
fn candle_wicks_are_contained_and_keyboard_deduplicates_each_candle() {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../fixtures/families/portable-cases.json"
    ))
    .unwrap();
    let mut case = cases
        .into_iter()
        .find(|c| c["name"] == "family-ohlc-volume")
        .unwrap();
    for axis in case["chart"]["definition"]["axes"].as_array_mut().unwrap() {
        axis["visible"] = serde_json::json!(false);
    }
    let mut session =
        chart_core::portable::Session::new(&case["chart"].to_string(), &case["data"].to_string())
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
    let chart = Arc::new(layout(session.prepare().unwrap(), &request, &Metrics).unwrap());
    let (from, to) = chart
        .scene()
        .items()
        .iter()
        .find_map(|item| match item.primitive {
            chart_core::scene::Primitive::Rule { from, to, .. }
                if item.layer == Some(LayerId::new(1)) =>
            {
                Some((from, to))
            }
            _ => None,
        })
        .unwrap();
    let p = Point::new(from.x(), from.y() + 0.9 * (to.y() - from.y())).unwrap();
    let mut inspector = Inspector::new(chart, 3., 16).unwrap();
    let occluded = Point::new(from.x(), from.y() + 0.1 * (to.y() - from.y())).unwrap();
    assert_eq!(
        inspector.query(occluded, InspectionMode::Containment).hits[0].layer,
        LayerId::new(2)
    );
    let hits = inspector.query(p, InspectionMode::Containment).hits;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].layer, LayerId::new(1));
    assert_eq!(key(&hits[0].target), 9007199254743001);
    let mut targets = std::collections::BTreeSet::new();
    for _ in 0..9 {
        inspector
            .dispatch(
                inspector.presented().scene().stamp(),
                InspectionAction::StepFocus { forward: true },
                InputOrigin::Keyboard,
            )
            .unwrap();
        assert!(
            targets.insert(chart_core::state::MarkTarget::from_inspected(
                &inspector.hits()[0],
                SourceEpoch::new(1)
            ))
        );
    }
    inspector
        .dispatch(
            inspector.presented().scene().stamp(),
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard,
        )
        .unwrap();
    assert_eq!(key(&inspector.hits()[0].target), 9007199254743001);

    // The first wick's center is outside the plot, but half its stroke remains visible.
    case["chart"]["definition"]["layers"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    case["chart"]["definition"]["axes"][0]["viewport"] =
        serde_json::to_value(Bounds::new(0.002, 4.002).unwrap()).unwrap();
    let mut session =
        chart_core::portable::Session::new(&case["chart"].to_string(), &case["data"].to_string())
            .unwrap();
    let chart = Arc::new(layout(session.prepare().unwrap(), &request, &Metrics).unwrap());
    let (from, to) = chart
        .scene()
        .items()
        .iter()
        .find_map(|item| match item.primitive {
            chart_core::scene::Primitive::Rule { from, to, .. }
                if item.layer == Some(LayerId::new(1)) =>
            {
                Some((from, to))
            }
            _ => None,
        })
        .unwrap();
    assert!((from.x() + 0.2).abs() < 1e-10);
    let inspector = Inspector::new(chart, 3., 16).unwrap();
    let p = Point::new(0.1, from.y() + 0.9 * (to.y() - from.y())).unwrap();
    let hit = inspector.query(p, InspectionMode::Containment);
    assert_eq!(hit.hits.len(), 1);
    assert_eq!(key(&hit.hits[0].target), 9007199254743001);
    assert!(
        inspector
            .query(
                Point::new(-0.1, p.y()).unwrap(),
                InspectionMode::Containment
            )
            .hits
            .is_empty()
    );
}
