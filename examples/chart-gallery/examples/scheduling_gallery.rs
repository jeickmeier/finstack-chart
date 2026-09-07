//! WP-19 actual bounded background workers and exact dense native inspection.
#[path = "../../../crates/chart-core/examples/common/dense_workload.rs"]
#[allow(dead_code)]
mod workload;
use chart_core::{services::*, state::*, transaction::*, *};
use gpui::{prelude::*, *};
use gpui_charts::*;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
struct Gallery {
    charts: Vec<Entity<ChartView>>,
    stores: Vec<DataStore>,
    tick: usize,
    running: bool,
    started: Instant,
    status: String,
    isolated_redraw: bool,
    _task: Task<()>,
    _subscriptions: Vec<Subscription>,
}
impl Gallery {
    fn advance(&mut self, cx: &mut Context<Self>) -> ChartResult<()> {
        self.tick += 1;
        for i in 0..self.charts.len() {
            let store = &mut self.stores[i];
            let source = store.snapshot();
            let mutation = Mutation::UpsertByKey(workload::batch(1, 100, self.tick)?);
            let outcome = store.apply(Transaction {
                id: TransactionId::new(format!("native-{}-{}", i, self.tick))?,
                epoch: source.get()?.epoch(),
                expected: vec![source.get()?.dataset(workload::DATA)?.version()],
                operations: vec![Operation {
                    dataset: workload::DATA,
                    mutation,
                }],
            });
            if !matches!(outcome, CommitOutcome::Applied(_)) {
                return Err(Diagnostic::error(
                    DiagnosticCode::Validation,
                    format!("{outcome:?}"),
                    "Inspect the proof transaction.",
                ));
            }
            self.charts[i].update(cx, |chart, cx| chart.queue_data(store.snapshot(), cx))?;
        }
        if self.tick == 24 {
            self.act("Inspect", cx);
        }
        if self.tick == 48 {
            self.act("Theme", cx);
        }
        self.log(cx, "arrival");
        if self.tick >= 160 {
            self.running = false;
            self.status =
                "Finished 160 committed updates per chart; pending work is draining.".into();
        }
        Ok(())
    }
    fn log(&self, cx: &App, event: &str) {
        println!(
            "{}",
            json!({"event":event,"tick":self.tick,"elapsed_ms":self.started.elapsed().as_millis().to_string(),"charts":self.charts.iter().map(|c|{let c=c.read(cx);json!({"schedule":c.scheduling_metrics(),"presented_store":c.inspector().map(|i|i.presented().scene().stamp().store),"density":c.density_metrics(),"layout_attempts":c.metrics().layout_attempts,"paints":c.metrics().paints,"error":c.diagnostic()})}).collect::<Vec<_>>()})
        );
    }
    fn act(&mut self, label: &str, cx: &mut Context<Self>) {
        let result = (|| -> ChartResult<()> {
            match label {
                "Run" => {
                    self.running = self.tick < 160;
                    self.status = "Running bounded workers".into();
                }
                "Pause" => {
                    self.running = false;
                    self.status = "Paused arrivals; accepted preparation continues".into();
                }
                "Inspect" => {
                    for chart in &self.charts {
                        chart.update(cx, |c, cx| {
                            c.dispatch_chart(
                                ChartAction::SetViewport(Viewport {
                                    x: Some((0., 500.)),
                                    y: None,
                                }),
                                cx,
                            )
                        })?;
                    }
                    self.status = "Changed viewport while work may be active".into();
                }
                "Theme" => {
                    for chart in &self.charts {
                        chart.update(cx, |c, cx| {
                            let mut r = c.layout_request().clone();
                            r.output_theme = chart_core::theme::ThemePatch {
                                background: Some(chart_core::theme::rgb(220, 230, 240)),
                                panel: Some(chart_core::theme::rgb(245, 240, 225)),
                                ..Default::default()
                            };
                            c.set_layout(r, cx)
                        })?;
                    }
                    self.status = "Changed destination theme".into();
                }
                "Dispose workers" => {
                    self.running = false;
                    for chart in &self.charts {
                        chart.update(cx, |c, cx| c.dispose_preparation(cx));
                    }
                    self.status =
                        "Workers disposed; existing immutable presentation retained".into();
                }
                _ => {}
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.status = e.message;
        }
        self.log(cx, label);
        cx.notify();
    }
}
impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        println!(
            "{}",
            json!({"event":"render","tick":self.tick,"active":window.is_window_active()})
        );
        let mut buttons = div().flex().gap_2();
        for label in ["Run", "Pause", "Inspect", "Theme", "Dispose workers"] {
            buttons = buttons.child(
                div()
                    .id(label)
                    .role(Role::Button)
                    .aria_label(label)
                    .px_3()
                    .py_2()
                    .bg(rgb(0xffffff))
                    .cursor_pointer()
                    .child(label)
                    .on_click(cx.listener(move |s, _, _, cx| s.act(label, cx))),
            );
        }
        let mut columns = vec![];
        for row in 0..2 {
            let mut views = div().flex().flex_1().min_h_0().gap_3();
            for i in row * 2..row * 2 + 2 {
                let c = self.charts[i].read(cx);
                let m = c.scheduling_metrics();
                let shown = c
                    .inspector()
                    .map(|i| i.presented().scene().stamp().store.get())
                    .unwrap_or(0);
                let dense = c
                    .density_metrics()
                    .map(|d| {
                        format!(
                            "{} raw → {} painted vertices",
                            d.raw_vertices, d.rendered_vertices
                        )
                    })
                    .unwrap_or_default();
                let text = format!(
                    "{} rows · Committed {} · Presented {} · Active {} · Pending {} · Coalesced {} · Stale {}",
                    self.stores[i]
                        .snapshot()
                        .get()
                        .unwrap()
                        .dataset(workload::DATA)
                        .unwrap()
                        .len(),
                    m.committed.map(|r| r.get()).unwrap_or(0),
                    shown,
                    m.active,
                    m.pending,
                    m.coalesced,
                    m.stale
                );
                views = views.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap_2()
                        .child(text)
                        .child(dense)
                        .child(div().flex_1().min_h_0().child(self.charts[i].clone())),
                );
            }
            columns.push(views);
        }
        div().size_full().flex().flex_col().p_4().gap_3().bg(rgb(0xecf1f8)).text_color(rgb(0x202b3c)).font_family("Noto Sans").child(div().text_xl().child("Bounded preparation across four independent charts")).child("One 100,000-row chart and three smaller charts. Ingestion commits every accepted update; only pending preparation is coalesced.").child(buttons).child(format!("Tick {} / 160 · {}",self.tick,self.status)).children(columns)
    }
}
fn main() {
    let isolated_redraw = std::env::args().any(|a| a == "--isolated-redraw");
    gpui_platform::application().run(move |cx: &mut App| {
        let font = NativeFont::load(
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::new(1),
                kind: ResourceKind::Font,
                byte_len: FONT.len() as u64,
            },
            Arc::from(FONT),
            "Noto Sans",
            cx,
        )
        .unwrap();
        let stores: Vec<_> = [100_000, 2_000, 3_000, 4_000]
            .into_iter()
            .map(|n| workload::store(1, n).unwrap())
            .collect();
        let inputs: Vec<_> = stores
            .iter()
            .map(|s| ChartInput::new(workload::definition(), s.snapshot(), font.clone()).unwrap())
            .collect();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1440.), px(960.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Scheduling Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    let charts: Vec<_> = inputs
                        .into_iter()
                        .map(|input| cx.new(|cx| ChartView::new(input, cx)))
                        .collect();
                    let subscriptions = if isolated_redraw {
                        vec![]
                    } else {
                        charts
                            .iter()
                            .map(|chart| cx.observe(chart, |_, _, cx| cx.notify()))
                            .collect()
                    };
                    let task = cx.spawn(async move |entity, cx| {
                        let mut tail = 0;
                        let mut warmup = 0;
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            let result = entity.update(cx, |this: &mut Gallery, cx| {
                                if this.isolated_redraw && this.tick == 0 && !this.running {
                                    warmup += 1;
                                    if warmup == 100 {
                                        this.act("Run", cx);
                                    }
                                }
                                if this.running {
                                    tail = 0;
                                    for _ in 0..4 {
                                        if !this.running {
                                            break;
                                        }
                                        if let Err(e) = this.advance(cx) {
                                            this.running = false;
                                            this.status = e.message;
                                            break;
                                        }
                                    }
                                } else if this.tick > 0 && tail < 100 {
                                    tail += 1;
                                    if tail % 10 == 0 {
                                        this.log(cx, "drain");
                                    }
                                    if tail == 100 && this.isolated_redraw {
                                        this.act("Dispose workers", cx);
                                    }
                                }
                                if !this.isolated_redraw {
                                    cx.notify();
                                }
                            });
                            if result.is_err() {
                                break;
                            }
                        }
                    });
                    Gallery {
                        charts,
                        stores,
                        tick: 0,
                        running: false,
                        started: Instant::now(),
                        status: "Ready; press Run".into(),
                        isolated_redraw,
                        _task: task,
                        _subscriptions: subscriptions,
                    }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
