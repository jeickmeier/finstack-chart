//! FIX-GG04 / GG2-03: Date/datetime inputs retain exact origins through numeric palettes.
use chart_core::{
    data::TimeUnit,
    grammar::{ColorInput, Numeric, NumericAesthetic},
    interpolate::Value,
    prelude::*,
    scales::MappedScale,
};
use serde_json::Value as Json;
#[test]
fn automatic_temporal_numeric_aesthetics_match_reference_and_survive_edits() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-defaults.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 60);
    for case in fixture["cases"].as_array().unwrap().iter().filter(|c| {
        matches!(
            c["channel"].as_str().unwrap(),
            "size" | "alpha" | "linewidth"
        )
    }) {
        for (unit, units) in [
            (TimeUnit::Seconds, 1_i64),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1_000_000),
            (TimeUnit::Nanoseconds, 1_000_000_000),
        ] {
            let inputs = case["inputs"].as_array().unwrap();
            let stamps = inputs
                .iter()
                .map(|v| {
                    v.as_i64().unwrap_or(0) * if case["kind"] == "date" { 86400 } else { 1 } * units
                })
                .collect::<Vec<_>>();
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
            let channel = case["channel"].as_str().unwrap();
            let target = match channel {
                "size" => NumericAesthetic::Size,
                "alpha" => NumericAesthetic::Alpha,
                _ => NumericAesthetic::StrokeWidth,
            };
            let mapping = match channel {
                "size" => aes().x("x").y("y").size("when"),
                "alpha" => aes().x("x").y("y").alpha("when"),
                _ => aes().x("x").y("y").linewidth("when"),
            };
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .layer(
                    (if channel == "linewidth" {
                        line()
                    } else {
                        points()
                    })
                    .name("marks"),
                )
                .build();
            if case["result"]["error"].is_string() {
                let result = figure.and_then(|figure| figure.chart()?.prepare());
                assert!(result.is_err(), "reference rejects temporal guide: {case}");
                continue;
            }
            let figure = figure.unwrap();
            let figure = Plot::from_json(&figure.to_json().unwrap()).unwrap();
            let original = figure.chart().unwrap().prepare().unwrap();
            let original_encoding = original.layers()[0]
                .numeric_scales()
                .get(&target)
                .unwrap_or_else(|| panic!("missing automatic scale: {case} {unit:?}"));
            for edited in [
                figure.clone(),
                figure
                    .edit()
                    .layer(
                        "marks",
                        (if channel == "linewidth" {
                            line()
                        } else {
                            points()
                        })
                        .color(rgb(1, 2, 3)),
                    )
                    .build()
                    .unwrap(),
            ] {
                let prepared = edited.chart().unwrap().prepare().unwrap();
                let layer = &prepared.layers()[0];
                let encoding = &layer.numeric_scales()[&target];
                assert_eq!(encoding.id, original_encoding.id, "{case}");
                assert_eq!(encoding.input, original_encoding.input, "{case}");
                let ColorInput::Numeric(Numeric::Timestamp { origin, .. }) = encoding.input else {
                    panic!("lost timestamp input")
                };
                let scale = MappedScale::for_numbers(encoding.scale.clone()).unwrap();
                let guide = scale.continuous_guide_entries(128, 65536).unwrap().unwrap();
                if let Some(expected) = case["result"]["guide"]["breaks"].as_array() {
                    assert_eq!(guide.len(), expected.len(), "{case}");
                    for (entry, (value, label)) in guide.iter().zip(
                        expected
                            .iter()
                            .zip(case["result"]["guide"]["labels"].as_array().unwrap()),
                    ) {
                        let stamp = value.as_i64().unwrap()
                            * if case["kind"] == "date" { 86400 } else { 1 }
                            * units;
                        assert_eq!(entry.value.0, (stamp - origin) as f64, "{case}");
                        assert_eq!(entry.label.as_deref(), label.as_str(), "{case}");
                    }
                } else {
                    assert!(guide.is_empty(), "{case}");
                }

                for (i, input) in inputs.iter().enumerate() {
                    let actual = scale
                        .numeric((!input.is_null()).then(|| (stamps[i] - origin) as f64))
                        .unwrap();
                    let expected = &case["result"]["values"][i];
                    if expected.is_null() {
                        assert!(
                            matches!(actual, Value::Missing | Value::Null),
                            "{actual:?} {case}"
                        )
                    } else {
                        let Value::Number(n) = actual else {
                            panic!("numeric scale output")
                        };
                        assert!(
                            (n.0 - expected.as_f64().unwrap()).abs() < 2e-12,
                            "{actual:?} {expected} {case} {unit:?}"
                        );
                    }
                }
                if channel == "size" {
                    assert_eq!(
                        layer.marks().len(),
                        case["result"]["values"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter(|v| !v.is_null())
                            .count()
                    );
                    for (mark, value) in layer.marks().iter().zip(
                        case["result"]["values"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter_map(Json::as_f64),
                    ) {
                        assert!((mark.style.radius - value).abs() < 2e-12, "{case}")
                    }
                }
            }
        }
    }
}

