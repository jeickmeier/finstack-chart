//! FIX-GG02: canonical semantics, explicit overrides and stage-order counterexamples.
use chart_core::{
    grammar::{Compiler, GroupValue, PreparedRows},
    prelude::*,
    state::ChartState,
};

#[test]
fn profile_provenance_grouping_and_revision_survive_primary_and_legacy_round_trips() {
    let data = Data::columns()
        .column("x", [0., 1., 2.])
        .column("y", [1., 3., 2.])
        .column("category", categorical(["A", "B", "A"]))
        .build()
        .unwrap();
    let legacy = plot(data.clone())
        .aes(aes().x("x").y("y").color("category"))
        .layer(points())
        .build()
        .unwrap();
    let gg = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("category"))
        .layer(points())
        .build()
        .unwrap();
    let prepared = gg.chart().unwrap().prepare().unwrap();
    let groups: Vec<_> = prepared.layers()[0]
        .marks()
        .iter()
        .map(|m| m.group.clone())
        .collect();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0], groups[2]);
    assert_ne!(groups[0], groups[1]);
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/cases.json")).unwrap();
    let reference = oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "inferred_groups")
        .unwrap();
    assert_eq!(
        reference["layers"][0]["columns"]["group"],
        serde_json::json!([1, 1, 2])
    );
    assert_eq!(prepared.definition(), gg.definition());
    let old = legacy.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&old).unwrap()["version"],
        1
    );
    let wire = gg.to_json().unwrap();
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let mut value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    // GG-04 adds a retained, population-trained default hue scale.
    assert_eq!(value["version"], 17);
    value["profile"] = "LibraryV1".into();
    assert!(Plot::from_json(&value.to_string()).is_err());
    let changed = gg.edit().profile(Profile::LibraryV1).build().unwrap();
    assert_ne!(changed.definition().revision, gg.definition().revision);
    assert_eq!(gg.to_json().unwrap(), wire);
    assert_eq!(legacy.to_json().unwrap(), old);
    let mut compiler = Compiler::new();
    let first = compiler
        .prepare(
            gg.definition(),
            &gg.source(),
            &ChartState::default(),
            gg.compile_limits(),
        )
        .unwrap();
    let second = compiler
        .prepare(
            changed.definition(),
            &changed.source(),
            &ChartState::default(),
            changed.compile_limits(),
        )
        .unwrap();
    assert_eq!(first.definition().profile(), Profile::Ggplot2_4_0_3);
    assert!(
        second.layers()[0]
            .marks()
            .iter()
            .all(|m| m.group == GroupValue::All)
    );
    let normalized = chart_core::portable::ChartEnvelope {
        version: 17,
        definition: gg.definition().clone(),
    };
    normalized.validate().unwrap();
    let encoded = serde_json::to_string(&normalized).unwrap();
    serde_json::from_str::<chart_core::portable::ChartEnvelope>(&encoded)
        .unwrap()
        .validate()
        .unwrap();
}

#[test]
fn inferred_multiple_discrete_fields_and_stat_override_use_exact_interactions() {
    let data = Data::columns()
        .column("x", categorical(["X", "X", "Y", "Y"]))
        .column("y", [1., 2., 3., 4.])
        .column("category", categorical(["A", "B", "A", "B"]))
        .build()
        .unwrap();
    let figure = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("category"))
        .layer(points())
        .layer(points().aes(aes().group_all()))
        .build()
        .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    assert_eq!(
        prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| &m.group)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert!(
        prepared.layers()[1]
            .marks()
            .iter()
            .all(|m| m.group == GroupValue::All)
    );
    let counted = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("category"))
        .layer(points().stat(count().group_all()))
        .build()
        .unwrap();
    let prepared = counted.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!("count rows")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].group, GroupValue::All);
}

