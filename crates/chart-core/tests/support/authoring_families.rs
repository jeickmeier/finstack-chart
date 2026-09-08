//! FIX-AUTH03: explicit primary components against the independent legacy fixture corpus.
use chart_core::{
    grammar::*,
    portable::{BatchWire, Session},
    prelude::*,
};
use serde_json::Value;
use std::sync::Arc;

use crate::authors;
fn compare(case: &Value, built: Plot) {
    let name = case["name"].as_str().unwrap();
    let source = built.source();
    let source = source.get().unwrap();
    for (old, new) in case["data"]["datasets"]
        .as_array()
        .unwrap()
        .iter()
        .zip(source.datasets())
    {
        let mut batch: BatchWire = serde_json::from_value(old["batch"].clone()).unwrap();
        for (field, actual) in batch.fields.iter_mut().zip(new.schema().fields()) {
            field.id = actual.id;
        }
        assert_eq!(
            serde_json::to_value(batch).unwrap(),
            serde_json::to_value(BatchWire::from_batch(new.chunks()[0].batch())).unwrap(),
            "{name}: exact fixture materialization"
        );
    }
    let mut reference: ChartDefinition =
        serde_json::from_value(case["chart"]["definition"].clone()).unwrap();
    reference.revision = built.definition().revision;
    let layer_ids = reference
        .layers
        .iter()
        .zip(&built.definition().layers)
        .map(|(a, b)| (a.id, b.id))
        .collect::<std::collections::BTreeMap<_, _>>();
    if let Some(theme) = &mut reference.theme {
        theme.layers = std::mem::take(&mut theme.layers)
            .into_iter()
            .map(|(id, style)| (layer_ids[&id], style))
            .collect();
    }
    if let Some(figure) = &mut reference.figure {
        for inset in &mut figure.insets {
            for layer in &mut inset.layers {
                *layer = layer_ids[layer];
            }
        }
    }
    // The fixture keeps its original data and semantics. Only allocated opaque owners
    // are aligned with the primary plot before executing either path.
    for (old, new) in reference.layers.iter_mut().zip(&built.definition().layers) {
        if let (DataRef::Dataset(old_id), DataRef::Dataset(new_id), Mappings::Source(aes)) =
            (old.data, new.data, &mut old.mappings)
        {
            let batch: BatchWire = serde_json::from_value(
                case["data"]["datasets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|d| {
                        d["id"].as_str().and_then(|s| s.parse::<u64>().ok()) == Some(old_id.get())
                    })
                    .unwrap()["batch"]
                    .clone(),
            )
            .unwrap();
            let source = built.source();
            let source = source.get().unwrap();
            let schema = source.dataset(new_id).unwrap().schema();
            let field_id = |old| {
                let name = &batch.fields.iter().find(|f| f.id == old).unwrap().name;
                schema.fields().iter().find(|f| &f.name == name).unwrap().id
            };
            for value in [
                &mut aes.x,
                &mut aes.y,
                &mut aes.x2,
                &mut aes.y2,
                &mut aes.low,
                &mut aes.high,
                &mut aes.size,
            ]
            .into_iter()
            .flatten()
            {
                match value {
                    Numeric::Field(id)
                    | Numeric::Category(id)
                    | Numeric::Timestamp { field: id, .. } => *id = field_id(*id),
                    Numeric::Literal(_) => {}
                }
            }
            aes.group = aes.group.map(field_id);
        }
        old.id = new.id;
        old.data = new.data;
        old.scales = new.scales;
        if let (Some(a), Some(b)) = (&mut old.color, &new.color) {
            a.id = b.id;
        }
    }
    for old in &mut reference.axes {
        old.id = built
            .definition()
            .axes
            .iter()
            .find(|new| new.side == old.side)
            .map_or(old.id, |new| new.id);
    }
    let reference = Chart::from_external(
        reference,
        built.source(),
        Arc::new(ExtensionRegistry::new()),
    )
    .unwrap();
    let mut actual = Session::from_runtime(built.chart().unwrap()).unwrap();
    let mut expected = Session::from_runtime(reference).unwrap();
    let a: Value = serde_json::from_str(&actual.semantics_json().unwrap()).unwrap();
    let b: Value = serde_json::from_str(&expected.semantics_json().unwrap()).unwrap();
    assert_eq!(
        a, b,
        "{name}: complete semantic rows/domains/membership/source/state"
    );
    let a = actual.prepare().unwrap();
    let b = expected.prepare().unwrap();
    for (left, right) in a
        .layers()
        .iter()
        .chain(a.panels().iter().flat_map(|p| p.chart.layers()))
        .zip(
            b.layers()
                .iter()
                .chain(b.panels().iter().flat_map(|p| p.chart.layers())),
        )
    {
        assert_eq!(
            left.marks(),
            right.marks(),
            "{name}: resolved geometry/style/targets"
        );
    }
}

fn check_cases(cases: Vec<(String, Plot)>, fixtures: &[&str]) -> Vec<(String, Plot)> {
    let references = fixtures
        .iter()
        .flat_map(|f| serde_json::from_str::<Vec<Value>>(f).unwrap())
        .map(|v| (v["name"].as_str().unwrap().to_owned(), v))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (name, plot) in &cases {
        compare(&references[name], plot.clone());
    }
    cases
}
pub fn cases() -> Vec<(String, Plot)> {
    check_cases(
        authors::cases(),
        &[
            include_str!("../../../../fixtures/statistics/portable-cases.json"),
            include_str!("../../../../fixtures/families/portable-cases.json"),
            include_str!("../../../../fixtures/facets/portable-cases.json"),
        ],
    )
}
pub fn composition_cases(
    bold: chart_core::services::ResourceDescriptor,
    arabic: chart_core::services::ResourceDescriptor,
) -> Vec<(String, Plot)> {
    check_cases(
        authors::composition_cases(bold, arabic),
        &[include_str!(
            "../../../../fixtures/composition/portable-cases.json"
        )],
    )
}
