//! Correlate chart paint stamps with benchmark-only drawable callbacks.
use gpui::{prelude::*, *};
use gpui_charts::ChartView;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
pub fn unix_ns() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}
pub fn begin() -> impl IntoElement {
    canvas(
        |_, _, _| {
            println!(
                "{}",
                json!({"event":"scene-frame-begin","unix_ns":unix_ns()})
            );
        },
        |_, _, _, _| {},
    )
    .w(px(1.))
    .h(px(1.))
}
pub fn end(charts: &[Entity<ChartView>]) -> impl IntoElement {
    let charts: Vec<_> = charts.iter().map(Entity::downgrade).collect();
    canvas(|_,_,_|(),move |_,_,_,cx| {
        println!("{}",json!({"event":"scene-frame-painted","unix_ns":unix_ns(),"charts":charts.iter().filter_map(WeakEntity::upgrade).map(|e|{let c=e.read(cx);json!({"stamp":c.inspector().map(|i|i.presented().scene().stamp()),"paints":c.metrics().paints,"layout_attempts":c.metrics().layout_attempts,"diagnostic":c.diagnostic()})}).collect::<Vec<_>>()}));
    }).w(px(1.)).h(px(1.))
}
