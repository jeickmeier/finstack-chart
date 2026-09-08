//! FIX-AUTH02: complete primary authoring with independent geometry/data expectations.
use chart_core::{
    data::{ColumnValues, TimeUnit},
    grammar::{PreparedGeometry, PreparedRows},
    prelude::*,
};

#[test]
fn automatic_timestamp_origins_align_independent_datasets_without_narrowing_ticks() {
    let origin = 1_712_345_678_901_234_567;
    let data = |name: &str, ticks: Vec<i64>| {
        Data::columns()
            .name(name)
            .column("time", timestamps(ticks, TimeUnit::Nanoseconds, "UTC"))
            .column("value", [1., 2.])
            .build()
            .unwrap()
    };
    let first = data("first", vec![origin, origin + 1]);
    let second = data("second", vec![origin + 2, origin + 3]);
    let built = plot(first.clone())
        .aes(aes().x("time").y("value"))
        .layer(points())
        .layer(points().data(second.clone()))
        .build()
        .unwrap();
    let prepared = built.chart().unwrap().prepare().unwrap();
    let xs = prepared
        .layers()
        .iter()
        .flat_map(|l| l.marks())
        .map(|m| {
            let PreparedGeometry::Point(p) = m.geometry else {
                panic!("point")
            };
            p.x()
        })
        .collect::<Vec<_>>();
    assert_eq!(xs, [0., 1., 2., 3.]);
    assert!(
        plot(first)
            .aes(aes().x("time").y("value"))
            .layer(points())
            .layer(points().data(second).aes(aes().x(Mapping::Timestamp {
                field: "time".into(),
                origin: origin + 2
            })))
            .build()
            .is_err(),
        "explicit incompatible origins must not be rewritten"
    );
}

#[test]
fn shared_automatic_color_catalog_is_consistent_across_data_and_updates() {
    use chart_core::transaction::CommitOutcome;
    let data = |name: &str, labels: Vec<&str>| {
        Data::columns()
            .name(name)
            .column("x", vec![1.; labels.len()])
            .column("y", vec![2.; labels.len()])
            .column("series", categorical(labels))
            .build()
            .unwrap()
    };
    let a = data("a", vec!["A", "B"]);
    let b = data("b", vec!["B", "C"]);
    let mut chart = plot(a)
        .aes(aes().x("x").y("y").color("series"))
        .layer(points())
        .layer(points().data(b.clone()))
        .build()
        .unwrap()
        .chart()
        .unwrap();
    let prepared = chart.prepare().unwrap();
    let first = prepared.layers()[0].color_legend().unwrap();
    assert_eq!(
        first
            .entries
            .iter()
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "C"]
    );
    assert_eq!(first, prepared.layers()[1].color_legend().unwrap());
    assert_eq!(
        prepared.layers()[0].marks()[1].style.color,
        prepared.layers()[1].marks()[0].style.color
    );
    assert_ne!(
        prepared.layers()[0].marks()[0].style.color,
        prepared.layers()[1].marks()[0].style.color
    );
    let tx = chart
        .transaction()
        .unwrap()
        .append(&b, data("new", vec!["D"]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    let updated = chart.prepare().unwrap();
    let legend = updated.layers()[0].color_legend().unwrap();
    assert_eq!(legend, updated.layers()[1].color_legend().unwrap());
    assert_eq!(
        legend
            .entries
            .iter()
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "C", "D"]
    );
    assert_eq!(&legend.entries[..3], &first.entries);
}

