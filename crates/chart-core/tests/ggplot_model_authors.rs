//! Independently authored model publication inputs exercise the public pipeline.
#[path = "../../../examples/common/ggplot_model_controls.rs"]
mod fixtures;
use chart_core::{grammar::*, prelude::*};
#[test]
fn model_authors_prepare_and_replay_with_ribbons() {
    for mode in 0..10 {
        let original = fixtures::author(mode).unwrap();
        let restored = Plot::from_json(&original.to_json().unwrap()).unwrap();
        let a = original.chart().unwrap().prepare().unwrap();
        let b = restored.chart().unwrap().prepare().unwrap();
        assert_eq!(a.layers()[0].marks(), b.layers()[0].marks());
        let PreparedRows::Statistical(rows) = a.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(rows.len(), if mode == 4 || mode == 5 { 123 } else { 41 });
        if mode != 4 && mode != 5 {
            assert!(
                a.layers()[0]
                    .marks()
                    .iter()
                    .any(|m| matches!(m.geometry, PreparedGeometry::BandRun { .. }))
            );
        }
    }
}

#[test]
fn public_model_rows_match_independent_r_predictions() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/model-controls.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let method = match name {
            "automatic-1000" => ModelMethod::Gam {
                basis_dimension: 10,
                knots: None,
                iterations: 120,
                tolerance: 1e-9,
            },
            "automatic-999" => ModelMethod::Auto,
            "quantile-fn-TRUE" | "quantile-br-TRUE" => ModelMethod::Quantile {
                solver: if name.contains("-fn-") {
                    ModelQuantileSolver::Fn
                } else {
                    ModelQuantileSolver::Br
                },
                probabilities: vec![0.25, 0.5, 0.75],
                iterations: 10000,
            },
            _ if name.starts_with("lm-weighted-") => ModelMethod::Linear,
            _ => continue,
        };
        let controls = &case["controls"];
        let data = controls["data"].as_array().unwrap();
        let data = Data::columns()
            .column(
                "x",
                data.iter()
                    .map(|r| r["x"].as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "y",
                data.iter()
                    .map(|r| r["y"].as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
            .column(
                "w",
                data.iter()
                    .map(|r| r["w"].as_f64().unwrap_or(1.))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let result = &case["result"]["value"];
        let expected = result.get("built").unwrap_or(result)["values"]
            .as_array()
            .unwrap();
        let mut grid = Vec::new();
        for row in expected {
            let x = row["x"].as_f64().unwrap();
            if !grid.contains(&x) {
                grid.push(x);
            }
        }
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .layer(
                smooth().stat(
                    model_stat(ModelOptions {
                        method,
                        xseq: Some(grid),
                        level: controls["level"].as_f64().unwrap_or(0.95),
                        ..Default::default()
                    })
                    .x("x")
                    .y("y")
                    .weight("w"),
                ),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!()
        };
        assert_eq!(rows.len(), expected.len(), "{name}");
        for (row, expected) in rows.iter().zip(expected) {
            for (field, key) in [
                (StatField::Y, "y"),
                (StatField::StandardError, "se"),
                (StatField::Lower, "ymin"),
                (StatField::Upper, "ymax"),
            ] {
                if let Some(value) = expected.get(key).and_then(|v| v.as_f64()) {
                    assert!(
                        (row.value(&field).unwrap() - value).abs() < 1e-7,
                        "{name}/{key}"
                    );
                }
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 7);
}

#[test]
fn model_native_sized_frames_construct_inspection_with_original_targets() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision,
        inspection::Inspector,
        layout::{LayoutRequest, layout},
        services::{
            ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
        },
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 6., 9., 3.)
        }
    }
    for mode in [0, 1, 2, 3, 7, 8] {
        let plot = fixtures::author(mode).unwrap();
        let request = LayoutRequest::new(
            Rect::new(0., 0., 480., 320.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = std::sync::Arc::new(
            layout(plot.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap(),
        );
        let inspector =
            Inspector::new(frame.clone(), 10., 32).unwrap_or_else(|e| panic!("model{mode}: {e:?}"));
        assert_eq!(
            inspector.semantic_targets().len(),
            41,
            "one keyboard identity per prediction"
        );
        for hit in inspector.semantic_targets() {
            assert!(
                frame
                    .targets()
                    .iter()
                    .flatten()
                    .any(|target| target == &hit.target)
            );
        }
    }
}
