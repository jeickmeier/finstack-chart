//! WP-18 native ingestion, retention, follow/freeze and explicitly historical pin proof.
use chart_core::{
    portable::Session,
    services::*,
    state::{FollowMode, *},
    transaction::CommitOutcome,
    *,
};
use gpui::{prelude::*, *};
use gpui_charts::*;
use serde_json::{Value, json};
use std::sync::Arc;
const ORIGIN: i64 = 9007199254741001;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
struct Gallery {
    chart: Entity<ChartView>,
    session: Session,
    sequence: u64,
    status: String,
    removed: usize,
    _subscriptions: Vec<Subscription>,
}
impl Gallery {
    fn transaction(&mut self, mutation: Value) -> Value {
        self.sequence += 1;
        let source = self.session.source();
        let source = source.get().unwrap();
        json!({"version":1,"id":format!("native-stream-{}",self.sequence),"epoch":source.epoch(),"expected":[source.dataset(DatasetId::new(1)).unwrap().version()],"operations":[{"dataset":"1","mutation":mutation}]})
    }
    fn refresh(&mut self, cx: &mut Context<Self>) -> ChartResult<()> {
        self.session.prepare()?;
        self.chart
            .update(cx, |chart, cx| chart.set_data(self.session.source(), cx))
    }
    fn act(&mut self, label: &str, cx: &mut Context<Self>) {
        let result = (|| -> ChartResult<String> {
            match label {
                "Enqueue" => {
                    let fixture: Value = serde_json::from_str(include_str!(
                        "../../../fixtures/streaming/replay.json"
                    ))
                    .unwrap();
                    let mut batch = fixture["data"]["datasets"][0]["batch"].clone();
                    batch["keys"] = json!([(9007199254743101u64 + self.sequence).to_string()]);
                    batch["columns"][0]["values"] =
                        json!({"Timestamp":[(ORIGIN+25+self.sequence as i64*5).to_string()]});
                    batch["columns"][1]["values"] =
                        json!({"Float64":[25.+(self.sequence%4) as f64*5.]});
                    for c in batch["columns"].as_array_mut().unwrap() {
                        c["validity"] = json!([true]);
                    }
                    let tx = self.transaction(json!({"AppendBatch":batch}));
                    self.session
                        .stream(&json!({"version":1,"operation":{"Enqueue":tx}}).to_string())
                }
                "Commit" => {
                    let result = self
                        .session
                        .stream(r#"{"version":1,"operation":"CommitNext"}"#)?;
                    self.refresh(cx)?;
                    let result: Value = serde_json::from_str(&result).unwrap();
                    Ok(if result.is_null() {
                        "Queue is empty".into()
                    } else {
                        format!(
                            "Committed queue result: {}",
                            result["outcome"]
                                .as_object()
                                .unwrap()
                                .keys()
                                .next()
                                .unwrap()
                        )
                    })
                }
                "Window" => {
                    let tx=self.transaction(json!({"SetRetention":{"EventTime":{"field":"1","width":"15","allowed_lateness":"2","watermark":(ORIGIN+30).to_string(),"late":"Reject"}}}));
                    let result = self.session.apply_transaction(&tx.to_string())?;
                    let label = match &result {
                        CommitOutcome::Applied(r) => format!(
                            "Supplied watermark +30 ns; evicted {} rows",
                            r.operations[0].evicted
                        ),
                        _ => format!("{result:?}"),
                    };
                    self.refresh(cx)?;
                    Ok(label)
                }
                "Pin first" => {
                    let target = self
                        .chart
                        .read(cx)
                        .inspector()
                        .and_then(|i| {
                            i.semantic_targets().next().map(|hit| {
                                MarkTarget::from_inspected(
                                    hit,
                                    i.presented().prepared().source().get().unwrap().epoch(),
                                )
                            })
                        })
                        .ok_or_else(|| {
                            Diagnostic::error(
                                DiagnosticCode::MissingResource,
                                "No presented target.",
                                "Wait for the chart to appear.",
                            )
                        })?;
                    self.chart.update(cx, |chart, cx| {
                        chart.dispatch_chart(
                            ChartAction::Select {
                                change: SelectionChange::Replace,
                                targets: vec![target.clone()],
                            },
                            cx,
                        )?;
                        chart.dispatch_chart(ChartAction::SetPinned(Some(target)), cx)
                    })?;
                    Ok("Selected and pinned the first presented observation".into())
                }
                _ => {
                    let action = match label {
                        "Freeze" => ChartAction::SetFollow(FollowMode::FreezePresentation),
                        "Resume" => ChartAction::ResumeLatest,
                        "Inspect" => ChartAction::SetViewport(Viewport {
                            x: Some((5., 20.)),
                            y: None,
                        }),
                        _ => ChartAction::SetPinned(None),
                    };
                    self.chart
                        .update(cx, |chart, cx| chart.dispatch_chart(action, cx))?;
                    Ok(label.into())
                }
            }
        })();
        self.status = result.unwrap_or_else(|e| e.message);
        cx.notify();
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let queue: Value = serde_json::from_str(
            &self
                .session
                .stream(r#"{"version":1,"operation":"Status"}"#)
                .unwrap(),
        )
        .unwrap();
        let source = self.session.source();
        let source = source.get().unwrap();
        let chart = self.chart.read(cx);
        let visible = chart
            .inspector()
            .map(|i| i.presented().scene().stamp().store.get())
            .unwrap_or(0);
        let details = format!(
            "Latest rows {} · Committed {} · Presented {} · Queued {} / 1 · Active selection {} · Removed {} · {:?}",
            source.dataset(DatasetId::new(1)).unwrap().len(),
            source.revision().get(),
            visible,
            queue["queue"]["transactions"],
            chart.state().selection().len(),
            self.removed,
            chart.state().follow()
        );
        let pin = chart
            .pinned_description()
            .unwrap()
            .map(|p| {
                format!(
                    "{} pinned observation: {} · {}",
                    if p.historical {
                        "Historical"
                    } else {
                        "Current"
                    },
                    p.target.description,
                    p.target
                        .cells
                        .iter()
                        .map(|c| format!("{}: {}", c.field, c.value))
                        .collect::<Vec<_>>()
                        .join(" · ")
                )
            })
            .unwrap_or_else(|| "No retained pin".into());
        let viewport = format!(
            "Horizontal window: {:?} · Presented window: {:?}",
            chart.state().viewport().x,
            chart
                .inspector()
                .map(|i| i.presented().prepared().state().viewport().x)
        );
        let viewport = format!(
            "{viewport} · Axis {:?}",
            chart
                .inspector()
                .and_then(|i| i.presented().axes().get(&ScaleId::new(0)))
                .map(|a| match &a.scale {
                    chart_core::layout::ResolvedScale::Utc(s) => format!(
                        "{}..{}",
                        s.viewport().start - ORIGIN,
                        s.viewport().end - ORIGIN
                    ),
                    _ => "other".into(),
                })
        );
        let mut buttons = div().flex().gap_2();
        for label in [
            "Pin first",
            "Freeze",
            "Enqueue",
            "Commit",
            "Window",
            "Resume",
            "Inspect",
            "Unpin",
        ] {
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
                    .on_click(cx.listener(move |this, _, _, cx| this.act(label, cx))),
            );
        }
        div().size_full().flex().flex_col().gap_3().p_4().bg(rgb(0xecf1f8)).text_color(rgb(0x202b3c)).font_family("Noto Sans")
            .child(div().text_xl().child("Streaming retention and historical inspection"))
            .child("Enqueue accepts one transaction; Commit applies it. Window uses a supplied watermark. Freeze keeps the visible snapshot while ingestion continues.")
            .child(buttons).child(details).child(self.status.clone()).child(pin).child(viewport)
            .child(div().flex_1().min_h_0().w_full().bg(rgb(0xffffff)).child(self.chart.clone()))
    }
}
fn main() {
    gpui_platform::application().run(|cx:&mut App|{
    let font=NativeFont::load(ResourceDescriptor{id:ResourceId::new(1),revision:Revision::new(1),kind:ResourceKind::Font,byte_len:FONT.len() as u64},Arc::from(FONT),"Noto Sans",cx).unwrap();
    let fixture:Value=serde_json::from_str(include_str!("../../../fixtures/streaming/replay.json")).unwrap();
    let mut session=Session::new(&fixture["chart"].to_string(),&fixture["data"].to_string()).unwrap();
    session.stream(r#"{"version":1,"operation":{"ConfigureQueue":{"transactions":1,"rows":4,"bytes":4096,"overload":"Backpressure"}}}"#).unwrap();
    let input=ChartInput::new(session.definition().clone(),session.source(),font).unwrap();
    cx.open_window(WindowOptions{window_bounds:Some(WindowBounds::Windowed(Bounds::centered(None,size(px(1140.),px(760.)),cx))),titlebar:Some(TitlebarOptions{title:Some("Finstack Streaming Proof".into()),..Default::default()}),..Default::default()},|_,cx|cx.new(|cx|{
        let chart=cx.new(|cx|ChartView::new(input,cx));
        let subscriptions=vec![cx.subscribe(&chart,|this:&mut Gallery,_,event,cx|{if let ChartHostEvent::DataReconciled(r)=event{this.removed+=r.removed_selection.len();}cx.notify();}),cx.observe(&chart,|_,_,cx|cx.notify())];
        Gallery{chart,session,sequence:0,status:"Ready".into(),removed:0,_subscriptions:subscriptions}
    })).unwrap();cx.activate(true);
});
}
