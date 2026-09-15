//! FIX-GG04: built-in numeric palettes composed with the shared binned policy.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
#[test]
fn binned_numeric_maps_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/binned-numeric-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 288);
    let mut compared = 0;
    for case in cases {
        // R size/linewidth wrappers have no `right` argument. Our explicit generic
        // binned policy corresponds to binned_scale, which does accept it.
        if case["control"] == "left"
            && (case["palette"] == "size" || case["palette"] == "linewidth")
        {
            continue;
        }
        compared += 1;
        let kind = match case["palette"].as_str().unwrap() {
            "size" => GgplotNumericPalette::Size,
            "area" => GgplotNumericPalette::Area,
            "alpha" => GgplotNumericPalette::Alpha,
            "linewidth" => GgplotNumericPalette::Linewidth,
            _ => unreachable!(),
        };
        let control = case["control"].as_str().unwrap();
        let policy = GgplotBinnedPolicy {
            breaks: match control {
                "explicit" => GgplotBreaks::Explicit([1., 3., 5., 7., 9.].map(Number).to_vec()),
                "empty_breaks" | "null_breaks" => GgplotBreaks::Explicit(vec![]),
                "count_eight" => GgplotBreaks::Equal(8.),
                "count_sixteen" => GgplotBreaks::Equal(16.),
                _ => GgplotBreaks::Nice(5.),
            },
            limits: match control {
                "limits" => Some([Some(Number(-1.)), Some(Number(5.))]),
                "partial_limits" => Some([None, Some(Number(5.))]),
                _ => None,
            },
            right: control != "left",
            ..Default::default()
        };
        let inputs: Vec<_> = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().map(Number))
            .collect();
        check_constructor(case, kind, policy.clone(), &inputs);
        let result = (|| -> chart_core::ChartResult<_> {
            let spec = numeric_spec(case, kind)?
                .with_ggplot(GgplotScalePolicy::Binned(Box::new(policy)))?
                .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))?
                .trained(&inputs)?;
            let json = serde_json::to_string(&spec).unwrap();
            let scale = MappedScale::for_numbers(serde_json::from_str(&json).unwrap())?;
            let values = inputs
                .iter()
                .map(|input| scale.numeric(input.map(|v| v.0)))
                .collect::<chart_core::ChartResult<Vec<_>>>()?;
            let guide = scale.binned_guide_entries(4096, 65536)?;
            Ok((values, guide))
        })();
        let expected = &case["raw_result"];
        if expected.get("error").is_some() {
            assert!(result.is_err(), "{case}");
            continue;
        }
        let (values, guide) = result.unwrap_or_else(|error| panic!("{error:?}: {case}"));
        let wanted = expected["values"].as_array().unwrap();
        assert!(wanted.len() == values.len() || wanted.len() == 1, "{case}");
        for (i, got) in values.iter().enumerate() {
            let want = &wanted[if wanted.len() == 1 { 0 } else { i }];
            if want.is_null() {
                assert_eq!(*got, Value::Missing, "{case}");
            } else {
                let Value::Number(got) = got else {
                    panic!("{got:?} {case}")
                };
                assert!(
                    (got.0 - want.as_f64().unwrap()).abs() < 2e-12,
                    "{got:?} {case}"
                );
            }
        }
        let entries = guide.unwrap();
        let cuts = expected["guide"]["breaks"].as_array().unwrap();
        assert_eq!(entries.len(), cuts.len(), "{case}");
        for (got, want) in entries.iter().zip(cuts) {
            assert!(
                (got.transformed.0 - want.as_f64().unwrap()).abs() < 2e-12,
                "{case}"
            );
        }
        assert_eq!(
            entries
                .iter()
                .map(|e| e.label.as_deref())
                .collect::<Vec<_>>(),
            expected["guide"]["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            "{case}"
        );
    }
    assert_eq!(compared, 276);
}

