//! SP-07: exact eligible populations across updates, independent batches and facets.
use chart_core::{
    data::*,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
    state::ChartState,
    transaction::*,
    *,
};
const D: DatasetId = DatasetId::new(71);
fn data(rows: &[(u64, u64, f64, &str)]) -> Data {
    Data::columns()
        .identity(D.get())
        .keys(rows.iter().map(|r| r.0))
        .column("key", rows.iter().map(|r| r.1).collect::<Vec<u64>>())
        .column("value", rows.iter().map(|r| r.2).collect::<Vec<f64>>())
        .column("panel", categorical(rows.iter().map(|r| r.3)))
        .build()
        .unwrap()
}
fn definition(initial: &Data, faceted: bool, classifier: bool) -> ChartDefinition {
    let scale = if classifier {
        ScaleConstructor::Quantile
            .create(ScaleOptions {
                range: Some(vec![Value::Text("red".into()), Value::Text("blue".into())]),
                ..Default::default()
            })
            .unwrap()
    } else {
        ScaleConstructor::Ordinal
            .create(ScaleOptions {
                range: Some(vec![
                    Value::Text("red".into()),
                    Value::Text("blue".into()),
                    Value::Text("green".into()),
                ]),
                ..Default::default()
            })
            .unwrap()
    };
    let mut p = plot(initial.clone())
        .aes(
            aes()
                .x("value")
                .y("value")
                .color(if classifier { "value" } else { "key" })
                .color_scale("shared"),
        )
        .scale(color_mapped(
            "shared",
            scale.mapped(ScaleTraining::Eligible).unwrap(),
        ))
        .layer(points());
    if faceted {
        p = p.facet(facet_wrap("panel").columns(2).free_y(true));
    }
    p.build().unwrap().definition().clone()
}
fn prepare(c: &mut Compiler, d: &ChartDefinition, s: &DataStore) -> PreparedChart {
    c.prepare(
        d,
        &s.snapshot(),
        &ChartState::default(),
        CompileLimits::default(),
    )
    .unwrap()
}
fn check(p: &PreparedChart, rows: &[(u64, u64, f64, &str)], classifier: bool) {
    let expected = if classifier {
        let mut values: Vec<_> = rows.iter().map(|r| r.2).collect();
        values.sort_by(f64::total_cmp);
        ScaleFunctionSpec::Classifier(ClassifierSpec {
            domain: ClassifierDomain::Quantile(
                values.into_iter().map(|v| Some(Number(v))).collect(),
            ),
            range: vec![Value::Text("red".into()), Value::Text("blue".into())],
            unknown: None,
        })
    } else {
        let mut keys = vec![];
        // Figure order is panel order, then authored rows, preserving exact UInt keys.
        for panel in if p.panels().is_empty() {
            vec![None]
        } else {
            vec![Some("A"), Some("B")]
        } {
            for r in rows
                .iter()
                .filter(|r| panel.is_none_or(|panel| r.3 == panel))
            {
                let key = ScaleKey::Unsigned(r.1);
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        ScaleFunctionSpec::Ordinal(OrdinalSpec {
            domain: keys,
            range: vec![
                Value::Text("red".into()),
                Value::Text("blue".into()),
                Value::Text("green".into()),
            ],
            unknown: OrdinalUnknown::Implicit,
        })
    };
    let mut marks = 0;
    for layer in p.layers() {
        let legend = layer.color_legend().unwrap();
        let actual = &legend.mapping.as_ref().unwrap().function;
        if classifier {
            // Prepared descriptors retain source order; the actual exact quantile cuts are independently checked.
            let ScaleFunctionSpec::Classifier(s) = &expected else {
                unreachable!()
            };
            let ClassifierDomain::Quantile(values) = &s.domain else {
                unreachable!()
            };
            let n = values.len();
            let median = if n % 2 == 0 {
                (values[n / 2 - 1].unwrap().0 + values[n / 2].unwrap().0) / 2.
            } else {
                values[n / 2].unwrap().0
            };
            let ScaleFunctionSpec::Classifier(s) = actual else {
                panic!("classifier")
            };
            assert_eq!(
                ClassifierScale::new(s.clone()).unwrap().thresholds(),
                vec![Some(Number(median))]
            );
            let ClassifierDomain::Quantile(samples) = &s.domain else {
                unreachable!()
            };
            assert_eq!(
                samples.len(),
                rows.len(),
                "Duplicates are samples; broadcast is not another population"
            );
        } else {
            assert_eq!(actual, &expected);
        }

        for mark in layer.marks() {
            let PreparedGeometry::Point(point) = mark.geometry else {
                panic!("point")
            };
            let r = rows.iter().find(|r| r.2 == point.x()).unwrap();
            let index = match &expected {
                ScaleFunctionSpec::Ordinal(s) => {
                    s.domain
                        .iter()
                        .position(|k| k == &ScaleKey::Unsigned(r.1))
                        .unwrap()
                        % 3
                }
                ScaleFunctionSpec::Classifier(s) => {
                    let ClassifierDomain::Quantile(values) = &s.domain else {
                        unreachable!()
                    };
                    let n = values.len();
                    let median = if n % 2 == 0 {
                        (values[n / 2 - 1].unwrap().0 + values[n / 2].unwrap().0) / 2.
                    } else {
                        values[n / 2].unwrap().0
                    };
                    usize::from(r.2 >= median)
                }
                _ => unreachable!(),
            };
            let (red, green, blue) = [(255, 0, 0), (0, 0, 255), (0, 128, 0)][index];
            assert_eq!(
                mark.style.color,
                scene::Color {
                    red,
                    green,
                    blue,
                    alpha: 255
                }
            );
            marks += 1;
        }
    }
    assert_eq!(marks, rows.len());
}
#[test]
fn ordinal_and_quantile_updates_match_independent_retained_batches_and_keep_old_owners() {
    let base = 9_007_199_254_740_992;
    for faceted in [false, true] {
        for classifier in [false, true] {
            let mut rows = vec![
                (1, base + 1, 0., "A"),
                (2, base + 2, 2., "B"),
                (3, base + 1, 4., "A"),
            ];
            let initial = data(&rows);
            let d = definition(&initial, faceted, classifier);
            let mut s = DataStore::new(
                SourceEpoch::new(1),
                vec![(D, initial.batch().clone())],
                DataLimits::default(),
            )
            .unwrap();
            let mut compiler = Compiler::new();
            let old = prepare(&mut compiler, &d, &s);
            check(&old, &rows, classifier);
            let old_marks: Vec<_> = old.layers().iter().map(|l| l.marks().to_vec()).collect();
            for step in 0..4 {
                let mutation = match step {
                    0 => {
                        let r = (4, base + 3, 100., "B");
                        rows.push(r);
                        Mutation::AppendBatch(data(&[r]).into())
                    }
                    1 => {
                        let r = (2, base + 3, 8., "B");
                        rows[1] = r;
                        Mutation::UpsertByKey(data(&[r]).into())
                    }
                    2 => {
                        rows.remove(0);
                        Mutation::RemoveKeys(vec![RowKey::new(1)])
                    }
                    _ => {
                        rows.remove(0);
                        Mutation::SetRetention(RetentionPolicy::Count(2))
                    }
                };
                let tx = Transaction {
                    id: TransactionId::new(format!("step{step}")).unwrap(),
                    epoch: SourceEpoch::new(1),
                    expected: vec![s.snapshot().get().unwrap().dataset(D).unwrap().version()],
                    operations: vec![Operation {
                        dataset: D,
                        mutation,
                    }],
                };
                assert!(matches!(s.apply(tx), CommitOutcome::Applied(_)));
                let updated = prepare(&mut compiler, &d, &s);
                check(&updated, &rows, classifier);
                let batch = DataStore::new(
                    SourceEpoch::new(1),
                    vec![(D, data(&rows).into())],
                    DataLimits::default(),
                )
                .unwrap();
                let fresh = prepare(&mut Compiler::new(), &d, &batch);
                check(&fresh, &rows, classifier);
                for (a, b) in updated.layers().iter().zip(fresh.layers()) {
                    assert_eq!(a.color_legend(), b.color_legend());
                }
                for (a, b) in old.layers().iter().zip(&old_marks) {
                    assert_eq!(a.marks(), b);
                }
            }
        }
    }
}

#[test]
fn faceted_quantiles_count_duplicate_observations_and_broadcast_population_once() {
    let rows = [
        (1, 1, 0., "A"),
        (2, 2, 0., "A"),
        (3, 3, 0., "A"),
        (4, 4, 100., "B"),
    ];
    let initial = data(&rows);
    for broadcast in [false, true] {
        let mut d = definition(&initial, true, true);
        if broadcast {
            d.layers[0].facet = FacetTarget::Broadcast;
        }
        let s = DataStore::new(
            SourceEpoch::new(1),
            vec![(D, initial.batch().clone())],
            DataLimits::default(),
        )
        .unwrap();
        let p = prepare(&mut Compiler::new(), &d, &s);
        for layer in p.layers() {
            let legend = layer.color_legend().unwrap();
            let ScaleFunctionSpec::Classifier(spec) = &legend.mapping.as_ref().unwrap().function
            else {
                unreachable!()
            };
            let ClassifierDomain::Quantile(samples) = &spec.domain else {
                unreachable!()
            };
            assert_eq!(samples.len(), 4);
            assert_eq!(
                samples.iter().filter(|n| **n == Some(Number(0.))).count(),
                3
            );
            assert_eq!(
                ClassifierScale::new(spec.clone()).unwrap().thresholds(),
                [Some(Number(0.))]
            );
        }
        assert_eq!(p.layers()[0].color_legend(), p.layers()[1].color_legend());
    }
}

#[test]
fn classifier_labels_are_compact_without_merging_distinct_cutpoints() {
    let scale = ScaleConstructor::Quantile
        .create(ScaleOptions {
            domain: Some(
                [0., 0., 1., 2., 3., 5., 8., 13., 21.]
                    .map(|n| ScaleInput::Number(Number(n)))
                    .to_vec(),
            ),
            range: Some(
                ["red", "green", "blue"]
                    .map(|s| Value::Text(s.into()))
                    .to_vec(),
            ),
            ..Default::default()
        })
        .unwrap();
    let guide = MappedScale::for_colors(scale.mapped(ScaleTraining::Authored).unwrap())
        .unwrap()
        .legend(
            ScaleId::new(1),
            scene::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        )
        .unwrap();
    assert_eq!(
        guide
            .entries
            .iter()
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>(),
        ["0 – 1.66667", "1.66667 – 6", "6 – 21"]
    );
    assert_eq!(
        guide.intervals[0].upper,
        Some(ScaleKey::Number(Number(1.6666666666666665)))
    );
    let scale = ScaleConstructor::Quantize
        .create(ScaleOptions {
            domain: Some(
                [1., 1.0000000002]
                    .map(|n| ScaleInput::Number(Number(n)))
                    .to_vec(),
            ),
            range: Some(["red", "blue"].map(|s| Value::Text(s.into())).to_vec()),
            ..Default::default()
        })
        .unwrap();
    let guide = MappedScale::for_colors(scale.mapped(ScaleTraining::Authored).unwrap())
        .unwrap()
        .legend(
            ScaleId::new(1),
            scene::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        )
        .unwrap();
    assert_eq!(guide.entries[0].0, "1 – 1.0000000001");
    assert_eq!(guide.entries[1].0, "1.0000000001 – 1.0000000002");
}

#[test]
fn numeric_thresholds_read_integer_columns_numerically_and_explicit_keys_stay_exact() {
    for integer in [false, true] {
        let mut d = Data::columns().column("x", [0., 1., 2.]);
        d = if integer {
            d.column("value", vec![0_i64, 5, 10])
        } else {
            d.column("value", vec![0., 5., 10.])
        };
        let data = d.build().unwrap();
        let scale = ScaleConstructor::Threshold
            .create(ScaleOptions {
                domain: Some(
                    [5., 10.]
                        .map(|n| ScaleInput::Key(ScaleKey::Number(Number(n))))
                        .to_vec(),
                ),
                range: Some([3., 6., 9.].map(Value::number).to_vec()),
                ..Default::default()
            })
            .unwrap();
        let p = plot(data)
            .aes(aes().x("x").y("value"))
            .layer(points().numeric_scale(
                NumericAesthetic::Size,
                "value",
                scale.mapped(ScaleTraining::Authored).unwrap(),
            ))
            .build()
            .unwrap();
        let prepared = Compiler::new()
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                CompileLimits::default(),
            )
            .unwrap();
        assert_eq!(
            prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| m.style.radius)
                .collect::<Vec<_>>(),
            [3., 6., 9.]
        );
    }
    let base = 9_007_199_254_740_992;
    let data = Data::columns()
        .column("x", [0., 1., 2.])
        .column("value", vec![base, base + 1, base + 2])
        .build()
        .unwrap();
    let spec = MappedScaleSpec::authored(ScaleFunctionSpec::Threshold(ThresholdSpec {
        domain: vec![ScaleKey::Unsigned(base + 1), ScaleKey::Unsigned(base + 2)],
        range: vec![Value::number(3.), Value::number(6.), Value::number(9.)],
        unknown: None,
    }));
    assert!(spec.categorical());
    let p = plot(data)
        .aes(aes().x("x").y("x"))
        .layer(points().numeric_scale(NumericAesthetic::Size, "value", spec))
        .build()
        .unwrap();
    let prepared = Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(
        prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| m.style.radius)
            .collect::<Vec<_>>(),
        [3., 6., 9.]
    );
}
