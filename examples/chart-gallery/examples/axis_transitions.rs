//! AXIS-06 actual native clock, interruption, reduced-motion and disposal proof.
#[path = "../../common/axis_transition_fixtures.rs"]
mod fixtures;
use gpui::{prelude::*, *};
use gpui_charts::{ChartInput, ChartView, NativeFont};
use std::time::{Duration, Instant};
struct Gallery {
    chart: Entity<ChartView>,
    _task: Task<()>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .flex()
            .flex_col()
            .child(
                div()
                    .p_3()
                    .text_color(rgb(0x222222))
                    .child("Axis transitions: move, enter, exit, interrupt, reduced motion"),
            )
            .child(div().flex_1().w_full().child(self.chart.clone()))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plots = fixtures::sequence()?;
    gpui_platform::application().run(move |cx| {
        let font=NativeFont::from_bytes(include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),"Noto Sans",cx).expect("supplied font");
        let input=ChartInput::from_plot(&plots[0],font).expect("primary plot").guide_transition(Duration::from_secs(6));
        cx.open_window(WindowOptions {window_bounds:Some(WindowBounds::Windowed(Bounds::centered(None,size(px(940.),px(380.)),cx))),titlebar:Some(TitlebarOptions {title:Some("Axis Transition Proof".into()),..Default::default()}),..Default::default()},|_,cx| {
            let chart=cx.new(|cx|ChartView::new(input,cx));
            cx.new(|cx| {
                let observed=chart.clone();
                let task=cx.spawn(async move |_,cx| {
                    let start=Instant::now();let mut phase=0;let mut previous=None;let mut frozen:Option<chart_export::FigureRequest>=None;
                    loop {
                        cx.background_executor().timer(Duration::from_millis(100)).await;
                        let elapsed=start.elapsed().as_secs_f64();
                        let done=observed.update(cx,|chart,cx| {
                            let next=if elapsed>=16. {5} else if elapsed>=15. {4} else if elapsed>=13. {3} else if elapsed>=6. {2} else if elapsed>=3. {1} else {0};
                            if next!=phase {
                                phase=next;
                                match phase {
                                    1|2=>{let expected=chart.chart().definition().revision;chart.apply_plot(&plots[phase],expected,cx).expect("native edit");},
                                    3=>{cx.set_reduce_motion(true);let expected=chart.chart().definition().revision;chart.apply_plot(&plots[1],expected,cx).expect("reduced motion edit");},
                                    4=>chart.dispose_preparation(cx),
                                    _=>{}
                                }
                                println!("{}",serde_json::json!({"event":"phase","phase":phase,"elapsed":elapsed}));
                            }
                            if let Ok(capture)=chart.capture_presented() {
                                let frames=capture.chart.guide_presentation();
                                if phase==2 && elapsed>=8. && frozen.is_none() {
                                    let bounds=capture.chart.scene().bounds();
                                    let fonts=chart_export::FontResources::new(capture.fonts.iter().map(|(d,b)|chart_export::FontResource::new(*d,b.clone())).collect::<chart_core::ChartResult<Vec<_>>>().expect("captured font descriptors")).expect("captured fonts");
                                    let mut profile=chart_export::PublicationProfile::new(chart_export::PageSize::points(bounds.width(),bounds.height()).expect("native capture page"),capture.layout.font).expect("native capture policy");profile.dpi=144;
                                    let p=capture.chart.prepared();
                                    frozen=Some(chart_export::FigureRequest::new(p.definition().clone(),p.source().clone(),capture.state.clone(),fonts,profile,chart_core::state::InteractionCapture::ALL).expect("native capture request").with_extensions(capture.extensions.clone()).with_origin_layout(capture.layout.clone()).with_displayed_layout(capture.chart.clone()).expect("frozen native geometry"));
                                    println!("{}",serde_json::json!({"event":"capture-acquired","elapsed":elapsed,"guides":frames}));
                                }

                                let snapshot=serde_json::to_value(&frames).expect("sample metadata");
                                if previous.as_ref()!=Some(&snapshot) {
                                    println!("{}",serde_json::json!({"event":"painted","phase":phase,"elapsed":elapsed,"guides":snapshot,"layout_attempts":chart.metrics().layout_attempts,"paints":chart.metrics().paints,"error":chart.diagnostic()}));
                                    previous=Some(snapshot);
                                }
                                if phase==5 {
                                    assert!(chart.diagnostic().is_none());
                                    assert!(frames.iter().flat_map(|g|&g.frame.ticks).all(|t|t.opacity==1.));
                                    assert!(chart.scheduling_metrics().disposed);
                                    let figure=frozen.as_ref().expect("mid-transition native capture").prepare().expect("deferred native publication");
                                    let out=std::path::PathBuf::from("docs/evidence/phase-2-axis-transitions");
                                    std::fs::create_dir_all(&out).expect("evidence directory");
                                    for (name,format) in [("svg",chart_export::Format::Svg),("pdf",chart_export::Format::Pdf),("png",chart_export::Format::Png)] {std::fs::write(out.join(format!("native-captured.{name}")),figure.export(format).expect("native sampled publication").bytes).expect("save native capture");}
                                    std::fs::write(out.join("native-captured.presentation.json"),figure.presentation_json().expect("native sample metadata")).expect("save native metadata");
                                    assert!(figure.layout().guide_presentation()[0].frame.ticks.iter().any(|t|t.opacity<1.));

                                    println!("{}",serde_json::json!({"event":"PASS","elapsed":elapsed,"layout_attempts":chart.metrics().layout_attempts,"paints":chart.metrics().paints,"disposed":true}));
                                    return true;
                                }
                            }
                            false
                        });
                        if done {break;}
                    }
                });
                Gallery {chart,_task:task}
            })
        }).expect("native window");
        cx.on_window_closed(|cx,_|cx.quit()).detach();cx.activate(true);
    });
    Ok(())
}