fn oracle_values(id: &str, field: &str) -> Vec<f64> {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/stages.json")).unwrap();
    value["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .unwrap()["layers"][0]["columns"][field]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect()
}

#[test]
fn logged_summary_projects_once_into_destination_geometry() {
    use chart_core::{grammar::StatField, layout::*, scene::Primitive, services::*};
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let figure = plot(
        Data::columns()
            .column("x", [1., 1., 1.])
            .column("y", [1., 10., 100.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(
        points()
            .stat(summary().x("y"))
            .after_stat(stat_aes().x(1.).y(StatField::Mean)),
    )
    .y_axis(y_axis().scale(scale_log(10.).domain(1., 100.)))
    .build()
    .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    let mut request = LayoutRequest::new(
        chart_core::Rect::new(0., 0., 400., 240.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: chart_core::ResourceId::new(1),
            revision: chart_core::Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.axes = figure.definition().axes.clone();
    let laid = layout(prepared, &request, &Metrics).unwrap();
    let center = laid
        .scene()
        .items()
        .iter()
        .find_map(|item| match item.primitive {
            Primitive::Point { center, .. } if item.layer.is_some() => Some(center),
            _ => None,
        })
        .unwrap();
    let panel = laid.plot().unwrap();
    assert!((center.y() - (panel.origin().y() + panel.height() / 2.)).abs() < 1e-10);
}

#[test]
fn after_stat_expressions_reduce_generated_populations_and_backtransform_log_values() {
    use chart_core::grammar::{
        BinField, Expression, ExpressionReduce, PreparedGeometry, StatField,
    };
    let figure = plot(
        Data::columns()
            .column("x", [1., 1., 1.])
            .column("y", [1., 10., 100.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(
        points()
            .stat(summary().x("y"))
            .after_stat(stat_aes().x(1.).y(Expression::read(StatField::Mean) * 2.)),
    )
    .y_axis(y_axis().scale(scale_log(10.)))
    .build()
    .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Point(point) = prepared.layers()[0].marks()[0].geometry else {
        panic!("mean point")
    };
    assert!((point.y() - oracle_values("log_after_stat_expression", "y")[0]).abs() < 1e-12);
    let count = Expression::read(BinField::Count);
    let figure = plot(
        Data::columns()
            .column("x", [1., 2., 5., 20., 50., 200., 500.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x"))
    .layer(
        histogram()
            .breaks(vec![1., 10., 100., 1000.])
            .after_bin(bin_aes().y(count.clone() / count.reduce(ExpressionReduce::Sum, false))),
    )
    .build()
    .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    for (mark, expected) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("after_stat_expression", "y"))
    {
        let PreparedGeometry::Rectangle { from, to } = mark.geometry else {
            panic!("bin rectangle")
        };
        assert!((from.y().max(to.y()) - expected).abs() < 1e-12);
    }
    assert_eq!(
        Plot::from_json(&figure.to_json().unwrap())
            .unwrap()
            .to_json()
            .unwrap(),
        figure.to_json().unwrap()
    );
}
#[test]
fn scale_transforms_limits_and_oob_precede_statistics_while_coordinates_do_not() {
    use chart_core::grammar::{PreparedGeometry, ScaleOob, StatField, ValueSpace};
    let data = Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [1., 10., 100.])
        .build()
        .unwrap();
    for (id, axis) in [
        ("log_mean", y_axis().scale(scale_log(10.))),
        (
            "coordinate_log_mean",
            y_axis().coordinate_scale(scale_log(10.)),
        ),
        (
            "scale_limit_mean",
            y_axis().scale(scale_linear().domain(1., 10.)),
        ),
        ("coordinate_zoom_mean", y_axis().viewport(1., 10.)),
        (
            "squish_mean",
            y_axis()
                .scale(scale_linear().domain(1., 10.))
                .oob(ScaleOob::Squish),
        ),
        (
            "keep_mean",
            y_axis()
                .scale(scale_linear().domain(1., 10.))
                .oob(ScaleOob::Keep),
        ),
    ] {
        let plot = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .stat(summary().x("y"))
                    .after_stat(stat_aes().x(1.).y(StatField::Mean)),
            )
            .y_axis(axis)
            .build()
            .unwrap();
        let prepared = plot.chart().unwrap().prepare().unwrap();
        let PreparedGeometry::Point(point) = prepared.layers()[0].marks()[0].geometry else {
            panic!("mean point")
        };
        assert!(
            (point.y() - oracle_values(id, "y")[0]).abs() < 1e-12,
            "{id}: {}",
            point.y()
        );
        if id == "log_mean" {
            assert!(matches!(
                prepared.layers()[0].domains().y_space,
                Some(ValueSpace::Scaled { .. })
            ));
        }
    }
    let data = Data::columns()
        .column("x", [1., 2., 5., 20., 50., 200., 500.])
        .build()
        .unwrap();
    for (id, axis) in [
        ("log_histogram", x_axis().scale(scale_log(10.))),
        (
            "coordinate_log_histogram",
            x_axis().coordinate_scale(scale_log(10.)),
        ),
    ] {
        let plot = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x"))
            .layer(histogram().breaks(vec![1., 10., 100., 1000.]))
            .x_axis(axis)
            .build()
            .unwrap();
        let prepared = plot.chart().unwrap().prepare().unwrap();
        let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
            panic!("bins")
        };
        assert_eq!(
            rows.iter().map(|r| r.count as f64).collect::<Vec<_>>(),
            oracle_values(id, "count")
        );
        assert_eq!(
            rows.iter().map(|r| r.start).collect::<Vec<_>>(),
            oracle_values(id, "xmin")
        );
        assert_eq!(
            rows.iter().map(|r| r.end).collect::<Vec<_>>(),
            oracle_values(id, "xmax")
        );
    }
}

#[test]
fn source_expressions_and_reductions_use_registered_data_before_filters() {
    use chart_core::grammar::{ExpressionReduce, PreparedGeometry};
    let data = Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [1., 10., 100.])
        .build()
        .unwrap();
    let figure = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x(source_expr("x") + 1.).y(source_expr("y") * 2.))
        .layer(points())
        .build()
        .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    for ((mark, x), y) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("source_expression", "x"))
        .zip(oracle_values("source_expression", "y"))
    {
        let PreparedGeometry::Point(p) = mark.geometry else {
            panic!("source point")
        };
        assert_eq!((p.x(), p.y()), (x, y));
    }
    let fraction = source_expr("y") / source_expr("y").reduce(ExpressionReduce::Sum, false);
    let figure = plot(data)
        .aes(aes().x("x").y(fraction))
        .layer(points().filter(filter("y").maximum(10.)))
        .build()
        .unwrap();
    let wire = figure.to_json().unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    for (mark, expected) in prepared.layers()[0]
        .marks()
        .iter()
        .zip([1. / 111., 10. / 111.])
    {
        let PreparedGeometry::Point(p) = mark.geometry else {
            panic!("fraction point")
        };
        assert!((p.y() - expected).abs() < 1e-14);
    }
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    let edited = figure.edit().build().unwrap();
    assert_eq!(edited.definition().layers, figure.definition().layers);
}

#[test]
fn after_scale_and_theme_expressions_preserve_defaults_overrides_and_old_snapshots() {
    use chart_core::{
        grammar::{AfterScaleAesthetic, ThemeRead},
        scene::Color,
        theme::GeometryTheme,
    };
    let data = Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [1., 10., 100.])
        .build()
        .unwrap();
    let modifier = scale_aes().size(after_scale_expr(AfterScaleAesthetic::Size) * 2.);
    let figure = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points().after_scale(modifier.clone()))
        .layer(points().size(3.).after_scale(modifier))
        .build()
        .unwrap();
    let retained = figure.chart().unwrap().prepare().unwrap();
    for (mark, expected) in retained.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("after_scale_expression", "size"))
    {
        assert_eq!(mark.style.radius, expected);
    }
    assert!(
        retained.layers()[1]
            .marks()
            .iter()
            .all(|m| m.style.radius == 6.)
    );
    let library = figure.edit().profile(Profile::LibraryV1).build().unwrap();
    assert!(
        library.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.radius == 6.)
    );
    assert!(
        retained.layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.radius == 3.)
    );
    let accent = Color {
        red: 0x12,
        green: 0x56,
        blue: 0xab,
        alpha: 255,
    };
    let themed = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .theme(theme().geometry(GeometryTheme {
            accent,
            ..Default::default()
        }))
        .layer(points().after_scale(scale_aes().color(from_theme(ThemeRead::Accent))))
        .build()
        .unwrap();
    let prepared = themed.chart().unwrap().prepare().unwrap();
    assert!(
        prepared.layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.color == accent)
    );
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/ggplot2/stages.json")).unwrap();
    let colors = &reference["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "theme_expression")
        .unwrap()["layers"][0]["columns"]["colour"];
    assert_eq!(
        colors,
        &serde_json::json!(["#1256ab", "#1256ab", "#1256ab"])
    );
    let json = themed.to_json().unwrap();
    assert_eq!(Plot::from_json(&json).unwrap().to_json().unwrap(), json);
}

