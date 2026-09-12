//! FIX-GG04: reference Date/datetime aesthetic guide arguments and population precedence.
use chart_core::{
    data::TimeUnit,
    grammar::NumericAesthetic,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::Value as Json;
fn spec(
    case: &Json,
    origin: i64,
    unit: TimeUnit,
    factor: f64,
) -> chart_core::ChartResult<MappedScaleSpec> {
    let mut scale = ggplot_numeric_default(GgplotNumericPalette::Size)?
        .with_timestamp_normalization(GgplotTimestampNormalization {
            origin,
            unit,
            date: case["kind"] == "date",
        })?;
    let control = case["control"].as_str().unwrap();
    if let Some(GgplotScalePolicy::Continuous { limits, .. }) = scale.ggplot.as_deref_mut() {
        if control == "limits" {
            *limits = Some([Some(Number(-factor)), Some(Number(5. * factor))]);
        }
        if control == "partial_limits" {
            *limits = Some([None, Some(Number(5. * factor))]);
        }
    }
    let mut args = GgplotTemporalGuideArguments {
        date: case["kind"] == "date",
        ..Default::default()
    };
    match control {
        "null_breaks" => args.breaks = GgplotTemporalBreaks::None,
        "empty_breaks" => args.breaks = GgplotTemporalBreaks::Explicit(vec![]),
        "explicit_breaks" | "explicit_labels" | "bad_labels" => {
            args.breaks = GgplotTemporalBreaks::Explicit(
                [-1., 0., 2., 5.].map(|v| Number(v * factor)).to_vec(),
            )
        }
        "width" => {
            args.breaks = GgplotTemporalBreaks::Width(
                case["width"]
                    .as_str()
                    .unwrap_or(if args.date { "2 days" } else { "2 secs" })
                    .into(),
            )
        }
        "count_two" => args.count = Some(2.),
        _ => (),
    }
    match control {
        "null_labels" => args.labels = GgplotGuideLabels::Hidden,
        "explicit_labels" => {
            args.labels = GgplotGuideLabels::Explicit(
                ["Before", "Start", "Two", "After"]
                    .map(|v| Some(v.into()))
                    .to_vec(),
            )
        }
        "bad_labels" => {
            args.labels = GgplotGuideLabels::Explicit(["A", "B"].map(|v| Some(v.into())).to_vec())
        }
        "format" => {
            args.format = Some(Box::new(GgplotTimeFormat {
                pattern: if args.date { "%Y-%m-%d" } else { "%H:%M:%S" }.into(),
                locale: None,
            }))
        }
        _ => (),
    }
    scale = scale.with_guide(GgplotScaleGuide::Temporal(GgplotTemporalGuide {
        origin,
        unit,
        zone: CalendarZone::Utc,
        arguments: args,
    }))?;
    if control == "hidden_guide" {
        scale = scale.with_guide(GgplotScaleGuide::Hidden)?;
    }
    Ok(scale)
}
#[test]
fn temporal_arguments_match_reference_mapping_guides_json_and_edits() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-guides.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 156);
    run_cases(
        fixture["cases"].as_array().unwrap(),
        &[
            (TimeUnit::Seconds, 1),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1000000),
            (TimeUnit::Nanoseconds, 1000000000),
        ],
    );
}
#[test]
fn temporal_width_units_fractional_alignment_and_rejections_match_reference() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-widths.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 60);
    // Microseconds cover fractional seconds and multi-year spans with exact offsets.
    run_cases(
        fixture["cases"].as_array().unwrap(),
        &[(TimeUnit::Microseconds, 1000000)],
    );
}
fn run_cases(cases: &[Json], units: &[(TimeUnit, i64)]) {
    for case in cases {
        for &(unit, multiplier) in units {
            let date = case["kind"] == "date";
            let factor = multiplier * if date { 86400 } else { 1 };
            let origin = 1704067200 * multiplier;
            let inputs = case["inputs"].as_array().unwrap();
            let stamps: Vec<_> = inputs
                .iter()
                .map(|v| {
                    let value = v
                        .as_f64()
                        .unwrap_or(if date { 19723. } else { 1704067200. });
                    let whole = value.trunc() as i64;
                    whole * factor + ((value - whole as f64) * factor as f64).round() as i64
                })
                .collect();
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("y", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "when",
                    timestamps(stamps.clone(), unit, "UTC")
                        .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                )
                .build()
                .unwrap();
            let figure = spec(case, origin, unit, factor as f64).and_then(|scale| {
                plot(data)
                    .profile(Profile::Ggplot2_4_0_3)
                    .aes(aes().x("x").y("y"))
                    .layer(points().name("marks").numeric_scale(
                        NumericAesthetic::Size,
                        Mapping::Timestamp {
                            field: "when".into(),
                            origin,
                        },
                        scale,
                    ))
                    .build()
            });
            let expected = &case["result"];
            if expected["error"].is_string() || expected["draw_error"].is_string() {
                assert!(
                    figure.and_then(|p| p.chart()?.prepare()).is_err(),
                    "expected failure: {case} {unit:?}"
                );
                continue;
            }
            let figure = figure.unwrap_or_else(|e| panic!("{case} {unit:?}: {e:?}"));
            let figure = Plot::from_json(&figure.to_json().unwrap()).unwrap();
            for current in [
                figure.clone(),
                figure
                    .edit()
                    .layer(
                        "marks",
                        points().stroke(rgb(1, 2, 3)).numeric_scale(
                            NumericAesthetic::Size,
                            Mapping::Timestamp {
                                field: "when".into(),
                                origin,
                            },
                            spec(case, origin, unit, factor as f64).unwrap(),
                        ),
                    )
                    .build()
                    .unwrap(),
            ] {
                let prepared = current
                    .chart()
                    .unwrap()
                    .prepare()
                    .unwrap_or_else(|e| panic!("{case} {unit:?}: {e:?}"));
                let layer = &prepared.layers()[0];
                let encoding = layer
                    .numeric_scales()
                    .get(&NumericAesthetic::Size)
                    .unwrap_or_else(|| panic!("missing scale: {case} {unit:?}"));
                let scale = MappedScale::for_numbers(encoding.scale.clone()).unwrap();
                let entries = scale.continuous_guide_entries(128, 65536).unwrap();
                let entries = entries.unwrap();
                if case["control"] == "hidden_guide"
                    || case["population"] == "empty" && case["control"] != "limits"
                {
                    assert!(entries.is_empty(), "{case}");
                } else {
                    let breaks = expected["guide"]["breaks"]
                        .as_array()
                        .unwrap_or_else(|| panic!("{case}"));
                    assert_eq!(entries.len(), breaks.len(), "{case} {unit:?}: {entries:?}");
                    for (index, (entry, expected_break)) in entries.iter().zip(breaks).enumerate() {
                        let absolute =
                            (i128::from(origin) + entry.value.0 as i128) as f64 / factor as f64;
                        assert!(
                            (absolute - expected_break.as_f64().unwrap()).abs() < 1e-8,
                            "{case}: {entry:?}"
                        );
                        let label = expected["guide"]["labels"]
                            .get(index)
                            .and_then(Json::as_str);
                        assert_eq!(entry.label.as_deref(), label, "{case} {unit:?}");
                    }
                }
                for ((stamp, input), expected_value) in stamps
                    .iter()
                    .zip(inputs)
                    .zip(expected["values"].as_array().unwrap())
                {
                    let value = scale
                        .numeric(if input.is_null() {
                            None
                        } else {
                            Some((i128::from(*stamp) - i128::from(origin)) as f64)
                        })
                        .unwrap();
                    if let Some(expected_value) = expected_value.as_f64() {
                        let Value::Number(actual) = value else {
                            panic!("{case}: {value:?}")
                        };
                        assert!(
                            (actual.0 - expected_value).abs() < 2e-12,
                            "{case}: {actual:?}"
                        );
                    } else {
                        assert!(
                            matches!(value, Value::Missing | Value::Null),
                            "{case}: {value:?}"
                        );
                    }
                }
            }
        }
    }
}
