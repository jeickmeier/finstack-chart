//! Replay independent action/input expectations through primary authoring and runtime methods.
use crate::authors;
use chart_core::{composition::ConnectorOrigin, prelude::*};
use chart_export::{
    Output,
    host::{Options, Runtime},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn subset(actual: &Value, expected: &Value) {
    match expected {
        Value::Object(v) => {
            for (k, v) in v {
                subset(&actual[k], v);
            }
        }
        Value::Array(v) => {
            assert_eq!(actual.as_array().unwrap().len(), v.len());
            for (a, b) in actual.as_array().unwrap().iter().zip(v) {
                subset(a, b);
            }
        }
        Value::Number(n) => {
            let b = n.as_f64().unwrap();
            let a = actual.as_f64().unwrap();
            assert!((a - b).abs() <= 1e-12 * b.abs().max(1.), "{a} != {b}");
        }
        _ => assert_eq!(actual, expected),
    }
}
struct Ids {
    epoch: (String, String),
    data: (String, String),
    layers: BTreeMap<String, String>,
}
impl Ids {
    fn new(case: &Value, plot: &Plot, chart: &Runtime) -> Self {
        Self {
            epoch: (
                case["data"]["epoch"].as_str().unwrap().into(),
                chart
                    .chart()
                    .unwrap()
                    .source()
                    .get()
                    .unwrap()
                    .epoch()
                    .get()
                    .to_string(),
            ),
            data: (
                case["data"]["datasets"][0]["id"].as_str().unwrap().into(),
                plot.data("data_0").unwrap().id().get().to_string(),
            ),
            layers: case["chart"]["definition"]["layers"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&plot.definition().layers)
                .map(|(a, b)| (a["id"].as_str().unwrap().to_owned(), b.id.get().to_string()))
                .collect(),
        }
    }
    fn value(&self, value: &Value, key: &str, reverse: bool) -> Value {
        fn pair(pair: &(String, String), reverse: bool) -> (&String, &String) {
            if reverse {
                (&pair.1, &pair.0)
            } else {
                (&pair.0, &pair.1)
            }
        }
        match value {
            Value::Object(v) => Value::Object(
                v.iter()
                    .map(|(k, v)| (k.clone(), self.value(v, k, reverse)))
                    .collect(),
            ),
            Value::Array(v) => {
                Value::Array(v.iter().map(|v| self.value(v, key, reverse)).collect())
            }
            Value::String(s) => {
                let mut s = s.clone();
                let found = match key {
                    "epoch" => Some(pair(&self.epoch, reverse)),
                    "dataset" => Some(pair(&self.data, reverse)),
                    "layer" => self.layers.iter().find_map(|(a, b)| {
                        let (a, b) = if reverse { (b, a) } else { (a, b) };
                        (a == &s).then_some((a, b))
                    }),
                    _ => None,
                };
                if let Some((a, b)) = found
                    && s == *a
                {
                    s = b.clone();
                }
                if key == "description" {
                    let (a, b) = pair(&self.data, reverse);
                    s = s.replace(&format!("in dataset {a}"), &format!("in dataset {b}"));
                }
                Value::String(s)
            }
            _ => value.clone(),
        }
    }
}
fn input_plot(case: &Value) -> Plot {
    let name = case["name"].as_str().unwrap();
    let source = authors::dataset(case, 0);
    let layer = match name {
        "input-bars" => bars().width(20.),
        "input-line" | "input-utc" => line(),
        _ => points(),
    };
    let mut p =
        plot(source)
            .aes(aes().x("field_1").y("field_2"))
            .layer(layer.name("observations").color(chart_core::scene::Color {
                red: 30,
                green: 125,
                blue: 180,
                alpha: 210,
            }));
    p = match name {
        "input-category" | "input-host-category-link" => p
            .x_axis(
                x_axis().visible(false).scale(
                    scale_point()
                        .categories(["Alpha", "Beta", "Gamma"])
                        .point_padding(0.5),
                ),
            )
            .y_axis(y_axis().visible(false)),
        "input-utc" => p
            .aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "field_1".into(),
                        origin: 1709164800000,
                    })
                    .y("field_2"),
            )
            .x_axis(
                x_axis()
                    .visible(false)
                    .scale(scale_utc().interval(UtcInterval::Days(1))),
            )
            .y_axis(y_axis().visible(false)),
        _ => p
            .x_axis(x_axis().visible(false).scale(scale_linear().domain(0., 4.)))
            .y_axis(y_axis().visible(false).scale(scale_linear().domain(0., 4.))),
    };
    if name == "input-host-tools" {
        p = p.layer(
            callout()
                .label(
                    labels()
                        .id("Threshold")
                        .at(0.5, 3.)
                        .text("Threshold")
                        .offset(0., -20.),
                )
                .to_data(3.5, 3.)
                .connector_origin(ConnectorOrigin::Anchor),
        );
    }
    p.build().unwrap()
}
fn json_result(result: ChartResult<String>, step: &Value) -> Value {
    match result {
        Ok(s) => {
            assert!(step.get("error").is_none(), "{}", step["name"]);
            serde_json::from_str(&s).unwrap()
        }
        Err(e) => {
            assert_eq!(
                json!(e.code.as_str()),
                step["error"],
                "{}: {}",
                step["name"],
                e.message
            );
            json!({"error":e.code.as_str()})
        }
    }
}
pub fn run(root: &Path, output: &Output) -> (Value, Value) {
    let read = |path: &str| {
        serde_json::from_str::<Value>(&std::fs::read_to_string(root.join(path)).unwrap()).unwrap()
    };
    let options = Options::new(400., 200., "pt")
        .unwrap()
        .layout(
            &chart_core::plot::host::Component::new("layout_options", "[]")
                .unwrap()
                .set("padding", "[0]")
                .unwrap()
                .set("font_size", "[10]")
                .unwrap(),
        )
        .unwrap();
    let mut inputs = vec![];
    for case in [
        read("fixtures/interaction/cases.json"),
        read("fixtures/host-tools/cases.json"),
    ]
    .iter()
    .flat_map(|v| v.as_array().unwrap())
    {
        let plot = input_plot(case);
        let mut chart = Runtime::new(&plot).unwrap();
        let ids = Ids::new(case, &plot, &chart);
        let mut stamp = chart.present(output, &options).unwrap().scene().stamp();
        let mut basis = None;
        let initial: Value = serde_json::from_str(&chart.semantics().unwrap()).unwrap();
        for raw in case["queries"].as_array().unwrap() {
            let step = ids.value(raw, "", false);
            println!("primary input {} / {}", case["name"], step["name"]);
            let before: Value = serde_json::from_str(&chart.state().unwrap()).unwrap();
            let mut result = Value::Null;
            if let Some(query) = step.get("query") {
                let gesture = step["gesture"] == true;
                let mut query_stamp = if gesture {
                    basis.unwrap_or(stamp)
                } else {
                    stamp
                };
                if step["stale"] == true {
                    query_stamp.layout = chart_core::Revision::new(999999);
                }
                result = json_result(
                    chart.query(&query.to_string(), gesture, Some(query_stamp)),
                    &step,
                );
                assert_eq!(
                    serde_json::from_str::<Value>(&chart.state().unwrap()).unwrap(),
                    before
                );
                if let Some(expected) = step.get("expect") {
                    subset(&result, expected);
                }
            }
            let action=step.get("action").cloned().or_else(||match step["apply"].as_str(){
                Some("annotation_preview")=>Some(json!({"PreviewGesture":{"id":step["id"],"preview":{"Annotation":result["annotation"]}}})),
                Some("linked")=>Some(result["action"].clone()),
                Some("windows")=>Some(json!({"SetAxisWindows":result["windows"]})),
                Some("preview")=>Some(json!({"PreviewGesture":{"id":step["id"],"preview":{"AxisWindows":result["windows"]}}})),
                Some("targets")=>Some(json!({"Select":{"change":step.get("change").unwrap_or(&json!("Replace")),"targets":result["targets"]}})),_=>None});
            if let Some(action) = action {
                if action.get("BeginGesture").is_some() {
                    basis = Some(stamp);
                }
                let origin = if step["apply"] == "linked" {
                    result["origin"].clone()
                } else {
                    json!("Pointer")
                };
                chart
                    .act(&action.to_string(), &origin.to_string(), None)
                    .unwrap();
                if action.get("CancelGesture").is_some() || action.get("CommitGesture").is_some() {
                    basis = None;
                }
            }
            let after: Value = serde_json::from_str(&chart.state().unwrap()).unwrap();
            if let Some(expected) = step.get("state") {
                subset(&after, expected);
            }
            if step["present"] == true {
                stamp = chart.present(output, &options).unwrap().scene().stamp();
            }
            let semantic: Value = serde_json::from_str(&chart.semantics().unwrap()).unwrap();
            assert_eq!(semantic["datasets"], initial["datasets"]);
            for (a, b) in semantic["layers"]
                .as_array()
                .unwrap()
                .iter()
                .zip(initial["layers"].as_array().unwrap())
            {
                assert_eq!(a["rows"], b["rows"]);
                assert_eq!(a["domains"], b["domains"]);
            }
            inputs.push(ids.value(
                &json!({"case":case["name"],"name":step["name"],"result":result,"state":after}),
                "",
                true,
            ));
        }
    }
    let case = json!({"chart":read("fixtures/actions/chart.json"),"data":read("fixtures/bindings/data.json")});
    let plot = plot(authors::dataset(&case, 0))
        .aes(aes().x("x").y("y"))
        .layer(
            histogram()
                .breaks(vec![0., 1., 2.])
                .color(chart_core::scene::Color {
                    red: 180,
                    green: 200,
                    blue: 215,
                    alpha: 140,
                }),
        )
        .layer(line())
        .layer(points())
        .layer(
            labels()
                .id("threshold")
                .figure_at(0.1, 0.1)
                .text("Threshold"),
        )
        .build()
        .unwrap();
    let mut chart = Runtime::new(&plot).unwrap();
    let ids = Ids::new(&case, &plot, &chart);
    chart.present(output, &options).unwrap();
    let mut actions = vec![];
    for raw in read("fixtures/actions/trace.json").as_array().unwrap() {
        let step = ids.value(raw, "", false);
        println!("primary action {}", step["name"]);
        let before: Value = serde_json::from_str(&chart.state().unwrap()).unwrap();
        let expected = step["expected_state"].as_str().map(|s| s.parse().unwrap());
        let result = json_result(
            chart.act(
                &step["action"].to_string(),
                &step.get("origin").unwrap_or(&json!("Control")).to_string(),
                expected,
            ),
            &step,
        );
        let after: Value = serde_json::from_str(&chart.state().unwrap()).unwrap();
        if let Some(expected) = step.get("expect") {
            subset(&result, expected);
        }
        if let Some(expected) = step.get("state") {
            subset(&after, expected);
        }
        if step.get("error").is_some() {
            assert_eq!(after, before);
        }
        if step["present"] == true {
            chart.present(output, &options).unwrap();
        }
        actions.push(ids.value(
            &json!({"name":step["name"],"result":result,"state":after}),
            "",
            true,
        ));
    }
    (json!(actions), json!(inputs))
}