#[test]
fn missing_discrete_values_and_float_group_keys_match_reference_partitions() {
    let missing = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("y", [1., 2., 3., 4.])
        .column(
            "category",
            categorical(["A", "unused", "B", "unused"]).validity(vec![true, false, true, false]),
        )
        .build()
        .unwrap();
    let numeric = Data::columns()
        .column("x", [1., 2., 3., 4., 5.])
        .column("y", [1., 2., 3., 4., 5.])
        .column(
            "g",
            column(vec![-0., 0., 1.5, 1.5, 0.]).validity(vec![true, true, true, true, false]),
        )
        .build()
        .unwrap();
    for (id, figure) in [
        (
            "missing_groups",
            plot(missing)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y").color("category"))
                .layer(points())
                .build()
                .unwrap(),
        ),
        (
            "numeric_groups",
            plot(numeric)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y").group("g"))
                .layer(points())
                .build()
                .unwrap(),
        ),
    ] {
        let prepared = figure.chart().unwrap().prepare().unwrap();
        let groups = prepared.layers()[0]
            .marks()
            .iter()
            .map(|m| &m.group)
            .collect::<Vec<_>>();
        let reference = oracle_values(id, "group");
        assert_eq!(groups.len(), reference.len());
        for i in 0..groups.len() {
            for j in 0..groups.len() {
                assert_eq!(
                    groups[i] == groups[j],
                    reference[i] == reference[j],
                    "{id}: {i}/{j}"
                );
            }
        }
    }
    let data = Data::columns()
        .column("x", [1., 2., 1., 2.])
        .column("y", [1., 3., 2., 4.])
        .column("category", categorical(["A", "A", "B", "B"]))
        .build()
        .unwrap();
    let figure = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("category"))
        .layer(line())
        .build()
        .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    let lines = prepared.layers()[0].marks();
    assert_eq!(lines.len(), 2);
    for (mark, expected) in lines.iter().zip([[1., 3.], [2., 4.]]) {
        let chart_core::grammar::PreparedGeometry::LineRun(points) = &mark.geometry else {
            panic!("line run")
        };
        assert_eq!(points.iter().map(|p| p.y()).collect::<Vec<_>>(), expected);
    }
}

