//! HIR-08 component measurements for the WP-22 handoff; no native frame-rate claim.
use chart_core::{
    ChartResult, Point, Rect,
    hierarchy::{LayoutSpec, Tiler, TreemapOptions},
    inspection::{InspectionMode, Inspector},
    prelude::*,
    scene::Primitive,
};
use chart_export::*;
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};
fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.
}
fn data(name: &str, n: usize, phase: usize) -> ChartResult<Data> {
    let parent = (0..n)
        .map(|i| {
            if i == 0 {
                None
            } else {
                Some(match name {
                    "long-chain" => i as u64,
                    "balanced-tree" => (i - 1) as u64 / 2 + 1,
                    _ => 1,
                })
            }
        })
        .collect::<Vec<_>>();
    let values = (0..n)
        .map(|i| {
            let leaf = match name {
                "long-chain" => i + 1 == n,
                "balanced-tree" => 2 * i + 1 >= n,
                _ => i > 0,
            };
            if !leaf {
                0.
            } else if name == "skewed-pack" && i == 1 {
                100_000. + phase as f64
            } else {
                1. + ((i * 17 + 0xF157_AC03usize + phase) % 13) as f64
            }
        })
        .collect::<Vec<_>>();
    Data::columns()
        .name("hierarchy")
        .keys(1..=n as u64)
        .column("id", (1..=n as u64).collect::<Vec<_>>())
        .column("parent", parent)
        .column("value", values)
        .build()
}
fn summary(samples: &[Value], key: &str) -> Value {
    let mut v = samples
        .iter()
        .map(|s| s[key].as_f64().unwrap())
        .collect::<Vec<_>>();
    v.sort_by(f64::total_cmp);
    json!({"p50_ms":v[(v.len()*50).div_ceil(100)-1],"p95_ms":v[(v.len()*95).div_ceil(100)-1],"max_ms":v[v.len()-1]})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    let output = Output::new(bytes)?;
    let mut records = vec![];
    for n in [100usize, 1000] {
        for name in [
            "balanced-tree",
            "long-chain",
            "high-fanout",
            "skewed-pack",
            "resquarify-resize",
        ] {
            let layer = match name {
                "skewed-pack" => hierarchy_pack("id", "parent"),
                "resquarify-resize" => {
                    hierarchy_treemap("id", "parent").hierarchy_layout(LayoutSpec::Treemap {
                        options: TreemapOptions {
                            tile: Tiler::Resquarify(1.618_033_988_749_895),
                            ..Default::default()
                        },
                        history: true,
                        padding_sides: Default::default(),
                        padding: None,
                        tiler: None,
                    })
                }
                _ => hierarchy_tree("id", "parent"),
            }
            .hierarchy_value("value");
            let p = plot(data(name, n, 0)?).layer(layer).build()?;
            let mut chart = p.chart()?;
            let options = || {
                export_options(PageSize::points(1200., 600.).unwrap())
                    .dpi(72)
                    .basis(CaptureBasis::Current)
            };
            let start = Instant::now();
            let initial = output.live_request(&chart, options())?.prepare()?;
            let cold_ms = elapsed(start);
            let mut request = initial.metadata().profile.layout.clone();
            request = request.with_hierarchy_history(initial.layout());
            let fonts =
                FontResources::new(vec![FontResource::new(request.font, Arc::from(bytes))?])?;
            let initial_keys = Arc::downgrade(
                initial
                    .layout()
                    .hierarchies()
                    .values()
                    .next()
                    .unwrap()
                    .prepared()
                    .source_keys(),
            );
            drop(initial);
            assert!(
                initial_keys.upgrade().is_none(),
                "captured rows must not pin a prior hierarchy"
            );
            let incoming = [data(name, n, 1)?, data(name, n, 2)?];
            let mut samples = vec![];
            let mut counts = Value::Null;
            for i in 0..40 {
                let start = Instant::now();
                let transaction = chart
                    .transaction()?
                    .replace("hierarchy", &incoming[i % 2])
                    .build()?;
                chart.apply_transaction(transaction)?;
                let ingest_ms = elapsed(start);
                let start = Instant::now();
                let prepared = chart.prepare()?;
                let prepare_ms = elapsed(start);
                if name == "resquarify-resize" {
                    request.bounds = Rect::new(
                        0.,
                        0.,
                        if i % 2 == 0 { 1200. } else { 600. },
                        if i % 2 == 0 { 600. } else { 1200. },
                    )?;
                }
                let start = Instant::now();
                let layout = Arc::new(chart_core::layout::layout(
                    prepared.clone(),
                    &request,
                    &fonts,
                )?);
                let layout_ms = elapsed(start);
                let start = Instant::now();
                let inspector = Inspector::new(layout.clone(), 3., 8)?;
                let index_ms = elapsed(start);
                let start = Instant::now();
                let hit = inspector.query(Point::new(600., 300.)?, InspectionMode::Auto);
                std::hint::black_box(hit);
                let hit_ms = elapsed(start);
                request = request.with_hierarchy_history(&layout);
                let start = Instant::now();
                let capture = output
                    .live_request(&chart, options())?
                    .with_hierarchy_history(&layout);
                let acquire_ms = elapsed(start);
                let start = Instant::now();
                let figure = capture.prepare()?;
                let publication_prepare_ms = elapsed(start);
                let start = Instant::now();
                let artifact = figure.export(Format::Svg)?;
                let svg_ms = elapsed(start);
                let h = layout.hierarchies().values().next().unwrap();
                assert!(h.history().membership_count() < n);
                if i == 39 {
                    counts = json!({"nodes":n,"leaves":h.prepared().hierarchy().leaves(h.prepared().hierarchy().root().handle())?.len(),"depth":h.prepared().hierarchy().root().height(),"marks":layout.scene().items().len(),"path_commands":layout.scene().items().iter().map(|s|match &s.primitive {Primitive::ShapePath {geometry,..}=>geometry.commands().len(),_=>0}).sum::<usize>(),"semantic_targets":inspector.semantic_targets().len(),"shared_source_members":h.prepared().source_keys().len(),"history_rows":h.history().row_count(),"history_members":h.history().membership_count(),"scene_json_bytes":figure.scene_json()?.len(),"svg_bytes":artifact.bytes.len()});
                }
                if i >= 10 {
                    samples.push(json!({"ingest_commit_ms":ingest_ms,"numeric_prepare_ms":prepare_ms,"layout_ms":layout_ms,"hit_index_ms":index_ms,"hit_query_ms":hit_ms,"capture_acquire_ms":acquire_ms,"publication_prepare_ms":publication_prepare_ms,"svg_encode_ms":svg_ms}));
                }
            }
            let keys = [
                "ingest_commit_ms",
                "numeric_prepare_ms",
                "layout_ms",
                "hit_index_ms",
                "hit_query_ms",
                "capture_acquire_ms",
                "publication_prepare_ms",
                "svg_encode_ms",
            ];
            let summary = keys
                .iter()
                .map(|k| ((*k).into(), summary(&samples, k)))
                .collect::<serde_json::Map<_, _>>();
            let budget = if n == 100 { 500. } else { 5000. };
            assert!(
                summary
                    .values()
                    .all(|s| s["p95_ms"].as_f64().unwrap() <= budget)
            );
            records.push(json!({"name":name,"nodes":n,"cold_publication_prepare_ms":cold_ms,"warmup":10,"samples":samples,"summary":summary,"counts":counts,"p95_stage_budget_ms":budget,"verdict":"PASS"}));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"version":1,"profile":"release","units":"milliseconds","bounds":[1200,600],"seed":"0xF157AC03; deterministic index schedules","records":records,"limitations":["CPU component stages only; no native frame-rate or ingest-to-present certification","Allocation-event counts and OS memory sampling remain WP-22 instrumentation","Scene JSON byte count is serialized output, not total live heap"]})
        )?
    );
    Ok(())
}
