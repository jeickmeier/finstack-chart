//! Primary authoring gallery: retained chart edits, native painting and inspection.
use chart_core::data::*;
use chart_core::grammar::ValueSpace;
use chart_core::inspection::Inspector;
use chart_core::prelude::{Data, LayerBuilder, Plot, aes, histogram, line, plot, points};
use chart_core::provenance::{ResolvedTarget, Target};
use chart_core::state::{ChartAction, Viewport};
use chart_core::*;
use gpui::{
    App, Bounds, Context, Entity, IntoElement, Render, Role, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use gpui_charts::{ChartInput, ChartView, NativeFont};
use std::{rc::Rc, sync::Arc};
const ORIGIN: i64 = 1_709_164_800_000_000_000;
fn source() -> ChartResult<Data> {
    Data::rows(0..21u32)
        .field("x", |n| f64::from(*n))
        .field("value", |n| {
            if (8..=10).contains(n) {
                None
            } else {
                Some((f64::from(*n) * 0.55).sin() * 3. + 5.)
            }
        })
        .timestamp(
            "time",
            |n| ORIGIN + i64::from(*n) * 17,
            TimeUnit::Nanoseconds,
            "UTC",
        )
        .keys(|n| u64::from(*n) + 1)
        .build()
}
fn mark(mode: usize) -> LayerBuilder {
    match mode {
        1 => points().aes(aes().x("x").y("value")),
        2 => histogram()
            .aes(aes().x("x"))
            .breaks(vec![0., 4., 8., 12., 16., 20.]),
        3 => line().aes(aes().x("time").y("value")),
        _ => line().aes(aes().x("x").y("value")),
    }
}
fn tooltip(fields: [FieldId; 3], i: &Inspector, _: &mut Window, _: &mut App) -> gpui::AnyElement {
    let mut labels = vec![];
    for hit in i.hits() {
        let label = match hit.target.resolve(
            i.presented()
                .prepared()
                .source()
                .get()
                .expect("owned snapshot"),
        ) {
            Ok(ResolvedTarget::Source(row)) => {
                let x = match row.value(fields[0]) {
                    Some(ValueRef::Float64(v)) => format!("{v:.0}"),
                    _ => "—".into(),
                };
                let y = match row.value(fields[1]) {
                    Some(ValueRef::Float64(v)) => format!("{v:.2}"),
                    _ => "—".into(),
                };
                let coordinate = if i
                    .presented()
                    .axes()
                    .values()
                    .any(|a| matches!(a.space, ValueSpace::Timestamp { .. }))
                {
                    match row.value(fields[2]) {
                        Some(ValueRef::Timestamp(t)) => chart_core::scales::format_utc(
                            t,
                            TimeUnit::Nanoseconds,
                            chart_core::scales::UtcInterval::Ticks(1),
                        )
                        .unwrap_or_else(|_| "time unavailable".into()),
                        _ => "time unavailable".into(),
                    }
                } else {
                    format!("x {x}")
                };
                format!("Observation {} · {coordinate} · value {y}", row.key().get())
            }
            Ok(ResolvedTarget::Aggregate { members, .. }) => {
                format!("Histogram bin · {} observations", members.len())
            }
            _ => "Source unavailable".into(),
        };
        eprintln!(
            "inspection: stamp={:?} target={:?} label={label}",
            i.presented().scene().stamp(),
            match &hit.target {
                Target::Source(s) => format!("row {}", s.key.get()),
                _ => "aggregate".into(),
            }
        );
        labels.push(label);
    }
    div()
        .id("inspection-status")
        .role(Role::Status)
        .text_color(rgb(0x1e394b))
        .children(labels.into_iter().map(|s| div().child(s)))
        .into_any_element()
}
struct Gallery {
    chart: Entity<ChartView>,
    source: Data,
    plot: Plot,
    font: NativeFont,
    mode: usize,
    generation: usize,
    compact: bool,
    validation: Option<String>,
}
impl Gallery {
    fn new(source: Data, font: NativeFont, cx: &mut Context<Self>) -> ChartResult<Self> {
        let plot = plot(source.clone())
            .layer(mark(0).name("observations"))
            .build()?;
        let chart = Self::mount(&plot, &source, &font, cx)?;
        Ok(Self {
            chart,
            source,
            plot,
            font,
            mode: 0,
            generation: 0,
            compact: false,
            validation: None,
        })
    }
    fn mount(
        plot: &Plot,
        source: &Data,
        font: &NativeFont,
        cx: &mut Context<Self>,
    ) -> ChartResult<Entity<ChartView>> {
        let fields = [
            source.field("x")?.id(),
            source.field("value")?.id(),
            source.field("time")?.id(),
        ];
        let input = ChartInput::from_plot(plot, font.clone())?
            .tooltip(Rc::new(move |i, w, cx| tooltip(fields, i, w, cx)));
        Ok(cx.new(|cx| ChartView::new(input, cx)))
    }
    fn select(&mut self, mode: usize, cx: &mut Context<Self>) {
        let candidate = self.plot.edit().layer("observations", mark(mode)).build();
        match candidate.and_then(|candidate| {
            self.chart.update(cx, |chart, cx| {
                chart.apply_plot(&candidate, chart.chart().definition().revision, cx)?;
                chart.dispatch_chart(ChartAction::Reset, cx)?;
                Ok(())
            })?;
            Ok(candidate)
        }) {
            Ok(plot) => {
                self.plot = plot;
                self.mode = mode;
            }
            Err(error) => eprintln!("fixture selection: {error}"),
        }
        cx.notify();
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut buttons = div().flex().gap_2();
        for (mode, label) in ["Line + gaps", "Scatter", "Histogram", "UTC precision"]
            .into_iter()
            .enumerate()
        {
            buttons = buttons.child(
                div()
                    .id(label)
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(if self.mode == mode {
                        rgb(0x244c68)
                    } else {
                        rgb(0xe4edf3)
                    })
                    .text_color(if self.mode == mode {
                        rgb(0xffffff)
                    } else {
                        rgb(0x244c68)
                    })
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| this.select(mode, cx))),
            );
        }
        let zoom = div()
            .id("zoom")
            .cursor_pointer()
            .child("Zoom")
            .on_click(cx.listener(|this, _, _, cx| {
                this.chart.update(cx, |c, cx| {
                    let domain = if this.mode == 3 {
                        (68., 272.)
                    } else {
                        (4., 16.)
                    };
                    let _ = c.dispatch_chart(
                        ChartAction::SetViewport(Viewport {
                            x: Some(domain),
                            y: None,
                        }),
                        cx,
                    );
                });
            }));
        let restore = div()
            .id("restore")
            .cursor_pointer()
            .child("Restore")
            .on_click(cx.listener(|this, _, _, cx| {
                this.chart.update(cx, |c, cx| {
                    let mut r = c.layout_request().clone();
                    r.font = this.font.descriptor();
                    let _ = c.set_layout(r, cx);
                    let _ = c.dispatch_chart(ChartAction::Reset, cx);
                    let _ = c.apply_plot(&this.plot, c.chart().definition().revision, cx);
                });
                this.compact = false;
                cx.notify();
            }));
        let malformed = div()
            .id("malformed")
            .cursor_pointer()
            .child("Invalid input")
            .on_click(cx.listener(|this, _, _, cx| {
                let invalid = this
                    .plot
                    .edit()
                    .layer("observations", points().aes(aes().x("absent").y("value")))
                    .build();
                this.validation = invalid.err().map(|e| e.message);
                cx.notify();
            }));
        let missing_font = div()
            .id("font-error")
            .cursor_pointer()
            .child("Missing font")
            .on_click(cx.listener(|this, _, _, cx| {
                this.chart.update(cx, |c, cx| {
                    let mut r = c.layout_request().clone();
                    r.font.id = ResourceId::new(999);
                    let _ = c.set_layout(r, cx);
                });
            }));
        let compact = div()
            .id("compact")
            .cursor_pointer()
            .child("Tiny bounds")
            .on_click(cx.listener(|this, _, _, cx| {
                this.compact = !this.compact;
                cx.notify();
            }));
        let remount = div().id("remount").cursor_pointer().child("Remount")
            .on_click(cx.listener(|this, _, window, cx| {
                let weak = this.chart.downgrade();
                let snapshot = this.chart.read(cx).inspector().map(|i| Arc::downgrade(i.presented()));
                this.generation += 1;
                let generation = this.generation;
                if let Ok(chart) = Self::mount(&this.plot,&this.source, &this.font, cx) {
                    this.chart = chart;
                }
                window.on_next_frame(move |window, _| {
                    window.on_next_frame(move |_, _| {
                        eprintln!(
                            "lifecycle: remount={generation}; old_entity_released={}; old_layout_released={}",
                            weak.upgrade().is_none(), snapshot.is_none_or(|s| s.upgrade().is_none()),
                        );
                    });
                });
                cx.notify();
            }));
        let report = div().id("report").cursor_pointer().child("Report")
            .on_click(cx.listener(|this, _, _, cx| {
                let c = this.chart.read(cx);
                eprintln!(
                    "report: mode={} generation={} metrics={:?} candidates={} stamp={:?} status={:?} error={:?}",
                    this.mode, this.generation, c.metrics(), c.inspector().map_or(0, Inspector::candidate_count),
                    c.inspector().map(|i| i.presented().scene().stamp()),
                    c.inspector().map(|i| i.presented().status()), c.diagnostic(),
                );
            }));
        let controls = div()
            .flex()
            .gap_3()
            .child(zoom)
            .child(restore)
            .child(malformed)
            .child(missing_font)
            .child(compact)
            .child(remount)
            .child(report);

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_6()
            .gap_4()
            .bg(rgb(0xf5f8fb))
            .text_color(rgb(0x203b4c))
            .font_family("Noto Sans")
            .child(div().text_xl().child("Native chart gallery"))
            .child(div().child(
                "Move over marks to inspect · Click chart, then use arrow keys · Escape clears",
            ))
            .child(buttons)
            .child(controls)
            .child(div().child(self.validation.clone().unwrap_or_default()))
            .child(if self.compact {
                div()
                    .w(px(20.))
                    .h(px(20.))
                    .bg(rgb(0xffffff))
                    .child(self.chart.clone())
            } else {
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .bg(rgb(0xffffff))
                    .rounded_lg()
                    .child(self.chart.clone())
            })
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = source()?;
    gpui_platform::application().run(move |cx| {
        let bytes: Arc<[u8]> = Arc::from(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
        );
        let font = match NativeFont::from_bytes(bytes, "Noto Sans", cx) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Font setup: {e}");
                cx.quit();
                return;
            }
        };
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1000.), px(680.)),
                    cx,
                ))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Finstack Native Charts".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Gallery::new(source, font, cx).expect("valid fixture")),
        );
        if let Err(e) = opened {
            eprintln!("Window setup failed: {e}");
            cx.quit();
            return;
        }
        cx.on_window_closed(|cx, _| {
            eprintln!("lifecycle: window closed");
            cx.quit();
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}
