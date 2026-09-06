//! WP-03 native path/text/resource/overlay experiment, deliberately outside public APIs.

#[path = "../../../fixtures/capability/support.rs"]
mod fixture;

use gpui::{
    App, Background, Bounds, ContentMask, Context, Entity, FocusHandle, KeyBinding, PathBuilder,
    Role, Window, WindowBounds, WindowOptions, actions, canvas, div, linear_color_stop,
    linear_gradient, point, prelude::*, px, rgb, size,
};
use gpui_kit::component::{
    Root,
    button::Button,
    input::{Input, InputState},
};
use resvg::{tiny_skia, usvg};
use std::{borrow::Cow, sync::Arc, time::Instant};

actions!(
    capability_spike,
    [
        /// Advance keyboard focus in the proof window.
        Next,
        /// Move keyboard focus backward in the proof window.
        Previous
    ]
);

struct Proof {
    tree: Arc<usvg::Tree>,
    input: Entity<InputState>,
    focus: FocusHandle,
    generation: usize,
    applied: String,
    profile_frames: usize,
}

impl Proof {
    fn new(tree: Arc<usvg::Tree>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Annotation text"));
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        eprintln!("lifecycle: mounted; scale_factor={}", window.scale_factor());
        Self {
            tree,
            input,
            focus,
            generation: 0,
            applied: "Ready".into(),
            profile_frames: if std::env::args().any(|arg| arg == "--profile") {
                40
            } else {
                0
            },
        }
    }
}

impl Drop for Proof {
    fn drop(&mut self) {
        eprintln!(
            "lifecycle: proof view dropped; generation={}",
            self.generation
        );
    }
}

fn color(color: usvg::Color, opacity: f32) -> gpui::Rgba {
    rgb((u32::from(color.red) << 16) | (u32::from(color.green) << 8) | u32::from(color.blue))
        .alpha(opacity)
}

fn paint(paint: &usvg::Paint, opacity: f32) -> Background {
    match paint {
        usvg::Paint::Color(c) => color(*c, opacity).into(),
        // This fixture intentionally has one horizontal, two-stop object-bounds gradient.
        usvg::Paint::LinearGradient(g) => {
            assert_eq!(g.stops().len(), 2, "unsupported gradient stops in proof");
            let a = &g.stops()[0];
            let b = &g.stops()[1];
            linear_gradient(
                90.0,
                linear_color_stop(
                    color(a.color(), a.opacity().get() * opacity),
                    a.offset().get(),
                ),
                linear_color_stop(
                    color(b.color(), b.opacity().get() * opacity),
                    b.offset().get(),
                ),
            )
            .color_space(gpui::ColorSpace::Srgb)
        }
        _ => panic!("unsupported fixture paint; do not silently substitute"),
    }
}

fn path(
    data: tiny_skia::Path,
    transform: tiny_skia::Transform,
    bounds: Bounds<gpui::Pixels>,
    scale: f32,
    background: Background,
    window: &mut Window,
) {
    let data = data.transform(transform).expect("finite fixture transform");
    let p = |p: tiny_skia::Point| {
        point(
            bounds.origin.x + px(p.x * scale),
            bounds.origin.y + px(p.y * scale),
        )
    };
    let mut builder = PathBuilder::fill().with_style(gpui::PathStyle::Fill(
        gpui::FillOptions::default().with_fill_rule(gpui::FillRule::NonZero),
    ));
    for segment in data.segments() {
        match segment {
            tiny_skia::PathSegment::MoveTo(a) => builder.move_to(p(a)),
            tiny_skia::PathSegment::LineTo(a) => builder.line_to(p(a)),
            tiny_skia::PathSegment::QuadTo(a, b) => builder.curve_to(p(b), p(a)),
            tiny_skia::PathSegment::CubicTo(a, b, c) => builder.cubic_bezier_to(p(c), p(a), p(b)),
            tiny_skia::PathSegment::Close => builder.close(),
        }
    }
    window.paint_path(
        builder.build().expect("fixture path tessellation"),
        background,
    );
}

