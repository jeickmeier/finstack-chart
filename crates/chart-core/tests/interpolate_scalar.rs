//! FIX-I01-B/D: unchanged offline oracle plus independent scalar/lifetime anchors.
use chart_core::{ChartResult, DiagnosticCode, interpolate::*};
use serde_json::Value;

fn number(v: &Value) -> f64 {
    match v.get("number").and_then(Value::as_str) {
        Some("NaN") => f64::NAN,
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        Some("-0") => -0.,
        _ => v.as_f64().unwrap(),
    }
}
fn compare(actual: f64, expected: &Value, id: &str) {
    let expected = number(&expected["value"]);
    if expected.is_nan() {
        assert!(actual.is_nan(), "{id}: {actual}");
    } else if expected.is_infinite() {
        assert_eq!(actual, expected, "{id}");
    } else if expected == 0. {
        assert_eq!(actual.to_bits(), expected.to_bits(), "{id}: signed zero");
    } else {
        assert!(
            (actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs(),
            "{id}: {actual} != {expected}"
        );
    }
}
#[test]
fn every_scalar_and_composition_case_matches_the_offline_oracle() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-interpolate/cases.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let op = case["op"].as_str().unwrap();
        let id = case["id"].as_str().unwrap();
        if op == "piecewise" && case["config"]["factory"].is_null() {
            continue;
        }
        if ![
            "interpolateNumber",
            "interpolateRound",
            "interpolateBasis",
            "interpolateBasisClosed",
            "interpolateDiscrete",
            "piecewise",
            "quantize",
        ]
        .contains(&op)
        {
            continue;
        }
        count += 1;
        let args = case["args"].as_array().unwrap();
        if op == "quantize" {
            let f =
                ScalarInterpolator::number(number(&args[0]["value"]), number(&args[1]["value"]));
            let values = quantize(&f, case["config"]["count"].as_u64().unwrap() as usize);
            if !case["adaptation"].is_null() {
                assert!(values.is_err(), "{id}");
                continue;
            }
            for (a, e) in values
                .unwrap()
                .into_iter()
                .zip(case["expected"]["value"].as_array().unwrap())
            {
                compare(a, e, id);
            }
            continue;
        }
        let controls = || {
            args[0]["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| number(&v["value"]))
                .collect::<Vec<_>>()
        };
        type Factory = Box<dyn Fn(f64) -> ChartResult<f64>>;
        let f: ChartResult<Factory> = match op {
            "interpolateNumber" | "interpolateRound" => {
                let a = number(&args[0]["value"]);
                let b = number(&args[1]["value"]);
                let f = if op == "interpolateRound" {
                    ScalarInterpolator::round(a, b)
                } else {
                    ScalarInterpolator::number(a, b)
                };
                Ok(Box::new(move |t| f.sample(t)))
            }
            "interpolateBasis" => ScalarInterpolator::basis(controls())
                .map(|f| Box::new(move |t| f.sample(t)) as Factory),
            "interpolateBasisClosed" => ScalarInterpolator::basis_closed(controls())
                .map(|f| Box::new(move |t| f.sample(t)) as Factory),
            "interpolateDiscrete" => {
                Discrete::new(controls()).map(|f| Box::new(move |t| f.sample(t)) as Factory)
            }
            "piecewise" => {
                let round = case["config"]["factory"] == "interpolateRound";
                Piecewise::new(&controls(), |a, b| {
                    Ok(if round {
                        ScalarInterpolator::round(*a, *b)
                    } else {
                        ScalarInterpolator::number(*a, *b)
                    })
                })
                .map(|f| Box::new(move |t| f.sample(t)) as Factory)
            }
            _ => unreachable!(),
        };
        if !case["adaptation"].is_null() {
            assert!(f.is_err(), "{id}");
            continue;
        }
        let f = f.unwrap();
        for (t, e) in case["times"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["expected"].as_array().unwrap())
        {
            compare(f(t.as_f64().unwrap()).unwrap(), e, id);
        }
    }
    assert_eq!(count, 56);
}

#[test]
fn independent_rounding_composition_and_owned_sampling_contracts() {
    let f = ScalarInterpolator::round(-1., 0.);
    assert_eq!(f.sample(0.5).unwrap().to_bits(), (-0_f64).to_bits());
    assert_eq!(
        ScalarInterpolator::round(-3., -2.).sample(0.5).unwrap(),
        -2.
    );
    assert_eq!(
        ScalarInterpolator::number(2., 10.).sample(1.25).unwrap(),
        12.
    );
    let linear = ScalarInterpolator::basis(vec![2., 10.]).unwrap();
    assert_eq!(linear.sample(-2.).unwrap(), 2.);
    assert_eq!(linear.sample(4.).unwrap(), 10.);
    assert_eq!(linear.sample(0.5).unwrap(), 6.);
    let closed = ScalarInterpolator::basis_closed(vec![0., 6., -3., 9.]).unwrap();
    for t in [0.125, 0.5, 0.875] {
        assert_eq!(closed.sample(t).unwrap(), closed.sample(t - 3.).unwrap());
    }
    let h = 1e-4;
    let left = (closed.sample(0.).unwrap() - closed.sample(-h).unwrap()) / h;
    let right = (closed.sample(h).unwrap() - closed.sample(0.).unwrap()) / h;
    assert!((left - right).abs() < 0.03, "C1 seam");
    let left2 = (closed.sample(-2. * h).unwrap() - 2. * closed.sample(-h).unwrap()
        + closed.sample(0.).unwrap())
        / (h * h);
    let right2 = (closed.sample(0.).unwrap() - 2. * closed.sample(h).unwrap()
        + closed.sample(2. * h).unwrap())
        / (h * h);
    assert!((left2 - right2).abs() < 0.5, "C2 seam");
    let calls = std::cell::Cell::new(0);
    let piece = Piecewise::new(&[0., 10., 0.], |a, b| {
        calls.set(calls.get() + 1);
        Ok(ScalarInterpolator::number(*a, *b))
    })
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(quantize(&piece, 5).unwrap(), vec![0., 5., 10., 5., 0.]);
    assert_eq!(piece.sample(-0.5).unwrap(), -10.);
    assert_eq!(piece.sample(1.5).unwrap(), -10.);
    assert_eq!(calls.get(), 2);
    let discrete = Discrete::new(vec![vec![1], vec![2]]).unwrap();
    let mut sampled = quantize(&discrete, 3).unwrap();
    sampled[1][0] = 99;
    assert_eq!(sampled[2], vec![2]);
    assert_eq!(discrete.sample(0.5).unwrap(), vec![2]);
    assert_eq!(discrete.sample(-1.).unwrap(), vec![1]);
    assert_eq!(discrete.sample(4.).unwrap(), vec![2]);
}

#[test]
fn parameter_controls_and_output_counts_are_bounded() {
    assert!(ScalarInterpolator::basis(vec![]).is_err());
    assert!(ScalarInterpolator::basis(vec![1.]).is_err());
    assert!(ScalarInterpolator::basis_closed(vec![]).is_err());
    assert!(Discrete::<u8>::new(vec![]).is_err());
    assert_eq!(
        ScalarInterpolator::basis(vec![0.; MAX_VALUES + 1])
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let f = ScalarInterpolator::number(0., 1.);
    assert!(f.sample(f64::NAN).is_err());
    assert!(f.sample(f64::INFINITY).is_err());
    for n in [0, 1, MAX_VALUES + 1] {
        assert!(quantize(&f, n).is_err());
    }
    assert_eq!(quantize(&f, 2).unwrap(), vec![0., 1.]);
}