#[test]
fn temporal_paint_defaults_match_values_and_guides() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-defaults.json"
    ))
    .unwrap();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["channel"].as_str(), Some("colour" | "fill")))
    {
        for (unit, units) in [
            (TimeUnit::Seconds, 1_i64),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1_000_000),
            (TimeUnit::Nanoseconds, 1_000_000_000),
        ] {
            let inputs = case["inputs"].as_array().unwrap();
            let factor = if case["kind"] == "date" { 86400 } else { 1 } * units;
            let data = Data::columns()
                .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column("y", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "when",
                    timestamps(
                        inputs
                            .iter()
                            .map(|v| v.as_i64().unwrap_or(0) * factor)
                            .collect(),
                        unit,
                        "UTC",
                    )
                    .validity(inputs.iter().map(|v| !v.is_null()).collect()),
                )
                .build()
                .unwrap();
            let fill = case["channel"] == "fill";
            let mapping = if fill {
                aes().x("x").y("y").fill("when")
            } else {
                aes().x("x").y("y").color("when")
            };
            let figure = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .layer(points().name("marks"))
                .build();
            if case["result"]["error"].is_string() {
                assert!(
                    figure.and_then(|figure| figure.chart()?.prepare()).is_err(),
                    "{case}"
                );
                continue;
            }
            let figure = figure.unwrap();
            let figure = Plot::from_json(&figure.to_json().unwrap()).unwrap();
            for edited in [
                figure.clone(),
                figure
                    .edit()
                    .layer("marks", points().radius(3.))
                    .build()
                    .unwrap(),
            ] {
                let prepared = edited.chart().unwrap().prepare().unwrap();
                let layer = &prepared.layers()[0];
                assert_eq!(layer.marks().len(), inputs.len(), "{case}");
                for (mark, expected) in layer
                    .marks()
                    .iter()
                    .zip(case["result"]["values"].as_array().unwrap())
                {
                    let actual = if fill {
                        mark.style.fill.unwrap()
                    } else {
                        mark.style.color
                    };
                    let expected = chart_core::color::parse_r(expected.as_str().unwrap())
                        .unwrap()
                        .resolve();
                    assert_eq!(actual, expected, "{case} {unit:?}");
                }
                let legend = if fill {
                    layer
                        .paint_legends()
                        .get(&chart_core::grammar::PaintAesthetic::Fill)
                } else {
                    layer.color_legend()
                };
                if let Some(expected) = case["result"]["guide"]["labels"].as_array() {
                    let legend = legend.unwrap();
                    assert_eq!(
                        legend
                            .numeric_breaks
                            .iter()
                            .map(|v| v.label.clone())
                            .collect::<Vec<_>>(),
                        expected
                            .iter()
                            .map(|v| v.as_str().map(str::to_owned))
                            .collect::<Vec<_>>(),
                        "{case}"
                    );
                }
            }
        }
    }
}