/// Replay the complete independent retained-stream trace using primary data and transactions.
pub fn run_stream(root: &Path, output: &Output) -> Value {
    use chart_core::{data::LateDataPolicy, transaction::Transaction};
    let case: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/streaming/replay.json")).unwrap(),
    )
    .unwrap();
    let plot = plot(authors::dataset(&case, 0))
        .transform(transform(
            "bins",
            bin().x("value").breaks(vec![0., 10., 20., 30., 50.]),
        ))
        .transform(transform(
            "summary",
            summary()
                .x("value")
                .quantiles(vec![0., 0.5, 1.])
                .empty_sum_zero(false),
        ))
        .layer(
            points()
                .aes(
                    aes()
                        .x(Mapping::Timestamp {
                            field: "event_time".into(),
                            origin: 9_007_199_254_741_001,
                        })
                        .y("value"),
                )
                .color(chart_core::scene::Color {
                    red: 30,
                    green: 125,
                    blue: 180,
                    alpha: 210,
                }),
        )
        .build()
        .unwrap();
    let mut chart = Runtime::new(&plot).unwrap();
    let ids = Ids::new(&case, &plot, &chart);
    let options = Options::new(400., 200., "pt")
        .unwrap()
        .layout(
            &chart_core::plot::host::Component::new("layout_options", "[]")
                .unwrap()
                .set("padding", "[0]")
                .unwrap()
                .set("font_size", "[10]")
                .unwrap(),
        )
        .unwrap();
    chart.present(output, &options).unwrap();
    let mut transactions: BTreeMap<String, Transaction> = BTreeMap::new();
    fn transaction(
        chart: &Runtime,
        wire: &Value,
        cache: &mut BTreeMap<String, Transaction>,
    ) -> Transaction {
        let id = wire["id"].as_str().unwrap();
        if let Some(t) = cache.get(id) {
            return t.clone();
        }
        let mut b = chart.transaction().unwrap().id(id);
        for operation in wire["operations"].as_array().unwrap() {
            let (kind, payload) = operation["mutation"]
                .as_object()
                .unwrap()
                .iter()
                .next()
                .unwrap();
            b = match kind.as_str() {
                "AppendBatch" | "UpsertByKey" | "ReplaceSnapshot" => {
                    let source = authors::dataset(
                        &json!({"data":{"datasets":[{"id":"1","batch":payload}]}}),
                        0,
                    );
                    b.data(
                        match kind.as_str() {
                            "AppendBatch" => "append",
                            "UpsertByKey" => "upsert",
                            _ => "replace",
                        },
                        "data_0".into(),
                        &source,
                    )
                    .unwrap()
                }
                "RemoveKeys" => b.remove(
                    "data_0".into(),
                    payload
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|k| k.as_str().unwrap().parse().unwrap())
                        .collect(),
                ),
                "AdvanceWatermark" => {
                    b.watermark("data_0".into(), payload.as_str().unwrap().parse().unwrap())
                }
                "SetRetention" => {
                    if let Some(event) = payload.get("EventTime") {
                        let event = json!({"width":event["width"],"watermark":event["watermark"],"allowed_lateness":event["allowed_lateness"],"late":if event["late"]=="Reject" {LateDataPolicy::Reject} else {LateDataPolicy::Drop}});
                        b.retain_event_time("data_0".into(), "event_time", &event.to_string())
                            .unwrap()
                    } else {
                        b.retain_count(
                            "data_0".into(),
                            payload.get("Count").map(|v| v.as_u64().unwrap() as usize),
                        )
                    }
                }
                _ => panic!("Unhandled stream mutation {kind}"),
            };
        }
        let built = b.build().unwrap();
        cache.insert(id.into(), built.clone());
        built
    }
    let mut trace = Vec::new();
    for raw in case["steps"].as_array().unwrap() {
        let step = ids.value(raw, "", false);
        let result = if let Some(wire) = step.get("transaction") {
            let t = transaction(&chart, wire, &mut transactions);
            chart.commit(&t)
        } else if let Some(stream) = step.get("stream") {
            if let Some(limits) = stream.get("ConfigureQueue") {
                let mut options =
                    chart_core::plot::host::Component::new("stream_options", "[]").unwrap();
                for key in ["transactions", "rows", "bytes", "overload"] {
                    options = options.set(key, &json!([limits[key]]).to_string()).unwrap();
                }
                chart
                    .stream(&options)
                    .map(|()| chart.stream_status().unwrap())
                    .map(|s| serde_json::from_str::<Value>(&s).unwrap()["limits"].to_string())
            } else if let Some(wire) = stream.get("Enqueue") {
                let t = transaction(&chart, wire, &mut transactions);
                chart.enqueue(&t)
            } else {
                match stream.as_str().unwrap() {
                    "CommitNext" => chart.commit_next(),
                    "Status" => chart.stream_status(),
                    "Pinned" => chart.pinned(),
                    _ => panic!("Unknown stream operation"),
                }
            }
        } else {
            chart.act(&step["action"].to_string(), "\"Control\"", None)
        };
        let result = json_result(result, &step);
        if let Some(expected) = step.get("expected") {
            subset(&result, expected);
        }
        let semantics: Value = serde_json::from_str(&chart.semantics().unwrap()).unwrap();
        assert_eq!(
            semantics["store_revision"], step["revision"],
            "{}",
            step["name"]
        );
        assert_eq!(semantics["datasets"][0]["retention"], step["retention"]);
        let rows: Vec<Value> = semantics["datasets"][0]["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|chunk| {
                chunk["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, key)| {
                        json!([
                            key,
                            chunk["columns"][0]["values"]["Timestamp"][i],
                            chunk["columns"][1]["values"]["Float64"][i]
                        ])
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(json!(rows), step["rows"], "{}", step["name"]);
        let state: Value = serde_json::from_str(&chart.state().unwrap()).unwrap();
        if step["present"] == true {
            chart.present(output, &options).unwrap();
        }
        trace.push(json!({"name":step["name"],"result":ids.value(&result,"",true),"state":ids.value(&state,"",true),"semantics":semantics}));
    }
    chart.dispose();
    assert_eq!(trace.len(), 70);
    json!(trace)
}
