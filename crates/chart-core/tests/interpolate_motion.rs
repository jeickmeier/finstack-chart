//! FIX-I01-F/G: actual browser transforms and independent camera/geometry anchors.
use chart_core::{interpolate::*, path::Affine};
use serde_json::Value;
fn close(a: f64, e: f64, abs: f64, rel: f64, id: &str) {
    assert!((a - e).abs() <= abs + rel * e.abs(), "{id}: {a} != {e}");
}
fn matrix(v: &Value) -> Affine {
    Affine::new(std::array::from_fn(|i| v[i].as_f64().unwrap())).unwrap()
}
fn canonical(a: &str, e: &str, id: &str) {
    let split = |s: &str| {
        s.split(')')
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let (name, values) = s.trim().split_once('(').unwrap();
                (
                    name.to_owned(),
                    values
                        .split(',')
                        .map(|v| {
                            let v = v.trim();
                            let unit = if v.ends_with("px") {
                                "px"
                            } else if v.ends_with("deg") {
                                "deg"
                            } else {
                                ""
                            };
                            (
                                v.trim_end_matches(unit).parse::<f64>().unwrap(),
                                unit.to_owned(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    let a = split(a);
    let e = split(e);
    assert_eq!(a.len(), e.len(), "{id}: template {a:?} != {e:?}");
    for ((an, av), (en, ev)) in a.iter().zip(e) {
        assert_eq!(an, &en, "{id}");
        assert_eq!(av.len(), ev.len(), "{id}");
        for ((a, au), (e, eu)) in av.iter().zip(ev) {
            assert_eq!(au, &eu, "{id}");
            close(*a, e, 1e-12, 1e-12, id);
        }
    }
}
#[test]
fn pinned_real_browser_transforms_match_decomposition_and_geometry() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-interpolate/transforms.json"
    ))
    .unwrap();
    for c in fixture["cases"].as_array().unwrap() {
        let id = c["id"].as_str().unwrap();
        let syntax = if c["syntax"] == "css" {
            TransformSyntax::Css
        } else {
            TransformSyntax::Svg
        };
        let a = parse_transform(c["a"].as_str().unwrap(), syntax);
        let b = parse_transform(c["b"].as_str().unwrap(), syntax);
        if !c["adaptation"].is_null() {
            assert!(a.is_err() || b.is_err(), "{id}");
            continue;
        }
        let a = a.unwrap_or_else(|e| panic!("{id}: {e}"));
        let b = b.unwrap_or_else(|e| panic!("{id}: {e}"));
        for (actual, expected) in [a, b].into_iter().zip(c["matrices"].as_array().unwrap()) {
            for (a, e) in actual
                .coefficients()
                .into_iter()
                .zip(matrix(expected).coefficients())
            {
                close(a, e, 1e-5, 1e-6, &format!("{id}/browser parser resolution"));
            }
        }
        // Resolve from exact browser coefficients to isolate decomposition from parser resolution.
        let f = TransformInterpolator::new(
            matrix(&c["matrices"][0]),
            matrix(&c["matrices"][1]),
            syntax,
        )
        .unwrap();
        let headless = TransformInterpolator::new(a, b, syntax).unwrap();
        for (t, e) in c["times"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["expected"].as_array().unwrap())
        {
            let t = t.as_f64().unwrap();
            canonical(&f.sample(t).unwrap(), e["text"].as_str().unwrap(), id);
            for actual in [f.matrix(t).unwrap(), headless.matrix(t).unwrap()] {
                for (a, e) in actual
                    .coefficients()
                    .into_iter()
                    .zip(matrix(&e["matrix"]).coefficients())
                {
                    close(a, e, 1e-5, 1e-6, id);
                }
                for (a, e) in actual
                    .point([2., 3.])
                    .unwrap()
                    .into_iter()
                    .zip(e["point"].as_array().unwrap())
                {
                    close(a, e.as_f64().unwrap(), 1e-5, 1e-6, id);
                }
            }
        }
    }
}
#[test]
fn all_zoom_rho_duration_cases_match_reference() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-interpolate/cases.json"
    ))
    .unwrap();
    let mut count = 0;
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["op"] == "interpolateZoom")
    {
        count += 1;
        let id = c["id"].as_str().unwrap();
        let view = |v: &Value| {
            ZoomView::new(std::array::from_fn(|i| {
                v["value"][i]["value"].as_f64().unwrap()
            }))
        };
        let a = view(&c["args"][0]);
        let b = view(&c["args"][1]);
        if c["adaptation"] == "RejectZoomView" {
            assert!(a.is_err() || b.is_err(), "{id}");
            continue;
        }
        let f = if let Some(rho) = c["config"].get("rho") {
            let rho: Number = serde_json::from_value(rho.clone()).unwrap();
            ZoomInterpolator::with_rho(a.unwrap(), b.unwrap(), rho.0)
        } else {
            ZoomInterpolator::new(a.unwrap(), b.unwrap())
        };
        if !c["adaptation"].is_null() {
            assert!(f.is_err(), "{id}");
            continue;
        }
        let f = f.unwrap_or_else(|e| panic!("{id}: {e}"));
        close(
            f.duration_ms(),
            c["duration"].as_f64().unwrap(),
            1e-12,
            1e-12,
            id,
        );
        assert_eq!(f.scheduling_duration_ms(), f.duration_ms().abs());
        for (t, e) in c["times"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["expected"].as_array().unwrap())
        {
            for (a, e) in f
                .sample(t.as_f64().unwrap())
                .unwrap()
                .values()
                .into_iter()
                .zip(e["value"].as_array().unwrap())
            {
                close(a, e["value"].as_f64().unwrap(), 1e-12, 1e-12, id);
            }
        }
    }
    assert_eq!(count, 17);
}
#[test]
fn independent_transform_order_reflection_and_zoom_invariants() {
    let svg = TransformSyntax::Svg;
    let css = TransformSyntax::Css;
    assert_eq!(
        parse_transform("translate(10 20) scale(2)", svg)
            .unwrap()
            .point([1., 1.])
            .unwrap(),
        [12., 22.]
    );
    assert_eq!(
        parse_transform("scale(2) translate(10 20)", svg)
            .unwrap()
            .point([1., 1.])
            .unwrap(),
        [22., 42.]
    );
    let centered = parse_transform("rotate(90,10,20)", svg).unwrap();
    assert_eq!(centered.point([10., 20.]).unwrap(), [10., 20.]);
    assert_eq!(centered.point([11., 20.]).unwrap(), [10., 21.]);
    let reflection = decompose(Affine::new([-2., 0., 0., 3., 4., 5.]).unwrap()).unwrap();
    assert_eq!(reflection.scale, [-2., 3.]);
    assert_eq!(
        reflection.matrix().unwrap().coefficients(),
        [-2., 0., 0., 3., 4., 5.]
    );
    let f = TransformInterpolator::from_text("rotate(350deg)", "rotate(10deg)", css).unwrap();
    let m = f.matrix(0.5).unwrap().coefficients();
    close(m[0], 1., 1e-12, 0., "short rotation");
    close(m[1], 0., 1e-12, 0., "short rotation");
    for text in [
        "translate(1em)",
        "translate(50%)",
        "rotateX(10deg)",
        "matrix(1,2,3)",
        "translate(1px,,2px)",
        "rotate()",
        "rotate(1e400deg)",
        "calc(2px)",
    ] {
        assert!(parse_transform(text, css).is_err(), "{text}");
    }
    for text in ["translate(1px)", "rotate(1,2)", "skewx(2)", "translate(1,)"] {
        assert!(parse_transform(text, svg).is_err(), "{text}");
    }
    assert!(parse_transform(&"scale(1) ".repeat(1025), svg).is_err());
    let a = ZoomView::new([2., 3., 16.]).unwrap();
    let b = ZoomView::new([2., 3., 1.]).unwrap();
    let f = ZoomInterpolator::new(a, b).unwrap();
    assert!(f.duration_ms() < 0.);
    close(
        f.sample(0.5).unwrap().values()[2],
        4.,
        1e-12,
        0.,
        "geometric width",
    );
    assert_eq!(f.sample(0.).unwrap(), a);
    close(f.sample(1.).unwrap().values()[2], 1., 1e-12, 0., "endpoint");
    let reverse = ZoomInterpolator::new(b, a).unwrap();
    close(
        f.duration_ms(),
        -reverse.duration_ms(),
        1e-12,
        1e-12,
        "reverse duration",
    );
    assert_eq!(ZoomInterpolator::with_rho(a, b, -5.).unwrap().rho(), 1e-3);
    assert!(ZoomView::new([0., 0., 0.]).is_err());
    assert!(ZoomView::new([f64::NAN, 0., 1.]).is_err());
    assert!(ZoomInterpolator::with_rho(a, b, f64::NAN).is_err());
    assert!(f.sample(f64::INFINITY).is_err());
}
