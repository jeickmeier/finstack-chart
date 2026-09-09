//! FIX-S05: independent pinned radial coordinates, all curves and links.
use chart_core::{
    DiagnosticCode,
    path::PathRequest,
    shape::{
        AreaRadial, Coordinate, CurveSpec, LineRadial, Link, LinkDatum, LinkRadial, ShapeLimits,
        point_radial,
    },
};
use serde_json::{Value, json};
fn near(actual: f64, expected: f64, id: &str) {
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs().max(1.),
        "{id}: {actual} != {expected}"
    );
}
fn compare(actual: &Value, expected: &Value, id: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => near(a.as_f64().unwrap(), b.as_f64().unwrap(), id),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{id}");
            for (a, b) in a.iter().zip(b) {
                compare(a, b, id);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{id}"
            );
            for (k, a) in a {
                compare(a, &b[k], id);
            }
        }
        _ => assert_eq!(actual, expected, "{id}"),
    }
}
#[test]
fn complete_radial_and_link_corpus_matches_reference() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/radial.json")).unwrap();
    for p in corpus["points"].as_array().unwrap() {
        compare(
            &json!(
                point_radial(p["angle"].as_f64().unwrap(), p["radius"].as_f64().unwrap()).unwrap()
            ),
            &p["point"],
            "pointRadial",
        );
    }
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let config = case["config"].clone();
        let result = match case["family"].as_str().unwrap() {
            "lineRadial" => serde_json::from_value::<LineRadial>(config)
                .unwrap()
                .generate(&serde_json::from_value::<Vec<Vec<f64>>>(case["input"].clone()).unwrap()),
            "areaRadial" => {
                let area = serde_json::from_value::<AreaRadial>(config).unwrap();
                let data: Vec<Vec<f64>> = serde_json::from_value(case["input"].clone()).unwrap();
                if case["helper"].is_null() {
                    area.generate(&data)
                } else {
                    area.boundary(serde_json::from_value(case["helper"].clone()).unwrap())
                        .generate(&data)
                }
            }
            "link" => serde_json::from_value::<Link>(config)
                .unwrap()
                .generate(&serde_json::from_value::<LinkDatum>(case["input"].clone()).unwrap()),
            "linkRadial" => serde_json::from_value::<LinkRadial>(config)
                .unwrap()
                .generate(&serde_json::from_value::<LinkDatum>(case["input"].clone()).unwrap()),
            _ => panic!("unhandled family"),
        };
        if case["finite"] == false {
            assert_eq!(
                result.unwrap_err().code,
                DiagnosticCode::NumericalDomain,
                "{id}"
            );
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let expected = PathRequest {
            version: 1,
            digits: Some(3.),
            limits: Default::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        }
        .build()
        .unwrap();
        compare(
            &serde_json::to_value(actual.geometry()).unwrap(),
            &serde_json::to_value(expected.geometry()).unwrap(),
            id,
        );
        let svg = actual.to_svg().unwrap();
        let reference_svg = case["svg"].as_str().unwrap_or("");
        if case["config"].get("digits") == Some(&Value::Null) && case["helper"].is_null() {
            // Unrounded transcendental results can differ by a platform ULP.
            // Apply the existing raw-coordinate contract to their decimal values;
            // rounded output remains byte-exact and topology was checked above.
            let values = |text: &str| -> Vec<f64> {
                text.split(|c: char| c == ',' || (c.is_ascii_alphabetic() && c != 'e' && c != 'E'))
                    .filter(|v| !v.is_empty())
                    .map(|v| v.parse().unwrap())
                    .collect()
            };
            compare(&json!(values(&svg)), &json!(values(reference_svg)), id);
        } else {
            assert_eq!(svg, reference_svg, "{id}");
        }
    }
    assert_eq!(corpus["points"].as_array().unwrap().len(), 40);
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 697);
}
#[test]
fn native_accessors_roundtrips_and_bounds_preserve_polar_contract() {
    let rows = [[0., 10.], [1., 30.], [2., 15.]];
    let line = LineRadial::new()
        .curve(CurveSpec::Natural)
        .unwrap()
        .defined(vec![true, false, true]);
    let decoded: LineRadial = serde_json::from_value(serde_json::to_value(&line).unwrap()).unwrap();
    assert_eq!(line, decoded);
    assert_eq!(
        line.generate(&rows).unwrap().geometry(),
        line.generate_by(&rows, |p, i, _| Ok((i != 1).then_some(*p)))
            .unwrap()
            .geometry()
    );
    let area = AreaRadial::new()
        .angle(Coordinate::Column(0))
        .radius(Coordinate::Column(1));
    let decoded: AreaRadial = serde_json::from_value(serde_json::to_value(&area).unwrap()).unwrap();
    assert_eq!(area, decoded);
    let radial = LinkRadial::new();
    assert_eq!(
        radial,
        serde_json::from_value(serde_json::to_value(&radial).unwrap()).unwrap()
    );
    assert!(serde_json::from_value::<LinkRadial>(json!({"curve":{"kind":"Linear"}})).is_err());
    assert!(
        serde_json::from_value::<AreaRadial>(json!({"angle":{"Constant":1},"end_angle":null}))
            .is_err()
    );
    assert!(
        AreaRadial::new()
            .curve(CurveSpec::Bundle { beta: 0.85 })
            .is_err()
    );
    assert!(point_radial(f64::NAN, 1.).is_err());
    assert!(point_radial(0., f64::INFINITY).is_err());
    assert!(LineRadial::new().generate(&[[f64::NAN, 1.]]).is_err());
    assert!(
        LineRadial::new()
            .defined(false)
            .generate(&[[f64::NAN, f64::INFINITY]])
            .unwrap()
            .geometry()
            .commands()
            .is_empty()
    );
    let calls = std::cell::Cell::new(0);
    assert_eq!(
        LinkRadial::new()
            .limits(ShapeLimits {
                max_points: 1,
                ..Default::default()
            })
            .generate_by(&(), |_| {
                calls.set(1);
                Ok([[0., 1.], [2., 3.]])
            })
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(calls.get(), 0);
    assert_eq!(
        LinkRadial::new()
            .generate_by(&(), |_| Ok([[0., f64::MAX], [1., f64::MAX]]))
            .unwrap_err()
            .code,
        DiagnosticCode::NumericalDomain
    );
    assert_eq!(
        Link::horizontal()
            .generate_by(&(), |_| Ok([[1., 2.], [9., 10.]]))
            .unwrap()
            .to_svg()
            .unwrap(),
        "M1,2C5,2,5,10,9,10"
    );
    assert_eq!(
        Link::vertical()
            .generate_by(&(), |_| Ok([[1., 2.], [9., 10.]]))
            .unwrap()
            .to_svg()
            .unwrap(),
        "M1,2C1,6,9,6,9,10"
    );
}
