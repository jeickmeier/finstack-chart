//! WP-10 canonical FIX-02–05 and operation-specific boundary invariants.
use chart_core::data::*;
use chart_core::grammar::*;
use chart_core::layout::*;
use chart_core::provenance::ResolvedTarget;
use chart_core::services::*;
use chart_core::state::*;
use chart_core::transaction::*;
use chart_core::*;
use std::sync::Arc;
const DATA: DatasetId = DatasetId::new(1);
const X: FieldId = FieldId::new(1);
const Y: FieldId = FieldId::new(2);
const G: FieldId = FieldId::new(3);
const C: FieldId = FieldId::new(4);
const L: LayerId = LayerId::new(1);
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../fixtures/statistics/canonical.json")).unwrap()
}
fn near(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() <= tolerance,
        "{a:.17} != {b:.17} (tol {tolerance})"
    );
}
fn batch(rows: &[(u64, Option<f64>, Option<f64>, i64)]) -> NormalizedBatch {
    let fields = [
        (X, "x", FieldKind::Float64),
        (Y, "y", FieldKind::Float64),
        (G, "group", FieldKind::Int64),
        (C, "category", FieldKind::Categorical),
    ]
    .into_iter()
    .map(|(id, name, kind)| Field {
        id,
        name: name.into(),
        kind,
        nullable: true,
        unit: None,
        label: None,
    })
    .collect();
    let schema = Arc::new(Schema::new(SchemaVersion::new(1), fields).unwrap());
    let col = |x: bool| {
        Column::new(
            ColumnValues::Float64(
                rows.iter()
                    .map(|r| if x { r.1 } else { r.2 }.unwrap_or(999.))
                    .collect(),
            ),
            rows.iter()
                .map(|r| if x { r.1 } else { r.2 }.is_some())
                .collect(),
            None,
        )
    };
    NormalizedBatch::new(
        schema,
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        vec![
            col(true),
            col(false),
            Column::new(
                ColumnValues::Int64(rows.iter().map(|r| r.3).collect()),
                vec![true; rows.len()],
                None,
            ),
            Column::new(
                ColumnValues::Categorical {
                    codes: vec![0; rows.len()],
                    dictionary: vec!["A".into()],
                },
                vec![true; rows.len()],
                None,
            ),
        ],
        DataLimits::default(),
    )
    .unwrap()
}
fn store(rows: &[(u64, Option<f64>, Option<f64>, i64)]) -> DataStore {
    DataStore::new(
        SourceEpoch::new(1),
        vec![(DATA, batch(rows))],
        DataLimits::default(),
    )
    .unwrap()
}
fn points(values: &[f64]) -> DataStore {
    store(
        &values
            .iter()
            .enumerate()
            .map(|(i, v)| (i as u64 + 1, Some(*v), Some(*v), i as i64))
            .collect::<Vec<_>>(),
    )
}
fn def(layer: Layer) -> ChartDefinition {
    ChartDefinition::new(Revision::new(1)).layer(layer)
}
fn prepare(d: &ChartDefinition, s: &DataStore) -> ChartResult<PreparedChart> {
    Compiler::new().prepare(
        d,
        &s.snapshot(),
        &ChartState::default(),
        CompileLimits::default(),
    )
}
fn rows(p: &PreparedChart) -> &[StatisticalRow] {
    let PreparedRows::Statistical(r) = p.layers()[0].table().rows() else {
        panic!("stat rows")
    };
    r
}
fn bins(p: &PreparedChart) -> &[BinnedRow] {
    let PreparedRows::Binned(r) = p.layers()[0].table().rows() else {
        panic!("bins")
    };
    r
}
fn summary(spec: SummarySpec) -> Layer {
    Layer::statistical(
        L,
        DATA,
        Statistic::summary(spec),
        Geom::Point,
        StatAes::new(StatField::Group, StatField::Mean),
    )
}
fn counts(spec: CountSpec) -> Layer {
    Layer::statistical(
        L,
        DATA,
        Statistic::count(spec),
        Geom::Point,
        StatAes::new(StatField::Group, StatField::Count),
    )
}
fn values(v: &serde_json::Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect()
}
#[test]
fn fix02_explicit_auto_constant_empty_and_overflow_bin_conservation() {
    let f = fixture();
    let values = values(&f["bin"]["values"]);
    let s = points(&values);
    let mut spec = BinSpec::new(X, vec![0., 1., 2.]);
    let p = prepare(&def(Layer::histogram(L, DATA, spec.clone())), &s).unwrap();
    assert_eq!(
        bins(&p).iter().map(|b| b.count).collect::<Vec<_>>(),
        vec![2, 2]
    );
    let outside = points(&[-2., 0., 0.5, 1., 2., 9.]);
    spec.outliers = OutlierPolicy::Overflow;
    let p = prepare(&def(Layer::histogram(L, DATA, spec.clone())), &outside).unwrap();
    assert_eq!(
        bins(&p).iter().map(|b| b.count).collect::<Vec<_>>(),
        vec![3, 3]
    );
    assert_eq!(p.layers()[0].table().operations()[0].counts.below, 1);
    assert_eq!(p.layers()[0].table().operations()[0].counts.above, 1);
    spec.outliers = OutlierPolicy::Error;
    assert!(prepare(&def(Layer::histogram(L, DATA, spec)), &outside).is_err());
    let automatic = |store: &DataStore| {
        let mut l = Layer::binned(L, DATA, Geom::Rectangle, BinAes::histogram());
        l.statistic = Statistic::auto_bin(AutoBinSpec::new(X));
        prepare(&def(l), store).unwrap()
    };
    let p = automatic(&s);
    assert_eq!(bins(&p).len(), 30);
    assert_eq!(bins(&p).iter().map(|b| b.count).sum::<u64>(), 4);
    assert_eq!(bins(&p)[0].start, 0.);
    assert_eq!(bins(&p)[29].end, 2.);
    let p = automatic(&points(&[10., 10.]));
    assert_eq!(bins(&p)[0].start, 9.5);
    assert_eq!(bins(&p)[29].end, 10.5);
    assert_eq!(bins(&p).iter().map(|b| b.count).sum::<u64>(), 2);
    let p = automatic(&points(&[]));
    assert_eq!(bins(&p)[0].start, 0.);
    assert_eq!(bins(&p)[29].end, 1.);
    assert!(bins(&p).iter().all(|b| b.count == 0));
}
#[test]
fn fix04_quantiles_stable_summary_missing_and_explicit_empty_sum() {
    let f = fixture();
    let mut spec = SummarySpec::new(X);
    spec.quantiles = values(&f["quantile"]["probabilities"]);
    let mut source = vec![
        (1, Some(0.), Some(0.), 0),
        (2, Some(10.), Some(0.), 0),
        (3, Some(20.), Some(0.), 0),
        (4, Some(30.), Some(0.), 0),
        (5, None, Some(0.), 0),
        (6, Some(f64::NAN), Some(0.), 0),
    ];
    let p = prepare(&def(summary(spec.clone())), &store(&source)).unwrap();
    let r = &rows(&p)[0];
    assert_eq!(r.count, 4);
    for (i, v) in values(&f["quantile"]["expected"]).iter().enumerate() {
        near(r.value(&StatField::Quantile(i)).unwrap(), *v, 1e-12);
    }
    assert_eq!(r.value(&StatField::Sum), Some(60.));
    assert_eq!(r.value(&StatField::Mean), Some(15.));
    assert_eq!(p.layers()[0].table().operations()[0].counts.invalid_stat, 2);
    assert_eq!(
        r.members.as_ref(),
        &[
            RowKey::new(1),
            RowKey::new(2),
            RowKey::new(3),
            RowKey::new(4)
        ]
    );
    source.reverse();
    let reordered = prepare(&def(summary(spec.clone())), &store(&source)).unwrap();
    assert_eq!(rows(&p), rows(&reordered));
    let p = prepare(&def(summary(spec.clone())), &points(&[1e16, 1., -1e16])).unwrap();
    assert_eq!(rows(&p)[0].value(&StatField::Sum), Some(1.));
    let p = prepare(&def(summary(spec.clone())), &points(&[])).unwrap();
    assert_eq!(rows(&p)[0].count, 0);
    assert!(rows(&p)[0].values.iter().all(|v| v.value.is_none()));
    spec.empty_sum_zero = true;
    let p = prepare(&def(summary(spec.clone())), &points(&[])).unwrap();
    assert_eq!(rows(&p)[0].value(&StatField::Sum), Some(0.));
    assert_eq!(rows(&p)[0].value(&StatField::Mean), None);
    spec.quantiles = vec![1.1];
    assert!(prepare(&def(summary(spec)), &points(&[0.])).is_err());
}
#[test]
fn count_requires_all_inputs_and_reports_invalid_strict_and_empty_groups() {
    let source = store(&[
        (1, Some(1.), Some(2.), 1),
        (2, None, Some(2.), 1),
        (3, Some(1.), Some(f64::INFINITY), 2),
    ]);
    let spec = CountSpec {
        required: vec![X.into(), Y.into()],
        grouping: Grouping::Field(G),
    };
    let p = prepare(&def(counts(spec.clone())), &source).unwrap();
    assert_eq!(
        rows(&p).iter().map(|r| r.count).collect::<Vec<_>>(),
        vec![1, 0]
    );
    assert_eq!(p.layers()[0].table().operations()[0].counts.invalid_stat, 2);
    let mut l = counts(spec);
    l.invalid = InvalidPolicy::Strict;
    assert!(prepare(&def(l), &source).is_err());
    let p = prepare(&def(counts(CountSpec::default())), &points(&[])).unwrap();
    assert_eq!(rows(&p)[0].count, 0);
}
#[test]
fn fix05_intercept_ols_zoom_filter_degenerate_and_large_origin() {
    let f = fixture();
    let source = store(
        &f["ols"]["points"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    i as u64 + 1,
                    Some(p[0].as_f64().unwrap()),
                    Some(p[1].as_f64().unwrap()),
                    0,
                )
            })
            .collect::<Vec<_>>(),
    );
    let layer = Layer::fit(L, DATA, OlsSpec::new(X, Y));
    let definition = def(layer.clone());
    let p = prepare(&definition, &source).unwrap();
    let r = &rows(&p)[0];
    near(r.value(&StatField::Intercept).unwrap(), 0.8, 1e-12);
    near(r.value(&StatField::Slope).unwrap(), 2.3, 1e-12);
    assert_eq!(r.members.len(), 4);
    assert!(matches!(
        r.target.resolve(source.snapshot().get().unwrap()).unwrap(),
        ResolvedTarget::Derived { .. }
    ));
    let mut state = ChartState::default();
    state
        .apply(
            &definition,
            ChartAction::SetViewport(Viewport {
                x: Some((0., 2.)),
                y: None,
            }),
        )
        .unwrap();
    let zoom = Compiler::new()
        .prepare(
            &definition,
            &source.snapshot(),
            &state,
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(rows(&p), rows(&zoom));
    let mut filtered = layer;
    filtered.filters = vec![SourceFilter {
        value: X.into(),
        minimum: None,
        maximum: Some(2.),
    }];
    let p = prepare(&def(filtered), &source).unwrap();
    let r = &rows(&p)[0];
    near(r.value(&StatField::Intercept).unwrap(), 1., 1e-12);
    near(r.value(&StatField::Slope).unwrap(), 2., 1e-12);
    assert_eq!(r.members.len(), 3);
    for data in [
        vec![],
        vec![(1, Some(1.), Some(2.), 0)],
        vec![(1, Some(1.), Some(2.), 0), (2, Some(1.), Some(3.), 0)],
    ] {
        assert!(prepare(&definition, &store(&data)).is_err());
    }
    let p = prepare(
        &definition,
        &store(&[
            (1, Some(1e12), Some(1.), 0),
            (2, Some(1e12 + 1.), Some(3.), 0),
            (3, Some(1e12 + 2.), Some(5.), 0),
        ]),
    )
    .unwrap();
    near(rows(&p)[0].value(&StatField::Slope).unwrap(), 2., 1e-12);
    near(rows(&p)[0].value(&StatField::Y).unwrap(), 1., 1e-12);
}
fn stack(normalize: bool) -> Layer {
    let mut l = Layer::new(
        L,
        DATA,
        Geom::Rectangle,
        SourceAes::new()
            .x(Numeric::Literal(0.))
            .x2(Numeric::Literal(1.))
            .y(Y)
            .y2(Numeric::Literal(0.))
            .group(G),
    );
    l.position = Position::Stack(StackSpec {
        order: (0..4).map(GroupValue::Int).collect(),
        normalize,
    });
    l
}
#[test]
fn fix03_mixed_sign_stack_normalize_and_reorder_domains() {
    let f = fixture();
    let v = values(&f["stack"]["values"]);
    let mut input: Vec<_> = v
        .iter()
        .enumerate()
        .map(|(i, v)| (i as u64 + 1, Some(0.), Some(*v), i as i64))
        .collect();
    for normalize in [false, true] {
        let p = prepare(&def(stack(normalize)), &store(&input)).unwrap();
        let (lower, upper) = if normalize {
            ("normalized_lower", "normalized_upper")
        } else {
            ("lower", "upper")
        };
        for (i, m) in p.layers()[0].marks().iter().enumerate() {
            let PreparedGeometry::Rectangle { from, to } = &m.geometry else {
                panic!()
            };
            near(from.y(), f["stack"][upper][i].as_f64().unwrap(), 1e-15);
            near(to.y(), f["stack"][lower][i].as_f64().unwrap(), 1e-15);
        }
        let d = p.domains().y.unwrap();
        assert_eq!(d.minimum, if normalize { -1. } else { -5. });
        assert_eq!(d.maximum, if normalize { 1. } else { 5. });
        input.reverse();
        let reverse = prepare(&def(stack(normalize)), &store(&input)).unwrap();
        for mark in p.layers()[0].marks() {
            let other = reverse.layers()[0]
                .marks()
                .iter()
                .find(|m| m.targets == mark.targets)
                .unwrap();
            assert_eq!(other.geometry, mark.geometry);
        }
        input.reverse();
    }
    let zero = store(&[(1, Some(0.), Some(0.), 0)]);
    let p = prepare(&def(stack(true)), &zero).unwrap();
    assert_eq!(p.domains().y.unwrap().maximum, 0.);
    let huge = store(&[
        (1, Some(0.), Some(f64::MAX), 0),
        (2, Some(0.), Some(f64::MAX), 1),
    ]);
    let p = prepare(&def(stack(true)), &huge).unwrap();
    assert_eq!(p.domains().y.unwrap().maximum, 1.);
    assert!(prepare(&def(stack(false)), &huge).is_err());
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 6., 9., 3.)
    }
}
fn layout_request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 400., 240.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::new(1),
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn centers(chart: &LaidOutChart) -> Vec<Point> {
    chart
        .scene()
        .items()
        .iter()
        .filter_map(|i| match i.primitive {
            scene::Primitive::Point { center, .. } if i.layer == Some(L) => Some(center),
            _ => None,
        })
        .collect()
}
#[test]
fn dodge_uses_band_width_and_preserves_missing_slots() {
    let s = store(&[(1, Some(0.), Some(1.), 0), (2, Some(0.), Some(2.), 2)]);
    let mut l = Layer::new(
        L,
        DATA,
        Geom::Point,
        SourceAes::new().x(Numeric::Category(C)).y(Y).group(G),
    );
    l.position = Position::Dodge(DodgeSpec {
        order: (0..3).map(GroupValue::Int).collect(),
        width: 0.9,
    });
    let p = Arc::new(prepare(&def(l.clone()), &s).unwrap());
    let laid = layout(p, &layout_request(), &Metrics).unwrap();
    let c = centers(&laid);
    let ResolvedScale::Band(b) = &laid.axes()[&ScaleId::new(0)].scale else {
        panic!()
    };
    let extent = b.extent("A").unwrap().unwrap();
    let width = (extent.end() - extent.start()) * 0.9;
    let center = b.center("A").unwrap().unwrap();
    near(c[0].x(), center - width / 3., 1e-10);
    near(c[1].x(), center + width / 3., 1e-10);
    l.position = Position::Dodge(DodgeSpec {
        order: vec![GroupValue::Int(0)],
        width: 0.9,
    });
    assert!(prepare(&def(l), &s).is_err());
}
#[test]
fn seeded_jitter_is_stable_in_data_and_display_units() {
    let mut source = vec![(99, Some(0.), Some(1.), 0), (7, Some(2.), Some(3.), 1)];
    for units in [JitterUnits::Data, JitterUnits::Display] {
        let mut l = Layer::new(L, DATA, Geom::Point, SourceAes::new().x(X).y(Y).group(G));
        l.position = Position::Jitter(JitterSpec {
            seed: u64::MAX,
            x: 0.25,
            y: 0.5,
            units,
        });
        let p = prepare(&def(l.clone()), &store(&source)).unwrap();
        source.reverse();
        let q = prepare(&def(l.clone()), &store(&source)).unwrap();
        for m in p.layers()[0].marks() {
            assert_eq!(
                m.geometry,
                q.layers()[0]
                    .marks()
                    .iter()
                    .find(|v| v.targets == m.targets)
                    .unwrap()
                    .geometry
            );
        }
        let a = layout(Arc::new(p), &layout_request(), &Metrics).unwrap();
        let b = layout(Arc::new(q), &layout_request(), &Metrics).unwrap();
        let mut other = centers(&b);
        other.reverse();
        assert_eq!(centers(&a), other);
        let mut plain = l.clone();
        plain.position = Position::Identity;
        let plain = prepare(&def(plain), &store(&source)).unwrap();
        let p = prepare(&def(l), &store(&source)).unwrap();
        if units == JitterUnits::Display {
            assert_eq!(p.domains(), plain.domains());
        } else {
            assert_ne!(p.domains(), plain.domains());
        }
        source.reverse();
    }
}
#[test]
fn transformed_stat_fields_are_not_transformed_again_and_mismatch_rejects() {
    let mut spec = SummarySpec::new(X);
    spec.space = StatSpace::Transformed(NumericTransform::affine(2., 10.));
    let s = points(&[0., 10.]);
    let p = prepare(&def(summary(spec)), &s).unwrap();
    assert_eq!(rows(&p)[0].value(&StatField::Mean), Some(20.));
    assert!(matches!(
        p.domains().y_space,
        Some(ValueSpace::Transformed { .. })
    ));
    let p = Arc::new(p);
    let mut req = layout_request();
    req.axes[1].scale = AxisScale::Linear(scales::ContinuousDomain {
        explicit: Some(scales::Bounds::new(0., 40.).unwrap()),
        ..Default::default()
    });
    let laid = layout(p, &req, &Metrics).unwrap();
    let y = centers(&laid)[0].y();
    let axis = &laid.axes()[&ScaleId::new(1)];
    let ResolvedScale::Linear(scale) = &axis.scale else {
        panic!()
    };
    assert_eq!(y, scale.map(20.).unwrap().unwrap());
    let mut fit = OlsSpec::new(X, Y);
    fit.x_space = StatSpace::Transformed(NumericTransform::affine(2., 10.));
    fit.y_space = StatSpace::Transformed(NumericTransform::affine(3., -4.));
    let p = prepare(&def(Layer::fit(L, DATA, fit)), &points(&[0., 1., 2.])).unwrap();
    near(rows(&p)[0].value(&StatField::Slope).unwrap(), 1.5, 1e-12);
    near(
        rows(&p)[0].value(&StatField::Intercept).unwrap(),
        -19.,
        1e-12,
    );
    let invalid = Layer::statistical(
        L,
        DATA,
        Statistic::count(CountSpec::default()),
        Geom::Point,
        StatAes::new(StatField::X, StatField::Mean),
    );
    assert_eq!(
        prepare(&def(invalid), &s).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
}

fn commit(s: &mut DataStore, mutation: Mutation) {
    let snapshot = s.snapshot();
    let snap = snapshot.get().unwrap();
    let tx = Transaction {
        id: TransactionId::new(format!("wp10-{}", snap.revision().get())).unwrap(),
        epoch: snap.epoch(),
        expected: vec![snap.dataset(DATA).unwrap().version()],
        operations: vec![Operation {
            dataset: DATA,
            mutation,
        }],
    };
    assert!(matches!(s.apply(tx), CommitOutcome::Applied(_)));
}
#[test]
fn exact_batch_fallback_after_append_correction_removal_and_reorder() {
    let mut s = store(&[
        (1, Some(0.), Some(1.), 0),
        (2, Some(1.), Some(3.), 0),
        (3, Some(2.), Some(5.), 0),
    ]);
    let stats = [
        Statistic::count(CountSpec {
            required: vec![X.into()],
            grouping: Grouping::All,
        }),
        Statistic::summary(SummarySpec::new(Y)),
        Statistic::ols(OlsSpec::new(X, Y)),
        Statistic::bin(BinSpec::new(X, vec![0., 1., 2., 4.])),
        Statistic::auto_bin(AutoBinSpec::new(X)),
    ];
    let definition =
        stats
            .iter()
            .enumerate()
            .fold(ChartDefinition::new(Revision::new(1)), |d, (i, stat)| {
                d.transform(TransformDefinition::new(
                    TransformId::new(i as u64),
                    DATA,
                    stat.clone(),
                ))
            });
    let mut compiler = Compiler::new();
    let first = compiler
        .prepare(
            &definition,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(first.metrics().evaluated_transforms, 5);
    let cached = compiler
        .prepare(
            &definition,
            &s.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(cached.metrics().reused_transforms, 5);
    for mutation in [
        Mutation::AppendBatch(batch(&[(4, Some(3.), Some(8.), 0)])),
        Mutation::UpsertByKey(batch(&[(2, Some(1.), Some(4.), 0)])),
        Mutation::RemoveKeys(vec![RowKey::new(4)]),
        Mutation::ReplaceSnapshot(batch(&[
            (3, Some(2.), Some(5.), 0),
            (1, Some(0.), Some(1.), 0),
            (2, Some(1.), Some(4.), 0),
        ])),
    ] {
        commit(&mut s, mutation);
        let current = compiler
            .prepare(
                &definition,
                &s.snapshot(),
                &ChartState::default(),
                CompileLimits::default(),
            )
            .unwrap();
        let reference = prepare(&definition, &s).unwrap();
        assert_eq!(current.metrics().evaluated_transforms, 5);
        for i in 0..5 {
            assert_eq!(
                current.transform(TransformId::new(i)),
                reference.transform(TransformId::new(i))
            );
            let cap = current.transform(TransformId::new(i)).unwrap().operations()[0].incremental;
            assert!(cap.full_recompute);
            let specialized = matches!(stats[i as usize].parameters, StatParameters::Bin(_));
            assert_eq!(
                (cap.append, cap.window, cap.correction),
                (specialized, specialized, specialized)
            );
        }
        // Independent final OLS: y=[1,4,5] at x=[0,1,2], slope=2, intercept=4/3.
        if s.snapshot().get().unwrap().dataset(DATA).unwrap().len() == 3 {
            let PreparedRows::Statistical(r) =
                current.transform(TransformId::new(2)).unwrap().rows()
            else {
                panic!()
            };
            near(r[0].value(&StatField::Slope).unwrap(), 2., 1e-12);
            near(r[0].value(&StatField::Intercept).unwrap(), 4. / 3., 1e-12);
        }
    }
    // Earlier output and model membership remain immutable and resolve at their pinned revision.
    let PreparedRows::Statistical(r) = first.transform(TransformId::new(2)).unwrap().rows() else {
        panic!()
    };
    assert_eq!(r[0].members.len(), 3);
    assert!(r[0].target.resolve(first.source().get().unwrap()).is_ok());
    assert!(r[0].target.resolve(s.snapshot().get().unwrap()).is_err());
}
#[test]
fn named_generated_output_is_shared_and_cannot_be_reinterpreted_as_source() {
    let t = TransformId::new(10);
    let node = TransformDefinition::new(t, DATA, Statistic::summary(SummarySpec::new(X)));
    let base = ChartDefinition::new(Revision::new(1)).transform(node);
    let l = Layer::statistical(
        L,
        t,
        Statistic::identity(),
        Geom::Point,
        StatAes::new(StatField::Group, StatField::Mean),
    );
    let p = prepare(
        &base.clone().layer(l.clone()).layer(Layer {
            id: LayerId::new(2),
            ..l
        }),
        &points(&[0., 10., 20., 30.]),
    )
    .unwrap();
    assert_eq!(p.metrics().evaluated_transforms, 1);
    assert_eq!(rows(&p)[0].value(&StatField::Mean), Some(15.));
    // The identity records differ by use, but the retained generated row allocation is shared.
    let (PreparedRows::Statistical(a), PreparedRows::Statistical(b)) =
        (p.layers()[0].table().rows(), p.layers()[1].table().rows())
    else {
        panic!()
    };
    assert!(Arc::ptr_eq(a, b));
    let bad = base.layer(Layer::new(L, t, Geom::Point, SourceAes::new().x(X).y(Y)));
    assert_eq!(
        prepare(&bad, &points(&[0.])).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
}
#[test]
fn strict_inputs_budgets_and_incompatible_position_contracts_reject() {
    let s = points(&[0., 1.]);
    let mut spec = SummarySpec::new(X);
    spec.grouping = Grouping::Field(G);
    let d = def(summary(spec));
    let limits = CompileLimits {
        max_groups: 1,
        ..Default::default()
    };
    assert_eq!(
        Compiler::new()
            .prepare(&d, &s.snapshot(), &ChartState::default(), limits)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let mut spec = AutoBinSpec::new(X);
    spec.bins = 0;
    let mut l = Layer::binned(L, DATA, Geom::Rectangle, BinAes::histogram());
    l.statistic = Statistic::auto_bin(spec);
    assert!(prepare(&def(l), &s).is_err());
    let mut l = stack(false);
    l.position = Position::Stack(StackSpec {
        order: vec![GroupValue::Int(0), GroupValue::Int(0)],
        normalize: false,
    });
    assert!(prepare(&def(l), &s).is_err());
    let mut l = summary(SummarySpec::new(X));
    l.position = Position::Jitter(JitterSpec {
        seed: 1,
        x: 1.,
        y: 0.,
        units: JitterUnits::Data,
    });
    assert!(prepare(&def(l), &s).is_err());
    let mut l = Layer::histogram(L, DATA, BinSpec::new(X, vec![0., 1., 2.]));
    l.mappings = Mappings::Binned(BinAes {
        x: BinNumeric::Literal(0.),
        y: BinField::End.into(),
        x2: Some(BinNumeric::Literal(1.)),
        y2: Some(BinNumeric::Literal(0.)),
        size: None,
    });
    if let StatParameters::Bin(s) = &mut l.statistic.parameters {
        s.space = StatSpace::Transformed(NumericTransform::affine(2., 10.));
    }
    l.position = Position::Stack(StackSpec {
        order: vec![GroupValue::All],
        normalize: false,
    });
    assert_eq!(
        prepare(&def(l), &s).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
}
#[test]
fn stored_d3_r7_reference_examples_match_only_the_agreed_quantile_definition() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/statistics/d3-quantiles.json"
    ))
    .unwrap();
    let mut spec = SummarySpec::new(X);
    spec.quantiles = values(&reference["probabilities"]);
    let p = prepare(&def(summary(spec)), &points(&values(&reference["values"]))).unwrap();
    for (i, value) in values(&reference["expected"]).iter().enumerate() {
        near(
            rows(&p)[0].value(&StatField::Quantile(i)).unwrap(),
            *value,
            1e-12,
        );
    }
}

#[test]
fn stable_accumulation_survives_intermediate_overflow_and_extreme_quantiles() {
    let s = points(&[f64::MAX, f64::MAX, -f64::MAX]);
    let mut spec = SummarySpec::new(X);
    spec.quantiles = vec![0., 0.25, 0.5, 1.];
    let p = prepare(&def(summary(spec)), &s).unwrap();
    assert_eq!(rows(&p)[0].value(&StatField::Sum), Some(f64::MAX));
    assert_eq!(rows(&p)[0].value(&StatField::Quantile(1)), Some(0.));
    assert!(
        prepare(
            &def(summary(SummarySpec::new(X))),
            &points(&[f64::MAX, f64::MAX])
        )
        .is_err()
    );
}
#[test]
fn zero_preserving_transformed_stacks_work_and_unused_bad_generated_fields_reject() {
    let mut spec = SummarySpec::new(Y);
    spec.grouping = Grouping::Field(G);
    spec.space = StatSpace::Transformed(NumericTransform::affine(2., 0.));
    let mut aes = StatAes::new(StatNumeric::Literal(0.), StatField::Sum);
    aes.x2 = Some(StatNumeric::Literal(1.));
    aes.y2 = Some(StatNumeric::Literal(0.));
    let mut layer = Layer::statistical(L, DATA, Statistic::summary(spec), Geom::Rectangle, aes);
    layer.position = Position::Stack(StackSpec {
        order: vec![GroupValue::Int(0), GroupValue::Int(1)],
        normalize: false,
    });
    let source = store(&[(1, Some(0.), Some(2.), 0), (2, Some(0.), Some(3.), 1)]);
    let p = prepare(&def(layer.clone()), &source).unwrap();
    assert_eq!(p.domains().y.unwrap().maximum, 10.);
    let Position::Stack(spec) = &mut layer.position else {
        panic!()
    };
    spec.normalize = true;
    let p = prepare(&def(layer), &source).unwrap();
    assert_eq!(p.domains().y.unwrap().maximum, 1.);
    assert_eq!(p.domains().y_space, Some(ValueSpace::Data));
    let mut l = counts(CountSpec::default());
    let Mappings::Statistical(a) = &mut l.mappings else {
        panic!()
    };
    a.x2 = Some(StatField::Slope.into());
    assert_eq!(
        prepare(&def(l), &points(&[1.])).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
}
