//! WP-05 FIX-01/02 semantic/geometry fixtures and compiler boundary counterexamples.
use chart_core::data::*;
use chart_core::grammar::*;
use chart_core::provenance::{ResolvedTarget, Target};
use chart_core::scene::Color;
use chart_core::state::*;
use chart_core::transaction::*;
use chart_core::{
    DatasetId, DiagnosticCode, FieldId, LayerId, Revision, RowKey, SchemaVersion, SourceEpoch,
    TransformId,
};
use std::{cell::Cell, rc::Rc, sync::Arc};

const DATA: DatasetId = DatasetId::new(9_007_199_254_740_993);
const X: FieldId = FieldId::new(1);
const Y: FieldId = FieldId::new(2);
const GROUP: FieldId = FieldId::new(3);
const L: LayerId = LayerId::new(1);
const BIN: TransformId = TransformId::new(99);
fn xy(rows: &[(u64, f64, Option<f64>)]) -> NormalizedBatch {
    let schema = Arc::new(
        Schema::new(
            SchemaVersion::new(0),
            vec![
                Field {
                    id: X,
                    name: "x".into(),
                    kind: FieldKind::Float64,
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
            ],
        )
        .unwrap(),
    );
    NormalizedBatch::new(
        schema,
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        vec![
            Column::new(
                ColumnValues::Float64(rows.iter().map(|r| r.1).collect()),
                vec![true; rows.len()],
                None,
            ),
            Column::new(
                ColumnValues::Float64(rows.iter().map(|r| r.2.unwrap_or(99.)).collect()),
                rows.iter().map(|r| r.2.is_some()).collect(),
                None,
            ),
        ],
        DataLimits::default(),
    )
    .unwrap()
}
fn store(rows: &[(u64, f64, Option<f64>)]) -> DataStore {
    DataStore::new(
        SourceEpoch::new(0),
        vec![(DATA, xy(rows))],
        DataLimits::default(),
    )
    .unwrap()
}
fn line() -> Layer {
    Layer::new(L, DATA, Geom::line(), SourceAes::new().x(X).y(Y))
}
fn definition(layer: Layer) -> ChartDefinition {
    ChartDefinition::new(Revision::new(1)).layer(layer)
}
fn prepare(def: &ChartDefinition, store: &DataStore) -> PreparedChart {
    Compiler::new()
        .prepare(
            def,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap()
}
fn runs(prepared: &PreparedChart) -> Vec<Vec<(f64, f64)>> {
    prepared.layers()[0]
        .marks()
        .iter()
        .map(|m| match &m.geometry {
            PreparedGeometry::LineRun(points) => points.iter().map(|p| (p.x(), p.y())).collect(),
            _ => panic!("line run"),
        })
        .collect()
}
fn bins(table: &PreparedTable) -> &[BinnedRow] {
    match table.rows() {
        PreparedRows::Binned(rows) => rows,
        _ => panic!("generated bin schema"),
    }
}
fn commit(store: &mut DataStore, mutation: Mutation) {
    let snapshot = store.snapshot();
    let input = snapshot.get().unwrap();
    let t = Transaction {
        id: TransactionId::new(format!("update-{}", input.revision().get())).unwrap(),
        epoch: input.epoch(),
        expected: vec![input.dataset(DATA).unwrap().version()],
        operations: vec![Operation {
            dataset: DATA,
            mutation,
        }],
    };
    assert!(matches!(store.apply(t), CommitOutcome::Applied(_)));
}

#[test]
fn fix01_null_y_is_two_isolated_runs_with_exact_source_targets() {
    let store = store(&[(u64::MAX, 0., Some(1.)), (2, 1., None), (3, 2., Some(3.))]);
    let prepared = prepare(&definition(line()), &store);
    assert_eq!(runs(&prepared), vec![vec![(0., 1.)], vec![(2., 3.)]]);
    assert_eq!(prepared.layers()[0].invalid_geometry(), 1);
    assert_eq!(
        prepared.domains().y,
        Some(Extent {
            minimum: 1.,
            maximum: 3.
        })
    );
    assert_eq!(prepared.diagnostics()[0].context.affected_rows, 1);
    assert_eq!(
        prepared.diagnostics()[0].context.row_samples,
        vec![RowKey::new(2)]
    );
    assert!(
        matches!(prepared.layers()[0].marks()[0].targets[0],Target::Source(source) if source.key==RowKey::new(u64::MAX) && source.dataset==DATA)
    );
    let mut connect = line();
    connect.geom = Geom::Line {
        order: LineOrder::X,
        connect_gaps: true,
    };
    assert_eq!(
        runs(&prepare(&definition(connect), &store)),
        vec![vec![(0., 1.), (2., 3.)]]
    );
    let mut strict = line();
    strict.invalid = InvalidPolicy::Strict;
    assert!(
        Compiler::new()
            .prepare(
                &definition(strict),
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .is_err()
    );
}

#[test]
fn fix02_histogram_is_bin_stat_plus_rectangles_with_complete_membership() {
    let store = store(&[
        (10, 0., Some(0.)),
        (11, 0.5, Some(0.)),
        (12, 1., Some(0.)),
        (13, 2., Some(0.)),
    ]);
    let recipe = Layer::histogram(L, DATA, BinSpec::new(X, vec![0., 1., 2.]));
    let mut composed = Layer::binned(L, DATA, Geom::Rectangle, BinAes::histogram());
    composed.statistic = Statistic::bin(BinSpec::new(X, vec![0., 1., 2.]));
    let prepared = prepare(&definition(recipe), &store);
    let explicit = prepare(&definition(composed), &store);
    assert_eq!(prepared.layers()[0].marks(), explicit.layers()[0].marks());
    let output = bins(prepared.layers()[0].table());
    assert_eq!(
        output
            .iter()
            .map(|b| (b.start, b.end, b.count))
            .collect::<Vec<_>>(),
        vec![(0., 1., 2), (1., 2., 2)]
    );
    assert!(
        matches!(prepared.layers()[0].table().schema(),OutputSchema::Binned {version,..} if *version==SchemaVersion::new(1))
    );
    for (i, expected) in [vec![10, 11], vec![12, 13]].iter().enumerate() {
        let ResolvedTarget::Aggregate { members, .. } = output[i]
            .target
            .resolve(prepared.source().get().unwrap())
            .unwrap()
        else {
            panic!("must be aggregate")
        };
        assert_eq!(
            members.iter().map(|r| r.key().get()).collect::<Vec<_>>(),
            *expected
        );
        let PreparedGeometry::Rectangle { from, to } = prepared.layers()[0].marks()[i].geometry
        else {
            panic!("rectangle")
        };
        assert_eq!(
            (from.x(), from.y(), to.x(), to.y()),
            (i as f64, 2., i as f64 + 1., 0.)
        );
    }
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: 0.,
            maximum: 2.
        })
    );
    assert_eq!(
        prepared.domains().y,
        Some(Extent {
            minimum: 0.,
            maximum: 2.
        })
    );
}

