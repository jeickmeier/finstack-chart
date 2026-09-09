//! FIX-S04: independent arc/pie geometry and layout expectations.
use chart_core::{
    path::PathRequest,
    shape::{Arc, ArcDatum, ArcParameters, Pie, PieAngles, PieOrder, ShapeLimits},
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

fn datum(v: &Value) -> ArcDatum {
    ArcDatum {
        inner_radius: v["innerRadius"].as_f64().unwrap(),
        outer_radius: v["outerRadius"].as_f64().unwrap(),
        start_angle: v["startAngle"].as_f64().unwrap(),
        end_angle: v["endAngle"].as_f64().unwrap(),
        pad_angle: v["padAngle"].as_f64().unwrap_or(0.),
    }
}
fn config(v: &Value) -> Arc {
    let mut arc = Arc::new();
    for (k, v) in v.as_object().unwrap() {
        let n = v.as_f64();
        arc = match k.as_str() {
            "innerRadius" => arc.inner_radius(n),
            "outerRadius" => arc.outer_radius(n),
            "startAngle" => arc.start_angle(n),
            "endAngle" => arc.end_angle(n),
            "padAngle" => arc.pad_angle(n),
            "cornerRadius" => arc.corner_radius(n.unwrap()),
            "padRadius" => arc.pad_radius(n),
            _ => panic!("unknown {k}"),
        };
    }
    arc
}
#[test]
fn full_arc_geometry_padding_corner_and_centroid_matrix() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/arc-pie.json")).unwrap();
    let cases = corpus["arcs"].as_array().unwrap();
    assert_eq!(cases.len(), 620);
    for c in cases {
        let id = c["id"].as_str().unwrap();
        let arc = config(&c["config"]);
        let d = datum(&c["input"]);
        if c["finite"] == false {
            assert!(arc.generate(d).is_err(), "{id}");
            continue;
        }
        let expected = PathRequest {
            version: 1,
            digits: Some(3.),
            limits: Default::default(),
            operations: serde_json::from_value(c["operations"].clone()).unwrap(),
        }
        .build()
        .unwrap();
        let actual = arc.generate(d).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        compare(
            &serde_json::to_value(actual.geometry()).unwrap(),
            &serde_json::to_value(expected.geometry()).unwrap(),
            id,
        );
        compare(&json!(arc.centroid(d).unwrap()), &c["centroid"], id);
        for digits in [0, 3] {
            assert_eq!(
                arc.clone()
                    .digits(Some(f64::from(digits)))
                    .unwrap()
                    .generate(d)
                    .unwrap()
                    .to_svg()
                    .unwrap(),
                c["svg"][digits.to_string()].as_str().unwrap(),
                "{id} digits={digits}"
            );
        }
        assert_eq!(actual.geometry(), arc.generate(d).unwrap().geometry());
    }
}
#[test]
fn full_pie_values_order_and_angle_matrix_retains_owned_data() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/arc-pie.json")).unwrap();
    let cases = corpus["pies"].as_array().unwrap();
    assert_eq!(cases.len(), 192);
    for c in cases {
        let id = c["id"].as_str().unwrap();
        let data = c["data"].as_array().unwrap();
        let pie = Pie::new()
            .start_angle(c["start"].as_f64().unwrap())
            .end_angle(c["end"].as_f64().unwrap())
            .pad_angle(c["pad"].as_f64().unwrap());
        let value = |d: &Value, _: usize, _: &[Value]| Ok(d["value"].as_f64().unwrap());
        let actual = match c["order"].as_str().unwrap() {
            "none" => pie.order(PieOrder::Input).layout_by(data, value),
            "valuesAscending" => pie.order(PieOrder::ValuesAscending).layout_by(data, value),
            "dataDescending" => pie.layout_by_comparator(data, value, |a, b| {
                b["label"]
                    .as_str()
                    .unwrap()
                    .cmp(a["label"].as_str().unwrap())
            }),
            _ => pie.layout_by(data, value),
        }
        .unwrap();
        let converted:Vec<_>=actual.iter().map(|a|json!({"data":a.data,"index":a.index,"value":a.value,"startAngle":a.start_angle,"endAngle":a.end_angle,"padAngle":a.pad_angle})).collect();
        compare(&json!(converted), &c["result"], id);
        for (a, d) in actual.iter().zip(data) {
            assert_eq!(&a.data, d);
        }
    }
}
#[test]
fn native_accessors_analytics_invalid_inputs_and_limits() {
    let d = ArcDatum {
        inner_radius: 0.,
        outer_radius: 10.,
        start_angle: 0.,
        end_angle: std::f64::consts::FRAC_PI_2,
        pad_angle: 0.,
    };
    let arc = Arc::new();
    let centroid = arc.centroid(d).unwrap();
    near(centroid[0], 5. / 2_f64.sqrt(), "centroid x");
    near(centroid[1], -5. / 2_f64.sqrt(), "centroid y");
    let native = arc
        .generate_by(&42, |v| {
            assert_eq!(*v, 42);
            Ok(ArcParameters {
                datum: d,
                corner_radius: 0.,
                pad_radius: None,
            })
        })
        .unwrap();
    assert_eq!(native.geometry(), arc.generate(d).unwrap().geometry());
    assert_eq!(
        arc.centroid_by(&d, |d| Ok(ArcParameters {
            datum: *d,
            corner_radius: 0.,
            pad_radius: None
        }))
        .unwrap(),
        centroid
    );
    assert!(Arc::new().outer_radius(Some(f64::NAN)).generate(d).is_err());
    assert!(Arc::new().digits(Some(-1.)).is_err());
    assert!(
        Arc::new()
            .limits(ShapeLimits {
                max_points: 0,
                ..Default::default()
            })
            .generate(d)
            .is_err()
    );
    assert!(Pie::new().layout(&[f64::INFINITY]).is_err());
    assert!(Pie::new().layout(&[f64::MAX, f64::MAX]).is_err());
    let p = Pie::new()
        .order(PieOrder::Input)
        .layout(&[1., 2., 1.])
        .unwrap();
    near(p[0].end_angle, std::f64::consts::FRAC_PI_2, "quarter");
    near(p[2].end_angle, std::f64::consts::TAU, "total");
    assert_eq!(p[0].arc_datum(0., 10.).end_angle, p[0].end_angle);
    let p = Pie::new().value(Some(2.)).layout(&[4., 7.]).unwrap();
    assert_eq!(p[0].value, 2.);
    assert_eq!(p[0].data, 4.);
    let p = Pie::new()
        .layout_by_value_comparator(
            &[4., 7.],
            |d, _, _| Ok(*d),
            |a, b| a.partial_cmp(&b).unwrap(),
        )
        .unwrap();
    assert_eq!(p[0].index, 0);
    let p = Pie::new()
        .layout_by_angles(
            &[1., 1.],
            |d, _, _| Ok(*d),
            |_| {
                Ok(PieAngles {
                    start_angle: 0.,
                    end_angle: 1.,
                    pad_angle: 0.,
                })
            },
        )
        .unwrap();
    assert_eq!(p[1].end_angle, 1.);
    let retained = Pie::new()
        .layout_materialized(&["a", "b"], &[2., 1.])
        .unwrap();
    assert_eq!(retained[0].data, "a");
    assert!(Pie::new().layout_materialized(&["a"], &[]).is_err());
    let mut called = false;
    assert!(
        Pie::new()
            .limits(ShapeLimits {
                max_points: 0,
                ..Default::default()
            })
            .layout_by(&[1.], |d, _, _| {
                called = true;
                Ok(*d)
            })
            .is_err()
    );
    assert!(!called);
    assert!(serde_json::from_value::<Arc>(json!({"wrong":2})).is_err());
    assert!(serde_json::from_value::<Pie>(json!({"wrong":2})).is_err());
}
