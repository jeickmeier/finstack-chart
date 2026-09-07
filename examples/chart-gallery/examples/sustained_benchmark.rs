//! PERF-03/05 wall-clock workload. Paint acknowledgements are explicitly CPU-side.
#[path = "common/native_probe.rs"]
mod probe;
#[path = "../../../crates/chart-core/examples/common/dense_workload.rs"]
#[allow(dead_code)]
mod workload;
use chart_core::{data::*, inspection::*, services::*, state::*, transaction::*, *};
use chart_export::*;
use gpui::{prelude::*, *};
use gpui_charts::*;
use serde_json::json;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
const RETAINED: usize = 100_000;
const BATCH: usize = 1_000;
const WARMUP: u64 = 100;
fn value(k: u64) -> f64 {
    if k.is_multiple_of(997) {
        150.
    } else {
        ((k.wrapping_add(0xF157AC03)) % 10_000) as f64 / 100.
    }
}
fn batch(
    schema: Arc<Schema>,
    keys: Vec<u64>,
    correction: Option<u64>,
) -> ChartResult<NormalizedBatch> {
    let len = keys.len();
    NormalizedBatch::new(
        schema,
        keys.iter().map(|k| RowKey::new(*k)).collect(),
        vec![
            Column::new(
                ColumnValues::Float64(keys.iter().map(|k| *k as f64).collect()),
                vec![true; len],
                None,
            ),
            Column::new(
                ColumnValues::Float64(
                    keys.iter()
                        .map(|k| value(*k) + if Some(*k) == correction { 0.5 } else { 0. })
                        .collect(),
                ),
                keys.iter().map(|k| k % 1729 != 1728).collect(),
                None,
            ),
            Column::new(ColumnValues::UInt64(vec![0; len]), vec![true; len], None),
        ],
        workload::limits(),
    )
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
struct Benchmark {
    chart: Option<Entity<ChartView>>,
    weak_chart: WeakEntity<ChartView>,
    store: DataStore,
    reference: BTreeMap<u64, f64>,
    frontier: u64,
    tick: u64,
    total: u64,
    started: Instant,
    pending: VecDeque<(u64, u128)>,
    painted: u64,
    queue: ExportQueue,
    tasks: Vec<Task<()>>,
    out: std::path::PathBuf,
    weak_scenes: Vec<std::sync::Weak<chart_core::layout::LaidOutChart>>,
    task: Option<Task<()>>,
}
impl Benchmark {
    fn nanos(&self) -> u128 {
        self.started.elapsed().as_nanos()
    }
    fn apply(&mut self, operations: Vec<Mutation>) -> CommitReceipt {
        let s = self.store.snapshot();
        let s = s.get().unwrap();
        match self.store.apply(Transaction {
            id: TransactionId::new(format!("stream-{}", self.tick)).unwrap(),
            epoch: s.epoch(),
            expected: vec![s.dataset(workload::DATA).unwrap().version()],
            operations: operations
                .into_iter()
                .map(|mutation| Operation {
                    dataset: workload::DATA,
                    mutation,
                })
                .collect(),
        }) {
            CommitOutcome::Applied(r) => r,
            outcome => panic!("accepted transaction failed: {outcome:?}"),
        }
    }
    fn check(&self) {
        let source = self.store.snapshot();
        let data = source.get().unwrap().dataset(workload::DATA).unwrap();
        assert_eq!(data.len(), self.reference.len());
        for row in data.rows() {
            let k = row.key().get();
            let expected = self.reference.get(&k).expect("unexpected retained key");
            assert_eq!(row.value(workload::X), Some(ValueRef::Float64(k as f64)));
            let y = if k % 1729 == 1728 {
                None
            } else {
                Some(ValueRef::Float64(*expected))
            };
            assert_eq!(row.value(workload::Y), y, "key {k}");
        }
    }
    fn advance(&mut self, cx: &mut Context<Self>) {
        self.tick += 1;
        let arrived = self.nanos();
        let arrival_unix_ns = probe::unix_ns();
        let candidate = self.frontier - 20;
        let correction = if candidate % 1729 == 1728 {
            candidate - 1
        } else {
            candidate
        };
        let mut keys: Vec<_> = (self.frontier + 1..self.frontier + BATCH as u64).collect();
        keys.push(correction);
        self.frontier += BATCH as u64 - 1;
        let source = self.store.snapshot();
        let schema = source
            .get()
            .unwrap()
            .dataset(workload::DATA)
            .unwrap()
            .schema()
            .clone();
        let b = batch(schema, keys.clone(), Some(correction)).unwrap();
        let mut ops = vec![Mutation::UpsertByKey(b)];
        for k in keys {
            self.reference
                .insert(k, value(k) + if k == correction { 0.5 } else { 0. });
        }
        while self.reference.len() > RETAINED {
            self.reference.pop_first();
        }
        let removed = if self.tick.is_multiple_of(10) {
            let k = self.frontier - 10;
            self.reference.remove(&k);
            ops.push(Mutation::RemoveKeys(vec![RowKey::new(k)]));
            1
        } else {
            0
        };
        let start = Instant::now();
        let receipt = self.apply(ops);
        let commit_ns = start.elapsed().as_nanos();
        assert_eq!(receipt.operations[0].inserted, BATCH - 1);
        assert_eq!(receipt.operations[0].updated, 1);
        assert_eq!(receipt.operations.get(1).map_or(0, |o| o.removed), removed);
        let revision = receipt.store_revision.get();
        self.pending.push_back((revision, arrived));
        assert!(
            self.pending.len() < 10_000,
            "presentation backlog exceeded explicit benchmark bound"
        );
        let chart = self.chart.as_ref().unwrap();
        chart
            .update(cx, |c, cx| c.queue_data(self.store.snapshot(), cx))
            .unwrap();
        if self.tick.is_multiple_of(20) {
            chart
                .update(cx, |c, cx| {
                    c.dispatch_chart(
                        ChartAction::SetViewport(Viewport {
                            x: Some((
                                self.frontier as f64 - 95_000.,
                                self.frontier as f64 + 1_000.,
                            )),
                            y: None,
                        }),
                        cx,
                    )
                })
                .unwrap();
        }
        println!(
            "{}",
            json!({"event":"commit","tick":self.tick,"measured":self.tick>WARMUP,"arrival_ns":arrived.to_string(),"arrival_unix_ns":arrival_unix_ns,"scheduled_ns":(u128::from(self.tick)*100_000_000).to_string(),"commit_ns":commit_ns.to_string(),"observed_ns":self.nanos().to_string(),"revision":revision,"counts":receipt.operations,"retained":self.reference.len(),"pending_ack":self.pending.len()})
        );
        if self.tick.is_multiple_of(100) {
            self.check();
            self.sample(cx, "reference-check");
        }
        if self.tick == WARMUP + 50
            || (self.tick > WARMUP && (self.tick - WARMUP).is_multiple_of(3000))
        {
            self.export(cx);
        }
    }
    fn observe(&mut self, cx: &mut Context<Self>) {
        let Some(chart) = self.chart.as_ref() else {
            return;
        };
        let now = self.nanos();
        let c = chart.read(cx);
        assert!(c.diagnostic().is_none(), "{:?}", c.diagnostic());
        let Some(inspector) = c.inspector() else {
            return;
        };
        let revision = inspector.presented().scene().stamp().store.get();
        if revision > self.painted {
            self.painted = revision;
            let mut acks = vec![];
            while self.pending.front().is_some_and(|p| p.0 <= revision) {
                let (r, arrival) = self.pending.pop_front().unwrap();
                acks.push(json!([r, (now - arrival).to_string()]));
            }
            println!(
                "{}",
                json!({"event":"cpu-paint-ack","tick":self.tick,"ns":now.to_string(),"revision":revision,"covered_commits":acks,"schedule":c.scheduling_metrics(),"density":c.density_metrics(),"bounds":inspector.presented().scene().bounds()})
            );
        }
        let p = inspector.presented().plot().unwrap();
        let x = p.origin().x() + p.width() * ((now / 20_000_000) % 101) as f64 / 100.;
        let point = chart_core::Point::new(x, p.origin().y() + p.height() * 0.5).unwrap();
        let start = Instant::now();
        let query = inspector.query(point, InspectionMode::NearestX);
        for h in &query.hits {
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
            json!({"event":"hover","tick":self.tick,"ns":now.to_string(),"query_ns":query_ns.to_string(),"dispatch_ns":start.elapsed().as_nanos().to_string(),"examined":query.examined})
        );
    }
    fn sample(&mut self, cx: &App, event: &str) {
        self.weak_scenes.retain(|w| w.strong_count() > 0);
        println!(
            "{}",
            json!({"event":event,"tick":self.tick,"ns":self.nanos().to_string(),"rss_kib":rss(),"export_metrics":self.queue.metrics(),"tracked_export_scenes_live":self.weak_scenes.len(),"chart_live":self.weak_chart.upgrade().is_some(),"schedule":self.chart.as_ref().map(|c|c.read(cx).scheduling_metrics())})
        );
    }
    fn export(&mut self, cx: &mut Context<Self>) {
        let captured = self
            .chart
            .as_ref()
            .unwrap()
            .read(cx)
            .capture_presented()
            .unwrap();
        self.weak_scenes.push(Arc::downgrade(&captured.chart));
        let prepared = captured.chart.prepared();
        let fonts = FontResources::new(
            captured
                .fonts
                .into_iter()
                .map(|(d, b)| FontResource::new(d, b))
                .collect::<ChartResult<Vec<_>>>()
                .unwrap(),
        )
        .unwrap();
        let mut profile =
            PublicationProfile::new(PageSize::points(600., 300.).unwrap(), captured.layout.font)
                .unwrap();
        profile.view = ViewMode::VisibleView;
        let request = FigureRequest::new(
            prepared.definition().clone(),
            prepared.source().clone(),
            captured.state,
            fonts,
            profile,
            InteractionCapture::default(),
        )
        .unwrap()
        .with_origin_scene(captured.chart.scene().stamp())
        .unwrap();
        let manifest = request.manifest().unwrap();
        let job = self.queue.submit(request, Format::Pdf).unwrap();
        let id = job.id();
        let out = self.out.clone();
        println!(
            "{}",
            json!({"event":"export-capture","tick":self.tick,"ns":self.nanos().to_string(),"id":id,"manifest":manifest,"rss_kib":rss()})
        );
        self.tasks.retain(|t| !t.is_ready());
        let task = cx.background_spawn(async move {
            let start = Instant::now();
            let artifact = job.run().unwrap();
            let elapsed = start.elapsed().as_nanos();
            std::fs::write(
                out.join(format!("stream-{}.pdf", id.get())),
                &artifact.bytes,
            )
            .unwrap();
            (artifact.metadata.manifest(), elapsed, artifact.bytes.len())
        });
        self.tasks.push(cx.spawn(async move |entity,cx| {let (manifest,elapsed,bytes)=task.await;let _=entity.update(cx,|this,cx|{println!("{}",json!({"event":"export-complete","tick":this.tick,"ns":this.nanos().to_string(),"id":id,"worker_ns":elapsed.to_string(),"bytes":bytes,"manifest":manifest}));this.sample(cx,"export-release");});}));
    }
}
impl Render for Benchmark {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tick.is_multiple_of(100) {
            println!(
                "{}",
                json!({"event":"window","tick":self.tick,"active":window.is_window_active(),"scale":window.scale_factor()})
            );
        }
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .child(probe::begin());
        root = root.child(div().h(px(45.)).child(format!(
            "Sustained stream: {} / {} batches · CPU painted {}",
            self.tick, self.total, self.painted
        )));
        if let Some(chart) = &self.chart {
            root = root
                .child(div().w(px(1202.)).h(px(602.)).child(chart.clone()))
                .child(probe::end(std::slice::from_ref(chart)));
        }
        if self.chart.is_none() {
            root = root.child(probe::end(&[]));
        }
        let _ = cx;
        root
    }
}
fn main() {
    let seconds = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(1800);
    let out = std::path::PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "artifacts/wp-22/stream".into()),
    );
    std::fs::create_dir_all(&out).unwrap();
    gpui_platform::application().run(move |cx| {
        let descriptor=ResourceDescriptor{id:ResourceId::new(1),revision:Revision::new(1),kind:ResourceKind::Font,byte_len:FONT.len() as u64};
        let font=NativeFont::load(descriptor,Arc::from(FONT),"Noto Sans",cx).unwrap();
        let schema=workload::batch(1,1,0).unwrap().schema().clone();
        let initial=batch(schema,(1..=RETAINED as u64).collect(),None).unwrap();
        let mut store=DataStore::new(SourceEpoch::new(1),vec![(workload::DATA,initial)],workload::limits()).unwrap();
        let source=store.snapshot();let source=source.get().unwrap();
        assert!(matches!(store.apply(Transaction{id:TransactionId::new("retention").unwrap(),epoch:source.epoch(),expected:vec![source.dataset(workload::DATA).unwrap().version()],operations:vec![Operation{dataset:workload::DATA,mutation:Mutation::SetRetention(RetentionPolicy::Count(RETAINED))}]}),CommitOutcome::Applied(_)));
        let input=ChartInput::new(workload::definition(),store.snapshot(),font).unwrap();
        cx.open_window(WindowOptions {kind:WindowKind::PopUp,window_bounds:Some(WindowBounds::Windowed(Bounds::centered(None,size(px(1240.),px(730.)),cx))),titlebar:Some(TitlebarOptions{title:Some("Finstack Sustained Benchmark".into()),..Default::default()}),..Default::default()},move |_,cx|cx.new(|cx| {
            let chart=cx.new(|cx|ChartView::new(input,cx));let weak_chart=chart.downgrade();
            let mut this=Benchmark{chart:Some(chart),weak_chart,store,reference:(1..=RETAINED as u64).map(|k|(k,value(k))).collect(),frontier:RETAINED as u64,tick:0,total:seconds*10+WARMUP,started:Instant::now(),pending:VecDeque::new(),painted:0,queue:ExportQueue::new(ExportLimits::default()).unwrap(),tasks:vec![],out,weak_scenes:vec![],task:None};
            println!("{}",json!({"event":"protocol","seed":"0xF157AC03","version":1,"pid":std::process::id(),"unix_ns":SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string(),"seconds":seconds,"warmup_batches":WARMUP,"batch_rows":BATCH,"period_ms":100,"retained":RETAINED,"presentation_boundary":"CPU paint acknowledgement; external display trace required"}));
            this.task=Some(cx.spawn(async move |entity,cx| {
                let mut tail=0;
                loop {
                    cx.background_executor().timer(Duration::from_millis(20)).await;
                    let done=entity.update(cx,|this:&mut Benchmark,cx| {
                        this.observe(cx);
                        if this.tick<this.total && this.nanos()>=u128::from(this.tick+1)*100_000_000 {this.advance(cx);}
                        if this.tick==this.total {
                            tail+=1;
                            if tail==250 {this.check();this.sample(cx,"drained");this.chart.as_ref().unwrap().update(cx,|c,cx|c.dispose_preparation(cx));this.chart=None;this.queue.dispose();cx.notify();}
                            if tail==300 {this.sample(cx,"disposed");assert!(this.weak_chart.upgrade().is_none());println!("{}",json!({"event":"complete","tick":this.tick,"ns":this.nanos().to_string(),"pending_ack":this.pending.len()}));cx.quit();return true;}
                        }
                        false
                    }).unwrap_or(true);
                    if done {break;}
                }
            }));this
        })).unwrap();cx.activate(true);
    });
}
