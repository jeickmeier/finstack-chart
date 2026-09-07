//! Preliminary PERF-01/02 CPU stages and exact lookup; no total-frame/GPU certification.
#[path = "../../chart-core/examples/common/dense_workload.rs"]
#[allow(dead_code)]
mod workload;
use chart_core::{
    dense::*, grammar::*, inspection::*, layout::*, services::*, state::ChartState, transaction::*,
    *,
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
    let repeats = std::env::var("DENSE_REPEATS")
        .ok()
        .map(|s| s.parse::<usize>().expect("positive repeat count"))
        .unwrap_or(1);
    assert!(repeats > 0);
    for (iteration, (name, series, rows)) in (0..repeats).flat_map(|iteration| {
        [("PERF-01", 10, 10_000), ("PERF-02", 1, 1_000_000)].map(|case| (iteration, case))
    }) {
        let before = rss();
        let start = Instant::now();
        let mut store = workload::store(series, rows)?;
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
        let start = Instant::now();
        let receipt = store.apply(Transaction {
            id: TransactionId::new(format!("benchmark-update-{iteration}"))?,
            epoch: source.get()?.epoch(),
            expected: vec![source.get()?.dataset(workload::DATA)?.version()],
            operations: vec![Operation {
                dataset: workload::DATA,
                mutation: Mutation::UpsertByKey(workload::batch(1, 100, iteration + 1)?),
            }],
        });
        let update_apply_ns = start.elapsed().as_nanos();
        assert!(matches!(receipt, CommitOutcome::Applied(_)));
        let updated_source = store.snapshot();
        assert_eq!(
            updated_source.get()?.dataset(workload::DATA)?.len(),
            series * rows
        );
        let expected = ((13 + iteration + 1) % 97) as f64;
        assert_eq!(
            updated_source
                .get()?
                .dataset(workload::DATA)?
                .row(RowKey::new(2))
                .unwrap()
                .value(workload::Y),
            Some(chart_core::data::ValueRef::Float64(expected))
        );
        assert_eq!(
            source
                .get()?
                .dataset(workload::DATA)?
                .row(RowKey::new(2))
                .unwrap()
                .value(workload::Y),
            Some(chart_core::data::ValueRef::Float64(13.))
        );
        let start = Instant::now();
        let updated = Arc::new(compiler.prepare(
            &d,
            &updated_source,
            &ChartState::default(),
            CompileLimits {
                max_prepared_rows: 1_100_000,
                max_vertices: 1_100_000,
                ..Default::default()
            },
        )?);
        let update_numeric_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let updated_chart = Arc::new(layout(updated.clone(), &request, &fonts)?);
        let update_layout_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let updated_dense = DenseChart::prepare(
            updated_chart.clone(),
            &DensityOptions::default(),
            request.limits,
        )?;
        let update_density_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let updated_inspector = Inspector::new(updated_chart.clone(), 10., 32)?;
        let update_index_ns = start.elapsed().as_nanos();
        let updated_memory = rss();
        let updated_weak = Arc::downgrade(&updated_chart);
        drop(updated_inspector);
        drop(updated_dense);
        drop(updated_chart);
        drop(updated);
        drop(updated_source);
        assert!(updated_weak.upgrade().is_none());
        drop(compiler);
        drop(source);
        drop(store);
        assert!(weak_prepared.upgrade().is_none() && weak_chart.upgrade().is_none());
        println!(
            "{}",
            json!({"workload":name,"iteration":iteration,"measured_stage_sample":repeats==1 || iteration>=10,"prepared_and_layout_released":true,"scope":"release CPU-stage sample with supplied Noto Sans numeric font size mapped one-to-one to scene units; 10 warmup and 100 hover samples; no native frame/GPU/presentation timings","chart_bounds":[1200,600],"plot_bounds":plot_bounds,"data_load_ns":ingest.to_string(),"numeric_ns":numeric.to_string(),"layout_ns":layout_ns.to_string(),"density_ns":dense_ns.to_string(),"index_ns":index_ns.to_string(),"update":{"apply_ns":update_apply_ns.to_string(),"numeric_ns":update_numeric_ns.to_string(),"layout_ns":update_layout_ns.to_string(),"density_ns":update_density_ns.to_string(),"index_ns":update_index_ns.to_string(),"rss_kib":updated_memory,"old_source_unchanged":true},"hover_p95_ns":p95.to_string(),"hover_ns":samples.iter().map(ToString::to_string).collect::<Vec<_>>(),"examined":tested,"density":metrics,"rss_kib":{"before":before,"after_numeric":numeric_rss,"after_density":dense_rss,"after_index":index_rss,"after_drop":rss()}})
        );
    }
    Ok(())
}
