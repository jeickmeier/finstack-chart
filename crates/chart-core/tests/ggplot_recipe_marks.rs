//! FIX-GG07 independent primitive/default-stat contracts.
use chart_core::{grammar::*, prelude::*};
#[test]
fn stat_sum_partitions_joint_positions_and_all_mapped_aesthetics() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/count-mark-controls.json"
    ))
    .unwrap();
    let data = Data::columns()
        .column("x", [1., 1., 1., 1., 2., 2.])
        .column("y", [1., 1., 1., 1., 2., 2.])
        .column("colour", [1., 1., 2., 2., 1., 1.])
        .column("shape", ["a", "b", "a", "a", "a", "a"])
        .column("weight", [1., 2., 3., -1., 0., 4.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().color("colour").shape("shape"))
        .layer(
            points()
                .recipe(BuiltinRecipe::Count(CountRecipe::default()))
                .stat(
                    count()
                        .sum_count()
                        .x("x")
                        .y("y")
                        .group("shape")
                        .count_partition("colour")
                        .count_partition("shape")
                        .count_weight("weight"),
                ),
        )
        .build()
        .unwrap();
    let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 4);
    let mut actual = rows
        .iter()
        .map(|r| {
            let value = |field| {
                r.values
                    .iter()
                    .find(|v| v.field == field)
                    .unwrap()
                    .value
                    .unwrap()
            };
            (
                value(StatField::X),
                value(StatField::Y),
                value(StatField::WeightedCount),
                value(StatField::Proportion),
                r.count,
            )
        })
        .collect::<Vec<_>>();
    let mut expected = fixture["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["x"].as_f64().unwrap(),
                r["y"].as_f64().unwrap(),
                r["n"].as_f64().unwrap(),
                r["prop"].as_f64().unwrap(),
                if r["n"].as_f64().unwrap() == 4. || r["prop"].as_f64().unwrap() == 2. / 7. {
                    2
                } else {
                    1
                },
            )
        })
        .collect::<Vec<_>>();
    let compare =
        |a: &(f64, f64, f64, f64, u64), b: &(f64, f64, f64, f64, u64)| a.partial_cmp(b).unwrap();
    actual.sort_by(compare);
    expected.sort_by(compare);
    for (a, b) in actual.iter().zip(expected) {
        assert_eq!((a.0, a.1, a.2, a.4), (b.0, b.1, b.2, b.4));
        assert!((a.3 - b.3).abs() < 1e-14);
    }
    for (row, mark) in rows.iter().zip(prepared.layers()[0].marks()) {
        let n = row
            .values
            .iter()
            .find(|v| v.field == StatField::WeightedCount)
            .unwrap()
            .value
            .unwrap();
        let expected = fixture["result"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["n"].as_f64() == Some(n))
            .unwrap()["size"]
            .as_f64()
            .unwrap();
        assert!(
            (mark.style.radius - expected).abs() < 1e-12,
            "{} != {}",
            mark.style.radius,
            expected
        );
        let color = mark.style.color;
        let hex = format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue);
        let shape = mark.aesthetics.get(&ValueAesthetic::Shape).unwrap();
        let shape = if let chart_core::interpolate::Value::Number(v) = shape {
            v.0
        } else {
            panic!("shape {shape:?}")
        };
        let x = row.value(&StatField::X).unwrap();
        let prop = row.value(&StatField::Proportion).unwrap();
        let reference = fixture["result"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| {
                v["x"].as_f64() == Some(x)
                    && v["n"].as_f64() == Some(n)
                    && (v["prop"].as_f64().unwrap() - prop).abs() < 1e-12
            })
            .unwrap();
        assert_eq!(hex, reference["colour"].as_str().unwrap());
        assert_eq!(shape, reference["shape"].as_f64().unwrap());
    }
    let targets = rows
        .iter()
        .map(|r| format!("{:?}", r.target))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(targets.len(), 4);
}
#[path = "../../../examples/common/ggplot_recipe_marks.rs"]
mod authors;
#[test]
fn columns_and_signed_spokes_retain_data_endpoints_and_original_targets() {
    let column = authors::author(1)
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    for (mark, (x, y)) in
        column.layers()[0]
            .marks()
            .iter()
            .zip([(1., 1.), (1., 1.), (2., -1.), (3., 2.)])
    {
        let PreparedGeometry::Rectangle { from, to } = &mark.geometry else {
            panic!()
        };
        assert!((from.x() - (x - 0.3)).abs() < 1e-12);
        assert!((to.x() - (x + 0.3)).abs() < 1e-12);
        assert_eq!(from.y(), y);
        assert_eq!(to.y(), 0.);
        assert_eq!(mark.targets.len(), 1);
    }
    let spoke = authors::author(4)
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    for (mark, (x, y)) in
        spoke.layers()[0]
            .marks()
            .iter()
            .zip([(2., 1.), (2., 1.), (2., -2.), (1., 2.)])
    {
        let PreparedGeometry::Recipe(recipe) = &mark.geometry else {
            panic!()
        };
        let PreparedRecipe::Mark(PreparedMarkRecipe::Spoke { to, .. }) = recipe.as_ref() else {
            panic!()
        };
        assert!((to.x() - x).abs() < 1e-12);
        assert!((to.y() - y).abs() < 1e-12);
        assert_eq!(mark.targets.len(), 1);
    }
}
#[test]
fn recipe_source_mutation_matches_fresh_batch_geometry() {
    fn data(y: f64) -> Data {
        Data::columns()
            .name("observations")
            .keys([11, 12])
            .column("x", [1., 2.])
            .column("y", [y, 3.])
            .column("end_x", [3., 4.])
            .column("end_y", [2., 1.])
            .build()
            .unwrap()
    }
    fn author(data: Data, mode: usize) -> Plot {
        let recipe = match mode {
            0 => BuiltinRecipe::Column(ColumnRecipe::default()),
            1 => BuiltinRecipe::Spoke(SpokeRecipe {
                angle: 1.,
                radius: 2.,
                ..Default::default()
            }),
            2 => BuiltinRecipe::Curve(CurveRecipe::default()),
            _ => BuiltinRecipe::Rug(RugRecipe::default()),
        };
        plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").x2("end_x").y2("end_y"))
            .layer(points().recipe(recipe))
            .build()
            .unwrap()
    }
    for mode in 0..4 {
        let mut chart = author(data(1.), mode).chart().unwrap();
        chart.prepare().unwrap();
        let transaction = chart
            .transaction()
            .unwrap()
            .upsert("observations", data(-2.))
            .build()
            .unwrap();
        chart.apply_transaction(transaction).unwrap();
        let live = chart.prepare().unwrap();
        let fresh = author(data(-2.), mode).chart().unwrap().prepare().unwrap();
        assert_eq!(
            live.layers()[0]
                .marks()
                .iter()
                .map(|m| &m.geometry)
                .collect::<Vec<_>>(),
            fresh.layers()[0]
                .marks()
                .iter()
                .map(|m| &m.geometry)
                .collect::<Vec<_>>()
        );
        assert!(
            live.layers()[0]
                .marks()
                .iter()
                .all(|m| m.targets.len() == 1)
        );
    }
}
#[test]
fn rugs_allow_one_axis_without_training_a_fake_zero() {
    for horizontal in [true, false] {
        let data = Data::columns().column("v", [10., 20.]).build().unwrap();
        let mapping = if horizontal {
            aes().x("v")
        } else {
            aes().y("v")
        };
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(mapping)
            .layer(points().recipe(BuiltinRecipe::Rug(RugRecipe {
                sides: "bltr".into(),
                ..Default::default()
            })))
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let layer = &prepared.layers()[0];
        assert_eq!(layer.marks().len(), 2);
        for mark in layer.marks() {
            let PreparedGeometry::Recipe(recipe) = &mark.geometry else {
                panic!()
            };
            let PreparedRecipe::Mark(PreparedMarkRecipe::Rug { x, y, .. }) = recipe.as_ref() else {
                panic!()
            };
            assert_eq!(x.is_some(), horizontal);
            assert_eq!(y.is_some(), !horizontal);
        }
        let domains = layer.domains();
        assert_eq!(domains.x.is_some(), horizontal);
        assert_eq!(domains.y.is_some(), !horizontal);
    }
}
#[test]
fn count_partitions_evaluated_expressions_instead_of_underlying_reads() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/count-mark-expression.json"
    ))
    .unwrap();
    let data = Data::columns()
        .column("x", [1., 1., 1., 1.])
        .column("y", [1., 1., 1., 1.])
        .column("v", [-1., 1., -2., 2.])
        .column("w", [1., 2., 3., 4.])
        .build()
        .unwrap();
    let scale = chart_core::scales::MappedScaleSpec::authored(
        chart_core::scales::ScaleFunctionSpec::GgplotNumericIdentity(Default::default()),
    );
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            points()
                .numeric_scale(
                    NumericAesthetic::Opacity,
                    source_expr("v").unary(ExpressionUnary::Abs) * 0.5,
                    scale,
                )
                .recipe(BuiltinRecipe::Count(CountRecipe::default()))
                .stat(count().sum_count().x("x").y("y").count_weight("w")),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(rows.len(), 2);
    for (row, expected) in rows.iter().zip(fixture["result"].as_array().unwrap()) {
        assert_eq!(row.count, 2);
        assert_eq!(row.value(&StatField::WeightedCount), expected["n"].as_f64());
        assert_eq!(row.value(&StatField::Proportion), expected["prop"].as_f64());
        assert_eq!(row.retained_numeric.len(), 1);
    }
    for (mark, expected) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(fixture["result"].as_array().unwrap())
    {
        assert!((mark.style.radius - expected["size"].as_f64().unwrap()).abs() < 1e-12);
    }
}
#[test]
fn column_reference_default_and_overridden_paints_match_source() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/column-mark-paints.json"
    ))
    .unwrap();
    for mode in ["default", "fixed", "mapped", "mapped_outline"] {
        let data = Data::columns()
            .column("x", [1., 2.])
            .column("y", [2., -1.])
            .column("g", ["A", "B"])
            .build()
            .unwrap();
        let mut layer = rectangle().recipe(BuiltinRecipe::Column(ColumnRecipe::default()));
        if mode == "fixed" {
            layer = layer.fill(rgb(255, 0, 0)).color(rgb(0, 0, 255));
        }
        if mode == "mapped" {
            layer = layer.aes(aes().fill("g").color("g"));
        }
        if mode == "mapped_outline" {
            layer = layer.fill(rgb(255, 0, 0)).aes(aes().color("g"));
        }
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        let hex =
            |c: chart_core::scene::Color| format!("#{:02X}{:02X}{:02X}", c.red, c.green, c.blue);
        for (mark, expected) in p.layers()[0]
            .marks()
            .iter()
            .zip(fixture["cases"][mode].as_array().unwrap())
        {
            let expected_fill = expected["fill"].as_str().unwrap();
            let expected_fill = match expected_fill {
                "red" => "#FF0000",
                v => v.strip_suffix("FF").filter(|_| v.len() == 9).unwrap_or(v),
            };
            assert_eq!(hex(mark.style.fill.unwrap()), expected_fill);
            if expected["colour"].is_null() {
                assert!(mark.style.stroke.is_none());
            } else {
                let color = expected["colour"].as_str().unwrap();
                assert_eq!(
                    hex(mark.style.stroke.unwrap()),
                    if color == "blue" { "#0000FF" } else { color }
                );
            }
            assert_eq!(
                mark.style.stroke_width,
                expected["linewidth"].as_f64().unwrap() * 72.27 / 25.4 * (72. / 96.)
            );
        }
    }
}
#[test]
fn count_recipe_preserves_mapped_shape_and_size_on_all_base_geometries() {
    let data = Data::columns()
        .column("x", [1., 1., 2.])
        .column("y", [2., 2., 3.])
        .column("shape", ["A", "A", "B"])
        .column("w", [1., 2., 4.])
        .build()
        .unwrap();
    struct Metrics;
    impl chart_core::services::TextMeasurer for Metrics {
        fn measure(
            &self,
            r: chart_core::services::TextRequest<'_>,
        ) -> chart_core::ChartResult<chart_core::services::TextMetrics> {
            chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let mut results = vec![];
    let mut glyphs = vec![];
    for base in [points(), rule(), rectangle()] {
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").shape("shape"))
            .layer(
                base.recipe(BuiltinRecipe::Count(CountRecipe::default()))
                    .stat(
                        count()
                            .sum_count()
                            .x("x")
                            .y("y")
                            .group("shape")
                            .count_weight("w"),
                    ),
            )
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        results.push(
            p.layers()[0]
                .marks()
                .iter()
                .map(|m| {
                    assert!(matches!(m.geometry, PreparedGeometry::Point(_)));
                    assert_eq!(m.targets.len(), 1);
                    (
                        m.geometry.clone(),
                        m.aesthetics.get(&ValueAesthetic::Shape).cloned(),
                        m.style.radius,
                    )
                })
                .collect::<Vec<_>>(),
        );
        let request = chart_core::layout::LayoutRequest::new(
            chart_core::Rect::new(0., 0., 600., 360.).unwrap(),
            chart_core::services::Units::LogicalPixels,
            chart_core::services::ResourceDescriptor {
                id: chart_core::ResourceId::new(1),
                revision: chart_core::Revision::INITIAL,
                kind: chart_core::services::ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = chart_core::layout::layout(p, &request, &Metrics).unwrap();
        let shapes = frame
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .filter_map(|i| match &i.primitive {
                chart_core::scene::Primitive::ShapePath { geometry, .. } => Some(geometry.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shapes.len(),
            2,
            "Count mapped shapes must reach scene glyphs"
        );
        assert_ne!(
            shapes[0].commands().len(),
            shapes[1].commands().len(),
            "circle and triangle retain distinct paths"
        );
        glyphs.push(shapes);
    }
    assert_eq!(glyphs[0], glyphs[1]);
    assert_eq!(glyphs[0], glyphs[2]);
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
}
#[test]
fn missing_mapped_column_and_spoke_controls_do_not_become_defaults() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/missing-recipe-controls.json"
    ))
    .unwrap();
    for family in ["column", "spoke"] {
        for mapped in [false, true] {
            let data = Data::columns()
                .column("x", [1., 2.])
                .column("y", [1., 2.])
                .column("w", [None, Some(1.)])
                .column("a", [None, Some(0.)])
                .column("r", [Some(1.), None])
                .build()
                .unwrap();
            let mut layer = if family == "column" {
                rectangle().recipe(BuiltinRecipe::Column(ColumnRecipe::default()))
            } else {
                rule().recipe(BuiltinRecipe::Spoke(SpokeRecipe::default()))
            };
            if mapped {
                layer = if family == "column" {
                    layer.recipe_value(RecipeAesthetic::Width, "w")
                } else {
                    layer
                        .recipe_value(RecipeAesthetic::Angle, "a")
                        .recipe_value(RecipeAesthetic::Radius, "r")
                };
            }
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(layer)
                .build()
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            let layer = &p.layers()[0];
            if mapped {
                let endpoint = if family == "column" { "xmin" } else { "xend" };
                let expected = fixture["values"][family]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| !r[endpoint].is_null())
                    .count();
                assert_eq!(layer.marks().len(), expected);
                if family == "column" {
                    let PreparedGeometry::Rectangle { from, to } = &layer.marks()[0].geometry
                    else {
                        panic!()
                    };
                    assert_eq!(from.x(), 1.5);
                    assert_eq!(to.x(), 2.5);
                }
                assert!(
                    layer
                        .domains()
                        .x
                        .is_some_and(|v| v.minimum <= 1. && v.maximum >= 2.),
                    "source x trains even if mark omitted"
                );
                assert!(
                    layer
                        .domains()
                        .y
                        .is_some_and(|v| v.minimum <= 1. && v.maximum >= 2.),
                    "source y trains even if mark omitted"
                );
            } else {
                assert_eq!(
                    layer.marks().len(),
                    2,
                    "absent controls use configured defaults"
                );
            }
        }
    }
}
