//! FIX-I01-E: all color routes/configurations, exact strings and independent anchors.
use chart_core::{color::*, interpolate::*};
use serde_json::Value;
fn color(v: &Value) -> ColorValue {
    match v["kind"].as_str().unwrap() {
        "Text" => parse(v["value"].as_str().unwrap())
            .unwrap()
            .unwrap_or_else(|| ColorValue::undefined(ColorSpace::Rgb)),
        "Color" => serde_json::from_value(v["value"].clone()).unwrap(),
        _ => panic!("color fixture"),
    }
}
fn num(v: &Value) -> f64 {
    match v.get("number").and_then(Value::as_str) {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        Some("-0") => -0.,
        _ => v.as_f64().unwrap(),
    }
}
#[test]
fn every_color_export_gamma_and_hue_matches_offline_reference() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-interpolate/cases.json"
    ))
    .unwrap();
    let mut failures = vec![];
    let mut count = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let op = c["op"].as_str().unwrap();
        let id = c["id"].as_str().unwrap();
        if op == "interpolateHue" {
            count += 1;
            let f = HueInterpolator::new(num(&c["args"][0]["value"]), num(&c["args"][1]["value"]));
            for (t, e) in c["times"]
                .as_array()
                .unwrap()
                .iter()
                .zip(c["expected"].as_array().unwrap())
            {
                let a = f.sample(t.as_f64().unwrap()).unwrap();
                let e = num(&e["value"]);
                assert!(
                    if e.is_nan() {
                        a.is_nan()
                    } else if e.is_infinite() {
                        a == e
                    } else {
                        (a - e).abs() <= 1e-12 + 1e-12 * e.abs()
                    },
                    "{id}: {a} != {e}"
                );
            }
            continue;
        }
        let route = match op {
            "interpolateRgb" => Some(ColorRoute::Rgb),
            "interpolateHsl" => Some(ColorRoute::Hsl),
            "interpolateHslLong" => Some(ColorRoute::HslLong),
            "interpolateLab" => Some(ColorRoute::Lab),
            "interpolateHcl" => Some(ColorRoute::Hcl),
            "interpolateHclLong" => Some(ColorRoute::HclLong),
            "interpolateCubehelix" => Some(ColorRoute::Cubehelix),
            "interpolateCubehelixLong" => Some(ColorRoute::CubehelixLong),
            _ => None,
        };
        let f = if let Some(route) = route {
            ColorInterpolator::new(
                route,
                color(&c["args"][0]),
                color(&c["args"][1]),
                c["config"].get("gamma").map(num),
            )
        } else if ["interpolateRgbBasis", "interpolateRgbBasisClosed"].contains(&op) {
            ColorInterpolator::rgb_basis(
                &c["args"][0]["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(color)
                    .collect::<Vec<_>>(),
                op.ends_with("Closed"),
            )
        } else {
            continue;
        };
        count += 1;
        if !c["adaptation"].is_null() {
            assert!(f.is_err(), "{id}");
            continue;
        }
        let f = f.unwrap();
        for (t, e) in c["times"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["expected"].as_array().unwrap())
        {
            let a = f.sample(t.as_f64().unwrap()).unwrap();
            let e = e["value"].as_str().unwrap();
            if a != e {
                failures.push(format!("{id}@{t}: {a} != {e}"));
            }
        }
    }
    assert_eq!(count, 106);
    assert!(
        failures.is_empty(),
        "{}\n{}",
        failures.len(),
        failures.join("\n")
    );
}
#[test]
fn floating_channels_opacity_hue_and_splines_keep_their_contracts() {
    let a: ColorValue = rgb(0., 0., 0.).opacity(0.25).into();
    let b: ColorValue = rgb(255., 255., 255.).opacity(0.75).into();
    let f = ColorInterpolator::new(ColorRoute::Rgb, a, b, None).unwrap();
    let mid = f.sample_color(0.5).unwrap().rgb();
    assert_eq!(mid, rgb(127.5, 127.5, 127.5).opacity(0.5));
    assert_eq!(f.sample(0.5).unwrap(), "rgba(128, 128, 128, 0.5)");
    let gamma = ColorInterpolator::new(ColorRoute::Rgb, a, b, Some(2.)).unwrap();
    assert!((gamma.sample_color(0.5).unwrap().rgb().r - 255. / 2_f64.sqrt()).abs() < 1e-12);
    let a: ColorValue = hsl(350., 1., 0.5).into();
    let b: ColorValue = hsl(10., 1., 0.5).into();
    assert_eq!(
        ColorInterpolator::new(ColorRoute::Hsl, a, b, None)
            .unwrap()
            .sample(0.5)
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        ColorInterpolator::new(ColorRoute::HslLong, a, b, None)
            .unwrap()
            .sample(0.5)
            .unwrap(),
        "rgb(0, 255, 255)"
    );
    assert_eq!(HueInterpolator::new(350., 10.).sample(0.5).unwrap(), 0.);
    assert_eq!(HueInterpolator::new(0., 180.).sample(0.5).unwrap(), 90.);
    assert_eq!(HueInterpolator::new(180., 0.).sample(0.5).unwrap(), 90.);
    let spline = ColorInterpolator::rgb_basis(
        &[
            rgb(0., 0., 0.).opacity(0.).into(),
            rgb(255., 255., 255.).opacity(0.2).into(),
        ],
        false,
    )
    .unwrap();
    assert_eq!(spline.sample_color(0.5).unwrap().rgb().opacity, 1.);
    let copy = spline.clone();
    let held = spline.sample_color(0.25).unwrap();
    spline.sample(0.75).unwrap();
    assert_eq!(held, copy.sample_color(0.25).unwrap());
    for gamma in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(ColorInterpolator::new(ColorRoute::Rgb, a, b, Some(gamma)).is_err());
    }
    assert!(ColorInterpolator::new(ColorRoute::Lab, a, b, Some(1.)).is_err());
}