#[test]
fn horizontal_histogram_and_column_defaults_reuse_stat_and_geometry_stages() {
    use chart_core::grammar::{PreparedGeometry, ValueSpace};
    let data = Data::columns()
        .column("value", [1., 2., 5., 20., 50., 200., 500.])
        .build()
        .unwrap();
    let figure = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().y("value"))
        .layer(histogram().breaks(vec![1., 10., 100., 1000.]))
        .y_axis(y_axis().scale(scale_log(10.)))
        .build()
        .unwrap();
    assert_eq!(
        figure.definition().layers[0].orientation,
        Orientation::Horizontal
    );
    let prepared = figure.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].orientation(), Orientation::Horizontal);
    assert!(matches!(
        prepared.layers()[0].domains().y_space,
        Some(ValueSpace::Scaled { .. })
    ));
    for ((mark, count), low) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("horizontal_log_histogram", "count"))
        .zip(oracle_values("horizontal_log_histogram", "ymin"))
    {
        let PreparedGeometry::Rectangle { from, to } = mark.geometry else {
            panic!("horizontal bin")
        };
        assert_eq!(from.x().max(to.x()), count);
        assert_eq!(from.y().min(to.y()), low);
        assert_eq!(from.y().max(to.y()), low + 1.);
    }
    let data = Data::columns()
        .column("category", categorical(["A", "B"]))
        .column("value", [3., 7.])
        .build()
        .unwrap();
    let columns = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("value").y("category"))
        .layer(bars().orientation(Orientation::Horizontal))
        .y_axis(y_axis().scale(scale_band()))
        .build()
        .unwrap();
    let prepared = columns.chart().unwrap().prepare().unwrap();
    for (mark, x) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("horizontal_columns", "x"))
    {
        let PreparedGeometry::Bar { from, to, .. } = mark.geometry else {
            panic!("horizontal bar")
        };
        assert_eq!(from.x().max(to.x()), x);
        assert_eq!(from.x().min(to.x()), 0.);
        assert_eq!(from.y(), to.y());
    }
    let counts = plot(
        Data::columns()
            .column("category", categorical(["A", "A", "B"]))
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().y("category"))
    .layer(bars())
    .y_axis(y_axis().scale(scale_band()))
    .build()
    .unwrap();
    let prepared = counts.chart().unwrap().prepare().unwrap();
    for (mark, count) in prepared.layers()[0]
        .marks()
        .iter()
        .zip(oracle_values("horizontal_count", "count"))
    {
        let PreparedGeometry::Bar { from, to, .. } = mark.geometry else {
            panic!("horizontal count")
        };
        assert_eq!(from.x().max(to.x()), count);
    }
    assert_eq!(
        Plot::from_json(&counts.to_json().unwrap())
            .unwrap()
            .to_json()
            .unwrap(),
        counts.to_json().unwrap()
    );
}

