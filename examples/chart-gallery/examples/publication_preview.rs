//! Native vector preview of FigureSnapshot::preview_svg(), without native text remeasurement.
//! This bounded example accepts exporter paths, straight dashes, two-stop gradients and rectangular clips.
use gpui::{
    App, Bounds, ContentMask, Context, PathBuilder, Pixels, Render, Window, WindowBounds,
    WindowOptions, canvas, div, point, prelude::*, px, rgb, size,
};
use resvg::{tiny_skia, usvg};
use std::sync::Arc;
struct Outline {
    path: tiny_skia::Path,
    color: gpui::Background,
    clip: Option<tiny_skia::Rect>,
}
struct Preview {
    outlines: Arc<Vec<Outline>>,
    width: f32,
    height: f32,
}
fn outlines(
    group: &usvg::Group,
    parent: tiny_skia::Transform,
    inherited: Option<tiny_skia::Rect>,
    result: &mut Vec<Outline>,
) -> Result<(), String> {
    if !group.filters().is_empty() || group.mask().is_some() || group.opacity().get() != 1. {
        return Err("Unsupported publication preview group effect".into());
    }
    let transform = parent.pre_concat(group.transform());
    let clip = if let Some(c) = group.clip_path() {
        if c.clip_path().is_some() {
            return Err("Nested clip-path resource is unsupported in this preview".into());
        }
        let bounds = c
            .root()
            .abs_bounding_box()
            .transform(transform.pre_concat(c.transform()))
            .ok_or("Invalid clip transform")?;
        if let Some(previous) = inherited {
            let Some(intersection) = bounds.intersect(&previous) else {
                return Ok(());
            };
            Some(intersection)
        } else {
            Some(bounds)
        }
    } else {
        inherited
    };
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => outlines(g, transform, clip, result)?,
            usvg::Node::Path(p) => {
                if !p.is_visible() {
                    continue;
                }
                let color = |paint: &usvg::Paint, alpha: f32| -> Result<gpui::Background, String> {
                    let rgba = |c: usvg::Color, a: f32| {
                        rgb((u32::from(c.red) << 16)
                            | (u32::from(c.green) << 8)
                            | u32::from(c.blue))
                        .alpha(a)
                    };
                    match paint {
                        usvg::Paint::Color(c) => Ok(rgba(*c, alpha).into()),
                        usvg::Paint::LinearGradient(g) => {
                            if g.stops().len() != 2
                                || g.stops()[0].offset().get() != 0.
                                || g.stops()[1].offset().get() != 1.
                            {
                                return Err("Preview requires two endpoint gradient stops".into());
                            }
                            let mut a = tiny_skia::Point::from_xy(g.x1(), g.y1());
                            let mut b = tiny_skia::Point::from_xy(g.x2(), g.y2());
                            g.transform().map_point(&mut a);
                            g.transform().map_point(&mut b);
                            let bounds = p.data().bounds();
                            let near = |a: f32, b: f32| (a - b).abs() < 0.01;
                            let angle = if near(a.y, b.y)
                                && near(a.x, bounds.left())
                                && near(b.x, bounds.right())
                            {
                                90.
                            } else if near(a.x, b.x)
                                && near(a.y, bounds.top())
                                && near(b.y, bounds.bottom())
                            {
                                180.
                            } else {
                                return Err(
                                    "Preview gradient is not full-bounds horizontal/vertical"
                                        .into(),
                                );
                            };
                            let from = &g.stops()[0];
                            let to = &g.stops()[1];
                            Ok(gpui::linear_gradient(
                                angle,
                                gpui::linear_color_stop(
                                    rgba(from.color(), from.opacity().get() * alpha),
                                    0.,
                                ),
                                gpui::linear_color_stop(
                                    rgba(to.color(), to.opacity().get() * alpha),
                                    1.,
                                ),
                            )
                            .color_space(gpui::ColorSpace::Srgb))
                        }
                        _ => Err("Unsupported publication preview paint".into()),
                    }
                };
                if let Some(fill) = p.fill() {
                    result.push(Outline {
                        path: p
                            .data()
                            .clone()
                            .transform(transform)
                            .ok_or("Invalid outline transform")?,
                        color: color(fill.paint(), fill.opacity().get())?,
                        clip,
                    });
                }
                if let Some(stroke) = p.stroke() {
                    let style = stroke.to_tiny_skia();
                    let path = if let Some(dash) = &style.dash {
                        p.data().dash(dash, 1.).ok_or("Invalid dashed outline")?
                    } else {
                        p.data().clone()
                    };
                    result.push(Outline {
                        path: path
                            .stroke(&style, 1.)
                            .ok_or("Invalid stroke outline")?
                            .transform(transform)
                            .ok_or("Invalid stroke transform")?,
                        color: color(stroke.paint(), stroke.opacity().get())?,
                        clip,
                    });
                }
            }
            _ => return Err("Preview requires FigureSnapshot's font-free vector outlines".into()),
        }
    }
    Ok(())
}
struct Draw {
    path: gpui::Path<Pixels>,
    color: gpui::Background,
    clip: Option<Bounds<Pixels>>,
}
fn prepare(
    items: &[Outline],
    bounds: Bounds<Pixels>,
    width: f32,
    height: f32,
) -> Result<Vec<Draw>, String> {
    let scale = (f32::from(bounds.size.width) / width).min(f32::from(bounds.size.height) / height);
    let origin = point(
        bounds.origin.x + (bounds.size.width - px(width * scale)) / 2.,
        bounds.origin.y,
    );
    let point_at =
        |p: tiny_skia::Point| point(origin.x + px(p.x * scale), origin.y + px(p.y * scale));
    items
        .iter()
        .map(|item| {
            let mut builder = PathBuilder::fill().with_style(gpui::PathStyle::Fill(
                gpui::FillOptions::default().with_fill_rule(gpui::FillRule::NonZero),
            ));
            for segment in item.path.segments() {
                match segment {
                    tiny_skia::PathSegment::MoveTo(p) => builder.move_to(point_at(p)),
                    tiny_skia::PathSegment::LineTo(p) => builder.line_to(point_at(p)),
                    tiny_skia::PathSegment::QuadTo(a, b) => {
                        builder.curve_to(point_at(b), point_at(a))
                    }
                    tiny_skia::PathSegment::CubicTo(a, b, c) => {
                        builder.cubic_bezier_to(point_at(c), point_at(a), point_at(b))
                    }
                    tiny_skia::PathSegment::Close => builder.close(),
                }
            }
            Ok(Draw {
                path: builder.build().map_err(|e| e.to_string())?,
                color: item.color,
                clip: item.clip.map(|r| {
                    Bounds::new(
                        point(origin.x + px(r.x() * scale), origin.y + px(r.y() * scale)),
                        size(px(r.width() * scale), px(r.height() * scale)),
                    )
                }),
            })
        })
        .collect()
}
impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let items = self.outlines.clone();
        let (width, height) = (self.width, self.height);
        let preview = canvas(
            move |bounds, _, _| prepare(&items, bounds, width, height),
            |_, draws: Result<Vec<Draw>, String>, window, cx| match draws {
                Ok(draws) => {
                    for draw in draws {
                        window.with_content_mask(
                            draw.clip.map(|bounds| ContentMask { bounds }),
                            |window| window.paint_path(draw.path, draw.color),
                        );
                    }
                }
                Err(e) => {
                    eprintln!("preview preparation failed: {e}");
                    cx.quit();
                }
            },
        )
        .size_full();
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_3()
            .bg(rgb(0xe8eef3))
            .child(div().child("Publication preview · 180 × 120 mm · exact export outlines"))
            .child(div().flex_1().min_h_0().w_full().child(preview))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            let bundled = std::env::current_exe().ok().and_then(|p| {
                p.parent()?
                    .parent()
                    .map(|p| p.join("Resources/publication.svg"))
            });
            bundled.filter(|p| p.is_file()).unwrap_or_else(|| {
                std::path::PathBuf::from("artifacts/wp-08/publication-preview.svg")
            })
        });
    let data = std::fs::read(file)?;
    let tree = usvg::Tree::from_data(
        &data,
        &usvg::Options {
            dpi: 72.,
            ..Default::default()
        },
    )?;
    let mut items = vec![];
    outlines(
        tree.root(),
        tiny_skia::Transform::identity(),
        None,
        &mut items,
    )?;
    let (width, height) = (tree.size().width(), tree.size().height());
    println!(
        "preview: {width}x{height} points, {} numeric paths; no font measurement",
        items.len()
    );
    gpui_platform::application().run(move |cx: &mut App| {
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1000.), px(730.)),
                    cx,
                ))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Finstack Publication Preview".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Preview {
                    outlines: Arc::new(items),
                    width,
                    height,
                })
            },
        );
        if let Err(e) = result {
            eprintln!("preview window failed: {e}");
            cx.quit();
            return;
        }
        cx.on_window_closed(|cx, _| {
            println!("preview closed");
            cx.quit();
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}
