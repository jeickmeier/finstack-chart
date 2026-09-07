//! WP-21 real native resize/freeze/recovery and repeated entity lifetime evidence.
use chart_core::state::FollowMode;
use chart_core::{data::*, grammar::*, portable::*, services::*, state::*, transaction::*, *};
use gpui::{prelude::*, *};
use gpui_charts::*;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
struct Gallery {
    chart: Entity<ChartView>,
    store: DataStore,
    definition: ChartDefinition,
    font: NativeFont,
    dimensions: (f32, f32),
    stage: usize,
    running: bool,
    status: String,
    old: Option<WeakEntity<ChartView>>,
    released: usize,
    observation: Subscription,
    _timer: Task<()>,
}
impl Gallery {
    fn log(&self, cx: &App, event: &str) {
        let chart = self.chart.read(cx);
        let capture = chart.capture_presented().ok();
        println!(
            "{}",
            json!({"event":event,"stage":self.stage,"released":self.released,
            "dimensions":[self.dimensions.0,self.dimensions.1],"error":chart.diagnostic(),
            "capture":capture.map(|c|json!({"stamp":c.chart.scene().stamp(),"bounds":c.chart.scene().bounds(),"profile":c.layout,"state":c.state.interaction_snapshot(),"source":c.chart.prepared().source().get().unwrap().revision()})),
            "layout_attempts":chart.metrics().layout_attempts,"paints":chart.metrics().paints,"status":self.status})
        );
    }
    fn step(&mut self, cx: &mut Context<Self>) -> ChartResult<()> {
        self.log(cx, "before-step");
        match self.stage {
            0 => {
                self.chart.update(cx, |c, cx| {
                    c.dispatch_chart(ChartAction::SetFollow(FollowMode::FreezePresentation), cx)
                })?;
                self.status = "Frozen original figure".into();
            }
            1 => {
                let source = self.store.snapshot();
                let source = source.get()?;
                let operations = source
                    .datasets()
                    .map(|d| {
                        let sign = if d.version().dataset == DatasetId::new(1) {
                            1.
                        } else {
                            -1.
                        };
                        let batch = NormalizedBatch::new(
                            d.schema().clone(),
                            vec![RowKey::new(9007199254743004)],
                            vec![
                                Column::new(ColumnValues::Float64(vec![3.]), vec![true], None),
                                Column::new(
                                    ColumnValues::Float64(vec![sign * 999.]),
                                    vec![true],
                                    None,
                                ),
                            ],
                            DataLimits::default(),
                        )?;
                        Ok(Operation {
                            dataset: d.version().dataset,
                            mutation: Mutation::UpsertByKey(batch),
                        })
                    })
                    .collect::<ChartResult<Vec<_>>>()?;
                assert!(matches!(
                    self.store.apply(Transaction {
                        id: TransactionId::new("hardening-correction")?,
                        epoch: source.epoch(),
                        expected: source.datasets().map(|d| d.version()).collect(),
                        operations
                    }),
                    CommitOutcome::Applied(_)
                ));
                self.chart.update(cx, |c, cx| {
                    c.set_data(self.store.snapshot(), cx)?;
                    let mut r = c.layout_request().clone();
                    r.output_theme.background = Some(chart_core::theme::rgb(245, 225, 215));
                    c.set_layout(r, cx)
                })?;
                self.dimensions = (480., 240.);
                self.status = "Frozen resize: retains original data and theme".into();
            }
            2 => {
                self.dimensions = (12., 12.);
                self.status = "Tiny frozen destination: recoverable limited layout".into();
            }
            3 => {
                self.dimensions = (800., 400.);
                self.status = "Recovered frozen projection at 800 × 400".into();
            }
            4 => {
                self.chart.update(cx, |c, cx| {
                    c.dispatch_chart(ChartAction::SetFollow(FollowMode::FollowLatest), cx)
                })?;
                self.status = "Resumed: latest ±999 values and updated theme".into();
            }
            5 => {
                let mut invalid = self.definition.clone();
                invalid.layers[0].data = DatasetId::new(999).into();
                let error = self
                    .chart
                    .update(cx, |c, cx| c.set_definition(invalid, cx))
                    .unwrap_err();
                println!("{}", json!({"event":"malformed-definition","error":error}));
                self.status = "Malformed update rejected; last valid chart retained".into();
            }
            6 => {
                self.chart
                    .update(cx, |c, cx| c.set_definition(self.definition.clone(), cx))?;
                let invalid = NativeFont::load(
                    ResourceDescriptor {
                        id: ResourceId::new(999),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 3,
                    },
                    Arc::from(&b"bad"[..]),
                    "Unavailable",
                    cx,
                )
                .err()
                .expect("bad font must fail");
                println!("{}", json!({"event":"unavailable-font","error":invalid}));
                self.status = "Recovered definition; malformed explicit font rejected".into();
            }
            7..=46 => {
                if self.stage % 2 == 1 {
                    self.old = Some(self.chart.downgrade());
                    let input = ChartInput::new(
                        self.definition.clone(),
                        self.store.snapshot(),
                        self.font.clone(),
                    )?;
                    self.chart = cx.new(|cx| ChartView::new(input, cx));
                    self.observation = cx.observe(&self.chart, |_, _, cx| cx.notify());
                    self.dimensions = if self.stage % 4 == 3 {
                        (600., 300.)
                    } else {
                        (800., 400.)
                    };
                } else {
                    assert!(
                        self.old.take().unwrap().upgrade().is_none(),
                        "unmounted chart must release"
                    );
                    self.released += 1;
                }
                self.status = format!(
                    "Repeated mount/resize/drop: {} old chart entities released",
                    self.released
                );
            }
            _ => {
                self.running = false;
                self.status = "PASS: frozen resize/recovery and 20 entity releases".into();
            }
        }
        self.stage += 1;
        self.log(cx, "after-step");
        cx.notify();
        Ok(())
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0xeaf0f6))
            .child("Native hardening: immutable freeze, destination resize, recovery and lifetime")
            .child(
                div()
                    .id("run")
                    .role(Role::Button)
                    .aria_label("Run native checks")
                    .p_2()
                    .bg(rgb(0xffffff))
                    .child("Run native checks")
                    .on_click(cx.listener(|s, _, _, cx| {
                        if s.stage < 48 {
                            s.running = true;
                            cx.notify();
                        }
                    })),
            )
            .child(
                div()
                    .id("step")
                    .role(Role::Button)
                    .aria_label("Next stage")
                    .p_2()
                    .bg(rgb(0xffffff))
                    .child("Next stage")
                    .on_click(cx.listener(|s, _, _, cx| {
                        if !s.running
                            && s.stage < 48
                            && let Err(e) = s.step(cx)
                        {
                            s.status = e.message;
                            cx.notify();
                        }
                    })),
            )
            .child(self.status.clone())
            .child(
                div()
                    .w(px(self.dimensions.0))
                    .h(px(self.dimensions.1))
                    .flex_none()
                    .child(self.chart.clone()),
            )
            .child(format!(
                "Stage {} · Released {} · Required original WP-21 scope",
                self.stage, self.released
            ))
    }
}
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let fixture: Value =
            serde_json::from_str(include_str!("../../../fixtures/live-export/replay.json"))
                .unwrap();
        let d: ChartEnvelope = serde_json::from_value(fixture["chart"].clone()).unwrap();
        let data: DataEnvelope = serde_json::from_value(fixture["data"].clone()).unwrap();
        let store = DataStore::new(
            data.epoch,
            data.datasets
                .into_iter()
                .map(|d| Ok((d.id, d.batch.into_batch()?)))
                .collect::<ChartResult<Vec<_>>>()
                .unwrap(),
            DataLimits::default(),
        )
        .unwrap();
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
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1050.), px(750.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Native Hardening".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    let input =
                        ChartInput::new(d.definition.clone(), store.snapshot(), font.clone())
                            .unwrap();
                    let chart = cx.new(|cx| ChartView::new(input, cx));
                    let observation = cx.observe(&chart, |_, _, cx| cx.notify());
                    let timer = cx.spawn(async move |entity, cx| {
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(750))
                                .await;
                            if entity
                                .update(cx, |s: &mut Gallery, cx| {
                                    if s.running
                                        && let Err(e) = s.step(cx)
                                    {
                                        s.running = false;
                                        s.status = e.message;
                                        s.log(cx, "FAILED");
                                        cx.notify();
                                    }
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
                    Gallery {
                        chart,
                        store,
                        definition: d.definition,
                        font,
                        dimensions: (700., 360.),
                        stage: 0,
                        running: false,
                        status: "Ready to run actual native checks".into(),
                        old: None,
                        released: 0,
                        observation,
                        _timer: timer,
                    }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
