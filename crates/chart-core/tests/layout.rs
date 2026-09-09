//! WP-06: destination geometry, measured layout and upstream semantic separation.
use chart_core::data::*;
use chart_core::grammar::*;
use chart_core::layout::*;
use chart_core::provenance::Target;
use chart_core::scales::*;
use chart_core::scene::Primitive;
use chart_core::services::*;
use chart_core::state::*;
use chart_core::transaction::*;
use chart_core::*;
use std::{cell::Cell, sync::Arc};

const DATA: DatasetId = DatasetId::new(10);
const X: FieldId = FieldId::new(1);
const Y: FieldId = FieldId::new(2);
const L: LayerId = LayerId::new(5);
const XS: ScaleId = ScaleId::new(0);
const YS: ScaleId = ScaleId::new(1);
const FONT: ResourceDescriptor = ResourceDescriptor {
    id: ResourceId::new(99),
    revision: Revision::new(7),
    kind: ResourceKind::Font,
    byte_len: 123,
};
struct Metrics {
    width: f64,
    calls: Cell<usize>,
    units: Units,
    fail: bool,
}
impl Metrics {
    fn new(width: f64) -> Self {
        Self {
            width,
            calls: Cell::new(0),
            units: Units::Points,
            fail: false,
        }
    }
}
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(*r.font, FONT);
        assert_eq!(r.units, self.units);
        assert_eq!(r.font_size, 12.);
        if self.fail {
            return Err(Diagnostic::error(
                DiagnosticCode::MissingResource,
                "Fixture font unavailable",
                "Supply the exact font.",
            ));
        }
        TextMetrics::new(self.width * r.text.chars().count() as f64, 8., 2.)
    }
}
fn batch(
    xs: ColumnValues,
    xkind: FieldKind,
    ys: &[f64],
    keys: Option<Vec<RowKey>>,
) -> NormalizedBatch {
    let fields = vec![
        Field {
            id: X,
            name: "x".into(),
            kind: xkind,
            nullable: true,
            unit: None,
            label: None,
        },
        Field {
            id: Y,
            name: "y".into(),
            kind: FieldKind::Float64,
            nullable: true,
            unit: None,
            label: None,
        },
    ];
    NormalizedBatch::new(
        Arc::new(Schema::new(SchemaVersion::new(1), fields).unwrap()),
        keys.unwrap_or_else(|| (0..ys.len()).map(|i| RowKey::new(i as u64 + 1)).collect()),
        vec![
            Column::new(xs, vec![true; ys.len()], None),
            Column::new(
                ColumnValues::Float64(ys.to_vec()),
                vec![true; ys.len()],
                None,
            ),
        ],
        DataLimits::default(),
    )
    .unwrap()
}
fn source(xs: &[f64], ys: &[f64]) -> DataStore {
    DataStore::new(
        SourceEpoch::new(1),
        vec![(
            DATA,
            batch(
                ColumnValues::Float64(xs.to_vec()),
                FieldKind::Float64,
                ys,
                None,
            ),
        )],
        DataLimits::default(),
    )
    .unwrap()
}
fn points() -> ChartDefinition {
    ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        L,
        DATA,
        Geom::Point,
        SourceAes::new().x(X).y(Y),
    ))
}
fn prepare(d: &ChartDefinition, s: &DataStore, state: &ChartState) -> Arc<PreparedChart> {
    Arc::new(
        Compiler::new()
            .prepare(d, &s.snapshot(), state, CompileLimits::default())
            .unwrap(),
    )
}
fn request() -> LayoutRequest {
    LayoutRequest::new(Rect::new(0., 0., 400., 240.).unwrap(), Units::Points, FONT)
}
fn fixed() -> LayoutRequest {
    let mut r = request();
    r.padding = 0.;
    for a in &mut r.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 10.).unwrap()));
    }
    r
}
fn centers(c: &LaidOutChart) -> Vec<Point> {
    c.scene()
        .items()
        .iter()
        .filter_map(|i| {
            if i.layer.is_some() {
                if let Primitive::Point { center, .. } = i.primitive {
                    Some(center)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-10, "{a} != {b}");
}

#[test]
fn known_linear_geometry_clips_targets_ranges_and_inverse_share_one_scene() {
    let p = prepare(
        &points(),
        &source(&[0., 5., 10.], &[0., 5., 10.]),
        &ChartState::default(),
    );
    let r = fixed();
    let out = layout(p.clone(), &r, &Metrics::new(5.)).unwrap();
    assert_eq!(out.status(), LayoutStatus::Ready);
    assert!(Arc::ptr_eq(out.prepared(), &p));
    assert_eq!(out.passes(), 1);
    assert_eq!(
        centers(&out),
        vec![
            Point::new(0., 240.).unwrap(),
            Point::new(200., 120.).unwrap(),
            Point::new(400., 0.).unwrap()
        ]
    );
    assert!(out.scene().items().iter().all(|i| i.clip == out.plot()));
    assert_eq!(out.targets().len(), out.scene().items().len());
    assert!(
        matches!(out.targets()[1][0],Target::Source(s) if s.key==RowKey::new(2)&&s.dataset==DATA)
    );
    let ResolvedScale::Linear(x) = &out.axes()[&XS].scale else {
        panic!("linear")
    };
    near(x.invert(200.).unwrap(), 5.);
    assert_eq!(out.scene().resources(), &[FONT]);
    assert_eq!(out.scene().units(), Units::Points);
    let mut reverse = r.clone();
    reverse.axes[0].range = Some(Bounds::new(400., 0.).unwrap());
    assert_eq!(
        centers(&layout(p, &reverse, &Metrics::new(5.)).unwrap())[0].x(),
        400.
    );
}

#[test]
fn destination_text_changes_margins_without_recomputing_statistics() {
    let p = prepare(
        &points(),
        &source(&[0., 10.], &[1000., 2000.]),
        &ChartState::default(),
    );
    let r = request();
    let narrow = Metrics::new(4.);
    let wide = Metrics::new(11.);
    let a = layout(p.clone(), &r, &narrow).unwrap();
    let b = layout(p.clone(), &r, &wide).unwrap();
    assert!(b.plot().unwrap().origin().x() > a.plot().unwrap().origin().x());
    assert!(b.plot().unwrap().width() < a.plot().unwrap().width());
    assert_ne!(centers(&a), centers(&b));
    assert!(Arc::ptr_eq(a.prepared(), b.prepared()));
    assert_eq!(
        a.prepared().layers()[0].table().operations(),
        b.prepared().layers()[0].table().operations()
    );
    assert!(a.passes() <= MAX_LAYOUT_PASSES && b.passes() <= MAX_LAYOUT_PASSES);
    assert!(narrow.calls.get() <= 4 * 2 * r.max_ticks);
    assert!(wide.calls.get() <= 4 * 2 * r.max_ticks);
    assert!(
        a.targets()
            .iter()
            .zip(a.scene().items())
            .all(|(t, i)| i.layer.is_some() || t.is_empty())
    );
    for item in b.scene().items() {
        if let Primitive::Text {
            font, font_size, ..
        } = item.primitive
        {
            assert_eq!(font, FONT.id);
            assert_eq!(font_size, 12.);
        }
    }
    let mut pixels = r.clone();
    pixels.units = Units::LogicalPixels;
    let mut m = Metrics::new(4.);
    m.units = Units::LogicalPixels;
    layout(p, &pixels, &m).unwrap();
}

#[test]
fn zoom_keeps_histogram_edges_counts_membership_domains_and_snapshot() {
    let s = source(&[0., 0.5, 1., 2.], &[0.; 4]);
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        L,
        DATA,
        BinSpec::new(X, vec![0., 1., 2.]),
    ));
    let mut state = ChartState::default();
    let mut compiler = Compiler::new();
    let p = Arc::new(
        compiler
            .prepare(&d, &s.snapshot(), &state, CompileLimits::default())
            .unwrap(),
    );
    state
        .apply(
            &d,
            ChartAction::SetViewport(Viewport {
                x: Some((0.5, 1.5)),
                y: None,
            }),
        )
        .unwrap();
    let zoom = Arc::new(
        compiler
            .prepare(&d, &s.snapshot(), &state, CompileLimits::default())
            .unwrap(),
    );
    let mut r = request();
    for a in &mut r.axes {
        a.visible = false;
    }
    r.padding = 0.;
    let a = layout(p.clone(), &r, &Metrics::new(5.)).unwrap();
    let b = layout(zoom.clone(), &r, &Metrics::new(5.)).unwrap();
    assert_eq!(
        p.layers()[0].table().rows(),
        zoom.layers()[0].table().rows()
    );
    assert_eq!(
        p.layers()[0].table().operations(),
        zoom.layers()[0].table().operations()
    );
    assert_eq!(p.domains(), zoom.domains());
    assert_eq!(a.targets(), b.targets());
    assert_ne!(a.scene().items(), b.scene().items());
    let Primitive::Rectangle { bounds, .. } = a.scene().items()[0].primitive else {
        panic!("histogram rectangle")
    };
    assert_eq!(bounds, Rect::new(0., 0., 200., 240.).unwrap());
    let Primitive::Rectangle { bounds, .. } = b.scene().items()[0].primitive else {
        panic!("histogram rectangle")
    };
    assert_eq!(bounds, Rect::new(-200., 0., 400., 240.).unwrap());
    assert_eq!(b.scene().items()[0].clip, b.plot());
    assert_eq!(b.scene().stamp().viewport, Revision::new(1));
}

#[test]
fn outside_policies_split_lines_and_annotation_overflow_is_explicit() {
    let s = source(&[1., 20., 2.], &[1., 2., 3.]);
    let mut d = points();
    d.layers[0].geom = Geom::Line {
        order: LineOrder::Authored,
        connect_gaps: false,
    };
    let p = prepare(&d, &s, &ChartState::default());
    let mut r = fixed();
    r.axes[0].outside = OutsidePolicy::Omit;
    let out = layout(p.clone(), &r, &Metrics::new(5.)).unwrap();
    assert_eq!(centers(&out).len(), 2);
    assert_eq!(
        out.targets().iter().map(Vec::len).collect::<Vec<_>>(),
        vec![1, 1]
    );
    assert!(
        out.diagnostics()
            .iter()
            .any(|d| d.message.contains("omitted 1"))
    );
    r.axes[0].outside = OutsidePolicy::Clamp;
    let clamped = layout(p, &r, &Metrics::new(5.)).unwrap();
    let Primitive::Path { commands, .. } = &clamped.scene().items()[0].primitive else {
        panic!("path")
    };
    assert!(matches!(commands[1],chart_core::scene::PathCommand::LineTo(p) if p.x()==400.));
    d.layers[0].clip = ClipPolicy::Figure;
    r.padding = 20.;
    r.axes[0].outside = OutsidePolicy::Extend;
    let overflow = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    assert_eq!(overflow.scene().items()[0].clip, Some(r.bounds));
    assert_ne!(Some(r.bounds), overflow.plot());
}

#[test]
fn independent_named_axes_accept_distinct_calculation_spaces_and_hidden_layers_train() {
    let s = source(&[0., 10.], &[1., 2.]);
    let mut d = points();
    d.layers.push(
        Layer::new(
            LayerId::new(6),
            DATA,
            Geom::Point,
            SourceAes::new().x(X).y(Numeric::Literal(1000.)),
        )
        .scaled(XS, ScaleId::new(44)),
    );
    let mut r = request();
    r.axes
        .push(AxisSpec::new(ScaleId::new(44), AxisSide::Right));
    let mut state = ChartState::default();
    state
        .apply(
            &d,
            ChartAction::SetLayerVisible {
                layer: LayerId::new(6),
                visible: false,
            },
        )
        .unwrap();
    let out = layout(prepare(&d, &s, &state), &r, &Metrics::new(5.)).unwrap();
    assert_eq!(centers(&out).len(), 2);
    let ResolvedScale::Linear(right) = &out.axes()[&ScaleId::new(44)].scale else {
        panic!("right linear")
    };
    assert_eq!(right.domain(), Bounds::new(950., 1050.).unwrap());
    let ResolvedScale::Linear(left) = &out.axes()[&YS].scale else {
        panic!("left linear")
    };
    assert_eq!(left.domain(), Bounds::new(1., 2.).unwrap());
    r.axes.pop();
    let m = Metrics::new(5.);
    assert_eq!(
        layout(prepare(&d, &s, &state), &r, &m).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    assert_eq!(m.calls.get(), 0);
    d.layers[1].scales.x = YS;
    assert_eq!(
        Compiler::new()
            .prepare(&d, &s.snapshot(), &state, CompileLimits::default())
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
}

#[test]
fn tiny_empty_and_empty_histogram_have_controlled_states_and_finite_geometry() {
    let p = prepare(&points(), &source(&[], &[]), &ChartState::default());
    let out = layout(p.clone(), &request(), &Metrics::new(5.)).unwrap();
    assert_eq!(out.status(), LayoutStatus::NoData);
    assert!(
        out.scene()
            .items()
            .iter()
            .any(|i| matches!(&i.primitive,Primitive::Text{text,..}if text=="No data"))
    );
    for a in out.axes().values() {
        let ResolvedScale::Linear(s) = &a.scale else {
            panic!("neutral linear")
        };
        assert_eq!(s.domain(), Bounds::new(0., 1.).unwrap());
    }
    let mut tiny = request();
    tiny.bounds = Rect::new(0., 0., 1., 1.).unwrap();
    let out = layout(p, &tiny, &Metrics::new(5.)).unwrap();
    assert_eq!(out.status(), LayoutStatus::NoSpace);
    assert!(out.plot().is_none());
    assert!(out.scene().items().is_empty());
    assert_eq!(
        out.diagnostics().last().unwrap().code,
        DiagnosticCode::LayoutPressure
    );
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        L,
        DATA,
        BinSpec::new(X, vec![0., 1., 2.]),
    ));
    assert_eq!(
        layout(
            prepare(&d, &source(&[], &[]), &ChartState::default()),
            &request(),
            &Metrics::new(5.)
        )
        .unwrap()
        .status(),
        LayoutStatus::NoData
    );
    let out = layout(
        prepare(&points(), &source(&[1.], &[1.]), &ChartState::default()),
        &request(),
        &Metrics::new(1000.),
    )
    .unwrap();
    assert_eq!(out.status(), LayoutStatus::NoSpace);
    assert!(out.passes() <= 4);
}

#[test]
fn resource_and_work_errors_precede_callbacks_and_preserve_previous_scene() {
    let p = prepare(
        &points(),
        &source(&[0., 10.], &[0., 10.]),
        &ChartState::default(),
    );
    let r = request();
    let old = layout(p.clone(), &r, &Metrics::new(5.)).unwrap();
    let mut limited = r.clone();
    limited.max_vertices = 1;
    let m = Metrics::new(5.);
    assert_eq!(
        layout(p.clone(), &limited, &m).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(m.calls.get(), 0);
    limited = r.clone();
    limited.limits.max_text_bytes = 0;
    assert_eq!(
        layout(p.clone(), &limited, &m).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(m.calls.get(), 0);
    limited = r.clone();
    limited.axes.push(limited.axes[0].clone());
    assert_eq!(
        layout(p.clone(), &limited, &m).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    assert_eq!(m.calls.get(), 0);
    let mut failed = Metrics::new(5.);
    failed.fail = true;
    let e = layout(p, &r, &failed).unwrap_err();
    assert_eq!(e.code, DiagnosticCode::MissingResource);
    assert_eq!(e.context.resource, Some(FONT.id));
    assert_eq!(e.context.resource_revision, Some(FONT.revision));
    assert_eq!(old.status(), LayoutStatus::Ready);
    assert_eq!(centers(&old).len(), 2);
}

#[test]
fn crowded_labels_are_thinned_deterministically_with_a_pressure_diagnostic() {
    let p = prepare(
        &points(),
        &source(&[0., 1.], &[0., 1.]),
        &ChartState::default(),
    );
    let mut r = request();
    r.bounds = Rect::new(0., 0., 180., 140.).unwrap();
    r.target_ticks = 50;
    let a = layout(p.clone(), &r, &Metrics::new(5.)).unwrap();
    let b = layout(p, &r, &Metrics::new(5.)).unwrap();
    assert_eq!(a.status(), LayoutStatus::Ready);
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::LayoutPressure)
    );
    assert_eq!(a.scene().items(), b.scene().items());
    assert!(a.axes()[&XS].ticks.len() < 50);
    assert!(a.passes() <= 4);
    assert!(
        a.axes()[&XS]
            .ticks
            .windows(2)
            .all(|w| w[0].position < w[1].position && w[0].label != w[1].label)
    );
}

#[test]
fn utc_large_origin_scene_uses_integer_inverse_and_calendar_labels() {
    let origin = 1_709_164_800_000_000_000_i64;
    let repr = TimestampType {
        unit: TimeUnit::Nanoseconds,
        timezone: "UTC".into(),
    };
    let b = batch(
        ColumnValues::Timestamp(vec![origin, origin + 17, origin + 257]),
        FieldKind::Timestamp(repr),
        &[0., 5., 10.],
        None,
    );
    let s = DataStore::new(SourceEpoch::new(1), vec![(DATA, b)], DataLimits::default()).unwrap();
    let mut d = points();
    d.layers[0].mappings = Mappings::Source(
        SourceAes::new()
            .x(Numeric::Timestamp { field: X, origin })
            .y(Y),
    );
    let p = prepare(&d, &s, &ChartState::default());
    let mut r = request();
    r.axes[0].scale = AxisScale::Utc {
        domain: None,
        interval: Some(UtcInterval::Ticks(100)),
    };
    let out = layout(p, &r, &Metrics::new(2.)).unwrap();
    let ResolvedScale::Utc(x) = &out.axes()[&XS].scale else {
        panic!("UTC")
    };
    assert_eq!(
        x.domain(),
        TimeBounds {
            start: origin,
            end: origin + 257
        }
    );
    for (center, expected) in centers(&out)
        .iter()
        .zip([origin, origin + 17, origin + 257])
    {
        assert!((i128::from(x.invert(center.x()).unwrap()) - i128::from(expected)).abs() <= 1);
    }
    assert!(
        out.axes()[&XS]
            .ticks
            .iter()
            .all(|t| t.label.contains("2024-02-29") && t.label.ends_with('Z'))
    );
    assert!(
        out.prepared()
            .source()
            .get()
            .unwrap()
            .dataset(DATA)
            .unwrap()
            .rows()
            .next()
            .unwrap()
            .value(X)
            .is_some_and(|v| v == ValueRef::Timestamp(origin))
    );
}

fn categorical(
    codes: Vec<u32>,
    dictionary: &[&str],
    ys: &[f64],
    keys: Option<Vec<RowKey>>,
) -> NormalizedBatch {
    batch(
        ColumnValues::Categorical {
            codes,
            dictionary: dictionary.iter().map(|s| (*s).into()).collect(),
        },
        FieldKind::Categorical,
        ys,
        keys,
    )
}
fn cat_definition() -> ChartDefinition {
    let mut d = points();
    d.layers[0].mappings = Mappings::Source(SourceAes::new().x(Numeric::Category(X)).y(Y));
    d
}
fn replace(s: &mut DataStore, b: NormalizedBatch) {
    let snap = s.snapshot();
    let input = snap.get().unwrap();
    let transaction = Transaction {
        id: TransactionId::new(format!("replace-{}", input.revision().get())).unwrap(),
        epoch: input.epoch(),
        expected: vec![input.dataset(DATA).unwrap().version()],
        operations: vec![Operation {
            dataset: DATA,
            mutation: Mutation::ReplaceSnapshot(b),
        }],
    };
    assert!(matches!(s.apply(transaction), CommitOutcome::Applied(_)));
}

#[test]
fn category_label_projection_survives_codes_reordering_removal_and_explicit_domain() {
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(
            DATA,
            categorical(vec![1, 0, 1], &["A", "B"], &[1., 2., 3.], None),
        )],
        DataLimits::default(),
    )
    .unwrap();
    let d = cat_definition();
    let mut r = fixed();
    r.axes[0].scale = AxisScale::Band(BandOptions {
        domain: Some(vec!["A".into(), "B".into()]),
        inner_padding: 0.,
        outer_padding: 0.,
    });
    let old = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    assert_eq!(
        centers(&old).iter().map(|p| p.x()).collect::<Vec<_>>(),
        vec![300., 100., 300.]
    );
    let ResolvedScale::Band(band) = &old.axes()[&XS].scale else {
        panic!("band")
    };
    assert_eq!(band.category_at(100.).unwrap(), Some("A"));
    assert!(!band.capabilities().numeric_inverse);
    // Same labels under new dictionary codes and authored row order; stable row keys retained.
    replace(
        &mut s,
        categorical(
            vec![1, 0],
            &["B", "A"],
            &[2., 1.],
            Some(vec![RowKey::new(2), RowKey::new(1)]),
        ),
    );
    let current = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    assert_eq!(
        centers(&current).iter().map(|p| p.x()).collect::<Vec<_>>(),
        vec![100., 300.]
    );
    assert!(matches!(current.targets()[0][0],Target::Source(t)if t.key==RowKey::new(2)));
    assert_eq!(centers(&old).len(), 3);
    r.axes[0].scale = AxisScale::Band(BandOptions {
        domain: Some(vec!["A".into()]),
        inner_padding: 0.,
        outer_padding: 0.,
    });
    let omitted = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    assert_eq!(centers(&omitted).len(), 1);
    assert_eq!(omitted.prepared().layers()[0].table().rows().len(), 2);
    assert!(
        omitted
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("omitted 1"))
    );
}