#[test]
fn facets_reject_same_ordinal_with_different_field_names() {
    let a = Data::columns()
        .column("x", [1.])
        .column("y", [2.])
        .column("panel", ["A"])
        .build()
        .unwrap();
    let b = Data::columns()
        .name("other")
        .column("x", [1.])
        .column("y", [2.])
        .column("unrelated", ["A"])
        .build()
        .unwrap();
    let base = || {
        plot(a.clone())
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_wrap("panel"))
    };
    let err = base().layer(points().data(b.clone())).build().unwrap_err();
    assert!(err.message.contains("panel") && err.message.contains("other"));
    assert!(
        base()
            .layer(
                points()
                    .data(b)
                    .facet_target(chart_core::grammar::FacetTarget::Broadcast)
            )
            .build()
            .is_ok()
    );
}
#[test]
fn columns_and_rows_prepare_identical_values_without_manual_ids() {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [10., 12., 11.])
        .build()
        .unwrap();
    let points_layer = points().name("observations");
    let handle = points_layer.handle().unwrap();
    let built = plot(data.clone())
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points_layer)
        .build()
        .unwrap();
    assert_eq!(built.layer("observations").unwrap(), handle);
    let mut chart = Chart::new(built.clone()).unwrap();
    let prepared = chart.prepare().unwrap();
    let PreparedGeometry::LineRun(vertices) = &prepared.layers()[0].marks()[0].geometry else {
        panic!("line")
    };
    assert_eq!(
        vertices.iter().map(|p| (p.x(), p.y())).collect::<Vec<_>>(),
        [(1., 10.), (2., 12.), (3., 11.)]
    );
    assert_eq!(prepared.layers()[1].marks().len(), 3);
    assert_eq!(built.clone().definition(), built.definition());
    let rows = Data::rows([(1., 10.), (2., 12.), (3., 11.)])
        .field("x", |r| r.0)
        .field("y", |r| r.1)
        .build()
        .unwrap();
    assert_eq!(rows.batch().columns(), data.batch().columns());
    assert_ne!(rows.id(), data.id());
    assert_ne!(rows.batch().keys(), data.batch().keys());
}
#[test]
fn exact_nullable_data_and_timestamps_survive_materialization() {
    let data = Data::columns()
        .column("id", [9_007_199_254_740_993_u64, u64::MAX])
        .column(
            "value",
            column(vec![Some(2.5), None])
                .unit("USD")
                .formatted(vec![Some("2.500".into()), None]),
        )
        .column(
            "time",
            timestamps(
                vec![1_712_345_678_901_234_567, 1_712_345_678_901_234_568],
                TimeUnit::Nanoseconds,
                "UTC",
            ),
        )
        .keys([u64::MAX - 1, u64::MAX])
        .build()
        .unwrap();
    assert_eq!(
        data.batch().columns()[0].values(),
        &ColumnValues::UInt64(vec![9_007_199_254_740_993, u64::MAX])
    );
    assert_eq!(data.batch().columns()[1].validity(), &[true, false]);
    assert_eq!(data.batch().columns()[1].formatted(0), Some("2.500"));
    let built = plot(data)
        .aes(aes().x("time").y("value"))
        .layer(points())
        .build()
        .unwrap();
    let prepared = built.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 1);
    assert_eq!(
        prepared.layers()[0].marks()[0].geometry,
        PreparedGeometry::Point(chart_core::Point::new(0., 2.5).unwrap())
    );
}
#[test]
fn histogram_lowers_to_the_existing_bin_kernel_and_source_membership() {
    let data = Data::columns().column("x", [0., 1., 2.]).build().unwrap();
    let keys = data.batch().keys().to_vec();
    let built = plot(data)
        .aes(aes().x("x"))
        .layer(histogram().breaks(vec![0., 1., 2.]))
        .build()
        .unwrap();
    let prepared = built.chart().unwrap().prepare().unwrap();
    let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
        panic!("bins")
    };
    assert_eq!(rows.iter().map(|r| r.count).collect::<Vec<_>>(), [1, 2]);
    let chart_core::provenance::Target::Aggregate { members, .. } = &rows[1].target else {
        panic!("aggregate")
    };
    assert_eq!(members.as_ref(), &keys[1..]);
}
#[test]
fn schema_errors_and_foreign_handles_reject_without_mutating_data() {
    assert!(
        Data::columns()
            .column("x", [1.])
            .column("x", [2.])
            .build()
            .is_err()
    );
    assert!(
        Data::columns()
            .column("x", [1.])
            .column("y", [2., 3.])
            .build()
            .is_err()
    );
    let a = Data::columns()
        .column("x", [1.])
        .column("y", [2.])
        .build()
        .unwrap();
    let b = Data::columns()
        .name("other")
        .column("x", [3.])
        .column("y", [4.])
        .build()
        .unwrap();
    let err = plot(a.clone())
        .aes(aes().x("missing").y("y"))
        .layer(points().name("dots"))
        .build()
        .unwrap_err();
    assert!(
        err.message.contains("missing")
            && err.message.contains("dots")
            && err.message.contains("data")
    );
    assert!(
        plot(a.clone())
            .aes(aes().x(b.field("x").unwrap()).y("y"))
            .layer(points())
            .build()
            .is_err()
    );
    assert_eq!(a.batch().len(), 1);
    let independent = plot(a)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(points().data(b))
        .build()
        .unwrap();
    assert_eq!(
        independent
            .chart()
            .unwrap()
            .prepare()
            .unwrap()
            .layers()
            .len(),
        2
    );
}

