//! FIX-C01: offline reference operations and independent floating/paint anchors.
use chart_core::color::*;
use serde_json::{Value, json};
fn number(v: &Value) -> f64 {
    match v.get("number").and_then(Value::as_str) {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        Some("-0") => -0.,
        _ => v.as_f64().unwrap(),
    }
}
fn space(name: &str) -> ColorSpace {
    match name {
        "rgb" => ColorSpace::Rgb,
        "hsl" => ColorSpace::Hsl,
        "lab" => ColorSpace::Lab,
        "hcl" | "lch" => ColorSpace::Hcl,
        "cubehelix" => ColorSpace::Cubehelix,
        _ => panic!("space"),
    }
}
fn input(v: &Value) -> Option<ColorValue> {
    if let Some(css) = v["parse"].as_str() {
        return parse(css).unwrap();
    }
    let name = v["constructor"].as_str().unwrap();
    if let Some(css) = v["css"].as_str() {
        return Some(ColorValue::from_css(css, space(name)).unwrap());
    }
    let args: Vec<_> = v["args"].as_array().unwrap().iter().map(number).collect();
    let value: ColorValue = match name {
        "rgb" => rgb(args[0], args[1], args[2]).into(),
        "hsl" => hsl(args[0], args[1], args[2]).into(),
        "lab" => lab(args[0], args[1], args[2]).into(),
        "hcl" => hcl(args[0], args[1], args[2]).into(),
        "lch" => lch(args[0], args[1], args[2]).into(),
        "cubehelix" => cubehelix(args[0], args[1], args[2]).into(),
        "gray" => gray(args[0]).into(),
        _ => panic!("constructor"),
    };
    Some(
        value.with_opacity(
            args.get(if name == "gray" { 1 } else { 3 })
                .copied()
                .unwrap_or(1.),
        ),
    )
}
fn compare(path: &str, actual: &Value, expected: &Value, failures: &mut Vec<String>) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(e)) => {
            let (a, e) = (a.as_f64().unwrap(), e.as_f64().unwrap());
            if (a - e).abs() > 1e-10 * e.abs().max(1.) {
                failures.push(format!("{path}: {a} != {e}"));
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            if a.len() != e.len() {
                failures.push(format!("{path}: object keys differ"));
            }
            for (k, e) in e {
                compare(
                    &format!("{path}/{k}"),
                    a.get(k).unwrap_or(&Value::Null),
                    e,
                    failures,
                );
            }
        }
        (Value::Array(a), Value::Array(e)) => {
            if a.len() != e.len() {
                failures.push(format!("{path}: array lengths differ"));
            }
            for (i, e) in e.iter().enumerate() {
                compare(
                    &format!("{path}/{i}"),
                    a.get(i).unwrap_or(&Value::Null),
                    e,
                    failures,
                );
            }
        }
        _ => {
            if actual != expected {
                failures.push(format!("{path}: {actual} != {expected}"));
            }
        }
    }
}
#[test]
fn complete_color_oracle_covers_all_constructors_and_inherited_operations() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-color/cases.json")).unwrap();
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-color/manifest.json"
    ))
    .unwrap();
    assert_eq!(manifest["exports"].as_array().unwrap().len(), 8);
    assert_eq!(manifest["cssNames"].as_array().unwrap().len(), 148);
    let mut failures = vec![];
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let expected = &case["expected"];
        let Some(value) = input(&case["input"]) else {
            assert!(expected.is_null(), "{id}");
            continue;
        };
        assert!(!expected.is_null(), "{id}: malformed CSS accepted");

        compare(
            &format!("{id}/value"),
            &json!(value),
            &expected["value"],
            &mut failures,
        );
        for name in ["rgb", "hsl", "lab", "hcl", "cubehelix"] {
            compare(
                &format!("{id}/convert/{name}"),
                &json!(value.convert(space(name))),
                &expected["conversions"][name],
                &mut failures,
            );
        }
        compare(
            &format!("{id}/displayable"),
            &json!(value.displayable()),
            &expected["displayable"],
            &mut failures,
        );
        for (name, text) in [
            ("formatHex", value.format_hex()),
            ("formatHex8", value.format_hex8()),
            ("formatRgb", value.format_rgb()),
            ("formatHsl", value.format_hsl()),
            ("toString", value.to_string()),
            ("hex", value.hex()),
        ] {
            compare(
                &format!("{id}/{name}"),
                &json!(text),
                &expected["formats"][name],
                &mut failures,
            );
        }
        let paint = value.to_paint();
        assert_eq!(
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                paint.red, paint.green, paint.blue, paint.alpha
            ),
            expected["formats"]["formatHex8"].as_str().unwrap(),
            "{id}/paint"
        );
        let mut copy = value.copy();
        for (name, v) in expected["copyPatch"].as_object().unwrap() {
            copy = copy.with_channel(name, number(v)).unwrap();
        }
        compare(
            &format!("{id}/copy"),
            &json!(copy),
            &expected["copy"],
            &mut failures,
        );
        for (i, op) in expected["brightness"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let k = (!op["k"].is_null()).then(|| number(&op["k"]));
            let result = if op["method"] == "brighter" {
                value.brighter(k)
            } else {
                value.darker(k)
            };
            compare(
                &format!("{id}/brightness/{i}"),
                &json!(result),
                &op["value"],
                &mut failures,
            );
        }
        compare(
            &format!("{id}/clamp"),
            &json!(value.clamp()),
            &expected["clamp"],
            &mut failures,
        );
        compare(
            &format!("{id}/sourceAfter"),
            &json!(value),
            &expected["sourceAfter"],
            &mut failures,
        );
        let descriptor = ColorDescriptor::new(value);
        assert_eq!(
            ColorDescriptor::from_json(&descriptor.to_json().unwrap()).unwrap(),
            descriptor,
            "{id}"
        );
    }
    assert!(
        failures.is_empty(),
        "{} discrepancies:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
#[test]
fn independent_color_math_and_quantization_preserve_floating_information() {
    let c = parse("#fea2").unwrap().unwrap();
    assert_eq!(c, rgb(255., 238., 170.).opacity(34. / 255.).into());
    assert_eq!(
        c.to_paint(),
        chart_core::scene::Color {
            red: 255,
            green: 238,
            blue: 170,
            alpha: 34
        }
    );
    assert_eq!(
        parse("rgb(10%,20%,30%)").unwrap().unwrap(),
        rgb(25.5, 51., 76.5).into()
    );
    let c: ColorValue = hsl(120., 0.5, 0.2).into();
    let r = c.rgb();
    for (a, e) in [(r.r, 25.5), (r.g, 76.5), (r.b, 25.5)] {
        assert!((a - e).abs() < 1e-12);
    }
    let parsed = parse("transparent").unwrap().unwrap();
    assert!(parsed.rgb().r.is_nan());
    assert_eq!(parsed.opacity(), 0.);
    let authored: ColorValue = rgb(10., 20., 30.).opacity(0.).into();
    assert_eq!(authored.rgb().r, 10.);
    assert_ne!(parsed, authored);
    assert_eq!(parse("not-a-color").unwrap(), None);
    let c: ColorValue = lab(50., 30., 40.).into();
    let cylindrical = c.hcl();
    assert_eq!(cylindrical.c, 50.);
    assert!((cylindrical.h - 53.13010235415598).abs() < 1e-12);
    assert_eq!(cylindrical.l, 50.);
    let roundtrip: ColorValue = cylindrical.into();
    let back = roundtrip.lab();
    assert!((back.a - 30.).abs() < 1e-12 && (back.b - 40.).abs() < 1e-12);
    let neutral: ColorValue = gray(50.).into();
    assert_eq!(neutral.lab().a, 0.);
    assert_eq!(neutral.lab().b, 0.);
    assert_eq!(neutral.hcl().c, 0.);
    assert!(neutral.hcl().h.is_nan());
    // This D50 primary anchor distinguishes the common but incorrect D65 conversion.
    let red: ColorValue = rgb(255., 0., 0.).into();
    let lab = red.lab();
    assert!((lab.l - 54.29173376861782).abs() < 1e-10);
    assert!((lab.a - 80.8124553179771).abs() < 1e-10);
    assert!((lab.b - 69.88504032350531).abs() < 1e-10);
    let original = rgb(70., 130., 180.);
    assert!(original.brighter(Some(2.)).b > 255.);
    assert_eq!(original.b, 180.);
    assert_eq!(original.brighter(Some(0.)), original);
    for x in [-0.5, 255.499_999] {
        assert!(rgb(x, 0., 0.).displayable());
    }
    for x in [-0.500_001, 255.5] {
        assert!(!rgb(x, 0., 0.).displayable());
    }
    assert_eq!(rgb(0.5, 1.5, 254.5).format_hex(), "#0102ff");
}
#[test]
fn color_descriptors_parser_budgets_and_channel_names_reject_malformed_inputs() {
    assert!(parse_with_limit("steelblue", 8).is_err());
    for css in [
        "rgb(1 2 3)",
        "rgb(1.5,2,3)",
        "rgba(1,2,3,1.)",
        "hsl(30,50 %,50%)",
        "\u{0085}red",
    ] {
        assert!(parse(css).unwrap().is_none(), "{css}");
    }
    let value: ColorValue = hsl(f64::NAN, 0., 0.5).opacity(f64::INFINITY).into();
    let descriptor = ColorDescriptor::new(value);
    let wire = descriptor.to_json().unwrap();
    assert!(wire.contains("NaN") && wire.contains("Infinity"));
    assert!(!wire.contains("null"));
    for bad in [
        wire.replace("\"version\":1", "\"version\":2"),
        wire.replace("NaN", "undefined"),
        wire.replace("{\"number\":\"NaN\"}", "null"),
        wire.replace("\"opacity\":", "\"extra\":1,\"opacity\":"),
    ] {
        assert!(ColorDescriptor::from_json(&bad).is_err(), "{bad}");
    }
    assert!(value.with_channel("r", 20.).is_err());
    assert_eq!(value.hsl().s, 0.);
}