#[test]
fn automatic_category_training_is_eligible_and_keeps_retained_relative_order() {
    let mut s = DataStore::new(
        SourceEpoch::new(1),
        vec![(
            DATA,
            categorical(vec![1, 0, 2], &["A", "B", "C"], &[1., 2., 3.], None),
        )],
        DataLimits::default(),
    )
    .unwrap();
    let mut d = cat_definition();
    let mut r = fixed();
    r.axes[0].scale = AxisScale::Auto;
    let a = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    let ResolvedScale::Band(b) = &a.axes()[&XS].scale else {
        panic!("band")
    };
    assert_eq!(b.domain(), &["B", "A", "C"]);
    d.layers[0].filters.push(SourceFilter {
        value: Y.into(),
        minimum: Some(2.),
        maximum: None,
    });
    let filtered = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    let ResolvedScale::Band(b) = &filtered.axes()[&XS].scale else {
        panic!("band")
    };
    assert_eq!(b.domain(), &["A", "C"]);
    replace(&mut s, categorical(vec![], &[], &[], None));
    let empty = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    let ResolvedScale::Band(b) = &empty.axes()[&XS].scale else {
        panic!("band")
    };
    assert!(b.domain().is_empty());
    assert_eq!(empty.status(), LayoutStatus::NoData);
    replace(
        &mut s,
        categorical(vec![0, 1], &["C", "B"], &[3., 2.], None),
    );
    let again = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    let ResolvedScale::Band(b) = &again.axes()[&XS].scale else {
        panic!("band")
    };
    assert_eq!(b.domain(), &["B", "C"]);
}

