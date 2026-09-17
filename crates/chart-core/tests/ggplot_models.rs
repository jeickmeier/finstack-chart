//! GG10 public model operation and prediction population contracts.
use chart_core::{grammar::*, prelude::*};
#[test]
fn weighted_linear_predictions_and_wire_use_shared_model_stage() {
    let data = Data::columns()
        .column("x", vec![0., 1., 2., 3., 4.])
        .column("y", vec![2., 5., 8., 11., 14.])
        .column("w", vec![1., 2., 1., 3., 1.])
        .build()
        .unwrap();
    let chart = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            line().aes(aes().x("x").y("y")).stat(
                model_stat(ModelOptions {
                    method: ModelMethod::Linear,
                    xseq: Some(vec![0.5, 2.5, 4.5]),
                    ..Default::default()
                })
                .weight("w"),
            ),
        )
        .build()
        .unwrap();
    assert_eq!(chart.chart().unwrap().definition().wire_version(), 77);
    let prepared = chart.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 3);
    for (r, y) in rows.iter().zip([3.5, 9.5, 15.5]) {
        assert!((r.value(&StatField::Y).unwrap() - y).abs() < 1e-11);
        assert!(r.value(&StatField::StandardError).unwrap() < 1e-11);
        assert_eq!(r.count, 5);
        assert_eq!(r.members.len(), 5);
        assert!(matches!(
            r.target,
            chart_core::provenance::Target::Derived { .. }
        ));
    }
}
#[test]
fn malformed_model_controls_reject_before_empty_evaluation() {
    let data = Data::columns()
        .column("x", Vec::<f64>::new())
        .column("y", Vec::<f64>::new())
        .build()
        .unwrap();
    let chart = plot(data)
        .layer(
            line()
                .aes(aes().x("x").y("y"))
                .stat(model_stat(ModelOptions {
                    level: 1.2,
                    ..Default::default()
                })),
        )
        .build();
    assert!(
        chart.is_err()
            || chart
                .unwrap()
                .chart()
                .and_then(|mut c| c.prepare())
                .is_err()
    );
}

