//! SP-01 measurement only: observe current APIs against the pinned reference catalog.
//! A successful runner does not mean parity; inspect the explicit missing/failing rows.
use chart_core::scales::{Bounds, ContinuousDomain, LinearScale, OutsidePolicy};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

fn bounds(v: &Value) -> Option<Bounds> {
    let values = v.as_array()?;
    if values.len() != 2 {
        return None;
    }
    Bounds::new(values[0].as_f64()?, values[1].as_f64()?).ok()
}
fn compare(id: &str, operation: &str, expected: &Value, actual: Value) -> Value {
    fn equal(a: &Value, b: &Value) -> bool {
        if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
            (a - b).abs() <= 1e-12 + 1e-12 * a.abs().max(b.abs())
        } else if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
        } else {
            a == b
        }
    }
    json!({"id":id,"operation":operation,"expected":expected,"actual":actual,"status":if equal(expected,&actual){"Pass"}else{"Fail"}})
}
fn main() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json"))
            .expect("valid pinned fixture");
    let inventory: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/inventory.json"
    ))
    .expect("valid pinned inventory");
    let mut observations = Vec::new();
    let mut methods = Vec::new();
    for factory in inventory["factories"].as_array().expect("factories") {
        for method in ["constructor", "map"].into_iter().chain(
            factory["methods"]
                .as_array()
                .expect("methods")
                .iter()
                .map(|v| v.as_str().expect("method")),
        ) {
            let partial = factory["factory"] == "scaleLinear"
                && matches!(
                    method,
                    "map" | "invert" | "ticks" | "domain" | "range" | "copy"
                );
            methods.push(json!({"factory":factory["factory"],"method":method,"disposition":if matches!(method,"constructor"|"copy"|"domain"|"range"|"rangeRound"|"clamp"|"unknown"|"base"|"exponent"|"constant"|"padding"|"paddingInner"|"paddingOuter"|"align"|"round"|"interpolate"|"interpolator"){"Typed immutable configuration; equivalent evaluated results"}else{"Equivalent operation on typed inputs; undefined reference results are tagged or diagnosed"},"owner":if matches!(method,"ticks"|"tickFormat"|"nice") && factory["owner"]!="SP-06"{json!("SP-05")}else{factory["owner"].clone()},"status":if partial{"Partial"}else{"Missing"},"reason":if partial{"Existing two-endpoint API is measurable; full method contract is unqualified."}else{"No complete D3-compatible standalone method has been implemented and qualified."}}));
        }
    }
    for case in corpus["cases"].as_array().expect("cases") {
        let id = case["id"].as_str().expect("id");
        if case["setup"].get("error").is_some() {
            observations.push(json!({"id":id,"status":"ReferenceDiagnostic","expected":case["setup"],"rust":"Unqualified"}));
            continue;
        }
        let config = &case["config"];
        let domain = bounds(&case["getters"]["domain"]["value"]);
        let range = bounds(&case["getters"]["range"]["value"]);
        let scale = if case["factory"] == "scaleLinear" && config.get("rangeRound").is_none() {
            domain.zip(range).and_then(|(d, r)| {
                LinearScale::resolve(
                    None,
                    ContinuousDomain::explicit(d),
                    r,
                    None,
                    if config["clamp"][0] == true {
                        OutsidePolicy::Clamp
                    } else {
                        OutsidePolicy::Extend
                    },
                )
                .ok()
            })
        } else {
            None
        };
        let Some(scale) = scale else {
            observations.push(json!({"id":id,"status":"Missing","owner":case["owner"],"reason":"Family, knot cardinality, degenerate endpoints or output policy lacks a qualified compatible API."}));
            continue;
        };
        for (index, x) in case["inputs"]
            .as_array()
            .expect("inputs")
            .iter()
            .enumerate()
        {
            if let Some(x) = x.as_f64() {
                let actual = match scale.map(x) {
                    Ok(Some(value)) => json!(value),
                    Ok(None) => json!({"kind":"Undefined"}),
                    Err(e) => json!({"diagnostic":e.to_string()}),
                };
                observations.push(compare(
                    id,
                    &format!("map[{index}]"),
                    &case["output"][index]["value"],
                    actual,
                ));
            } else {
                observations.push(json!({"id":id,"operation":format!("map[{index}]"),"status":"Missing","reason":"Typed missing/exceptional input contract is not present in the existing finite coordinate API."}));
            }
        }
        for row in case["queries"]["invert"].as_array().expect("inverse") {
            if let Some(x) = row["input"].as_f64() {
                let actual = match scale.invert(x) {
                    Ok(v) => json!(v),
                    Err(e) => json!({"diagnostic":e.to_string()}),
                };
                observations.push(compare(id, &format!("invert({x})"), &row["value"], actual));
            }
        }
        for row in case["queries"]["ticks"].as_array().expect("ticks") {
            if let Some(n) = row["count"].as_u64() {
                let actual = match scale.ticks(n as usize, 2000) {
                    Ok(v) => json!(v.iter().map(|v| v.value).collect::<Vec<_>>()),
                    Err(e) => json!({"diagnostic":e.to_string()}),
                };
                observations.push(compare(id, &format!("ticks({n})"), &row["value"], actual));
            } else {
                observations.push(json!({"id":id,"operation":"ticks","count":row["count"],"status":"Missing","reason":"Existing count uses usize, excluding fractional count hints."}));
            }
        }
    }
    for (index, case) in corpus["format_cases"]
        .as_array()
        .expect("format cases")
        .iter()
        .enumerate()
    {
        observations.push(json!({"id":format!("tickFormat-{index}"),"status":"Missing","owner":"SP-05","input":case,"reason":"Complete scale specifiers and precision inference are not implemented."}));
    }
    let local: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/d3-scale/local-time.json"
    ))
    .expect("local time corpus");
    for zone in local["zones"].as_array().expect("zones") {
        for case in zone["records"].as_array().expect("records") {
            observations.push(json!({"id":case["id"],"zone":zone["zone"],"status":"Missing","owner":"SP-06","reason":"Supplied local calendar scale, intervals and formatting are not implemented."}));
        }
    }
    methods
        .push(json!({"factory":"tickFormat","method":"call","owner":"SP-05","status":"Missing"}));
    methods.push(
        json!({"factory":"scaleImplicit","method":"policy","owner":"SP-03","status":"Missing"}),
    );
    let counts = observations.iter().fold(BTreeMap::new(), |mut acc, v| {
        *acc.entry(v["status"].as_str().expect("status"))
            .or_insert(0usize) += 1;
        acc
    });
    let report = json!({"schema_version":1,"stage":"SP-01 measurement only","gate":"G-SCALE OPEN","factories":inventory["factories"].as_array().expect("factories").len(),"reference_cases":corpus["cases"].as_array().expect("cases").len(),"format_cases":corpus["format_cases"].as_array().expect("format cases").len(),"format_status":"Missing SP-05","method_dispositions":methods,"counts":counts,"observations":observations});
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/scale-gap-report.json"));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&report).expect("report JSON"),
    )
    .expect("write report");
    println!(
        "SP-01 measured current API; counts: {}. G-SCALE remains OPEN.",
        report["counts"]
    );
}
