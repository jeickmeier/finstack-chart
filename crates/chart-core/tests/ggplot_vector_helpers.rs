//! Independent vector-helper source and materialization contracts.
use chart_core::plot::{CutOptions, CutSpec, ResolutionOptions, cut, resolution};
#[test]
fn pinned_vector_helper_contracts() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/vector-helper-controls.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let op = case["operation"].as_str().unwrap();
        let o = &case["options"];
        let x: Vec<_> = case["x"].as_array().map_or_else(
            || vec![case["x"].as_f64()],
            |a| a.iter().map(serde_json::Value::as_f64).collect(),
        );
        if op == "resolution" {
            let actual = resolution(
                &x,
                ResolutionOptions {
                    zero: o["zero"].as_bool().unwrap(),
                    integer: case["integer"].as_bool().unwrap(),
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(
                (actual - case["result"]["value"].as_f64().unwrap()).abs() < 1e-14,
                "{name}: {actual}"
            );
            continue;
        }
        let spec = match op {
            "cut_interval" => {
                if let Some(length) = o["length"].as_f64() {
                    CutSpec::IntervalLength { length }
                } else {
                    CutSpec::Interval {
                        bins: o["n"].as_u64().unwrap() as usize,
                    }
                }
            }
            "cut_number" => CutSpec::Number {
                bins: o["n"].as_u64().unwrap() as usize,
            },
            "cut_width" => CutSpec::Width {
                width: o["width"].as_f64().unwrap(),
                center: o["center"].as_f64(),
                boundary: o["boundary"].as_f64(),
            },
            _ => unreachable!(),
        };
        let options = CutOptions {
            right: o["right"].as_bool().unwrap_or(o["closed"] != "left"),
            labels: o["labels"]
                .as_array()
                .map(|a| a.iter().map(|s| s.as_str().unwrap().to_string()).collect()),
            ordered: o["ordered_result"].as_bool().unwrap_or(false),
            ..Default::default()
        };
        let actual = cut(&x, spec, options);
        if case["result"]["error"].is_string() {
            assert!(actual.is_err(), "{name}");
            continue;
        }
        let actual = actual.unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(
            serde_json::to_value(&actual.codes).unwrap(),
            case["result"]["codes"],
            "{name} codes"
        );
        assert_eq!(
            serde_json::to_value(&actual.levels).unwrap(),
            case["result"]["levels"],
            "{name} levels"
        );
        assert_eq!(
            actual.ordered,
            case["result"]["ordered"].as_bool().unwrap(),
            "{name}"
        );
    }
}
#[test]
fn materialization_preserves_unused_levels_and_missing_rows() {
    let c = cut(
        &[Some(0.), None, Some(3.)],
        CutSpec::Interval { bins: 3 },
        CutOptions::default(),
    )
    .unwrap();
    let d = chart_core::plot::Data::columns()
        .column("bins", c.column())
        .build()
        .unwrap();
    assert_eq!(c.codes, vec![Some(0), None, Some(2)]);
    assert_eq!(c.levels.len(), 3);
    let p = chart_core::plot::plot(d)
        .layer(chart_core::plot::blank())
        .build()
        .unwrap();
    assert!(p.chart().unwrap().prepare().is_ok());
    assert!(
        cut(
            &[Some(1.)],
            CutSpec::Interval { bins: usize::MAX },
            CutOptions::default()
        )
        .is_err()
    );
}

#[test]
fn standalone_summary_matches_independent_sample_and_missing_contract() {
    use chart_core::{grammar::SummaryHelper, plot::summarize};
    let helper = SummaryHelper::MeanSe { mult: 1. };
    let result = summarize(
        &[Some(1.), None, Some(f64::NAN), Some(2.), Some(3.)],
        &helper,
    )
    .unwrap();
    let se = (1_f64 / 3.).sqrt();
    for (actual, expected) in result.into_iter().zip([2., 2. - se, 2. + se]) {
        assert!((actual.unwrap() - expected).abs() < 1e-14);
    }
    assert_eq!(summarize(&[None], &helper).unwrap(), [None; 3]);
    assert!(summarize(&[Some(f64::INFINITY)], &helper).is_err());
    let singleton = summarize(&[Some(2.)], &helper).unwrap();
    assert_eq!(singleton, [Some(2.), None, None]);
}