#[test]
fn category_catalogs_merge_by_label_across_layers_and_reject_numeric_statistics() {
    let other = DatasetId::new(11);
    let s = DataStore::new(
        SourceEpoch::new(1),
        vec![
            (DATA, categorical(vec![0, 1], &["A", "B"], &[1., 2.], None)),
            (other, categorical(vec![0, 1], &["B", "C"], &[3., 4.], None)),
        ],
        DataLimits::default(),
    )
    .unwrap();
    let mut d = cat_definition();
    d.layers.push(Layer::new(
        LayerId::new(6),
        other,
        Geom::Point,
        SourceAes::new().x(Numeric::Category(X)).y(Y),
    ));
    let mut r = fixed();
    r.axes[0].scale = AxisScale::Band(BandOptions {
        inner_padding: 0.,
        outer_padding: 0.,
        domain: None,
    });
    let out = layout(
        prepare(&d, &s, &ChartState::default()),
        &r,
        &Metrics::new(5.),
    )
    .unwrap();
    let c = centers(&out);
    near(c[0].x(), 400. / 6.);
    near(c[1].x(), 200.);
    near(c[2].x(), 200.);
    near(c[3].x(), 400. * 5. / 6.);
    d.layers[0].statistic = Statistic::bin(BinSpec::new(Numeric::Category(X), vec![0., 1., 2.]));
    assert_eq!(
        Compiler::new()
            .prepare(
                &d,
                &s.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
}

#[test]
fn visibility_has_its_own_scene_stamp_and_does_not_change_trained_domains() {
    let d = points();
    let s = source(&[0., 10.], &[0., 10.]);
    let mut state = ChartState::default();
    let a = layout(prepare(&d, &s, &state), &request(), &Metrics::new(5.)).unwrap();
    state
        .apply(
            &d,
            ChartAction::SetLayerVisible {
                layer: L,
                visible: false,
            },
        )
        .unwrap();
    let b = layout(prepare(&d, &s, &state), &request(), &Metrics::new(5.)).unwrap();
    assert_ne!(a.scene().stamp().state, b.scene().stamp().state);
    assert_eq!(a.scene().stamp().viewport, b.scene().stamp().viewport);
    assert_eq!(a.prepared().domains(), b.prepared().domains());
    assert_eq!(b.status(), LayoutStatus::NoData);
}

#[test]
fn invalid_scale_options_reject_even_at_tiny_bounds_before_measurement() {
    let p = prepare(
        &points(),
        &source(&[0., 10.], &[0., 10.]),
        &ChartState::default(),
    );
    let mut r = request();
    r.bounds = Rect::new(0., 0., 1., 1.).unwrap();
    r.axes[0].scale = AxisScale::Linear(ContinuousDomain {
        padding: f64::NAN,
        ..Default::default()
    });
    let m = Metrics::new(5.);
    assert_eq!(
        layout(p.clone(), &r, &m).unwrap_err().code,
        DiagnosticCode::NumericalDomain
    );
    assert_eq!(m.calls.get(), 0);
    r.axes[0].scale = AxisScale::Band(BandOptions::default());
    assert_eq!(
        layout(p, &r, &m).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    assert_eq!(m.calls.get(), 0);
}

#[test]
fn representable_narrow_numeric_domains_have_finite_scene_and_distinct_labels() {
    let p = prepare(
        &points(),
        &source(&[1e16, 1e16 + 2., 1e16 + 4.], &[0., 5., 10.]),
        &ChartState::default(),
    );
    let mut r = request();
    r.bounds = Rect::new(0., 0., 1000., 300.).unwrap();
    let out = layout(p, &r, &Metrics::new(2.)).unwrap();
    assert_eq!(out.status(), LayoutStatus::Ready);
    assert_eq!(centers(&out).len(), 3);
    assert!(
        out.axes()[&XS]
            .ticks
            .windows(2)
            .all(|p| p[0].label != p[1].label)
    );
    let ResolvedScale::Linear(s) = &out.axes()[&XS].scale else {
        panic!("linear")
    };
    for (p, x) in centers(&out).iter().zip([1e16, 1e16 + 2., 1e16 + 4.]) {
        assert!((s.invert(p.x()).unwrap() - x).abs() <= 2.);
    }
}

#[test]
fn projection_overflow_and_category_or_guide_budgets_fail_without_replacing_a_scene() {
    let previous = layout(
        prepare(
            &points(),
            &source(&[0., 10.], &[0., 10.]),
            &ChartState::default(),
        ),
        &fixed(),
        &Metrics::new(5.),
    )
    .unwrap();
    let p = prepare(
        &points(),
        &source(&[0., f64::MAX], &[0., 10.]),
        &ChartState::default(),
    );
    let e = layout(p, &fixed(), &Metrics::new(5.)).unwrap_err();
    assert_eq!(e.code, DiagnosticCode::PrecisionLoss);
    assert_eq!(e.context.stamp.unwrap().definition, Revision::new(1));
    assert_eq!(centers(&previous).len(), 2);
    let s = DataStore::new(
        SourceEpoch::new(1),
        vec![(
            DATA,
            categorical(vec![0, 1], &["alpha", "beta"], &[1., 2.], None),
        )],
        DataLimits::default(),
    )
    .unwrap();
    let p = prepare(&cat_definition(), &s, &ChartState::default());
    let m = Metrics::new(5.);
    let mut r = request();
    r.max_categories = 1;
    assert_eq!(
        layout(p.clone(), &r, &m).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(m.calls.get(), 0);
    r = request();
    r.limits.max_text_bytes = 4;
    assert_eq!(
        layout(p.clone(), &r, &m).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(m.calls.get(), 0);
    r = request();
    r.limits.max_items = 2;
    assert_eq!(
        layout(p, &r, &m).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(m.calls.get(), 0);
}

#[test]
fn wp11_portable_log_axis_and_secondary_units_use_shared_projection() {
    let s = source(&[-1., 1., 10., 100.], &[0., 10., 20., 30.]);
    let mut log = AxisSpec::new(XS, AxisSide::Bottom);
    // Isolate unit-guide alignment from separately tested cross-axis corner-label thinning.
    log.visible = false;
    log.scale = AxisScale::Nonlinear {
        transform: ScaleTransform::Log { base: 10. },
        domain: ContinuousDomain::default(),
    };
    let mut secondary = AxisSpec::new(ScaleId::new(8), AxisSide::Right);
    secondary.scale = AxisScale::Secondary {
        source: YS,
        factor: 1.8,
        offset: 32.,
    };
    let d = points()
        .axis(log)
        .axis(AxisSpec::new(YS, AxisSide::Left))
        .axis(secondary);
    let roundtrip: ChartDefinition =
        serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(roundtrip, d);
    let prepared = prepare(&d, &s, &ChartState::default());
    let laid = layout(prepared.clone(), &request(), &Metrics::new(4.)).unwrap();
    let ResolvedScale::Nonlinear(log) = &laid.axes()[&XS].scale else {
        panic!("log")
    };
    assert!((log.domain().start() - 1.).abs() < 1e-12);
    assert!((log.domain().end() - 100.).abs() < 1e-10);
    assert_eq!(prepared.layers()[0].marks().len(), 4);
    assert_eq!(
        laid.scene()
            .items()
            .iter()
            .filter(|i| i.layer == Some(L))
            .count(),
        3
    );
    let ResolvedScale::Secondary { domain, .. } = &laid.axes()[&ScaleId::new(8)].scale else {
        panic!("secondary")
    };
    assert_eq!(*domain, Bounds::new(32., 86.).unwrap());
    assert_eq!(
        laid.axes()[&YS]
            .ticks
            .iter()
            .map(|t| t.position)
            .collect::<Vec<_>>(),
        laid.axes()[&ScaleId::new(8)]
            .ticks
            .iter()
            .map(|t| t.position)
            .collect::<Vec<_>>()
    );
}

#[test]
fn wp11_area_and_ribbon_split_missing_and_validate_bounds() {
    let s = source(&[0., 1., 2., 3., 4.], &[1., 2., f64::NAN, 4., 3.]);
    let d = ChartDefinition::new(Revision::INITIAL).layer(Layer::new(
        L,
        DATA,
        Geom::area(),
        SourceAes::new().x(X).y(Y),
    ));
    let prepared = prepare(&d, &s, &ChartState::default());
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    assert_eq!(prepared.layers()[0].invalid_geometry(), 1);
    assert_eq!(prepared.layers()[0].domains().y.unwrap().minimum, 0.);
    let laid = layout(prepared, &request(), &Metrics::new(4.)).unwrap();
    assert_eq!(
        laid.scene()
            .items()
            .iter()
            .filter(|i| matches!(i.primitive, Primitive::FilledPath { .. }))
            .count(),
        2
    );
    let d = ChartDefinition::new(Revision::INITIAL).layer(Layer::new(
        L,
        DATA,
        Geom::ribbon(),
        SourceAes::new().x(X).y(Y).y2(Numeric::Literal(3.)),
    ));
    let prepared = prepare(&d, &s, &ChartState::default());
    assert_eq!(prepared.layers()[0].invalid_geometry(), 2);
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    let mut req = request();
    req.axes[1].scale = AxisScale::Nonlinear {
        transform: ScaleTransform::Log { base: 10. },
        domain: ContinuousDomain::default(),
    };
    assert!(
        layout(
            prepare(
                &ChartDefinition::new(Revision::INITIAL).layer(Layer::new(
                    L,
                    DATA,
                    Geom::area(),
                    SourceAes::new().x(X).y(Y)
                )),
                &s,
                &ChartState::default()
            ),
            &req,
            &Metrics::new(4.)
        )
        .is_err()
    );
}

#[test]
fn wp11_ohlc_validation_and_volume_invalidity_are_independent() {
    let s = source(&[0., 1., 2., 3.], &[2., 3., 8., -1.]);
    let price = Layer::ohlc(
        L,
        DATA,
        SourceAes::new()
            .x(X)
            .y(Y)
            .y2(Numeric::Literal(3.))
            .bounds(Numeric::Literal(1.), Numeric::Literal(5.)),
        8.,
    );
    let volume = Layer::volume(LayerId::new(6), DATA, X, Y, 8.).scaled(XS, ScaleId::new(8));
    let d = ChartDefinition::new(Revision::INITIAL)
        .layer(price)
        .layer(volume);
    let prepared = prepare(&d, &s, &ChartState::default());
    assert_eq!(prepared.layers()[0].invalid_geometry(), 2);
    assert_eq!(prepared.layers()[1].invalid_geometry(), 1);
    assert_eq!(prepared.layers()[0].marks().len(), 4);
    assert_eq!(prepared.layers()[1].marks().len(), 3);
    assert_eq!(
        prepared.layers()[0].domains().y.unwrap(),
        Extent {
            minimum: 1.,
            maximum: 5.
        }
    );
    let mut req = request();
    req.axes
        .push(AxisSpec::new(ScaleId::new(8), AxisSide::Right));
    let laid = layout(prepared, &req, &Metrics::new(4.)).unwrap();
    assert_eq!(
        laid.scene()
            .items()
            .iter()
            .filter(|i| i.layer == Some(L))
            .count(),
        4
    );
}

#[test]
fn wp11_mapped_color_changes_styles_and_metadata_without_numeric_domain_changes() {
    let s = source(&[0., 1., 2.], &[0., 5., 10.]);
    let d = points();
    let before = prepare(&d, &s, &ChartState::default());
    let mut d = d.clone();
    let black = chart_core::scene::Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    let white = chart_core::scene::Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };
    d.layers[0].color = Some(ColorEncoding {
        title: None,
        id: ScaleId::new(20),
        input: ColorInput::Numeric(Numeric::Field(Y)),
        scale: ColorScale::Continuous {
            domain: Bounds::new(0., 10.).unwrap(),
            palette: vec![black.into(), white.into()],
            clamp: true,
            missing: black.into(),
        },
    });
    let after = prepare(&d, &s, &ChartState::default());
    assert_eq!(before.scale_domains(), after.scale_domains());
    assert_eq!(after.layers()[0].marks()[1].style.color.red, 128);
    assert_eq!(after.layers()[0].color_legend().unwrap().entries.len(), 2);
    let limits = CompileLimits {
        max_groups: 1,
        ..Default::default()
    };
    assert!(
        Compiler::new()
            .prepare(&d, &s.snapshot(), &ChartState::default(), limits)
            .is_err()
    );
}
