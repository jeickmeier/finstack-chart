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
        SourceAes::new().x(X).y(Y),
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