fn check_constructor(
    case: &serde_json::Value,
    kind: GgplotNumericPalette,
    policy: GgplotBinnedPolicy,
    inputs: &[Option<Number>],
) {
    use chart_core::prelude::*;
    let expected = &case["result"];
    let result = (|| -> chart_core::ChartResult<_> {
        let scale = numeric_spec(case, kind)?
            .with_ggplot(GgplotScalePolicy::Binned(Box::new(policy)))?
            .with_guide(if case["control"] == "null_breaks" {
                GgplotScaleGuide::Hidden
            } else {
                GgplotScaleGuide::BinnedBins(GgplotGuideLabels::Automatic)
            })?;
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "v",
                inputs.iter().map(|v| v.map(|v| v.0)).collect::<Vec<_>>(),
            )
            .build()?;
        let channel = match kind {
            GgplotNumericPalette::Alpha => NumericAesthetic::Alpha,
            GgplotNumericPalette::Linewidth => NumericAesthetic::StrokeWidth,
            _ => NumericAesthetic::Size,
        };
        let layer = if kind == GgplotNumericPalette::Linewidth {
            rule()
        } else {
            points()
        };
        let plot = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).x2("x").y2(2.))
            .layer(layer.numeric_scale(channel, "v", scale))
            .build()?;
        let wire = plot.to_json()?;
        let restored = Plot::from_json(&wire)?;
        assert_eq!(wire, restored.to_json()?);
        restored.chart()?.prepare()
    })();
    if expected.get("error").is_some() || expected["draw_error"].is_string() {
        assert!(result.is_err(), "{case}");
        return;
    }
    let prepared = result.unwrap_or_else(|e| panic!("{e:?}: {case}"));
    let entries = prepared.layers()[0]
        .numeric_value_guides()
        .values()
        .flatten()
        .filter(|e| e.visible)
        .collect::<Vec<_>>();
    let keys = expected["guide_keys"].as_array().unwrap();
    if entries.is_empty() {
        assert!(keys.is_empty(), "{case}");
        return;
    }
    assert_eq!(keys.len(), 1, "{case}");
    let keys = &keys[0];
    assert_eq!(
        entries.len(),
        keys["values"].as_array().unwrap().len(),
        "{case}"
    );
    for (index, entry) in entries.iter().enumerate() {
        let position = index as f64 / (entries.len() - 1) as f64;
        assert!(
            (position - keys["values"][index].as_f64().unwrap()).abs() < 2e-12,
            "{case}"
        );
        assert_eq!(
            entry.label.as_deref(),
            keys["labels"][index].as_str(),
            "{case}"
        );
        let want = &keys["mapped"][index];
        if want.is_null() {
            assert_eq!(entry.mapped, Some(Value::Missing), "{case}");
        } else {
            let Some(Value::Number(got)) = entry.mapped else {
                panic!("{case}")
            };
            assert!((got.0 - want.as_f64().unwrap()).abs() < 2e-12, "{case}");
        }
    }
}

fn numeric_spec(
    case: &serde_json::Value,
    kind: GgplotNumericPalette,
) -> chart_core::ChartResult<MappedScaleSpec> {
    let mut spec = ggplot_numeric_default(kind)?;
    let control = case["control"].as_str().unwrap();
    let range = if kind == GgplotNumericPalette::Area {
        match control {
            "custom_range" => Some([0., 9.]),
            "reverse_range" => Some([0., 2.]),
            "constant_range" => Some([0., 0.]),
            _ => None,
        }
    } else {
        match control {
            "custom_range" => Some([2., 9.]),
            "reverse_range" => Some([9., 2.]),
            "constant_range" => Some([3.5, 3.5]),
            _ => None,
        }
    };
    if let Some(range) = range {
        let ScaleFunctionSpec::Interpolated(ref mut interpolated) = spec.function else {
            unreachable!()
        };
        let ScaleRangeFunction::Interpolate(
            chart_core::interpolate::InterpolationSpec::PowerRange {
                range: ref mut output_range,
                ..
            },
        ) = interpolated.output
        else {
            unreachable!()
        };
        *output_range = range.map(Number);
    }
    Ok(spec)
}
