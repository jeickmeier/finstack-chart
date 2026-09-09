//! SP-07/FIX-20: every pinned standalone family and method through typed operation transport.
use chart_core::{
    interpolate::Number,
    portable::{self, ScaleChange, ScaleQuery},
    scales::{ScaleConstructor, ScaleOptions},
};
use serde_json::Value as Json;

fn equivalent(a: &Json, e: &Json, exact: bool) -> bool {
    if a.is_number() && e.is_number() {
        let (a, e) = (a.as_f64().unwrap(), e.as_f64().unwrap());
        return a == e || !exact && (a - e).abs() <= 1e-12 + 1e-12 * e.abs();
    }
    match (a, e) {
        (Json::Array(a), Json::Array(e)) => {
            a.len() == e.len() && a.iter().zip(e).all(|(a, e)| equivalent(a, e, exact))
        }
        (Json::Object(a), Json::Object(e)) => {
            a.len() == e.len()
                && a.iter()
                    .all(|(k, a)| e.get(k).is_some_and(|e| equivalent(a, e, exact)))
        }
        _ => a == e,
    }
}
#[test]
fn every_pinned_operation_and_declared_adaptation_is_observable() {
    let corpus: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/operations.json"
    ))
    .unwrap();
    let mut failures = vec![];
    let (mut cases, mut operations, mut adaptations, mut diagnostics) = (0, 0, 0, 0);
    for c in corpus["cases"].as_array().unwrap() {
        cases += 1;
        let family: ScaleConstructor = serde_json::from_value(c["family"].clone()).unwrap();
        let options: ScaleOptions = portable::decode(&c["options"].to_string()).unwrap();
        let mut scale = match family.create(options) {
            Ok(s) if c.get("setup_error").is_none() => s,
            Err(_) if c.get("setup_error").is_some() => {
                diagnostics += 1;
                continue;
            }
            result => {
                failures.push(format!("{} setup {result:?}", c["id"]));
                continue;
            }
        };
        let restored =
            chart_core::scales::StandaloneScale::from_json(&scale.to_json().unwrap()).unwrap();
        assert_eq!(restored.to_json().unwrap(), scale.to_json().unwrap());
        for o in c["operations"].as_array().unwrap() {
            operations += 1;
            if o.get("adaptation").is_some() {
                adaptations += 1;
            }
            let original = scale.to_json().unwrap();
            let run=|| -> chart_core::ChartResult<(String,Option<chart_core::scales::StandaloneScale>)> {
                let changed=o.get("change").map(|v|portable::decode::<ScaleChange>(&v.to_string()).and_then(|change|change.apply(&scale))).transpose()?;
                let selected=changed.as_ref().unwrap_or(&scale);
                let result=portable::decode::<ScaleQuery>(&o["query"].to_string())?.execute(selected)?;
                Ok((result,changed))
            };
            match run() {
                Ok((actual, changed)) => {
                    let json: Json = serde_json::from_str(&actual).unwrap();
                    let actual = o["select"]
                        .as_str()
                        .and_then(|p| json.pointer(p))
                        .unwrap_or(&json);
                    if o.get("diagnostic").is_some()
                        || !equivalent(actual, &o["expected"], o["exact"] == true)
                    {
                        failures.push(format!(
                            "{} {}: {actual} expected {}",
                            c["id"],
                            o["name"],
                            o.get("expected").unwrap_or(&o["diagnostic"])
                        ));
                    }
                    if o["persist"] == true {
                        scale = changed.expect("persisted change");
                    }
                }
                Err(e) => {
                    let code = serde_json::to_value(e.code).unwrap();
                    let allowed = o
                        .get("diagnostic")
                        .or_else(|| o.get("allowed_diagnostic"))
                        .and_then(Json::as_array);
                    if !allowed.is_some_and(|v| v.contains(&code)) {
                        failures.push(format!(
                            "{} {}: {e}; expected {}",
                            c["id"], o["name"], o["expected"]
                        ));
                    } else {
                        diagnostics += 1;
                    }
                }
            }
            if o["persist"] != true {
                assert_eq!(
                    scale.to_json().unwrap(),
                    original,
                    "failed or temporary change must preserve source"
                );
            }
        }
    }
    assert_eq!(cases, 661);
    assert_eq!(operations, 19562);
    assert_eq!(adaptations, 1112);
    println!(
        "FIX-20 cases={cases} operations={operations} adaptations={adaptations} diagnostics={diagnostics}"
    );
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.into_iter().take(80).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn standalone_named_options_are_atomic_and_capabilities_are_checked() {
    use chart_core::{
        DiagnosticCode,
        interpolate::Value,
        scales::{ScaleInput, ScaleKey},
    };
    let numeric = ScaleConstructor::Linear
        .create(ScaleOptions::default())
        .unwrap();
    assert_eq!(
        numeric.map(ScaleInput::Number(Number(0.25))).unwrap(),
        Value::number(0.25)
    );
    assert!(
        ScaleOptions {
            padding: Some(0.5),
            ..Default::default()
        }
        .apply(&numeric)
        .is_err()
    );
    assert_eq!(
        numeric.range().unwrap(),
        vec![Value::number(0.), Value::number(1.)]
    );
    let categorical = ScaleConstructor::Ordinal
        .create(ScaleOptions {
            range: Some(vec![Value::Text("a".into()), Value::Text("b".into())]),
            ..Default::default()
        })
        .unwrap();
    let key = ScaleKey::Unsigned(u64::MAX);
    assert_eq!(
        categorical.map(ScaleInput::Key(key.clone())).unwrap(),
        Value::Missing
    );
    let trained = categorical.train(vec![key.clone()]).unwrap();
    assert_eq!(
        trained.map(ScaleInput::Key(key)).unwrap(),
        Value::Text("a".into())
    );
    assert!(categorical.domain().is_empty());
    assert_eq!(
        trained.invert(1.).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    let threshold = ScaleConstructor::Threshold
        .create(ScaleOptions {
            unknown: Some(Value::Text("missing".into())),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        threshold
            .map(ScaleInput::Key(ScaleKey::Number(Number(f64::NAN))))
            .unwrap(),
        Value::Text("missing".into())
    );
    let ns = ScaleConstructor::Utc
        .create(ScaleOptions {
            unit: Some(chart_core::data::TimeUnit::Nanoseconds),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(ns.domain()[0], ScaleInput::Time(946_684_800_000_000_000));
    let mut wire: Json = serde_json::from_str(&numeric.to_json().unwrap()).unwrap();
    wire["version"] = 2.into();
    assert!(chart_core::scales::StandaloneScale::from_json(&wire.to_string()).is_err());
    assert!(portable::decode::<ScaleQuery>(r#"{"Invert":null}"#).is_err());
}