#[test]
fn heterogeneous_typed_sources_and_shared_histogram_overlay_use_one_engine() {
    struct Observation {
        value: f64,
    }
    struct Annotation {
        left: f64,
        right: f64,
        height: f64,
    }
    let source = TypedRows::snapshot(
        DATA,
        Revision::INITIAL,
        (1..=4).map(RowKey::new).collect(),
        vec![
            Observation { value: 0. },
            Observation { value: 0.5 },
            Observation { value: 1. },
            Observation { value: 2. },
        ],
        4,
    )
    .unwrap();
    let notes = DatasetId::new(7);
    let a = FieldId::new(21);
    let b = FieldId::new(22);
    let y = FieldId::new(23);
    let annotations = TypedRows::snapshot(
        notes,
        Revision::INITIAL,
        vec![RowKey::new(90)],
        vec![Annotation {
            left: -1.,
            right: 3.,
            height: 3.,
        }],
        1,
    )
    .unwrap();
    let observations = TypedDataBuilder::new(source.get().unwrap(), SchemaVersion::new(0))
        .float(X, "value", |r| Some(r.value))
        .finish(DataLimits::default())
        .unwrap();
    let rules = TypedDataBuilder::new(annotations.get().unwrap(), SchemaVersion::new(0))
        .float(a, "left", |r| Some(r.left))
        .float(b, "right", |r| Some(r.right))
        .float(y, "height", |r| Some(r.height))
        .finish(DataLimits::default())
        .unwrap();
    let store = DataStore::new(
        SourceEpoch::new(0),
        vec![(DATA, observations), (notes, rules)],
        DataLimits::default(),
    )
    .unwrap();
    let def = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(
            BIN,
            DATA,
            Statistic::bin(BinSpec::new(X, vec![0., 1., 2.])),
        ))
        .layer(Layer::binned(L, BIN, Geom::Rectangle, BinAes::histogram()))
        .layer(Layer::binned(
            LayerId::new(2),
            BIN,
            Geom::Point,
            BinAes::new(BinField::Midpoint, BinField::Count),
        ))
        .layer(
            Layer::new(
                LayerId::new(3),
                notes,
                Geom::Rule,
                SourceAes::new().x(a).y(y).x2(b).y2(y),
            )
            .independent(),
        );
    let prepared = prepare(&def, &store);
    assert_eq!(prepared.metrics().evaluated_transforms, 1);
    assert_eq!(
        prepared
            .layers()
            .iter()
            .map(PreparedLayer::id)
            .collect::<Vec<_>>(),
        vec![L, LayerId::new(2), LayerId::new(3)]
    );
    let (PreparedRows::Binned(a), PreparedRows::Binned(b)) = (
        prepared.layers()[0].table().rows(),
        prepared.layers()[1].table().rows(),
    ) else {
        panic!("bins")
    };
    assert!(Arc::ptr_eq(a, b));
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: -1.,
            maximum: 3.
        })
    );
    assert_eq!(
        prepared.domains().y,
        Some(Extent {
            minimum: 0.,
            maximum: 3.
        })
    );
    assert!(
        matches!(prepared.layers()[2].marks()[0].targets[0],Target::Source(s) if s.dataset==notes)
    );
}

