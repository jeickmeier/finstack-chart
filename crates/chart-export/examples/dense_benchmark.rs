//! Preliminary PERF-01/02 CPU stages and exact lookup; no total-frame/GPU certification.
#[path = "../../chart-core/examples/common/dense_workload.rs"]
#[allow(dead_code)]
mod workload;
use chart_core::{
    dense::*, grammar::*, inspection::*, layout::*, services::*, state::ChartState, *,
};
use chart_export::{FontResource, FontResources};
use serde_json::json;
use std::{sync::Arc, time::Instant};
struct Measure(FontResources);
impl TextMeasurer for Measure {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.0.measure(TextRequest {
            units: Units::Points,
            ..r
        })
    }
}
fn rss() -> Option<u64> {
    String::from_utf8(
        std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?
            .stdout,
    )
    .ok()?
    .trim()
    .parse()
    .ok()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("fixtures/capability/fonts/NotoSans-Regular.ttf")?;
    let descriptor = ResourceDescriptor {
        id: ResourceId::new(1),
        revision: Revision::new(1),
        kind: ResourceKind::Font,
        byte_len: bytes.len() as u64,
    };
    let fonts = Measure(FontResources::new(vec![FontResource::new(
        descriptor,
        Arc::from(bytes),
    )?])?);
    for (name, series, rows) in [("PERF-01", 10, 10_000), ("PERF-02", 1, 1_000_000)] {
        let before = rss();
        let start = Instant::now();
        let store = workload::store(series, rows)?;
        let ingest = start.elapsed().as_nanos();
        let source = store.snapshot();
        let d = workload::definition();
        let mut compiler = Compiler::new();
        let start = Instant::now();
        let prepared = Arc::new(compiler.prepare(
            &d,
            &source,
            &ChartState::default(),
            CompileLimits {
                max_prepared_rows: 1_100_000,
                max_vertices: 1_100_000,
                ..Default::default()
            },
        )?);
        let numeric = start.elapsed().as_nanos();
        let numeric_rss = rss();
        let mut request = LayoutRequest::new(
            Rect::new(0., 0., 1200., 600.)?,
            Units::LogicalPixels,
            descriptor,
        );
        request.max_vertices = 1_100_000;
        request.limits.max_path_commands = 1_100_000;
        request.limits.max_items = 1_100_000;
        let start = Instant::now();
        let chart = Arc::new(layout(prepared.clone(), &request, &fonts)?);
        let layout_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let dense = DenseChart::prepare(chart.clone(), &DensityOptions::default(), request.limits)?;
        let dense_ns = start.elapsed().as_nanos();
        let dense_rss = rss();
        let start = Instant::now();
        let inspector = Inspector::new(chart.clone(), 10., 32)?;
        let index_ns = start.elapsed().as_nanos();
        let index_rss = rss();
        let plot = chart.plot().unwrap();
        let mut samples = vec![];
        let mut tested = vec![];
        for i in 0..110 {
            let p = Point::new(
                plot.origin().x() + plot.width() * (i % 100) as f64 / 99.,
                plot.origin().y() + plot.height() * 0.5,
            )?;
            let start = Instant::now();
            let q = inspector.query(p, InspectionMode::NearestX);
            for h in &q.hits {
                std::hint::black_box(inspector.describe_target(h, &ChartState::default())?);
            }
            let elapsed = start.elapsed().as_nanos();
            if i >= 10 {
                samples.push(elapsed);
                tested.push(q.examined);
            }
        }
        assert!(dense.metrics().rendered_vertices < dense.metrics().raw_vertices);
        assert!(tested.iter().all(|n| *n <= series * 4));
        assert_eq!(
            source.get()?.dataset(workload::DATA)?.lookup_entries(),
            series * rows
        );
        let mut sorted = samples.clone();
        sorted.sort_unstable();
        let p95 = sorted[(sorted.len() * 95).div_ceil(100) - 1];
        let weak_prepared = Arc::downgrade(&prepared);
        let weak_chart = Arc::downgrade(&chart);
        let metrics = dense.metrics().clone();
        let plot_bounds = [
            plot.origin().x(),
            plot.origin().y(),
            plot.width(),
            plot.height(),
        ];
        drop(inspector);
        drop(dense);
        drop(chart);
        drop(prepared);
        drop(compiler);
        drop(source);
        drop(store);
        assert!(weak_prepared.upgrade().is_none() && weak_chart.upgrade().is_none());
        println!(
            "{}",
            json!({"workload":name,"prepared_and_layout_released":true,"scope":"single release CPU-stage run with supplied Noto Sans numeric font size mapped one-to-one to scene units; 10 warmup and 100 hover samples; no native frame/GPU/presentation timings","chart_bounds":[1200,600],"plot_bounds":plot_bounds,"data_load_ns":ingest.to_string(),"numeric_ns":numeric.to_string(),"layout_ns":layout_ns.to_string(),"density_ns":dense_ns.to_string(),"index_ns":index_ns.to_string(),"hover_p95_ns":p95.to_string(),"hover_ns":samples.iter().map(ToString::to_string).collect::<Vec<_>>(),"examined":tested,"density":metrics,"rss_kib":{"before":before,"after_numeric":numeric_rss,"after_density":dense_rss,"after_index":index_rss,"after_drop":rss()}})
        );
    }
    Ok(())
}
