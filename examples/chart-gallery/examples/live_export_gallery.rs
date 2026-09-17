//! Actual native FIX-14: presented captures export on workers while atomic ingestion continues.
use chart_core::{
    data::*,
    prelude::{Data, Plot, aes, labels, line, plot},
    state::*,
    transaction::*,
    *,
};
use chart_export::*;
use gpui::{prelude::*, *};
use gpui_charts::*;
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
struct Gallery {
    chart: Entity<ChartView>,
    plot: Plot,
    output: Output,
    queue: ExportQueue,
    tasks: BTreeMap<Revision, Task<()>>,
    latest: Option<ExportCancellation>,
    tick: u64,
    running: bool,
    status: String,
    outputs: Vec<String>,
    out: PathBuf,
    started: Instant,
    _timer: Task<()>,
    _observation: Subscription,
}
fn batch(y: f64) -> ChartResult<Data> {
    Data::columns()
        .column("x", [3.])
        .column("y", [y])
        .keys([9007199254743004])
        .build()
}

impl Gallery {
    fn log(&self, cx: &App, event: &str) {
        let c = self.chart.read(cx);
        println!(
            "{}",
            json!({"event":event,"tick":self.tick,"elapsed_ms":self.started.elapsed().as_millis().to_string(),"committed":c.chart().source().get().unwrap().revision(),"presented":c.inspector().map(|i|i.presented().scene().stamp()),"exports":self.queue.metrics(),"preparation":c.scheduling_metrics(),"error":c.diagnostic(),"status":self.status})
        );
    }
    fn advance(&mut self, cx: &mut Context<Self>) -> ChartResult<()> {
        self.tick += 1;
        let tx = self
            .chart
            .read(cx)
            .chart()
            .transaction()?
            .id(format!("native-live-{}", self.tick))
            .upsert("positive", batch(self.tick as f64)?)
            .upsert("negative", batch(-(self.tick as f64))?)
            .build()?;
        let result = self.chart.update(cx, |chart, cx| chart.commit(tx, cx))?;
        if !matches!(result, CommitOutcome::Applied(_)) {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                format!("{result:?}"),
                "Inspect the atomic live proof.",
            ));
        }
        if self.tick.is_multiple_of(10) {
            let edited = self
                .plot
                .edit()
                .annotation(
                    labels()
                        .id("threshold")
                        .figure_at(0.1, 0.1)
                        .text(format!("Live annotation {}", self.tick)),
                )
                .build()?;
            let expected = self.chart.read(cx).chart().definition().revision;
            self.chart
                .update(cx, |c, cx| c.apply_plot(&edited, expected, cx))?;
            self.plot = edited;
        }
        if self.tick == 40 {
            self.chart.update(cx, |c, cx| {
                let mut layout = c.layout_request().clone();
                layout.output_theme = chart_core::theme::ThemePatch {
                    background: Some(chart_core::theme::rgb(245, 225, 215).into()),
                    panel: Some(chart_core::theme::rgb(250, 240, 230).into()),
                    ..Default::default()
                };
                c.set_layout(layout, cx)
            })?;
        }
        if self.tick >= 400 {
            self.running = false;
            self.status = "400 coherent commits completed; export work drains independently".into();
        }
        self.log(cx, "commit");
        Ok(())
    }
    fn capture(&mut self, format: Format, full: bool, cx: &mut Context<Self>) -> ChartResult<()> {
        let started = Instant::now();
        let chart = self.chart.read(cx).chart();
        let request = self.output.live_request(
            chart,
            export_options(PageSize::points(500., 300.)?)
                .dpi(144)
                .view(if full {
                    ViewMode::FullDomain
                } else {
                    ViewMode::VisibleView
                }),
        )?;
        let source = request.source().get()?;
        let values: [f64; 2] = ["positive", "negative"].map(|name| {
            let dataset = chart.data(name).unwrap().id();
            let field = self.plot.data(name).unwrap().field("y").unwrap().id();
            match source
                .dataset(dataset)
                .unwrap()
                .row(RowKey::new(9007199254743004))
                .unwrap()
                .value(field)
                .unwrap()
            {
                ValueRef::Float64(v) => v,
                _ => panic!("numeric source"),
            }
        });
        assert_eq!(values[0] + values[1], 0.);
        let captured_revision = source.revision();
        let manifest = request.manifest()?;
        let job = self.queue.submit(request, format)?;
        let id = job.id();
        self.latest = Some(job.cancellation());
        println!(
            "{}",
            json!({"event":"capture","job":id,"tick":self.tick,"capture_ns":started.elapsed().as_nanos().to_string(),"source_pair":values,"format":format,"manifest":manifest})
        );
        let out = self.out.clone();
        let task=cx.background_spawn(async move{let started=Instant::now();let result=job.run_with_observer(|phase|{println!("{}",json!({"event":"export-phase","job":id,"phase":phase,"elapsed_ns":started.elapsed().as_nanos().to_string()}));if phase!=ExportPhase::Encoded{std::thread::sleep(Duration::from_millis(2000));println!("{}",json!({"event":"export-resume","job":id,"phase":phase,"elapsed_ns":started.elapsed().as_nanos().to_string()}));}Ok(())});(result,started.elapsed(),out)});
        self.tasks.insert(id,cx.spawn(async move |entity,cx|{let(result,elapsed,out)=task.await;let _=entity.update(cx,|this,cx|{this.tasks.remove(&id);match result{Ok(artifact)=>{let extension=match artifact.format{Format::Svg=>"svg",Format::Pdf=>"pdf",Format::Png=>"png",Format::Jpeg=>"jpeg",Format::Tiff=>"tiff",Format::Bmp=>"bmp",Format::PostScript=>"ps",Format::Eps=>"eps",Format::PicTeX=>"tex",Format::Emf=>"emf"};let name=format!("native-live-{}.{extension}",id.get());let save_started=Instant::now();let result=std::fs::write(out.join(&name),&artifact.bytes).and_then(|()|std::fs::write(out.join(format!("native-live-{}.json",id.get())),serde_json::to_vec_pretty(&artifact.metadata.manifest()).unwrap()));match result{Ok(())=>{if this.outputs.len()==8{this.outputs.remove(0);}this.outputs.push(format!("{name}: captured commit {}, completed at {}",artifact.metadata.stamp.store.get(),this.tick));this.status="Coherent export saved while the live chart advanced".into();},Err(e)=>this.status=e.to_string()};println!("{}",json!({"event":"export-complete","job":id,"tick":this.tick,"worker_elapsed_ms":elapsed.as_millis().to_string(),"save_ns":save_started.elapsed().as_nanos().to_string(),"metadata":artifact.metadata.manifest(),"bytes":artifact.bytes.len(),"exports":this.queue.metrics()}));},Err(e)=>{this.status=format!("Export {}: {}",id.get(),e.message);println!("{}",json!({"event":"export-error","job":id,"tick":this.tick,"error":e,"exports":this.queue.metrics()}));}}this.log(cx,"export-returned");cx.notify();});}));
        self.running = self.tick < 400;
        self.status = format!(
            "Captured job {} at visible commit {}; worker deliberately pauses twice",
            id.get(),
            captured_revision.get()
        );
        cx.notify();
        Ok(())
    }
    fn act(&mut self, label: &str, cx: &mut Context<Self>) {
        let result = (|| -> ChartResult<()> {
            match label {
                "Capture pair" => {
                    self.capture(Format::Svg, false, cx)?;
                    self.capture(Format::Pdf, true, cx)?;
                    match self.capture(Format::Png, false, cx) {
                        Err(e) if e.code == DiagnosticCode::ResourceLimit => {
                            self.status =
                                "Two captures accepted; third rejected by the explicit job bound"
                                    .into();
                            println!(
                                "{}",
                                json!({"event":"capacity-rejected","tick":self.tick,"error":e})
                            );
                        }
                        other => {
                            other?;
                            return Err(Diagnostic::error(
                                DiagnosticCode::Validation,
                                "Expected bounded export rejection",
                                "Inspect job capacity.",
                            ));
                        }
                    }
                }
                "PNG" => self.capture(Format::Png, true, cx)?,
                "Cancel latest" => {
                    if let Some(c) = &self.latest {
                        c.cancel();
                    }
                    self.status =
                        "Requested cancellation; active phase releases at its next boundary".into();
                }
                "Freeze" => {
                    self.chart.update(cx, |c, cx| {
                        c.dispatch_chart(
                            ChartAction::SetFollow(
                                chart_core::state::FollowMode::FreezePresentation,
                            ),
                            cx,
                        )
                    })?;
                }
                "Resume" => {
                    self.chart
                        .update(cx, |c, cx| c.dispatch_chart(ChartAction::ResumeLatest, cx))?;
                }
                "Stop" => {
                    self.running = false;
                    self.status = "Stopped ingestion; accepted exports may finish".into();
                }
                "Dispose" => {
                    self.running = false;
                    self.queue.dispose();
                    self.chart.update(cx, |c, cx| c.dispose_preparation(cx));
                    self.status = "Disposed export/preparation queues".into();
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
        let mut controls = div().flex().gap_2();
        for label in [
            "Capture pair",
            "PNG",
            "Cancel latest",
            "Freeze",
            "Resume",
            "Stop",
            "Dispose",
        ] {
            controls = controls.child(
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
        let c = self.chart.read(cx);
        let shown = c
            .inspector()
            .map(|i| i.presented().scene().stamp().store.get())
            .unwrap_or(0);
        let m = self.queue.metrics();
        div().size_full().flex().flex_col().p_4().gap_3().bg(rgb(0xeaf0f8)).text_color(rgb(0x263448)).font_family("Noto Sans").child(div().text_xl().child("Coherent publication during atomic live updates")).child("Visible SVG and full-domain PDF capture one painted revision. Two worker pauses allow later data, annotation and theme edits.").child(controls).child(format!("Committed {} · Painted {} · Export pending {} · Running {} · Input bytes {} · Window active {}",self.tick,shown,m.pending,m.running,m.input_bytes,window.is_window_active())).child(self.status.clone()).child(div().flex_1().min_h_0().child(self.chart.clone())).child(div().id("export-results").h(px(150.)).flex_shrink_0().overflow_y_scroll().children(self.outputs.iter().cloned().map(|s|div().child(s))))
    }
}
fn main() {
    let out = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("artifacts/wp-20/native-ui"));
    std::fs::create_dir_all(&out).unwrap();
    gpui_platform::application().run(move |cx: &mut App| {
        let data = |name: &str, sign: f64| {
            Data::columns()
                .name(name)
                .column("x", (0..8).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "y",
                    (0..8).map(|i| sign * (i * i) as f64).collect::<Vec<_>>(),
                )
                .keys(9007199254743001..9007199254743009)
                .build()
                .unwrap()
        };
        let plot = plot(data("positive", 1.))
            .aes(aes().x("x").y("y"))
            .layer(line().color(chart_core::theme::rgb(35, 90, 150)))
            .layer(
                line()
                    .data(data("negative", -1.))
                    .color(chart_core::theme::rgb(190, 60, 65)),
            )
            .layer(
                labels()
                    .id("threshold")
                    .figure_at(0.1, 0.1)
                    .text("Before capture"),
            )
            .build()
            .unwrap();
        let output = Output::new(FONT).unwrap();
        let font = NativeFont::from_bytes(FONT, "Noto Sans", cx).unwrap();
        let input = ChartInput::from_plot(&plot, font).unwrap();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1220.), px(850.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Live Export Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    let chart = cx.new(|cx| ChartView::new(input, cx));
                    chart
                        .update(cx, |c, cx| {
                            c.dispatch_chart(
                                ChartAction::SetViewport(Viewport {
                                    x: Some((2., 5.)),
                                    y: None,
                                }),
                                cx,
                            )
                        })
                        .unwrap();
                    let observation = cx.observe(&chart, |_, _, cx| cx.notify());
                    let timer = cx.spawn(async move |entity, cx| {
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(50))
                                .await;
                            let result = entity.update(cx, |s: &mut Gallery, cx| {
                                if s.running {
                                    if let Err(e) = s.advance(cx) {
                                        s.running = false;
                                        s.status = e.message;
                                    }
                                    cx.notify();
                                }
                            });
                            if result.is_err() {
                                break;
                            }
                        }
                    });
                    Gallery {
                        chart,
                        plot,
                        output,
                        queue: ExportQueue::new(ExportLimits::default()).unwrap(),
                        tasks: BTreeMap::new(),
                        latest: None,
                        tick: 0,
                        running: false,
                        status: "Ready; capture a pair to start ingestion".into(),
                        outputs: vec![],
                        out,
                        started: Instant::now(),
                        _timer: timer,
                        _observation: observation,
                    }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
