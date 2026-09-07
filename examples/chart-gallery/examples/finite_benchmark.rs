//! Finite native PERF-01/02/04 CPU, GPU and actual-presentation workloads.
#[path = "common/native_probe.rs"]
mod probe;
#[path = "../../../crates/chart-core/examples/common/dense_workload.rs"]
#[allow(dead_code)]
mod workload;
use chart_core::{
    data::*, dense::DensityOptions, grammar::*, inspection::*, services::*, transaction::*, *,
};
use gpui::{
    prelude::*,
    profiler::{FrameEvent, FrameTimingCollector},
    *,
};
use gpui_charts::*;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn scatter_batch(rows: usize, phase: usize) -> ChartResult<NormalizedBatch> {
    let schema = workload::batch(1, 1, 0)?.schema().clone();
    NormalizedBatch::new(
        schema,
        (1..=rows as u64).map(RowKey::new).collect(),
        vec![
            Column::new(
                ColumnValues::Float64(
                    (0..rows)
                        .map(|i| ((i as u64 * 1664525 + 0xF157AC03) % 1048576) as f64 / 1024.)
                        .collect(),
                ),
                vec![true; rows],
                None,
            ),
            Column::new(
                ColumnValues::Float64(
                    (0..rows)
                        .map(|i| {
                            ((i as u64 * 22695477 + 0xF157AC03 + phase as u64 * 7) % 1048576) as f64
                                / 1024.
                        })
                        .collect(),
                ),
                vec![true; rows],
                None,
            ),
            Column::new(ColumnValues::UInt64(vec![0; rows]), vec![true; rows], None),
        ],
        workload::limits(),
    )
}
struct Benchmark {
    mode: String,
    charts: Vec<Entity<ChartView>>,
    weak: Vec<WeakEntity<ChartView>>,
    stores: Vec<DataStore>,
    tick: u64,
    update: u64,
    collector: FrameTimingCollector,
    task: Option<Task<()>>,
}
impl Benchmark {
    fn profile(&mut self) {
        for event in self.collector.collect_unseen() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            match event {
                FrameEvent::Draw(t) => println!(
                    "{}",
                    json!({"event":"gpui-draw","tick":self.tick,"start_unix_ns":(now-t.draw_start.elapsed().as_nanos()).to_string(),"end_unix_ns":(now-t.draw_end.elapsed().as_nanos()).to_string(),"draw_ns":t.draw_duration().as_nanos().to_string(),"invalidations":t.invalidations})
                ),
                FrameEvent::Present(t) => println!(
                    "{}",
                    json!({"event":"gpui-submit","tick":self.tick,"start_unix_ns":(now-t.present_start.elapsed().as_nanos()).to_string(),"end_unix_ns":(now-t.present_end.elapsed().as_nanos()).to_string(),"submit_ns":t.present_duration().as_nanos().to_string()})
                ),
            }
        }
    }
    fn sample(&self, cx: &App, event: &str) {
        let rss = String::from_utf8(
            std::process::Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        println!(
            "{}",
            json!({"event":event,"mode":self.mode,"tick":self.tick,"rss_kib":rss.trim().parse::<u64>().ok(),"charts":self.charts.iter().map(|c|{let c=c.read(cx);json!({"schedule":c.scheduling_metrics(),"density":c.density_metrics(),"layouts":c.metrics().layout_attempts,"bounds":c.inspector().map(|i|i.presented().scene().bounds()),"index_candidates":c.inspector().map(chart_core::inspection::Inspector::candidate_count)})}).collect::<Vec<_>>(),"live_entities":self.weak.iter().filter(|w|w.upgrade().is_some()).count()})
        );
    }
    fn step(&mut self, cx: &mut Context<Self>) {
        self.tick += 1;
        self.profile();
        for (i, chart) in self.charts.iter().enumerate() {
            let c = chart.read(cx);
            assert!(c.diagnostic().is_none(), "{:?}", c.diagnostic());
            if let Some(inspector) = c.inspector() {
                let p = inspector.presented().plot().unwrap();
                let point = chart_core::Point::new(
                    p.origin().x() + p.width() * (self.tick % 101) as f64 / 100.,
                    p.origin().y() + p.height() * 0.5,
                )
                .unwrap();
                let start = Instant::now();
                let q = inspector.query(
                    point,
                    if i == 0 && self.mode.starts_with("dashboard") {
                        InspectionMode::NearestPoint
                    } else {
                        InspectionMode::NearestX
                    },
                );
                for h in &q.hits {
                    std::hint::black_box(inspector.describe_target(h, c.state()).unwrap());
                }
                let query_ns = start.elapsed().as_nanos();
                let start = Instant::now();
                chart
                    .update(cx, |c, cx| {
                        c.dispatch_inspection(
                            InspectionAction::Hover(Some(point)),
                            InputOrigin::Programmatic,
                            cx,
                        )
                    })
                    .unwrap();
                println!(
                    "{}",
                    json!({"event":"finite-hover","tick":self.tick,"chart":i,"measured":self.tick>120 && self.tick<=720,"query_ns":query_ns.to_string(),"dispatch_ns":start.elapsed().as_nanos().to_string(),"examined":q.examined})
                );
            }
        }
        if self.mode.starts_with("dashboard") && self.tick <= 720 && self.tick.is_multiple_of(6) {
            self.update += 1;
            for (i, chart) in self.charts.iter().enumerate() {
                let source = self.stores[i].snapshot();
                let source = source.get().unwrap();
                let start = Instant::now();
                let receipt = self.stores[i].apply(Transaction {
                    id: TransactionId::new(format!("finite-{}-{}", i, self.update)).unwrap(),
                    epoch: source.epoch(),
                    expected: vec![source.dataset(workload::DATA).unwrap().version()],
                    operations: vec![Operation {
                        dataset: workload::DATA,
                        mutation: Mutation::UpsertByKey(if i == 0 {
                            scatter_batch(100, self.update as usize).unwrap()
                        } else {
                            workload::batch(1, 100, self.update as usize).unwrap()
                        }),
                    }],
                });
                assert!(matches!(receipt, CommitOutcome::Applied(_)));
                chart
                    .update(cx, |c, cx| c.queue_data(self.stores[i].snapshot(), cx))
                    .unwrap();
                println!(
                    "{}",
                    json!({"event":"finite-commit","tick":self.tick,"chart":i,"revision":self.update,"commit_ns":start.elapsed().as_nanos().to_string()})
                );
            }
        }
        if self.tick.is_multiple_of(120) {
            self.sample(cx, "finite-sample");
        }
        if self.tick == 780 {
            self.sample(cx, "finite-drained");
            for c in &self.charts {
                c.update(cx, |c, cx| c.dispose_preparation(cx));
            }
            self.charts.clear();
            cx.notify();
        }
        if self.tick == 840 {
            self.sample(cx, "finite-disposed");
            assert!(self.weak.iter().all(|w| w.upgrade().is_none()));
            cx.quit();
        }
    }
}
impl Render for Benchmark {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        println!(
            "{}",
            json!({"event":"finite-window","mode":self.mode,"tick":self.tick,"scale":window.scale_factor(),"active":window.is_window_active()})
        );
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .child(probe::begin())
            .child(
                div()
                    .h(px(28.))
                    .child(format!("{} · {}", self.mode, self.tick)),
            );
        if let Some(first) = self.charts.first() {
            root = root.child(
                div()
                    .w(px(1202.))
                    .h(px(if self.mode.starts_with("dashboard") {
                        300.
                    } else {
                        602.
                    }))
                    .child(first.clone()),
            );
            if self.charts.len() > 1 {
                for row in self.charts[1..].chunks(4) {
                    root = root.child(
                        div().flex().h(px(160.)).children(
                            row.iter()
                                .map(|c| div().w(px(300.)).h(px(160.)).child(c.clone())),
                        ),
                    );
                }
            }
        }
        root.child(probe::end(&self.charts))
    }
}
fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "lines".into());
    assert!(["lines", "million", "dashboard-raw", "dashboard-dense"].contains(&mode.as_str()));
    gpui::profiler::set_trace_enabled(true);
    gpui_platform::application().run(move |cx| {
        let font=NativeFont::load(ResourceDescriptor{id:ResourceId::new(1),revision:Revision::new(1),kind:ResourceKind::Font,byte_len:FONT.len() as u64},Arc::from(FONT),"Noto Sans",cx).unwrap();
        let sizes=if mode.starts_with("dashboard") {let mut s=vec![(1,50_000)];s.extend(std::iter::repeat_n((1,10_000),12));s} else if mode=="million" {vec![(1,1_000_000)]} else {vec![(10,10_000)]};
        let stores:Vec<_>=sizes.iter().enumerate().map(|(i,(s,n))| if mode.starts_with("dashboard") && i==0 {DataStore::new(SourceEpoch::new(1),vec![(workload::DATA,scatter_batch(*n,0).unwrap())],workload::limits()).unwrap()} else {workload::store(*s,*n).unwrap()}).collect();
        let inputs:Vec<_>=stores.iter().enumerate().map(|(i,s)|{let mut d=workload::definition();if mode.starts_with("dashboard")&&i==0 {d.layers[0].geom=Geom::Point;}ChartInput::new(d,s.snapshot(),font.clone()).unwrap()}).collect();
        let dashboard=mode.starts_with("dashboard");
        cx.open_window(WindowOptions{kind:WindowKind::PopUp,window_bounds:Some(WindowBounds::Windowed(Bounds::centered(None,size(px(1240.),px(if dashboard{920.}else{730.})),cx))),titlebar:Some(TitlebarOptions{title:Some("Finstack Finite Benchmark".into()),..Default::default()}),..Default::default()},move |_,cx|cx.new(|cx|{
            let charts:Vec<_>=inputs.into_iter().map(|input|cx.new(|cx|{let mut chart=ChartView::new(input,cx);let mut layout=chart.layout_request().clone();layout.limits.max_items=1_100_000;layout.limits.max_path_commands=1_100_000;chart.set_layout(layout,cx).unwrap();if mode=="dashboard-raw" {chart.set_density(DensityOptions{line_bucket_width:None,candle_bucket_width:None,..Default::default()},cx).unwrap();}chart})).collect();
            let weak=charts.iter().map(Entity::downgrade).collect();
            println!("{}",json!({"event":"finite-protocol","mode":mode,"warmup_ticks":120,"measured_ticks":600,"period_ms":16,"seed":"0xF157AC03","sizes":sizes,"profiler":"GPUI full Window::draw and platform submission; external MTLDrawable/GPU callbacks"}));
            let mut this=Benchmark{mode,charts,weak,stores,tick:0,update:0,collector:FrameTimingCollector::new(),task:None};
            this.task=Some(cx.spawn(async move |entity,cx|{loop {cx.background_executor().timer(Duration::from_millis(16)).await;if entity.update(cx,|this:&mut Benchmark,cx|{this.step(cx);this.tick>=840}).unwrap_or(true){break;}}}));this
        })).unwrap();cx.activate(true);
    });
}
