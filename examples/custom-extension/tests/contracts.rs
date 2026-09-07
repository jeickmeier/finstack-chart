//! Independent FIX-17 semantic and capability assertions through the public API.
use chart_core::{data::*, grammar::*, state::*, transaction::*, *};
use chart_extension_example::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn rows(p: &PreparedChart) -> &[StatisticalRow] {
    let PreparedRows::Statistical(rows) = p.layers()[0].table().rows() else {
        panic!("generated schema")
    };
    rows
}
#[test]
fn custom_histogram_has_own_schema_exact_membership_density_and_shared_domains() {
    let p = prepare(false).unwrap();
    assert!(
        matches!(p.layers()[0].table().schema(),OutputSchema::Custom{operation,..}if operation.id==HISTOGRAM)
    );
    assert_eq!(rows(&p).iter().map(|r| r.count).collect::<Vec<_>>(), [3, 3]);
    for (i, r) in rows(&p).iter().enumerate() {
        assert_eq!(r.value(&StatField::Custom("left".into())), Some(i as f64));
        assert_eq!(r.value(&StatField::Custom("density".into())), Some(0.5));
        assert_eq!(
            r.members.iter().map(|k| k.get()).collect::<Vec<_>>(),
            (1..=3)
                .map(|k| 9007199254743000 + i as u64 * 3 + k)
                .collect::<Vec<_>>()
        );
        r.target.resolve(p.source().get().unwrap()).unwrap();
    }
    assert_eq!(p.layers()[0].table().operations()[0].counts.invalid_stat, 1);
    assert_eq!(
        p.domains().x,
        Some(Extent {
            minimum: 0.,
            maximum: 2.
        })
    );
    assert_eq!(
        p.domains().y,
        Some(Extent {
            minimum: 0.,
            maximum: 0.5
        })
    );
    assert!(
        p.layers()[0]
            .marks()
            .iter()
            .all(|m| matches!(m.geometry, PreparedGeometry::Polygon(_)))
    );
    assert_eq!(p.layers()[0].interactions()[&0].keyboard_order, 2);
    assert_eq!(p.layers()[0].interactions()[&1].keyboard_order, 1);
}
#[test]
fn custom_transform_reuses_exact_tables_and_recomputes_after_corrections_like_fresh_batch() {
    let mut store = store().unwrap();
    let d = definition(false);
    let mut c = Compiler::with_extensions(registry().unwrap());
    let mut state = ChartState::default();
    let first = c
        .prepare(&d, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    state
        .apply(
            &d,
            ChartAction::SetViewport(Viewport {
                x: Some((0.25, 1.5)),
                y: None,
            }),
        )
        .unwrap();
    let view = c
        .prepare(&d, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert!(Arc::ptr_eq(
        first.transform(TransformId::new(7)).unwrap(),
        view.transform(TransformId::new(7)).unwrap()
    ));
    assert_eq!(rows(&first), rows(&view));
    let snap = store.snapshot();
    let data = snap.get().unwrap().dataset(DatasetId::new(1)).unwrap();
    let batch = NormalizedBatch::new(
        data.schema().clone(),
        vec![RowKey::new(9007199254743001)],
        vec![Column::new(
            ColumnValues::Float64(vec![1.75]),
            vec![true],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap();
    let t = Transaction {
        id: TransactionId::new("correction").unwrap(),
        epoch: SourceEpoch::new(1),
        expected: vec![data.version()],
        operations: vec![Operation {
            dataset: DatasetId::new(1),
            mutation: Mutation::UpsertByKey(batch),
        }],
    };
    assert!(matches!(store.apply(t), CommitOutcome::Applied { .. }));
    let updated = c
        .prepare(&d, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    let fresh = Compiler::with_extensions(registry().unwrap())
        .prepare(&d, &store.snapshot(), &state, CompileLimits::default())
        .unwrap();
    assert_eq!(rows(&updated), rows(&fresh));
    assert_eq!(
        rows(&updated).iter().map(|r| r.count).collect::<Vec<_>>(),
        [2, 4]
    );
    assert!(
        (rows(&updated)[0]
            .value(&StatField::Custom("density".into()))
            .unwrap()
            - 1. / 3.)
            .abs()
            < 1e-15
    );
    assert_eq!(rows(&first)[0].count, 3);
    assert!(!Arc::ptr_eq(
        first.layers()[0].table(),
        updated.layers()[0].table()
    ));
    let mut changed = d.clone();
    if let StatParameters::Custom(p) = &mut changed.transforms[0].statistic.parameters {
        p.values["edges"] = serde_json::json!([0., 0.5, 2.]);
    }
    let reparam = c
        .prepare(
            &changed,
            &store.snapshot(),
            &state,
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(
        rows(&reparam).iter().map(|r| r.count).collect::<Vec<_>>(),
        [1, 5]
    );
}
struct Bad {
    mode: u8,
    calls: Arc<AtomicUsize>,
}
impl CustomStat for Bad {
    fn descriptor(&self) -> ExtensionDescriptor {
        let mut d = DensityHistogram.descriptor();
        if self.mode == 5 {
            d.incremental.append = true;
        }
        d
    }
    fn schema(
        &self,
        d: &DatasetSnapshot,
        p: &ExtensionParameters,
        l: CompileLimits,
    ) -> ChartResult<Vec<StatColumn>> {
        let mut fields = DensityHistogram.schema(d, p, l)?;
        if self.mode == 6 {
            fields[0].space = ValueSpace::Categorical {
                categories: vec!["wrong".into()],
            };
        }
        if self.mode == 7 {
            let mut space = ValueSpace::Data;
            for _ in 0..25 {
                space = ValueSpace::Transformed {
                    input: Box::new(space),
                    transform: NumericTransform::affine(1., 0.),
                };
            }
            fields[0].space = space;
        }
        Ok(fields)
    }
    fn evaluate(&self, i: CustomStatInput<'_>) -> ChartResult<CustomStatOutput> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let mut o = DensityHistogram.evaluate(i)?;
        match self.mode {
            0 => o.rows[0].values[0].value = Some(f64::NAN),
            1 => o.rows[0].count += 1,
            2 => o.rows[0].values[0].field = StatField::Custom("absent".into()),
            3 => {
                o.rows[0].target =
                    chart_core::provenance::Target::Source(chart_core::provenance::SourceRef {
                        dataset: DatasetId::new(1),
                        key: o.rows[0].members[0],
                    })
            }
            4 => o.invalid_rows.push(RowKey::new(999)),
            _ => {}
        }
        Ok(o)
    }
}
#[test]
fn registry_schema_and_output_failures_are_typed_and_do_not_replace_valid_results() {
    let d = definition(false);
    let store = store().unwrap();
    assert_eq!(
        Compiler::new()
            .prepare(
                &d,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    for (mode, code) in [
        (0, DiagnosticCode::SchemaConflict),
        (1, DiagnosticCode::Validation),
        (2, DiagnosticCode::SchemaConflict),
        (3, DiagnosticCode::Validation),
        (4, DiagnosticCode::Validation),
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut r = ExtensionRegistry::new();
        r.register_stat(Arc::new(Bad {
            mode,
            calls: calls.clone(),
        }))
        .unwrap();
        r.register_geom(Arc::new(HistogramBars { native: false }))
            .unwrap();
        let mut c = Compiler::with_extensions(Arc::new(r));
        assert_eq!(
            c.prepare(
                &d,
                &store.snapshot(),
                &ChartState::default(),
                CompileLimits::default()
            )
            .unwrap_err()
            .code,
            code
        );
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    let mut r = ExtensionRegistry::new();
    assert_eq!(
        r.register_stat(Arc::new(Bad {
            mode: 5,
            calls: Arc::new(AtomicUsize::new(0))
        }))
        .unwrap_err()
        .code,
        DiagnosticCode::UnsupportedCapability
    );
    r.register_stat(Arc::new(DensityHistogram)).unwrap();
    assert_eq!(
        r.register_stat(Arc::new(DensityHistogram))
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
    let mut d = definition(false);
    d.layers[0].mappings =
        Mappings::Source(SourceAes::new().x(FieldId::new(1)).y(Numeric::Literal(1.)));
    assert_eq!(
        Compiler::with_extensions(registry().unwrap())
            .prepare(
                &d,
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
fn bounded_custom_paint_and_hit_regions_cannot_exhaust_core_or_invent_targets() {
    let d = definition(false);
    let source = store().unwrap().snapshot();
    let limits = CompileLimits {
        max_vertices: 12,
        ..CompileLimits::default()
    };
    assert_eq!(
        Compiler::with_extensions(registry().unwrap())
            .prepare(&d, &source, &ChartState::default(), limits)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let polygon = HitGeometry::Polygon(vec![
        Point::new(0., 0.).unwrap(),
        Point::new(2., 0.).unwrap(),
        Point::new(1., 2.).unwrap(),
    ]);
    assert!(polygon.contains(Point::new(1., 1.).unwrap()));
    assert!(polygon.contains(Point::new(0., 0.).unwrap()));
    assert!(!polygon.contains(Point::new(0.1, 1.9).unwrap()));
    let p = prepare(true).unwrap();
    assert!(matches!(
        p.layers()[0].marks()[0].geometry,
        PreparedGeometry::NativePaint { .. }
    ));
    assert!(
        !registry()
            .unwrap()
            .geometry_descriptor(&OperationRef::new(NATIVE_BARS, Revision::new(1)))
            .unwrap()
            .portable
    );
}

#[test]
fn schema_spaces_and_parameter_budgets_are_checked_before_evaluation() {
    let source = store().unwrap().snapshot();
    for (mode, code) in [
        (6, DiagnosticCode::SchemaConflict),
        (7, DiagnosticCode::ResourceLimit),
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut r = ExtensionRegistry::new();
        r.register_stat(Arc::new(Bad {
            mode,
            calls: calls.clone(),
        }))
        .unwrap();
        r.register_geom(Arc::new(HistogramBars { native: false }))
            .unwrap();
        let result = Compiler::with_extensions(Arc::new(r)).prepare(
            &definition(false),
            &source,
            &ChartState::default(),
            CompileLimits::default(),
        );
        assert_eq!(result.unwrap_err().code, code);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }
    for geometry in [false, true] {
        let mut d = definition(false);
        let value = serde_json::Value::Array(vec![serde_json::Value::Null; 4097]);
        if geometry {
            d.layers[0].geometry_extension.as_mut().unwrap().parameters = value;
        } else if let StatParameters::Custom(p) = &mut d.transforms[0].statistic.parameters {
            p.values = value;
        }
        assert_eq!(
            Compiler::with_extensions(registry().unwrap())
                .prepare(
                    &d,
                    &source,
                    &ChartState::default(),
                    CompileLimits::default()
                )
                .unwrap_err()
                .code,
            DiagnosticCode::ResourceLimit
        );
    }
}