#[test]
fn shared_statistics_use_source_grouping_and_scale_context_once() {
    use chart_core::grammar::{PreparedGeometry, StatField};
    let data = Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [1., 10., 100.])
        .column("group", categorical(["A", "A", "B"]))
        .build()
        .unwrap();
    let base = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .transform(transform("mean", summary().x("y")))
        .transform(transform("copy", identity_stat()).from_transform("mean"))
        .y_axis(y_axis().scale(scale_log(10.)));
    let figure = base
        .clone()
        .layer(
            points()
                .from_transform("mean")
                .after_stat(stat_aes().x(1.).y(StatField::Mean)),
        )
        .layer(
            points()
                .from_transform("copy")
                .after_stat(stat_aes().x(2.).y(StatField::Mean)),
        )
        .build()
        .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    for layer in prepared.layers() {
        let PreparedGeometry::Point(p) = layer.marks()[0].geometry else {
            panic!("point")
        };
        assert!((p.y() - oracle_values("log_mean", "y")[0]).abs() < 1e-12);
    }
    let wire = figure.to_json().unwrap();
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let conflict = base
        .axis(y_axis().name("linear"))
        .layer(
            points()
                .from_transform("mean")
                .after_stat(stat_aes().x(1.).y(StatField::Mean)),
        )
        .layer(
            points()
                .from_transform("copy")
                .axes("x", "linear")
                .after_stat(stat_aes().x(2.).y(StatField::Mean)),
        )
        .build();
    assert!(conflict.is_err());
    assert!(
        conflict
            .err()
            .unwrap()
            .message
            .contains("incompatible positional scale")
    );
    let grouped = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .transform(transform("mean", summary().x("y")).aes(aes().color("group")))
        .layer(
            points()
                .from_transform("mean")
                .after_stat(stat_aes().x(1.).y(StatField::Mean)),
        )
        .y_axis(y_axis().scale(scale_log(10.)))
        .build()
        .unwrap();
    let prepared = grouped.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!("summary")
    };
    assert_eq!(rows.len(), 2);
    let values: Vec<_> = prepared.layers()[0]
        .marks()
        .iter()
        .map(|m| {
            let PreparedGeometry::Point(p) = m.geometry else {
                panic!("point")
            };
            p.y()
        })
        .collect();
    assert_eq!(values, vec![0.5, 2.]);
}