#[test]
fn explicit_timestamp_origins_survive_edits_and_authored_scales_keep_their_policy() {
    use chart_core::scales::{GgplotNumericPalette, ggplot_numeric_default};
    for (unit, factor) in [
        (TimeUnit::Seconds, 1_i64),
        (TimeUnit::Milliseconds, 1000),
        (TimeUnit::Microseconds, 1_000_000),
        (TimeUnit::Nanoseconds, 1_000_000_000),
    ] {
        let epoch = 1_704_067_200 * factor;
        let origin = epoch - 37 * factor;
        let data = Data::columns()
            .column("x", vec![0., 1., 2., 3.])
            .column("y", vec![0., 1., 2., 3.])
            .column(
                "when",
                timestamps(
                    [0, 1, 4, 9].map(|delta| epoch + delta * factor).to_vec(),
                    unit,
                    "UTC",
                ),
            )
            .build()
            .unwrap();
        for (target, kind) in [
            (NumericAesthetic::Size, GgplotNumericPalette::Size),
            (NumericAesthetic::Alpha, GgplotNumericPalette::Alpha),
            (
                NumericAesthetic::StrokeWidth,
                GgplotNumericPalette::Linewidth,
            ),
        ] {
            let input = Mapping::Timestamp {
                field: "when".into(),
                origin,
            };
            let mapping = match target {
                NumericAesthetic::Size => aes().x("x").y("y").size(input.clone()),
                NumericAesthetic::Alpha => aes().x("x").y("y").alpha(input.clone()),
                _ => aes().x("x").y("y").linewidth(input.clone()),
            };
            let p = plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .layer(points().name("marks"))
                .build()
                .unwrap();
            let a = p.chart().unwrap().prepare().unwrap();
            let q = p
                .edit()
                .layer("marks", points().stroke(rgb(1, 2, 3)))
                .build()
                .unwrap();
            let b = q.chart().unwrap().prepare().unwrap();
            assert_eq!(
                a.layers()[0].numeric_scales(),
                b.layers()[0].numeric_scales()
            );
            assert!(
                matches!(a.layers()[0].numeric_scales()[&target].input,ColorInput::Numeric(Numeric::Timestamp{origin:actual,..}) if actual==origin)
            );
            let manual = plot(data.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(points().numeric_scale(target, input, ggplot_numeric_default(kind).unwrap()))
                .build()
                .unwrap();
            assert!(
                manual.chart().unwrap().prepare().unwrap().layers()[0].numeric_scales()[&target]
                    .scale
                    .guide
                    .is_none()
            );
        }
    }
}

#[test]
fn temporal_precision_uses_absolute_reference_normalization_independent_of_guides() {
    use chart_core::scales::GgplotScaleGuide;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-precision.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 180);
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["channel"].as_str(), Some("size" | "alpha" | "linewidth")))
    {
        let epoch = case["epoch"].as_i64().unwrap() * 1_000_000_000;
        let stamps: Vec<_> = case["offsets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| epoch + (v.as_f64().unwrap() * 1e9).round() as i64)
            .collect();
        let data = Data::columns()
            .column("x", vec![0., 1., 2., 3.])
            .column("y", vec![0., 1., 2., 3.])
            .column(
                "when",
                timestamps(stamps.clone(), TimeUnit::Nanoseconds, "UTC"),
            )
            .build()
            .unwrap();
        let (mapping, target) = match case["channel"].as_str().unwrap() {
            "size" => (aes().x("x").y("y").size("when"), NumericAesthetic::Size),
            "alpha" => (aes().x("x").y("y").alpha("when"), NumericAesthetic::Alpha),
            _ => (
                aes().x("x").y("y").linewidth("when"),
                NumericAesthetic::StrokeWidth,
            ),
        };
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(mapping)
            .layer(points())
            .build()
            .unwrap();
        let prepared = figure.chart().unwrap().prepare().unwrap();
        let encoding = &prepared.layers()[0].numeric_scales()[&target];
        let ColorInput::Numeric(Numeric::Timestamp { origin, .. }) = encoding.input else {
            panic!("timestamp lost")
        };
        let mut spec = encoding.scale.clone();
        if case["hidden"] == true {
            spec.guide = Some(Box::new(GgplotScaleGuide::Hidden));
        }
        let spec = serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        let scale = MappedScale::for_numbers(spec).unwrap();
        for (stamp, expected) in stamps.iter().zip(case["values"].as_array().unwrap()) {
            let Value::Number(actual) = scale
                .numeric(Some((i128::from(*stamp) - i128::from(origin)) as f64))
                .unwrap()
            else {
                panic!("numeric output")
            };
            assert!(
                (actual.0 - expected.as_f64().unwrap()).abs() < 2e-12,
                "actual {} expected {expected}: {case}",
                actual.0
            );
        }
    }
}

#[test]
fn temporal_precision_paint_and_hidden_guide_match_reference() {
    fn hide_guides(value: &mut Json) {
        match value {
            Json::Object(fields) => {
                if fields
                    .get("guide")
                    .is_some_and(|g| g.get("Temporal").is_some())
                {
                    fields.insert("guide".into(), Json::String("Hidden".into()));
                }
                for child in fields.values_mut() {
                    hide_guides(child);
                }
            }
            Json::Array(values) => {
                for child in values {
                    hide_guides(child);
                }
            }
            _ => (),
        }
    }
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/temporal-aesthetic-precision.json"
    ))
    .unwrap();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| matches!(c["channel"].as_str(), Some("colour" | "fill")))
    {
        let epoch = case["epoch"].as_i64().unwrap() * 1_000_000_000;
        let stamps: Vec<_> = case["offsets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| epoch + (v.as_f64().unwrap() * 1e9).round() as i64)
            .collect();
        let data = Data::columns()
            .column("x", vec![0., 1., 2., 3.])
            .column("y", vec![0., 1., 2., 3.])
            .column("when", timestamps(stamps, TimeUnit::Nanoseconds, "UTC"))
            .build()
            .unwrap();
        let fill = case["channel"] == "fill";
        let mapping = if fill {
            aes().x("x").y("y").fill("when")
        } else {
            aes().x("x").y("y").color("when")
        };
        let figure = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(mapping)
            .layer(points())
            .build()
            .unwrap();
        let mut wire: Json = serde_json::from_str(&figure.to_json().unwrap()).unwrap();
        assert_eq!(wire["version"], 25);
        if case["hidden"] == true {
            hide_guides(&mut wire);
        }
        let figure = Plot::from_json(&serde_json::to_string(&wire).unwrap()).unwrap();
        let prepared = figure.chart().unwrap().prepare().unwrap();
        for (mark, expected) in prepared.layers()[0]
            .marks()
            .iter()
            .zip(case["values"].as_array().unwrap())
        {
            let actual = if fill {
                mark.style.fill.unwrap()
            } else {
                mark.style.color
            };
            assert_eq!(
                actual,
                chart_core::color::parse_r(expected.as_str().unwrap())
                    .unwrap()
                    .resolve(),
                "{case}"
            );
        }
        assert_eq!(prepared.layers()[0].marks().len(), 4);
    }
}