#[test]
fn grid_infers_empty_combinations_and_build_rejects_invalid_catalogs() {
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., 4., 6.])
        .column("row", ["A", "A", "B"])
        .column("column", ["I", "II", "I"])
        .build()
        .unwrap();
    let built = plot(data.clone())
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_grid("row", "column"))
        .build()
        .unwrap();
    let prepared = built.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.panels().len(), 4);
    assert_eq!(
        prepared
            .panels()
            .iter()
            .map(|p| p.chart.layers()[0].marks().len())
            .collect::<Vec<_>>(),
        [1, 1, 1, 0]
    );
    assert!(
        plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_grid("row", "column").columns(2))
            .build()
            .is_err()
    );
    assert!(
        plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_wrap("row").columns(0))
            .build()
            .is_err()
    );
    assert!(
        plot(data)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_grid("row", "row"))
            .build()
            .is_err()
    );
}

#[test]
fn component_options_reject_incompatible_owners_without_preparation() {
    let data = Data::columns()
        .column("x", [1.])
        .column("y", [2.])
        .build()
        .unwrap();
    for layer in [
        points().bins(3),
        line().breaks(vec![0., 1.]),
        points().baseline(0.),
    ] {
        assert!(
            plot(data.clone())
                .aes(aes().x("x").y("y"))
                .layer(layer)
                .build()
                .is_err()
        );
    }
    for axis in [
        x_axis().scale(scale_log(1.)),
        x_axis().rotation(f64::NAN),
        x_axis().scale(scale_linear().padding(-1.)),
    ] {
        assert!(
            plot(data.clone())
                .aes(aes().x("x").y("y"))
                .layer(points())
                .x_axis(axis)
                .build()
                .is_err()
        );
    }
}

#[test]
fn labels_are_single_annotations_and_callout_components_preserve_the_endpoint() {
    use chart_core::composition::{Anchor, ConnectorOrigin};
    let data = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., 4., 6.])
        .build()
        .unwrap();
    let built = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(labels().at(2., 4.).text("Peak").id("peak"))
        .layer(
            callout()
                .to_data(3., 6.)
                .connector_origin(ConnectorOrigin::Anchor)
                .label(labels().at(1., 6.).text("Leader")),
        )
        .title(title("Title"))
        .subtitle(subtitle("Subtitle"))
        .build()
        .unwrap();
    let figure = built.definition().figure.as_ref().unwrap();
    assert_eq!(figure.annotations.len(), 2);
    assert_eq!(figure.annotations[0].id, "peak");
    assert!(matches!(
        figure.annotations[1].callout,
        Some(Anchor::Data { .. })
    ));
    assert_eq!(
        figure.annotations[1].connector_origin,
        ConnectorOrigin::Anchor
    );
    assert_eq!(
        built.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .len(),
        3
    );
}

