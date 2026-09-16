//! FIX-GG06: independent reference scalar helpers, count/summary partitions and exact provenance.
use chart_core::{grammar::*, prelude::*};
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/count-summary-controls.json"
    ))
    .unwrap()
}
fn near(a: Option<f64>, b: &serde_json::Value) {
    if b.is_null() {
        assert_eq!(a, None)
    } else {
        let b = b.as_f64().unwrap();
        let a = a.unwrap();
        assert!((a - b).abs() <= 1e-7_f64.max(b.abs() * 2e-12), "{a} != {b}")
    }
}
#[test]
fn all_scalar_helpers_match_independent_r_values() {
    let f = fixture();
    for c in f["helpers"].as_array().unwrap() {
        let h: SummaryHelper = serde_json::from_value(c["helper"].clone()).unwrap();
        let x: Vec<f64> = serde_json::from_value(c["x"].clone()).unwrap();
        let actual = summary_values(&h, &x).unwrap();
        for (a, b) in actual.into_iter().zip(c["expected"].as_array().unwrap()) {
            near(a, b)
        }
    }
}
fn data() -> Data {
    Data::columns()
        .column("x", [1., 1., 2., 2., 4., 4.])
        .column("y", [1., 3., 4., 8., 2., 6.])
        .column(
            "w",
            [Some(2.), Some(-1.), Some(0.), Some(3.), None, Some(-2.)],
        )
        .column("g", [1i64, 1, 1, 1, 2, 2])
        .build()
        .unwrap()
}
fn rows(stat: StatBuilder) -> Vec<StatisticalRow> {
    let p = plot(data()).layer(points().stat(stat)).build().unwrap();
    let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    rows.to_vec()
}
#[test]
fn weighted_count_and_summary_are_partitioned_and_retain_exact_members() {
    let f = fixture();
    let counts = rows(count().ggplot_count().x("x").group("g").count_weight("w"));
    assert_eq!(counts.len(), 3);
    for (a, b) in counts.iter().zip(f["count"].as_array().unwrap()) {
        for (field, key) in [
            (StatField::X, "x"),
            (StatField::WeightedCount, "count"),
            (StatField::Proportion, "prop"),
            (StatField::Width, "width"),
        ] {
            near(a.value(&field), &b[key])
        }
        assert_eq!(a.count, 2);
        assert_eq!(a.members.len(), 2);
    }
    assert_ne!(counts[0].target, counts[1].target);
    let summary = rows(
        summary()
            .x("x")
            .y("y")
            .group("g")
            .summary_helper(SummaryHelper::default()),
    );
    for (a, b) in summary.iter().zip(f["summary"].as_array().unwrap()) {
        for (field, key) in [
            (StatField::X, "x"),
            (StatField::Y, "y"),
            (StatField::Lower, "ymin"),
            (StatField::Upper, "ymax"),
        ] {
            near(a.value(&field), &b[key])
        }
    }
}
#[test]
fn binned_summary_shares_closed_breaks_and_retains_only_member_rows() {
    let f = fixture();
    for (cl, name) in [(BinClosure::Right, "right"), (BinClosure::Left, "left")] {
        let actual = rows(
            summary()
                .x("x")
                .y("y")
                .group("g")
                .summary_bins(SummaryBins {
                    breaks: Some(vec![0., 2., 5.]),
                    options: GgplotBinOptions {
                        closed: cl,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );
        let expected = f["bins"][name].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected) {
            for (field, key) in [
                (StatField::X, "x"),
                (StatField::Y, "y"),
                (StatField::Lower, "ymin"),
                (StatField::Upper, "ymax"),
                (StatField::Width, "width"),
            ] {
                near(a.value(&field), &b[key])
            }
        }
        assert_eq!(actual.iter().map(|r| r.count).sum::<u64>(), 6)
    }
}
#[test]
fn helper_validation_and_legacy_contracts_remain_explicit() {
    assert!(summary_values(&SummaryHelper::MeanClNormal { confidence: 1. }, &[1., 2.]).is_err());
    assert!(
        summary_values(
            &SummaryHelper::MeanClBoot {
                confidence: 0.95,
                draws: vec![vec![0]]
            },
            &[1., 2.]
        )
        .is_err()
    );
    assert!(summary_values(&SummaryHelper::default(), &[f64::INFINITY]).is_err());
    let raw = rows(count().group("g"));
    assert_eq!(raw.iter().map(|r| r.count).collect::<Vec<_>>(), [4, 2]);
    assert!(
        raw.iter()
            .all(|r| r.value(&StatField::WeightedCount).is_none())
    );
}

#[test]
fn seeded_bootstrap_is_builtin_bounded_and_matches_explicit_draw_oracle() {
    let h = SummaryHelper::mean_cl_boot();
    let x = [1., 2., 4., 8.];
    let a = summary_values(&h, &x).unwrap();
    assert_eq!(a, summary_values(&h, &x).unwrap());
    assert_eq!(a[0], Some(3.75));
    assert!(a[1].unwrap() >= 1. && a[2].unwrap() <= 8.);
    let singleton = summary_values(&h, &[7.]).unwrap();
    assert_eq!(singleton, [Some(7.); 3]);
    assert!(
        summary_values(
            &SummaryHelper::MeanClBootSeeded {
                confidence: 0.95,
                samples: 0,
                seed: 0
            },
            &x
        )
        .is_err()
    );
    // Published SplitMix64 seed-zero prefix modulo four, two independently fixed resamples.
    let explicit = SummaryHelper::MeanClBoot {
        confidence: 0.95,
        draws: vec![vec![3, 0, 3, 0], vec![3, 2, 1, 0]],
    };
    let seeded = SummaryHelper::MeanClBootSeeded {
        confidence: 0.95,
        samples: 2,
        seed: 0,
    };
    assert_eq!(
        summary_values(&seeded, &x).unwrap(),
        summary_values(&explicit, &x).unwrap()
    );
}
#[test]
fn grouped_and_binned_summaries_match_batch_after_source_mutations() {
    use chart_core::transaction::CommitOutcome;
    type Input = (u64, f64, f64, i64);
    fn batch(rows: &[Input]) -> Data {
        Data::columns()
            .name("observations")
            .keys(rows.iter().map(|r| r.0))
            .column("x", rows.iter().map(|r| r.1).collect::<Vec<_>>())
            .column("y", rows.iter().map(|r| r.2).collect::<Vec<_>>())
            .column("g", rows.iter().map(|r| r.3).collect::<Vec<_>>())
            .build()
            .unwrap()
    }
    fn author(data: Data, binned: bool) -> Plot {
        let stat = summary()
            .x("x")
            .y("y")
            .group("g")
            .summary_helper(SummaryHelper::mean_cl_boot());
        let stat = if binned {
            stat.summary_bins(SummaryBins {
                breaks: Some(vec![0., 2., 5.]),
                ..Default::default()
            })
        } else {
            stat
        };
        plot(data).layer(points().stat(stat)).build().unwrap()
    }
    for binned in [false, true] {
        let mut expected = vec![
            (1, 1., 1., 1),
            (2, 1., 3., 1),
            (3, 2., 4., 1),
            (4, 4., 6., 2),
        ];
        let mut chart = author(batch(&expected), binned).chart().unwrap();
        for step in 0..4 {
            let tx = match step {
                0 => {
                    let add = [(5, 2., 8., 1), (6, 4., 2., 2)];
                    expected.extend(add);
                    chart
                        .transaction()
                        .unwrap()
                        .append("observations", batch(&add))
                        .build()
                        .unwrap()
                }
                1 => {
                    let changed = (2, 4., 10., 2);
                    expected[1] = changed;
                    chart
                        .transaction()
                        .unwrap()
                        .upsert("observations", batch(&[changed]))
                        .build()
                        .unwrap()
                }
                2 => {
                    expected.retain(|r| r.0 != 1);
                    chart
                        .transaction()
                        .unwrap()
                        .remove("observations", [1])
                        .build()
                        .unwrap()
                }
                _ => {
                    expected = expected[expected.len() - 3..].to_vec();
                    chart
                        .transaction()
                        .unwrap()
                        .retain_count("observations", Some(3))
                        .build()
                        .unwrap()
                }
            };
            assert!(matches!(
                chart.apply_transaction(tx).unwrap(),
                CommitOutcome::Applied(_)
            ));
            let live = chart.prepare().unwrap();
            let fresh = author(batch(&expected), binned)
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            let PreparedRows::Statistical(a) = live.layers()[0].table().rows() else {
                panic!()
            };
            let PreparedRows::Statistical(b) = fresh.layers()[0].table().rows() else {
                panic!()
            };
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b.iter()) {
                assert_eq!(a.group, b.group);
                assert_eq!(a.count, b.count);
                assert_eq!(a.values, b.values);
                assert_eq!(a.members, b.members);
            }
        }
    }
}

#[test]
fn automatic_transformed_and_faceted_summary_bins_match_reference() {
    let f = fixture();
    for mode in ["automatic", "transformed", "faceted", "free"] {
        println!("checking {mode}");
        let stat = summary()
            .x("x")
            .y("y")
            .group("g")
            .summary_bins(SummaryBins {
                bins: 3,
                ..Default::default()
            });
        let mut builder = plot(data())
            .profile(Profile::Ggplot2_4_0_3)
            .layer(points().stat(stat));
        if mode == "transformed" {
            builder = builder.y_axis(y_axis().scale(scale_log(10.)));
        }
        if mode == "faceted" || mode == "free" {
            builder = builder.facet(facet_wrap("g").free_x(mode == "free"));
        }
        let prepared = builder.build().unwrap().chart().unwrap().prepare().unwrap();
        let panels = if prepared.panels().is_empty() {
            vec![prepared.as_ref()]
        } else {
            prepared.panels().iter().map(|p| p.chart.as_ref()).collect()
        };
        let actual = panels
            .iter()
            .flat_map(|p| {
                let PreparedRows::Statistical(rows) = p.layers()[0].table().rows() else {
                    panic!()
                };
                rows.iter()
            })
            .collect::<Vec<_>>();
        let expected = f["extra"][mode].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{mode}");
        for (a, b) in actual.iter().zip(expected) {
            for (field, key) in [
                (StatField::X, "x"),
                (StatField::Y, "y"),
                (StatField::Lower, "ymin"),
                (StatField::Upper, "ymax"),
                (StatField::Width, "width"),
            ] {
                near(a.value(&field), &b[key]);
            }
        }
    }
}

#[test]
fn fixed_facet_histograms_reuse_the_same_pre_statistic_range() {
    let prepared = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").group("g"))
        .layer(histogram().stat(bin().bins(3).ggplot_bin(GgplotBinOptions::default())))
        .facet(facet_wrap("g"))
        .build()
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    let f = fixture();
    let expected = f["extra"]["histogram"].as_array().unwrap();
    let actual = prepared
        .panels()
        .iter()
        .flat_map(|p| {
            let PreparedRows::Binned(rows) = p.chart.layers()[0].table().rows() else {
                panic!()
            };
            rows.iter()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
        near(Some(a.start), &b["xmin"]);
        near(Some(a.end), &b["xmax"]);
        near(Some(a.count as f64), &b["count"]);
    }
}

#[test]
fn all_source_layers_train_reference_bin_ranges_and_infinite_count_weights_reject() {
    let d = Data::columns().column("x", [1., 2.]).build().unwrap();
    let overlay = Data::columns()
        .name("overlay")
        .column("x", [0., 10.])
        .column("y", [1., 1.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram().stat(bin().bins(3).ggplot_bin(GgplotBinOptions::default())))
        .layer(points().data(overlay).aes(aes().x("x").y("y")))
        .build()
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    let PreparedRows::Binned(rows) = p.layers()[0].table().rows() else {
        panic!()
    };
    let f = fixture();
    let expected = f["extra"]["overlay"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for (a, b) in rows.iter().zip(expected) {
        near(Some(a.start), &b["xmin"]);
        near(Some(a.end), &b["xmax"]);
        near(Some(a.count as f64), &b["count"]);
    }
    let d = Data::columns()
        .column("x", [1., 1.])
        .column("w", [1., f64::INFINITY])
        .build()
        .unwrap();
    let p = plot(d)
        .layer(points().stat(count().ggplot_count().x("x").count_weight("w")))
        .build()
        .unwrap();
    assert!(p.chart().unwrap().prepare().is_err());
}

#[test]
fn explicit_limits_define_bins_even_when_outside_values_are_kept() {
    use chart_core::interpolate::Number;
    let d = Data::columns().column("x", [1., 2., 10.]).build().unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram().stat(bin().bins(3).ggplot_bin(GgplotBinOptions::default())))
        .x_axis(
            x_axis()
                .numeric_limits(Some([Some(Number::from(0.)), Some(Number::from(5.))]))
                .oob(ScaleOob::Keep),
        )
        .build()
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    let PreparedRows::Binned(rows) = p.layers()[0].table().rows() else {
        panic!()
    };
    let f = fixture();
    let expected = f["extra"]["kept_limits"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for (a, b) in rows.iter().zip(expected) {
        near(Some(a.start), &b["xmin"]);
        near(Some(a.end), &b["xmax"]);
        near(Some(a.count as f64), &b["count"]);
    }
}

#[test]
fn horizontal_bin_training_uses_physical_y_and_its_facet_freedom() {
    let f = fixture();
    for mode in ["fixed", "free_x", "free_y"] {
        let d = Data::columns()
            .column("y", [0., 1., 100., 101.])
            .column("g", ["A", "A", "B", "B"])
            .build()
            .unwrap();
        let p = plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().y("y"))
            .layer(
                histogram()
                    .orientation(Orientation::Horizontal)
                    .stat(bin().x("y").bins(3).ggplot_bin(GgplotBinOptions::default())),
            )
            .facet(
                facet_wrap("g")
                    .free_x(mode == "free_x")
                    .free_y(mode == "free_y"),
            )
            .build()
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        let actual = p
            .panels()
            .iter()
            .flat_map(|p| {
                let PreparedRows::Binned(rows) = p.chart.layers()[0].table().rows() else {
                    panic!()
                };
                rows.iter()
            })
            .collect::<Vec<_>>();
        let expected = f["extra"][format!("horizontal_{mode}")].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{mode}");
        for (a, b) in actual.iter().zip(expected) {
            near(Some(a.start), &b["ymin"]);
            near(Some(a.end), &b["ymax"]);
            near(Some(a.count as f64), &b["count"]);
        }
    }
    let d = Data::columns().column("y", [1., 2.]).build().unwrap();
    let overlay = Data::columns()
        .name("overlay")
        .column("x", [1., 1.])
        .column("y", [0., 10.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().y("y"))
        .layer(
            histogram()
                .orientation(Orientation::Horizontal)
                .stat(bin().x("y").bins(3).ggplot_bin(GgplotBinOptions::default())),
        )
        .layer(points().data(overlay).aes(aes().x("x").y("y")))
        .build()
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    let PreparedRows::Binned(rows) = p.layers()[0].table().rows() else {
        panic!()
    };
    let expected = f["extra"]["horizontal_overlay"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for (a, b) in rows.iter().zip(expected) {
        near(Some(a.start), &b["ymin"]);
        near(Some(a.end), &b["ymax"]);
        near(Some(a.count as f64), &b["count"]);
    }
}