#[test]
fn inherited_schema_and_generated_stage_mismatches_fail_with_context() {
    let store = store(&[(1, 0., Some(1.))]);
    let inherited = ChartDefinition::new(Revision::new(1))
        .mapped(SourceAes::new().x(FieldId::new(999)).y(Y))
        .layer(Layer::new(L, DATA, Geom::Point, SourceAes::new()));
    let e = Compiler::new()
        .prepare(
            &inherited,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap_err();
    assert_eq!(e.code, DiagnosticCode::SchemaConflict);
    assert_eq!(e.context.layer, Some(L));
    assert_eq!(e.context.field, Some(FieldId::new(999)));
    let mut overridden = inherited;
    overridden.layers[0].mappings = Mappings::Source(SourceAes::new().x(X));
    assert_eq!(prepare(&overridden, &store).layers()[0].marks().len(), 1);
    let mut wrong = line();
    wrong.statistic = Statistic::bin(BinSpec::new(X, vec![0., 1.]));
    assert_eq!(
        Compiler::new()
            .prepare(
                &definition(wrong),
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
    let wrong = Layer::binned(
        L,
        DATA,
        Geom::Point,
        BinAes::new(BinField::Midpoint, BinField::Count),
    );
    assert!(
        Compiler::new()
            .prepare(
                &definition(wrong),
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .is_err()
    );
}

#[test]
fn edges_outliers_invalid_inputs_and_empty_bins_have_explicit_counts() {
    let store = store(&[
        (1, -1., None),
        (2, 0., None),
        (3, 0.5, None),
        (4, 1., None),
        (5, 2., None),
        (6, 3., None),
        (7, f64::NAN, None),
        (8, f64::INFINITY, None),
    ]);
    let def = definition(Layer::histogram(L, DATA, BinSpec::new(X, vec![0., 1., 2.])));
    let prepared = prepare(&def, &store);
    let record = prepared.layers()[0].table().operations().last().unwrap();
    assert_eq!(
        record.counts,
        PopulationCounts {
            input: 8,
            invalid_stat: 2,
            below: 1,
            above: 1,
            output: 2,
            ..Default::default()
        }
    );
    assert_eq!(
        bins(prepared.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![2, 2]
    );
    assert_eq!(prepared.diagnostics().len(), 2);
    let empty = prepare(&def, &store_empty());
    assert_eq!(
        bins(empty.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );
    let mut error_spec = BinSpec::new(X, vec![0., 1., 2.]);
    error_spec.outliers = OutlierPolicy::Error;
    assert!(
        Compiler::new()
            .prepare(
                &definition(Layer::histogram(L, DATA, error_spec)),
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .is_err()
    );
    for edges in [
        vec![],
        vec![0.],
        vec![0., 0.],
        vec![2., 1.],
        vec![0., f64::NAN],
        vec![0., f64::INFINITY],
    ] {
        assert!(
            Compiler::new()
                .prepare(
                    &definition(Layer::histogram(L, DATA, BinSpec::new(X, edges))),
                    &store.snapshot(),
                    &ChartState::default(),
                    CompileLimits::default()
                )
                .is_err()
        );
    }
}
fn store_empty() -> DataStore {
    store(&[])
}

#[test]
fn viewport_and_visibility_leave_stat_population_domains_and_cache_unchanged() {
    let store = store(&[(1, 0., None), (2, 0.5, None), (3, 1., None), (4, 2., None)]);
    let def = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(
            BIN,
            DATA,
            Statistic::bin(BinSpec::new(X, vec![0., 1., 2.])),
        ))
        .layer(Layer::binned(L, BIN, Geom::Rectangle, BinAes::histogram()));
    let mut compiler = Compiler::new();
    let mut state = ChartState::default();
    let first = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert!(
        state
            .apply(
                &def,
                ChartAction::SetViewport(Viewport {
                    x: Some((0., 1.)),
                    y: None
                })
            )
            .unwrap()
            .viewport_changed
    );
    state
        .apply(
            &def,
            ChartAction::SetLayerVisible {
                layer: L,
                visible: false,
            },
        )
        .unwrap();
    let zoomed = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert_eq!(first.domains(), zoomed.domains());
    assert!(!zoomed.layers()[0].visible());
    assert_eq!(
        bins(first.layers()[0].table()),
        bins(zoomed.layers()[0].table())
    );
    assert_eq!(
        zoomed.metrics(),
        PreparationMetrics {
            evaluated_transforms: 0,
            reused_transforms: 1,
            evaluated_layers: 0,
            reused_layers: 1
        }
    );
    assert!(Arc::ptr_eq(
        first.transform(BIN).unwrap(),
        zoomed.transform(BIN).unwrap()
    ));
    let mut filtered = def.clone();
    filtered.transforms[0].filters.push(SourceFilter {
        value: X.into(),
        minimum: Some(1.),
        maximum: None,
    });
    let result = compiler
        .prepare(
            &filtered,
            &store.snapshot(),
            &state,
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(result.metrics().evaluated_transforms, 1);
    assert_eq!(
        bins(result.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(
        result.transform(BIN).unwrap().operations()[0]
            .counts
            .filtered,
        2
    );
    assert_eq!(
        bins(first.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![2, 2]
    );
}

#[test]
fn graph_cycles_unknown_operations_and_forward_dependencies_are_checked() {
    let store = store(&[(1, 0., Some(1.))]);
    let second = TransformId::new(100);
    let mut def = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(BIN, second, Statistic::identity()))
        .transform(TransformDefinition::new(
            second,
            DATA,
            Statistic::identity(),
        ))
        .layer(Layer::new(L, BIN, Geom::Point, SourceAes::new().x(X).y(Y)));
    let prepared = prepare(&def, &store);
    assert_eq!(prepared.metrics().evaluated_transforms, 2);
    assert_eq!(prepared.layers()[0].marks().len(), 1);
    def.transforms[1].input = DataRef::Transform(BIN);
    assert_eq!(
        Compiler::new()
            .prepare(
                &def,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    def.transforms[1].input = DataRef::Transform(TransformId::new(999));
    assert_eq!(
        Compiler::new()
            .prepare(
                &def,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::MissingResource
    );
    for operation in [
        OperationRef::new("native.closure", Revision::new(1)),
        OperationRef::new("chart.identity", Revision::new(2)),
        OperationRef::new("chart.bin", Revision::new(1)),
    ] {
        let mut layer = line();
        layer.statistic.operation = operation;
        assert_eq!(
            Compiler::new()
                .prepare(
                    &definition(layer),
                    &store.snapshot(),
                    &ChartState::default(),
                    CompileLimits::default()
                )
                .unwrap_err()
                .code,
            DiagnosticCode::UnsupportedCapability
        );
    }
}

#[test]
fn transformed_statistics_record_space_once_and_reject_mixed_axes() {
    let store = store(&[
        (1, 0., Some(0.)),
        (2, 0.5, Some(0.)),
        (3, 1., Some(0.)),
        (4, 2., Some(0.)),
    ]);
    let mut spec = BinSpec::new(X, vec![10., 12., 14.]);
    spec.space = StatSpace::Transformed(NumericTransform::affine(2., 10.));
    let def = definition(Layer::histogram(L, DATA, spec.clone()));
    let prepared = prepare(&def, &store);
    assert_eq!(
        bins(prepared.layers()[0].table())
            .iter()
            .map(|r| (r.start, r.end, r.count))
            .collect::<Vec<_>>(),
        vec![(10., 12., 2), (12., 14., 2)]
    );
    assert!(matches!(
        prepared.domains().x_space,
        Some(ValueSpace::Transformed { .. })
    ));
    assert_eq!(
        prepared.layers()[0].table().operations()[0].parameters,
        StatParameters::Bin(spec)
    );
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: 10.,
            maximum: 14.
        })
    );
    let mixed = def.layer(Layer::new(
        LayerId::new(2),
        DATA,
        Geom::Point,
        SourceAes::new().x(X).y(Y),
    ));
    assert_eq!(
        Compiler::new()
            .prepare(
                &mixed,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
}

#[test]
fn cache_invalidates_on_correction_parameters_and_snapshot_identity() {
    let mut store = store(&[(1, 0., Some(0.)), (2, 1., Some(0.))]);
    let mut def = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(
            BIN,
            DATA,
            Statistic::bin(BinSpec::new(X, vec![0., 1., 2.])),
        ))
        .layer(Layer::binned(L, BIN, Geom::Rectangle, BinAes::histogram()));
    let mut compiler = Compiler::new();
    let state = ChartState::default();
    let old = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    commit(&mut store, Mutation::UpsertByKey(xy(&[(1, 1.5, Some(0.))])));
    let changed = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert_eq!(changed.metrics().evaluated_transforms, 1);
    assert_eq!(
        bins(changed.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(
        bins(old.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![1, 1]
    );
    let old_target = &bins(old.layers()[0].table())[0].target;
    assert!(old_target.resolve(old.source().get().unwrap()).is_ok());
    assert_eq!(
        old_target
            .resolve(changed.source().get().unwrap())
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    def.layers[0].style.color = Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    assert_eq!(
        compiler
            .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
            .unwrap()
            .metrics()
            .reused_transforms,
        1
    );
    if let StatParameters::Bin(spec) = &mut def.transforms[0].statistic.parameters {
        spec.edges = vec![0., 2.];
    }
    assert_eq!(
        compiler
            .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
            .unwrap()
            .metrics()
            .evaluated_transforms,
        1
    );
    let independent = store_empty(); // Same epoch/revision is insufficient to identify a snapshot.
    assert_eq!(
        compiler
            .prepare(
                &def,
                &independent.snapshot(),
                &state,
                CompileLimits::default()
            )
            .unwrap()
            .metrics()
            .evaluated_transforms,
        1
    );
}

#[test]
fn budgets_and_invalid_definitions_keep_last_prepared_result_and_cache_valid() {
    let store = store(&[(1, 0., Some(1.)), (2, 1., Some(2.))]);
    let def = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(BIN, DATA, Statistic::identity()))
        .layer(Layer::new(L, BIN, Geom::Point, SourceAes::new().x(X).y(Y)));
    let mut compiler = Compiler::new();
    let state = ChartState::default();
    let first = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    for limits in [
        CompileLimits {
            max_layers: 0,
            ..Default::default()
        },
        CompileLimits {
            max_transforms: 0,
            ..Default::default()
        },
        CompileLimits {
            max_prepared_rows: 3,
            ..Default::default()
        },
        CompileLimits {
            max_vertices: 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            compiler
                .prepare(&def, &store.snapshot(), &state, limits)
                .unwrap_err()
                .code,
            DiagnosticCode::ResourceLimit
        );
    }
    let mut invalid = def.clone();
    invalid.layers[0].style.radius = f64::NAN;
    assert!(
        compiler
            .prepare(
                &invalid,
                &store.snapshot(),
                &state,
                CompileLimits::default()
            )
            .is_err()
    );
    let next = compiler
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert_eq!(next.metrics().reused_transforms, 1);
    assert_eq!(first.layers()[0].marks(), next.layers()[0].marks());
}

#[test]
fn grouped_bins_use_stable_labels_and_lines_never_cross_groups_or_unknown_x() {
    struct Row {
        x: f64,
        y: Option<f64>,
        group: &'static str,
    }
    let rows = TypedRows::snapshot(
        DATA,
        Revision::INITIAL,
        (1..=4).map(RowKey::new).collect(),
        vec![
            Row {
                x: 1.,
                y: Some(2.),
                group: "z",
            },
            Row {
                x: 0.,
                y: Some(1.),
                group: "a",
            },
            Row {
                x: 0.,
                y: Some(1.),
                group: "z",
            },
            Row {
                x: 2.,
                y: Some(3.),
                group: "a",
            },
        ],
        4,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(0))
        .float(X, "x", |r| Some(r.x))
        .float(Y, "y", |r| r.y)
        .category(GROUP, "group", |r| Some(r.group.into()))
        .finish(DataLimits::default())
        .unwrap();
    let store = DataStore::new(
        SourceEpoch::new(0),
        vec![(DATA, batch)],
        DataLimits::default(),
    )
    .unwrap();
    let lines = prepare(
        &definition(Layer::new(
            L,
            DATA,
            Geom::line(),
            SourceAes::new().x(X).y(Y).group(GROUP),
        )),
        &store,
    );
    assert_eq!(
        runs(&lines),
        vec![vec![(0., 1.), (1., 2.)], vec![(0., 1.), (2., 3.)]]
    );
    let mut spec = BinSpec::new(X, vec![0., 1., 2.]);
    spec.grouping = Grouping::Field(GROUP);
    let hist = prepare(&definition(Layer::histogram(L, DATA, spec)), &store);
    assert_eq!(
        bins(hist.layers()[0].table())
            .iter()
            .map(|r| (r.group.clone(), r.count))
            .collect::<Vec<_>>(),
        vec![
            (GroupValue::Text("z".into()), 1),
            (GroupValue::Text("z".into()), 1),
            (GroupValue::Text("a".into()), 1),
            (GroupValue::Text("a".into()), 1)
        ]
    );
    let data = crate::store(&[
        (1, 3., Some(3.)),
        (2, f64::NAN, Some(9.)),
        (3, 1., Some(1.)),
        (4, 2., Some(2.)),
    ]);
    assert_eq!(
        runs(&prepare(&definition(line()), &data)),
        vec![vec![(3., 3.)], vec![(1., 1.), (2., 2.)]]
    );
}

#[test]
fn equal_x_ties_keep_insertion_ordinals_after_replacement_and_authored_order_is_explicit() {
    let mut store = store(&[(1, 1., Some(10.)), (2, 1., Some(20.)), (3, 0., Some(0.))]);
    commit(
        &mut store,
        Mutation::ReplaceSnapshot(xy(&[
            (2, 1., Some(20.)),
            (3, 0., Some(0.)),
            (1, 1., Some(10.)),
        ])),
    );
    assert_eq!(
        runs(&prepare(&definition(line()), &store)),
        vec![vec![(0., 0.), (1., 10.), (1., 20.)]]
    );
    let mut authored = line();
    authored.geom = Geom::Line {
        order: LineOrder::Authored,
        connect_gaps: false,
    };
    assert_eq!(
        runs(&prepare(&definition(authored), &store)),
        vec![vec![(1., 20.), (0., 0.), (1., 10.)]]
    );
}

#[test]
fn mapped_sizes_are_distinct_from_constant_style_and_all_endpoints_train_domains() {
    let store = store(&[(1, 1., Some(2.)), (2, 2., Some(0.)), (3, 3., Some(-1.))]);
    let points = Layer::new(
        L,
        DATA,
        Geom::Point,
        SourceAes::new().x(X).y(Numeric::Literal(4.)).size(Y),
    );
    let prepared = prepare(&definition(points), &store);
    assert_eq!(prepared.layers()[0].marks().len(), 1);
    assert_eq!(prepared.layers()[0].marks()[0].style.radius, 2.);
    assert_eq!(prepared.layers()[0].invalid_geometry(), 2);
    let rect = Layer::new(
        L,
        DATA,
        Geom::Rectangle,
        SourceAes::new()
            .x(Numeric::Literal(-5.))
            .y(Y)
            .x2(X)
            .y2(Numeric::Literal(0.)),
    );
    let prepared = prepare(&definition(rect), &store);
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: -5.,
            maximum: 3.
        })
    );
    assert_eq!(
        prepared.domains().y,
        Some(Extent {
            minimum: -1.,
            maximum: 2.
        })
    );
    assert!(
        prepared.layers()[0]
            .marks()
            .iter()
            .all(|m| matches!(m.geometry, PreparedGeometry::Rectangle { .. }))
    );
}

#[test]
fn typed_accessors_run_once_without_send_bounds_and_do_not_escape_to_prepared_scene() {
    struct Row {
        drops: Rc<Cell<usize>>,
        value: i64,
    }
    impl Drop for Row {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let calls = Rc::new(Cell::new(0));
    let mut typed = TypedRows::snapshot(
        DATA,
        Revision::new(17),
        vec![RowKey::new(u64::MAX)],
        vec![Row {
            drops: drops.clone(),
            value: i64::MAX,
        }],
        1,
    )
    .unwrap();
    let counter = calls.clone();
    let normalized = TypedDataBuilder::new(typed.get().unwrap(), SchemaVersion::new(0))
        .int64(X, "exact", move |r| {
            counter.set(counter.get() + 1);
            Some(r.value)
        })
        .float(Y, "y", |_| Some(1.))
        .finish(DataLimits::default())
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        normalized.column(X).unwrap().value(0),
        Some(ValueRef::Int64(i64::MAX))
    );
    typed.dispose();
    assert_eq!(drops.get(), 1);
    let store = DataStore::new(
        SourceEpoch::new(0),
        vec![(DATA, normalized)],
        DataLimits::default(),
    )
    .unwrap();
    let prepared = prepare(&definition(line()), &store);
    assert_eq!(prepared.layers()[0].invalid_geometry(), 1);
    assert!(prepared.layers()[0].marks().is_empty());
    assert_eq!(calls.get(), 1);
    let source =
        TypedRows::snapshot(DATA, Revision::INITIAL, vec![RowKey::new(1)], vec![1.], 1).unwrap();
    let build = || {
        TypedDataBuilder::new(source.get().unwrap(), SchemaVersion::new(0)).float(X, "x", |r| {
            calls.set(calls.get() + 1);
            Some(*r)
        })
    };
    assert!(
        build()
            .finish(DataLimits {
                max_batch_rows: 0,
                ..Default::default()
            })
            .is_err()
    );
    assert!(
        build()
            .float(X, "duplicate", |r| {
                calls.set(calls.get() + 1);
                Some(*r)
            })
            .finish(DataLimits::default())
            .is_err()
    );
    assert_eq!(calls.get(), 1);
}

#[test]
fn precise_timestamps_keep_their_origin_and_incompatible_origins_fail() {
    let origin = 9_000_000_000_000_000_000_i64;
    let typed = TypedRows::snapshot(
        DATA,
        Revision::INITIAL,
        vec![RowKey::new(u64::MAX)],
        vec![origin + 17],
        1,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(typed.get().unwrap(), SchemaVersion::new(0))
        .timestamp(
            X,
            "time",
            TimestampType {
                unit: TimeUnit::Nanoseconds,
                timezone: "UTC".into(),
            },
            |v| Some(*v),
        )
        .float(Y, "y", |_| Some(1.))
        .finish(DataLimits::default())
        .unwrap();
    let store = DataStore::new(
        SourceEpoch::new(0),
        vec![(DATA, batch)],
        DataLimits::default(),
    )
    .unwrap();
    let layer = Layer::new(
        L,
        DATA,
        Geom::Point,
        SourceAes::new()
            .x(Numeric::Timestamp { field: X, origin })
            .y(Y),
    );
    let def = definition(layer);
    let prepared = prepare(&def, &store);
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: 17.,
            maximum: 17.
        })
    );
    assert!(
        matches!(prepared.domains().x_space,Some(ValueSpace::Timestamp {origin:o,..}) if o==origin)
    );
    let mixed = def.layer(Layer::new(
        LayerId::new(2),
        DATA,
        Geom::Point,
        SourceAes::new()
            .x(Numeric::Timestamp {
                field: X,
                origin: origin - 1,
            })
            .y(Y),
    ));
    assert_eq!(
        Compiler::new()
            .prepare(
                &mixed,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
}

#[test]
fn state_actions_are_atomic_idempotent_and_preserve_prepared_snapshots() {
    let store = store(&[(1, 1., Some(1.))]);
    let def = definition(line());
    let mut state = ChartState::default();
    let first = Compiler::new()
        .prepare(&def, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert!(!state.apply(&def, ChartAction::Reset).unwrap().changed);
    let change = state
        .apply(
            &def,
            ChartAction::SetLayerVisible {
                layer: L,
                visible: false,
            },
        )
        .unwrap();
    assert_eq!(change.revision, Revision::new(1));
    assert!(!change.viewport_changed);
    assert!(
        !state
            .apply(
                &def,
                ChartAction::SetLayerVisible {
                    layer: L,
                    visible: false
                }
            )
            .unwrap()
            .changed
    );
    let before = state.clone();
    assert!(
        state
            .apply(
                &def,
                ChartAction::SetViewport(Viewport {
                    x: Some((0., 0.)),
                    y: None
                })
            )
            .is_err()
    );
    assert!(
        state
            .apply(
                &def,
                ChartAction::SetLayerVisible {
                    layer: LayerId::new(999),
                    visible: true
                }
            )
            .is_err()
    );
    assert_eq!(state, before);
    state
        .apply(
            &def,
            ChartAction::SetViewport(Viewport {
                x: Some((2., -1.)),
                y: None,
            }),
        )
        .unwrap();
    assert_eq!(state.viewport_revision(), Revision::new(1));
    state.apply(&def, ChartAction::Reset).unwrap();
    assert_eq!(state.revision(), Revision::new(3));
    assert!(state.is_visible(L));
    assert_eq!(first.state().revision(), Revision::INITIAL);
    assert!(first.layers()[0].visible());
}

#[test]
fn repeated_invalid_geometry_has_bounded_diagnostics_and_old_handles_remain_valid() {
    let store = store(&(0..100).map(|i| (i, i as f64, None)).collect::<Vec<_>>());
    let mut handle = store.snapshot();
    let mut compiler = Compiler::new();
    let prepared = compiler
        .prepare(
            &definition(line()),
            &handle,
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(prepared.diagnostics().len(), 1);
    assert_eq!(prepared.diagnostics()[0].context.affected_rows, 100);
    assert_eq!(prepared.diagnostics()[0].context.row_samples.len(), 32);
    handle.dispose();
    compiler.clear_cache();
    assert_eq!(
        prepared
            .source()
            .get()
            .unwrap()
            .dataset(DATA)
            .unwrap()
            .len(),
        100
    );
    assert_eq!(
        compiler
            .prepare(
                &definition(line()),
                &handle,
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::DisposedHandle
    );
}

#[test]
fn schema_preflight_precedes_population_work_and_null_filters_are_reported() {
    let store = store(&[(1, 0., None), (2, 1., Some(2.))]);
    let def = definition(Layer::new(
        L,
        DATA,
        Geom::Point,
        SourceAes::new().x(FieldId::new(999)).y(Y),
    ));
    let e = Compiler::new()
        .prepare(
            &def,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits {
                max_prepared_rows: 0,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(e.code, DiagnosticCode::SchemaConflict);
    assert_eq!(e.context.field, Some(FieldId::new(999)));
    let mut histogram = Layer::histogram(L, DATA, BinSpec::new(X, vec![0., 1., 2.]));
    histogram.filters.push(SourceFilter {
        value: Y.into(),
        minimum: Some(0.),
        maximum: None,
    });
    let prepared = prepare(&definition(histogram), &store);
    let record = prepared.layers()[0].table().operations().last().unwrap();
    assert_eq!(record.counts.invalid_filter, 1);
    assert_eq!(record.counts.invalid_stat, 0);
    assert_eq!(
        bins(prepared.layers()[0].table())
            .iter()
            .map(|r| r.count)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        prepared.diagnostics()[0].context.row_samples,
        vec![RowKey::new(1)]
    );
}

#[test]
fn signed_zero_x_values_use_ordinal_ties_and_rectangles_preserve_exact_endpoints() {
    let store = store(&[(1, 0., Some(1.)), (2, -0., Some(2.))]);
    assert_eq!(
        runs(&prepare(&definition(line()), &store))[0]
            .iter()
            .map(|p| p.1)
            .collect::<Vec<_>>(),
        vec![1., 2.]
    );
    let rect = Layer::new(
        L,
        DATA,
        Geom::Rectangle,
        SourceAes::new()
            .x(Numeric::Literal(-1e16))
            .y(Numeric::Literal(0.))
            .x2(Numeric::Literal(1.))
            .y2(Numeric::Literal(1.)),
    );
    let prepared = prepare(&definition(rect), &store);
    assert_eq!(
        prepared.domains().x,
        Some(Extent {
            minimum: -1e16,
            maximum: 1.
        })
    );
    let PreparedGeometry::Rectangle { from, to } = prepared.layers()[0].marks()[0].geometry else {
        panic!("rectangle")
    };
    assert_eq!((from.x(), to.x()), (-1e16, 1.)); // No x + width cancellation at this boundary.
}

#[test]
fn generated_midpoints_avoid_overflow_and_subnormal_loss() {
    let empty = store_empty();
    for (left, right, expected) in [
        (f64::from_bits(1), f64::from_bits(2), f64::from_bits(2)),
        (f64::MAX * 0.5, f64::MAX, f64::MAX * 0.75),
    ] {
        let mut layer = Layer::binned(
            L,
            DATA,
            Geom::Point,
            BinAes::new(BinField::Midpoint, BinField::Count),
        );
        layer.statistic = Statistic::bin(BinSpec::new(X, vec![left, right]));
        let prepared = prepare(&definition(layer), &empty);
        let PreparedGeometry::Point(p) = prepared.layers()[0].marks()[0].geometry else {
            panic!("point")
        };
        assert_eq!(p.x(), expected);
    }
}