#[test]
fn shared_transform_runs_once_and_generated_color_uses_its_own_stage() {
    use chart_core::grammar::{StatField, StatNumeric};
    let data = Data::columns()
        .column("x", [1., 3., 5., 7.])
        .column("group", ["A", "A", "B", "B"])
        .build()
        .unwrap();
    let averages = transform("means", summary().x("x").group("group"));
    let handle = averages.handle().unwrap();
    let built = plot(data.clone())
        .transform(transform("copy", identity_stat()).from_transform(handle))
        .transform(averages)
        .scale(color_continuous("mean", 0., 8.))
        .layer(
            points().from_transform("copy").after_stat(
                stat_aes()
                    .x(StatField::Group)
                    .y(StatField::Mean)
                    .color(StatField::Mean)
                    .color_scale("mean"),
            ),
        )
        .layer(
            bars().from_transform(handle).after_stat(
                stat_aes()
                    .x(StatField::Group)
                    .y(StatField::Mean)
                    .y2(StatNumeric::Literal(0.)),
            ),
        )
        .build()
        .unwrap();
    assert_eq!(built.transform("means").unwrap(), handle);
    let mut chart = built.chart().unwrap();
    let prepared = chart.prepare().unwrap();
    assert_eq!(prepared.metrics().evaluated_transforms, 2);
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    let colors: Vec<_> = prepared.layers()[0]
        .marks()
        .iter()
        .map(|m| m.style.color)
        .collect();
    assert_ne!(colors[0], colors[1]);
    let again = chart.prepare().unwrap();
    assert!(std::sync::Arc::ptr_eq(
        prepared.layers()[0].table(),
        again.layers()[0].table()
    ));
    assert!(
        plot(data.clone())
            .transform(transform("cycle", identity_stat()).from_transform("cycle"))
            .build()
            .is_err()
    );
    assert!(
        plot(data)
            .layer(points().from_transform("absent"))
            .build()
            .is_err()
    );
}

#[test]
fn nondefault_recipe_and_primary_axis_options_survive_later_components() {
    use chart_core::layout::AxisSide;
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [10., 12.])
        .build()
        .unwrap();
    let before = plot(data.clone())
        .layer(bars().baseline(4.).aes(aes().x("x").y("y")))
        .build()
        .unwrap();
    let after = plot(data.clone())
        .layer(bars().aes(aes().x("x").y("y")).baseline(4.))
        .build()
        .unwrap();
    let a = before.chart().unwrap().prepare().unwrap();
    let b = after.chart().unwrap().prepare().unwrap();
    assert_eq!(a.layers()[0].domains().y.unwrap().minimum, 4.);
    assert_eq!(a.layers()[0].marks(), b.layers()[0].marks());
    let authored = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .y_axis(y_axis().label("Price").scale(scale_log(10.)))
        .y_axis(y_axis().name("volume").side(AxisSide::Right))
        .build()
        .unwrap();
    let price = authored
        .definition()
        .axes
        .iter()
        .find(|a| a.id == authored.axis("y").unwrap().id())
        .unwrap();
    assert_eq!(price.title.as_ref().unwrap().lines[0][0].text, "Price");
    assert_eq!(authored.definition().axes.len(), 3);
    let edited = authored
        .edit()
        .x_axis(x_axis().label("Time"))
        .y_axis(y_axis().label("Value"))
        .build()
        .unwrap();
    assert_eq!(edited.axis("y").unwrap(), authored.axis("y").unwrap());
    assert_eq!(
        edited.axis("volume").unwrap(),
        authored.axis("volume").unwrap()
    );
}

#[test]
fn imported_identity_preserves_seeded_jitter_and_rejects_ambiguous_data_owners() {
    let batch = || {
        Data::columns()
            .identity(1)
            .column("x", [1., 2.])
            .column("y", [3., 4.])
            .keys([11, 12])
            .build()
            .unwrap()
    };
    let authored = |data: Data| {
        plot(data)
            .aes(aes().x("x").y("y"))
            .layer(points().position(jitter(42).displacement(0.5, 1.)))
            .build()
            .unwrap()
    };
    let first = authored(batch());
    let _unrelated = Data::columns()
        .column("other", vec![0.; 100])
        .build()
        .unwrap();
    let second = authored(batch());
    let a = first.chart().unwrap().prepare().unwrap();
    let b = second.chart().unwrap().prepare().unwrap();
    assert_ne!(
        first.definition().layers[0].id,
        second.definition().layers[0].id
    );
    for (a, b) in a.layers()[0].marks().iter().zip(b.layers()[0].marks()) {
        assert_eq!(a.geometry, b.geometry);
    }
    assert_eq!(first.data("data").unwrap().id().get(), 1);
    let shared = batch();
    assert!(
        plot(shared.clone())
            .aes(aes().x("x").y("y"))
            .layer(points().data(shared))
            .build()
            .is_ok()
    );
    let e = plot(batch())
        .aes(aes().x("x").y("y"))
        .layer(points().data(batch()))
        .build()
        .unwrap_err();
    assert_eq!(e.code, DiagnosticCode::SchemaConflict);
}