#[test]
fn smooth_ribbon_and_quantile_curves_keep_distinct_runs() {
    let data = Data::columns()
        .column("x", vec![0., 1., 2., 3., 4., 5.])
        .column("y", vec![0., 1.5, 1.8, 3.7, 3.5, 5.8])
        .build()
        .unwrap();
    let mut p = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            smooth()
                .aes(aes().x("x").y("y"))
                .stat(model_stat(ModelOptions {
                    method: ModelMethod::Linear,
                    n: 12,
                    ..Default::default()
                })),
        )
        .build()
        .unwrap()
        .chart()
        .unwrap();
    let prepared = p.prepare().unwrap();
    let marks = prepared.layers()[0].marks();
    assert_eq!(marks.len(), 2);
    assert!(matches!(
        marks[0].geometry,
        PreparedGeometry::BandRun { .. }
    ));
    assert!(matches!(marks[1].geometry, PreparedGeometry::LineRun(_)));
    assert_eq!(marks[0].targets.len(), 12);
    assert_eq!(marks[1].targets.len(), 12);
    assert_eq!(marks[0].style.fill.unwrap().alpha, 102);
    assert_eq!(marks[1].style.color.alpha, 255);
    let mut q = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(quantile().aes(aes().x("x").y("y")))
        .build()
        .unwrap()
        .chart()
        .unwrap();
    let prepared = q.prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 3);
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 300);
    assert_eq!(
        rows.iter()
            .map(|r| r.group.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[test]
fn explicit_model_input_and_source_expression_resolve() {
    let data = Data::columns()
        .column("predictor", vec![0., 1., 2., 3.])
        .column("response", vec![1., 3., 5., 7.])
        .build()
        .unwrap();
    let mut chart = plot(data)
        .layer(
            line().stat(
                model_stat(ModelOptions {
                    method: ModelMethod::Linear,
                    n: 3,
                    ..Default::default()
                })
                .input("predictor")
                .y("response"),
            ),
        )
        .build()
        .unwrap()
        .chart()
        .unwrap();
    let prepared = chart.prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 3);
    assert!((rows[1].value(&StatField::Y).unwrap() - 4.).abs() < 1e-12);
}

struct RegisteredModel {
    portable: bool,
    wrong_length: bool,
}
impl CustomModel for RegisteredModel {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.model", chart_core::Revision::new(1), self.portable)
    }
    fn validate(&self, p: &serde_json::Value) -> chart_core::ChartResult<()> {
        if p.is_null() {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Invalid model parameters.",
                "Use null.",
            ))
        }
    }
    fn predict(
        &self,
        input: ModelInput<'_>,
        _: &serde_json::Value,
    ) -> chart_core::ChartResult<Vec<ModelEstimate>> {
        assert_eq!(input.x.len(), input.y.len());
        assert_eq!(input.x.len(), input.weights.len());
        Ok(input
            .grid
            .iter()
            .take(if self.wrong_length {
                0
            } else {
                input.grid.len()
            })
            .map(|x| ModelEstimate {
                mean: Some(2. * x),
                standard_error: Some(0.5),
                lower: Some(2. * x - 1.),
                upper: Some(2. * x + 1.),
            })
            .collect())
    }
}
#[test]
fn registered_models_validate_alignment_and_portability() {
    use std::sync::Arc;
    for (portable, wrong_length) in [(true, false), (false, false), (true, true)] {
        let mut registry = ExtensionRegistry::default();
        registry
            .register_model(Arc::new(RegisteredModel {
                portable,
                wrong_length,
            }))
            .unwrap();
        let p = plot(
            Data::columns()
                .column("x", vec![0., 1., 2.])
                .column("y", vec![0., 1., 4.])
                .build()
                .unwrap(),
        )
        .extensions(Arc::new(registry))
        .layer(
            smooth()
                .aes(aes().x("x").y("y"))
                .stat(model_stat(ModelOptions {
                    method: ModelMethod::Registered {
                        operation: OperationRef::new("test.model", chart_core::Revision::new(1)),
                        parameters: serde_json::Value::Null,
                    },
                    xseq: Some(vec![0.5, 1.5]),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        assert_eq!(p.to_json().is_ok(), portable);
        let result = p.chart().unwrap().prepare();
        assert_eq!(result.is_ok(), !wrong_length);
        if let Ok(prepared) = result {
            let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
                panic!()
            };
            assert_eq!(rows[0].value(&StatField::Y), Some(1.));
            assert_eq!(rows[1].value(&StatField::Y), Some(3.));
        }
    }
}

#[test]
fn automatic_method_uses_largest_group_across_facets() {
    for large in [999, 1000] {
        let x = (0..large)
            .map(|i| i as f64 / (large - 1) as f64)
            .chain((0..40).map(|i| i as f64 / 39.))
            .collect::<Vec<_>>();
        let y = x
            .iter()
            .map(|x| libm::sin(6. * x) + 0.1 * libm::cos(37. * x))
            .collect::<Vec<_>>();
        let group = std::iter::repeat_n("large", large)
            .chain(std::iter::repeat_n("small", 40))
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("x", x)
            .column("y", y)
            .column("panel", group)
            .build()
            .unwrap();
        let method = if large < 1000 {
            ModelMethod::Loess {
                span: 0.75,
                degree: 2,
                cell: 0.2,
                surface: LoessSurface::Interpolate,
                family: LoessFamily::Gaussian,
                iterations: 4,
                normalize: true,
                exact_statistics: false,
                approximate_trace: false,
            }
        } else {
            ModelMethod::Gam {
                basis_dimension: 10,
                knots: None,
                iterations: 120,
                tolerance: 1e-9,
            }
        };
        let build = |method| {
            plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .layer(
                    line()
                        .aes(aes().x("x").y("y"))
                        .stat(model_stat(ModelOptions {
                            method,
                            xseq: Some(vec![0.2, 0.5, 0.8]),
                            se: false,
                            ..Default::default()
                        })),
                )
                .facet(facet_wrap("panel").free_x(true))
                .build()
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap()
        };
        let automatic = build(ModelMethod::Auto);
        let explicit = build(method);
        assert_eq!(automatic.panels().len(), 2);
        for (a, b) in automatic.panels().iter().zip(explicit.panels()) {
            let PreparedRows::Statistical(a) = a.chart.layers()[0].table().rows() else {
                panic!()
            };
            let PreparedRows::Statistical(b) = b.chart.layers()[0].table().rows() else {
                panic!()
            };
            assert_eq!(a.len(), 3);
            assert_eq!(b.len(), 3);
            for (a, b) in a.iter().zip(b.iter()) {
                assert!(
                    (a.value(&StatField::Y).unwrap() - b.value(&StatField::Y).unwrap()).abs()
                        < 1e-12
                );
            }
        }
    }
}

#[test]
fn model_updates_match_batch_and_preserve_old_snapshots() {
    type Input = (u64, f64, f64);
    fn data(rows: &[Input]) -> Data {
        Data::columns()
            .name("observations")
            .keys(rows.iter().map(|r| r.0))
            .column("x", rows.iter().map(|r| r.1).collect::<Vec<_>>())
            .column("y", rows.iter().map(|r| r.2).collect::<Vec<_>>())
            .build()
            .unwrap()
    }
    fn author(data: Data, mode: usize) -> Plot {
        let method = match mode {
            0 => ModelMethod::Linear,
            1 => ModelMethod::Loess {
                span: 1.,
                degree: 2,
                cell: 0.2,
                surface: LoessSurface::Direct,
                family: LoessFamily::Gaussian,
                iterations: 4,
                normalize: true,
                exact_statistics: false,
                approximate_trace: false,
            },
            _ => ModelMethod::Gam {
                basis_dimension: 6,
                knots: None,
                iterations: 120,
                tolerance: 1e-9,
            },
        };
        let stat = model_stat(ModelOptions {
            method,
            n: 5,
            se: false,
            ..Default::default()
        })
        .x("x")
        .y("y");
        plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(points().stat(stat))
            .build()
            .unwrap()
    }
    for mode in 0..3 {
        let mut expected = (1..=12)
            .map(|i| (i, i as f64, libm::sin(i as f64)))
            .collect::<Vec<_>>();
        let mut chart = author(data(&expected), mode).chart().unwrap();
        let old = chart.prepare().unwrap();
        let old_rows = match old.layers()[0].table().rows() {
            PreparedRows::Statistical(rows) => rows.to_vec(),
            _ => panic!(),
        };
        for operation in 0..3 {
            let transaction = match operation {
                0 => {
                    let added = (13, 13., 0.5);
                    expected.push(added);
                    chart
                        .transaction()
                        .unwrap()
                        .append("observations", data(&[added]))
                        .build()
                        .unwrap()
                }
                1 => {
                    let changed = (2, 2.5, 6.);
                    expected[1] = changed;
                    chart
                        .transaction()
                        .unwrap()
                        .upsert("observations", data(&[changed]))
                        .build()
                        .unwrap()
                }
                _ => {
                    expected.retain(|r| r.0 != 1);
                    chart
                        .transaction()
                        .unwrap()
                        .remove("observations", [1])
                        .build()
                        .unwrap()
                }
            };
            assert!(matches!(
                chart.apply_transaction(transaction).unwrap(),
                chart_core::transaction::CommitOutcome::Applied(_)
            ));
            let live = chart.prepare().unwrap();
            let fresh = author(data(&expected), mode)
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            let (PreparedRows::Statistical(a), PreparedRows::Statistical(b)) = (
                live.layers()[0].table().rows(),
                fresh.layers()[0].table().rows(),
            ) else {
                panic!()
            };
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b.iter()) {
                assert_eq!(a.values, b.values);
                assert_eq!(a.count, b.count);
                assert_eq!(a.members, b.members);
            }
        }
        let PreparedRows::Statistical(unchanged) = old.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(unchanged.as_ref(), old_rows.as_slice());
    }
}
#[test]
fn integer_smoothing_grids_preserve_source_type_and_quantile_default() {
    for integer in [false, true] {
        let columns = Data::columns().column("y", vec![1., 5., 9.]);
        let data = if integer {
            columns.column("x", vec![0i64, 2, 4])
        } else {
            columns.column("x", vec![0., 2., 4.])
        }
        .build()
        .unwrap();
        let p = plot(data)
            .layer(
                line()
                    .aes(aes().x("x").y("y"))
                    .stat(model_stat(ModelOptions {
                        method: ModelMethod::Linear,
                        n: 7,
                        ..Default::default()
                    })),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(rows.len(), if integer { 3 } else { 7 });
    }
}

#[test]
fn coordinate_zoom_preserves_fit_while_source_filter_refits() {
    let data = Data::columns()
        .column("x", vec![0., 1., 2., 3., 4., 5.])
        .column("y", vec![0., 1., 4., 9., 16., 25.])
        .build()
        .unwrap();
    let build = |zoom, filtered| {
        let mut layer = line()
            .aes(aes().x("x").y("y"))
            .stat(model_stat(ModelOptions {
                method: ModelMethod::Linear,
                xseq: Some(vec![1., 2., 3.]),
                ..Default::default()
            }));
        if filtered {
            layer = layer.filter(filter("x").maximum(3.));
        }
        let mut p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .layer(layer);
        if zoom {
            p = p.coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
                xlim: Some([
                    Some(chart_core::composition::ScaleValue::Number(1.)),
                    Some(chart_core::composition::ScaleValue::Number(3.)),
                ]),
                ..Default::default()
            }));
        }
        p.build().unwrap().chart().unwrap().prepare().unwrap()
    };
    let full = build(false, false);
    let zoomed = build(true, false);
    let filtered = build(false, true);
    let table = |p: &PreparedChart| match p.layers()[0].table().rows() {
        PreparedRows::Statistical(rows) => rows
            .iter()
            .map(|r| (r.values.clone(), r.count))
            .collect::<Vec<_>>(),
        _ => panic!(),
    };
    assert_eq!(table(&full), table(&zoomed));
    assert_ne!(table(&full), table(&filtered));
}
