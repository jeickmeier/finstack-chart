//! GG09: pinned empirical and theoretical samples through the actual compiler.
use chart_core::{grammar::*, prelude::*};
fn fixtures() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/univariate-controls.json"
    ))
    .unwrap()
}
fn near(actual: Option<f64>, expected: &serde_json::Value) {
    let expected = expected.as_f64().or_else(|| match expected.as_str() {
        Some("Inf") => Some(f64::INFINITY),
        Some("-Inf") => Some(f64::NEG_INFINITY),
        _ => None,
    });
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(
            a == b || (a - b).abs() <= 1e-10_f64.max(b.abs() * 1e-12),
            "{a} != {b}"
        ),
        (a, b) => assert_eq!(a, b),
    }
}
#[test]
fn qq_distributions_and_lines_match_all_pinned_samples() {
    for case in fixtures()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["family"].as_str(), Some("qq" | "qq_line")))
    {
        let sample = case["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["sample"].as_f64().unwrap())
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("sample", sample.clone())
            .build()
            .unwrap();
        let distribution = match case["params"]["distribution"].as_str().unwrap() {
            "uniform" => AnalyticFunction::UniformQuantile { min: 0., max: 1. },
            "logistic" => AnalyticFunction::LogisticQuantile {
                location: 0.,
                scale: 1.,
            },
            _ => AnalyticFunction::default(),
        };
        let stat = univariate_stat(UnivariateKind::Qq {
            distribution,
            quantiles: None,
            line: case["family"] == "qq_line",
            probabilities: [0.25, 0.75],
            full_range: false,
        })
        .input("sample");
        let plot = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(points().stat(stat))
            .build()
            .unwrap();
        let restored = Plot::from_json(&plot.to_json().unwrap()).unwrap();
        let prepared = restored.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        let expected = case["rows"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len(), "{}", case["name"]);
        for (row, reference) in rows.iter().zip(expected) {
            near(row.value(&StatField::X), &reference["x"]);
            near(row.value(&StatField::Y), &reference["y"]);
            assert_eq!(row.members.len(), sample.len());
            assert_eq!(row.count, sample.len() as u64);
        }
    }
}
#[test]
fn ecdf_reference_rows_and_membership_survive_compilation() {
    for case in fixtures()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["family"] == "ecdf" && c["params"].get("weighted").is_some())
    {
        let input = case["input"].as_array().unwrap();
        let data = Data::columns()
            .column(
                "x",
                input
                    .iter()
                    .map(|r| r["x"].as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "w",
                input.iter().map(|r| r["w"].as_f64()).collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let mut stat = univariate_stat(UnivariateKind::Ecdf {
            n: case["params"]["n"].as_u64().map(|n| n as usize),
            pad: case["params"]["pad"].as_bool().unwrap(),
        })
        .input("x");
        if case["params"]["weighted"] == true {
            stat = stat.weight("w");
        }
        let prepared = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(points().stat(stat))
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        let expected = case["rows"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len(), "{}", case["name"]);
        for (row, reference) in rows.iter().zip(expected) {
            near(row.value(&StatField::X), &reference["x"]);
            near(row.value(&StatField::Y), &reference["y"]);
            assert_eq!(row.members.len(), input.len());
        }
    }
}
#[test]
fn function_evaluates_inverse_log_coordinates_before_output_projection() {
    for logarithmic in [false, true] {
        let c = fixtures();
        let case = c["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["family"] == "function" && c["params"]["log_x"] == logarithmic)
            .unwrap();
        let f = AnalyticFunction::Expression(Expression::read(FunctionArgument::Value).binary(
            ExpressionBinary::Multiply,
            Expression::read(FunctionArgument::Value),
        ));
        let stat = univariate_stat(UnivariateKind::Function {
            function: f,
            n: 5,
            range: None,
        })
        .input("x");
        let mut builder = plot(Data::columns().column("x", [1., 100.]).build().unwrap())
            .profile(Profile::Ggplot2_4_0_3)
            .layer(line().stat(stat));
        if logarithmic {
            builder = builder.x_axis(x_axis().scale(scale_log(10.)));
        }
        let prepared = builder.build().unwrap().chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        for (row, expected) in rows.iter().zip(case["rows"].as_array().unwrap()) {
            near(row.value(&StatField::X), &expected["x"]);
            near(row.value(&StatField::Y), &expected["y"]);
        }
    }
}
#[test]
fn matrix_connections_preserve_reference_coordinates_and_full_membership() {
    let fixture = fixtures();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "connect-44")
        .unwrap();
    let data = Data::columns()
        .column("x", [3., 1., 2.])
        .column("y", [2., -1., 4.])
        .build()
        .unwrap();
    let prepared = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            line().stat(
                connect_stat(Connection::Matrix(vec![[0., 0.], [0.25, 0.75], [1., 1.]]))
                    .x("x")
                    .y("y"),
            ),
        )
        .build()
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    let expected = case["rows"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for (row, reference) in rows.iter().zip(expected) {
        near(row.value(&StatField::X), &reference["x"]);
        near(row.value(&StatField::Y), &reference["y"]);
        assert_eq!(row.members.len(), 3);
    }
}
struct AffineFunction {
    portable: bool,
    wrong_length: bool,
}
impl CustomAnalyticFunction for AffineFunction {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.affine", chart_core::Revision::new(1), self.portable)
    }
    fn validate(&self, parameters: &serde_json::Value) -> chart_core::ChartResult<()> {
        assert!(parameters.is_null());
        Ok(())
    }
    fn evaluate(
        &self,
        input: &[f64],
        _: &serde_json::Value,
    ) -> chart_core::ChartResult<Vec<Option<f64>>> {
        if self.wrong_length {
            Ok(vec![])
        } else {
            Ok(input.iter().map(|x| Some(2. * x + 1.)).collect())
        }
    }
}
#[test]
fn registered_function_and_distribution_enforce_identity_alignment_and_portability() {
    use std::sync::Arc;
    for portable in [false, true] {
        let mut registry = ExtensionRegistry::default();
        registry
            .register_analytic_function(Arc::new(AffineFunction {
                portable,
                wrong_length: false,
            }))
            .unwrap();
        let registry = Arc::new(registry);
        let descriptor = AnalyticFunction::Registered {
            operation: OperationRef::new("test.affine", chart_core::Revision::new(1)),
            parameters: serde_json::Value::Null,
        };
        for kind in [
            UnivariateKind::Function {
                function: descriptor.clone(),
                n: 3,
                range: Some([0., 1.]),
            },
            UnivariateKind::Qq {
                distribution: descriptor.clone(),
                quantiles: Some(vec![0., 0.5, 1.]),
                line: false,
                probabilities: [0.25, 0.75],
                full_range: false,
            },
        ] {
            let make = || {
                plot(Data::columns().column("x", [0., 0.5, 1.]).build().unwrap())
                    .layer(points().stat(univariate_stat(kind.clone()).input("x")))
            };
            assert!(make().build().is_err());
            let p = make().extensions(registry.clone()).build().unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
                panic!()
            };
            let field = if matches!(kind, UnivariateKind::Function { .. }) {
                StatField::Y
            } else {
                StatField::X
            };
            assert_eq!(
                rows.iter()
                    .map(|r| r.value(&field).unwrap())
                    .collect::<Vec<_>>(),
                vec![1., 2., 3.]
            );
            assert_eq!(p.to_json().is_ok(), portable);
        }
    }
    let mut registry = ExtensionRegistry::default();
    registry
        .register_analytic_function(Arc::new(AffineFunction {
            portable: true,
            wrong_length: true,
        }))
        .unwrap();
    let p = plot(Data::columns().column("x", [0., 1.]).build().unwrap())
        .extensions(Arc::new(registry))
        .layer(
            function_curve(AnalyticFunction::Registered {
                operation: OperationRef::new("test.affine", chart_core::Revision::new(1)),
                parameters: serde_json::Value::Null,
            })
            .stat(
                function_stat(AnalyticFunction::Registered {
                    operation: OperationRef::new("test.affine", chart_core::Revision::new(1)),
                    parameters: serde_json::Value::Null,
                })
                .input("x"),
            ),
        )
        .build()
        .unwrap();
    assert!(p.chart().unwrap().prepare().is_err());
}
#[test]
fn analytical_aesthetics_drop_only_varying_columns_and_unique_keeps_tuples() {
    let data = Data::columns()
        .column("x", [1., 1., 2., 3.])
        .column("y", [2., 2., 3., 4.])
        .column("fixed", [2., 2., 2., 2.])
        .build()
        .unwrap();
    for stat in [ecdf_stat().input("x"), density_stat().input("x")] {
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .layer(
                points()
                    .aes(aes().color("y").size("fixed").group_all())
                    .stat(stat),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| {
            r.retained
                .component(data.field("fixed").unwrap().id())
                .is_some()
        }));
        assert!(rows.iter().all(|r| {
            r.retained
                .component(data.field("y").unwrap().id())
                .is_none()
        }));
    }
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(points().stat(unique_stat().x("x").y("y")))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|r| r.count == 1
        && r.members.len() == 1
        && matches!(r.target, chart_core::provenance::Target::Source(_))));
}
#[test]
fn function_without_mapped_input_uses_unit_range_or_explicit_axis_limits() {
    for bounds in [None, Some([2., 6.])] {
        let mut builder = plot(
            Data::columns()
                .column("unrelated", [7., 9.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .layer(function_curve(AnalyticFunction::Expression(
            Expression::read(FunctionArgument::Value),
        )));
        if let Some([lo, hi]) = bounds {
            builder = builder.x_axis(xlim(PositionalLimits::Numeric(vec![
                Some(chart_core::interpolate::Number(lo)),
                Some(chart_core::interpolate::Number(hi)),
            ])));
        }
        let prepared = builder.build().unwrap().chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        let [lo, hi] = bounds.unwrap_or([0., 1.]);
        assert_eq!(rows.len(), 101);
        assert_eq!(rows[0].value(&StatField::X), Some(lo));
        assert_eq!(rows[100].value(&StatField::X), Some(hi));
    }
}
#[test]
fn reference_area_defaults_to_grouped_alignment_and_stack() {
    for case in fixtures()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["family"] == "align")
    {
        let input = case["input"].as_array().unwrap();
        let data = Data::columns()
            .column(
                "x",
                input
                    .iter()
                    .map(|r| r["x"].as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "y",
                input
                    .iter()
                    .map(|r| r["y"].as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "g",
                input
                    .iter()
                    .map(|r| r["g"].as_str().unwrap().to_string())
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(area().aes(aes().x("x").y("y").group("g")))
            .build()
            .unwrap();
        assert!(matches!(
            p.definition().layers[0].position,
            Position::GgplotStack(_)
        ));
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        let expected = case["rows"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len());
        for (row, reference) in rows.iter().zip(expected) {
            near(row.value(&StatField::X), &reference["x"]);
            near(row.value(&StatField::Y), &reference["y"]);
        }
    }
}
#[test]
fn analytical_updates_match_fresh_batch_and_preserve_old_snapshot() {
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
        let stat = match mode {
            0 => ecdf_stat().input("x"),
            1 => qq_stat().input("y"),
            2 => unique_stat().x("x").y("y"),
            3 => connect_stat(Connection::Matrix(vec![[0., 0.], [0.5, 0.5], [1., 1.]]))
                .x("x")
                .y("y"),
            4 => align_stat().x("x").y("y"),
            _ => function_stat(AnalyticFunction::Expression(Expression::read(
                FunctionArgument::Value,
            )))
            .input("x"),
        };
        plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(points().stat(stat))
            .build()
            .unwrap()
    }
    for mode in 0..6 {
        let mut expected = vec![(1, 1., 2.), (2, 2., 3.), (3, 4., 1.)];
        let mut chart = author(data(&expected), mode).chart().unwrap();
        let old = chart.prepare().unwrap();
        let old_rows = match old.layers()[0].table().rows() {
            PreparedRows::Statistical(rows) => rows.to_vec(),
            _ => panic!(),
        };
        for operation in 0..3 {
            let transaction = match operation {
                0 => {
                    let added = (4, 3., 5.);
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
struct PopulationShift;
impl PreparedTransform for PopulationShift {
    fn forward(&self, v: f64) -> f64 {
        v + 1.
    }
    fn inverse(&self, v: f64) -> f64 {
        v - 1.
    }
    fn is_pointwise(&self) -> bool {
        false
    }
    fn forward_batch(&self, v: &[f64]) -> chart_core::ChartResult<Vec<f64>> {
        Ok(v.iter().map(|x| x + v.len() as f64).collect())
    }
    fn inverse_batch(&self, v: &[f64]) -> chart_core::ChartResult<Vec<f64>> {
        Ok(v.iter().map(|x| x - v.len() as f64).collect())
    }
    fn domain(&self) -> [chart_core::interpolate::Number; 2] {
        [f64::NEG_INFINITY, f64::INFINITY].map(chart_core::interpolate::Number)
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        false
    }
}
impl CustomTransformFactory for PopulationShift {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.population_shift", chart_core::Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> chart_core::ChartResult<()> {
        Ok(())
    }
    fn compile(
        &self,
        _: &serde_json::Value,
    ) -> chart_core::ChartResult<std::sync::Arc<dyn PointwiseTransform>> {
        Ok(std::sync::Arc::new(Self))
    }
}
#[test]
fn function_outputs_receive_one_complete_registered_vector_transform() {
    let mut registry = ExtensionRegistry::default();
    registry
        .register_transform(std::sync::Arc::new(PopulationShift))
        .unwrap();
    let transform = chart_core::scales::ScaleTransform::Ggplot {
        transform: chart_core::scales::GgplotTransform::Registered {
            selection: TransformSelection::new(TransformOperation {
                operation: OperationRef::new("test.population_shift", chart_core::Revision::new(1)),
                parameters: serde_json::Value::Null,
            })
            .into(),
        },
    };
    let p = plot(Data::columns().column("x", [1., 3.]).build().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(std::sync::Arc::new(registry))
        .y_axis(y_axis().scale(chart_core::plot::scale_transform(transform)))
        .layer(
            points().stat(
                univariate_stat(UnivariateKind::Function {
                    function: AnalyticFunction::Expression(Expression::read(
                        FunctionArgument::Value,
                    )),
                    n: 3,
                    range: None,
                })
                .input("x"),
            ),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let y = prepared.layers()[0].domains().y.unwrap();
    assert_eq!(y.minimum, 4.);
    assert_eq!(y.maximum, 6.);
}
#[test]
fn named_connections_select_shared_step_curves() {
    for (connection, direction) in [
        (Connection::Hv, StepDirection::Hv),
        (Connection::Vh, StepDirection::Vh),
        (Connection::Mid, StepDirection::Mid),
    ] {
        let p = plot(
            Data::columns()
                .column("x", [1., 2.])
                .column("y", [2., 4.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .layer(line().stat(connect_stat(connection).x("x").y("y")))
        .build()
        .unwrap();
        assert_eq!(
            p.definition().layers[0].recipe,
            Some(BuiltinRecipe::Step(direction))
        );
        assert!(
            !p.chart().unwrap().prepare().unwrap().layers()[0]
                .marks()
                .is_empty()
        );
    }
}
