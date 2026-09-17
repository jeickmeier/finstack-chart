//! FIX-GG16 native recipe and target proof.
#[path = "../../common/ggplot_integration_controls.rs"]
mod fixtures;
#[path = "common/native_probe.rs"]
mod native_probe;
use gpui::{prelude::*, *};
use gpui_charts::{ChartInput, ChartView, NativeFont};
struct Gallery {
    charts: Vec<(String, Entity<ChartView>)>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let charts = self
            .charts
            .iter()
            .map(|(_, chart)| chart.clone())
            .collect::<Vec<_>>();
        let width = if self.charts.len() == 1 { 1440. } else { 360. };
        let height = if self.charts.len() == 1 { 640. } else { 320. };
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .child(native_probe::begin())
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .children(self.charts.iter().map(|(name, chart)| {
                        div()
                            .w(px(width))
                            .h(px(height + 24.))
                            .child(div().h(px(26.)).px(px(10.)).child(name.clone()))
                            .child(div().w(px(width)).h(px(height)).child(chart.clone()))
                    })),
            )
            .child(native_probe::end(&charts))
            .child(interaction_probe(&charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let modes = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v < fixtures::CASES)
        .map_or_else(|| (0..fixtures::CASES).collect::<Vec<_>>(), |v| vec![v]);
    let plots = modes
        .into_iter()
        .map(|mode| Ok((format!("Integration {mode}"), fixtures::author(mode)?)))
        .collect::<chart_core::ChartResult<Vec<_>>>()?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let inputs = plots
            .iter()
            .map(|(name, plot)| {
                (
                    name.to_owned(),
                    ChartInput::from_plot(plot, font.clone()).expect("scale input"),
                )
            })
            .collect::<Vec<_>>();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1440.), px(720.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("GGplot Integration Controls".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    charts: inputs
                        .into_iter()
                        .map(|(name, input)| (name, cx.new(|cx| ChartView::new(input, cx))))
                        .collect(),
                })
            },
        )
        .expect("native color window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}

fn interaction_probe(charts: &[Entity<ChartView>]) -> impl IntoElement {
    let charts: Vec<_> = charts.iter().map(Entity::downgrade).collect();
    canvas(|_,_,_|(), move |_,_,_,cx| {
        if std::env::var_os("GG18_PROGRAMMATIC_PROBE").is_some() {
            static PHASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            use std::sync::atomic::Ordering;
            if let Some(entity) = charts.first().and_then(WeakEntity::upgrade) {
                if PHASE.load(Ordering::Relaxed) == 0 && entity.read(cx).inspector().is_some() {
                    PHASE.store(1, Ordering::Relaxed);
                    cx.defer(move |cx| entity.update(cx, |chart, cx| {
                        use chart_core::{inspection::{InspectionAction,InputOrigin},state::{ChartAction,MarkTarget,SelectionChange,Viewport}};
                        let inspector = chart.inspector().unwrap();
                        let hit = inspector.semantic_targets().next().unwrap().clone();
                        let epoch = inspector.presented().prepared().source().get().unwrap().epoch();
                        let target = MarkTarget::from_inspected(&hit, epoch);
                        assert!(chart.dispatch_inspection(InspectionAction::Hover(Some(hit.position)), InputOrigin::Programmatic, cx).unwrap());
                        assert!(!chart.state().hover().is_empty());
                        chart.dispatch_chart(ChartAction::Select{change:SelectionChange::Replace,targets:vec![target]},cx).unwrap();
                        chart.dispatch_chart(ChartAction::SetViewport(Viewport{x:Some((-3.,3.)),y:None}),cx).unwrap();
                    }));
                } else if PHASE.load(Ordering::Relaxed) == 1 && entity.read(cx).capture_presented().is_ok_and(|c| c.state.viewport().x == Some((-3.,3.))) {
                    PHASE.store(2, Ordering::Relaxed);
                    cx.defer(move |cx| entity.update(cx, |chart, cx| {
                        let point = chart.inspector().unwrap().semantic_targets().next().unwrap().position;
                        chart.dispatch_inspection(chart_core::inspection::InspectionAction::Hover(Some(point)), chart_core::inspection::InputOrigin::Programmatic, cx).unwrap();
                    }));
                } else if PHASE.load(Ordering::Relaxed) == 2 {
                    let chart = entity.read(cx);
                    if let Ok(capture) = chart.capture_presented()
                        && capture.state.viewport().x == Some((-3.,3.)) && !capture.state.hover().is_empty() {
                            assert_eq!(capture.state.selection(), chart.state().selection());
                            assert_eq!(capture.state.hover(), chart.state().hover());
                            assert!(!capture.state.selection().is_empty());
                            assert!(!capture.state.hover().is_empty());
                            assert_eq!(capture.chart.scene().stamp().viewport, chart.state().viewport_revision());
                            println!("{}",serde_json::json!({"event":"integration-programmatic-PASS","stamp":capture.chart.scene().stamp(),"state_revision":capture.state.revision(),"selection":capture.state.selection(),"hover":capture.state.hover(),"viewport":capture.state.viewport()}));
                            PHASE.store(3,Ordering::Relaxed);
                    }
                }
            }
        }
        println!("{}",serde_json::json!({"event":"integration-interactions","charts":charts.iter().filter_map(WeakEntity::upgrade).map(|e|{
            let c=e.read(cx); let state=c.state();
            serde_json::json!({"stamp":c.inspector().map(|i|i.presented().scene().stamp()),"state_revision":state.revision(),"viewport":state.viewport(),"windows":state.axis_windows(),"selection":state.selection().iter().take(4).collect::<Vec<_>>(),"hover":state.hover().iter().take(4).collect::<Vec<_>>()})
        }).collect::<Vec<_>>()}));
    }).w(px(1.)).h(px(1.))
}