fn group(
    group: &usvg::Group,
    parent: tiny_skia::Transform,
    bounds: Bounds<gpui::Pixels>,
    scale: f32,
    window: &mut Window,
) {
    let transform = parent.pre_concat(group.transform());
    assert!(
        group.filters().is_empty() && group.mask().is_none() && group.opacity().get() == 1.0,
        "unsupported group effect in fixture"
    );
    // Only the fixture's axis-aligned rectangular clip is supported by this experiment.
    let mask = group.clip_path().map(|clip| {
        assert_eq!(clip.id(), "panel");
        let r = clip.root().abs_bounding_box();
        ContentMask {
            bounds: Bounds::new(
                point(
                    bounds.origin.x + px(r.x() * scale),
                    bounds.origin.y + px(r.y() * scale),
                ),
                size(px(r.width() * scale), px(r.height() * scale)),
            ),
        }
    });
    window.with_content_mask(mask, |window| {
        for node in group.children() {
            match node {
                usvg::Node::Group(g) => self::group(g, transform, bounds, scale, window),
                usvg::Node::Text(t) => self::group(t.flattened(), transform, bounds, scale, window),
                usvg::Node::Path(p) => {
                    if !p.is_visible() {
                        continue;
                    }
                    if let Some(fill) = p.fill() {
                        path(
                            p.data().clone(),
                            transform,
                            bounds,
                            scale,
                            paint(fill.paint(), fill.opacity().get()),
                            window,
                        );
                    }
                    if let Some(stroke) = p.stroke() {
                        let options = stroke.to_tiny_skia();
                        let data = if let Some(dash) = &options.dash {
                            p.data().dash(dash, scale).expect("fixture dash")
                        } else {
                            p.data().clone()
                        };
                        let outline = data
                            .stroke(&options, scale)
                            .expect("fixture stroke outline");
                        path(
                            outline,
                            transform,
                            bounds,
                            scale,
                            paint(stroke.paint(), stroke.opacity().get()),
                            window,
                        );
                    }
                }
                _ => panic!("unsupported fixture node; no implicit raster fallback"),
            }
        }
    });
}

impl Render for Proof {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.profile_frames > 0 {
            self.profile_frames -= 1;
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
        let tree = self.tree.clone();
        div().id("proof").role(Role::Application).aria_label("Chart capability proof")
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &Next, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &Previous, window, cx| window.focus_prev(cx)))
            .size_full().bg(rgb(0xe8eef0)).text_color(rgb(0x203444)).font_family("Noto Sans")
            .flex().flex_col().p_4().gap_3()
            .child(div().flex().gap_3().items_center()
                .child(div().w(px(340.)).child(Input::new(&self.input)))
                .child(Button::new("apply").label("Apply annotation").on_click(cx.listener(|this, _, _, cx| {
                    this.applied = this.input.read(cx).value().to_string();
                    eprintln!("input: applied {:?}", this.applied); cx.notify();
                })))
                .child(Button::new("remount").label("Remount input").on_click(cx.listener(|this, _, window, cx| {
                    let weak = this.input.downgrade();
                    this.input = cx.new(|cx| InputState::new(window, cx).placeholder("Annotation text"));
                    this.generation += 1;
                    let generation = this.generation;
                    window.on_next_frame(move |window, _| window.on_next_frame(move |_, _| {
                        eprintln!("lifecycle: remount {generation}; old_input_released={}", weak.upgrade().is_none());
                    }));
                    cx.notify();
                }))))
            .child(div().id("status").role(Role::Status).aria_label(format!("Applied: {}; input generation {}", self.applied, self.generation))
                .child(format!("Applied: {}    |    Input generation: {}", self.applied, self.generation)))
            .child(div().id("figure").role(Role::Image).aria_label("Vector capability fixture: curves, clipped dashes, caps, gradient and rotated rich text")
                .flex_1().min_h_0().child(canvas(move |_,_,_| {}, move |bounds,_,window,_| {
                    let start = Instant::now();
                    let scale = (f32::from(bounds.size.width)/tree.size().width())
                        .min(f32::from(bounds.size.height)/tree.size().height());
                    if !scale.is_finite() || scale <= 0.0 { return; }
                    window.with_content_mask(Some(ContentMask { bounds }), |window| group(tree.root(), tiny_skia::Transform::identity(), bounds, scale, window));
                    eprintln!("native fixture path conversion + tessellation + submission: {:.3} ms; canvas={}x{} logical px; figure_width={:.2}; device_scale={}", start.elapsed().as_secs_f64()*1000.0, f32::from(bounds.size.width), f32::from(bounds.size.height), tree.size().width()*scale, window.scale_factor());
                }).size_full()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let tree = Arc::new(fixture::tree()?);
    eprintln!(
        "native cold fixture load + shape: {:.3} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_kit::init(cx);
        cx.text_system()
            .add_fonts(
                fixture::FONTS
                    .iter()
                    .map(|(_, bytes)| Cow::Borrowed(*bytes))
                    .collect(),
            )
            .expect("load explicit fonts");
        cx.bind_keys([
            KeyBinding::new("tab", Next, None),
            KeyBinding::new("shift-tab", Previous, None),
        ]);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(880.)),
                    cx,
                ))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("WP-03 Capability Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let proof = cx.new(|cx| Proof::new(tree, window, cx));
                cx.new(|cx| Root::new(proof, window, cx))
            },
        )
        .expect("open proof window");
        cx.on_window_closed(|cx, _| {
            eprintln!("lifecycle: window closed");
            cx.quit();
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}
