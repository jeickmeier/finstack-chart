//! Primary data/plot/components reused directly by native and headless destinations.
use chart_core::prelude::*;
use chart_export::{Format, Output, PageSize, export_options};
use gpui::{
    AppContext, Context, Entity, IntoElement, Render, Window, WindowOptions, div, prelude::*, px,
    rgb,
};
use gpui_charts::{ChartInput, ChartView, NativeFont};

struct Gallery {
    chart: Entity<ChartView>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .p_4()
            .child(self.chart.clone())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = Data::columns()
        .column("day", [1., 2., 3., 1., 2., 3.])
        .column("value", [10., 12., 11., 8., 9., 10.])
        .column(
            "series",
            ["Alpha", "Alpha", "Alpha", "Beta", "Beta", "Beta"],
        )
        .build()?;
    let plot = plot(data)
        .aes(aes().x("day").y("value").group("series").color("series"))
        .layer(line())
        .layer(points().size(4.))
        .title(title("Daily observations"))
        .subtitle(subtitle("Shared native and publication authoring"))
        .x_axis(x_axis().label("Day"))
        .y_axis(y_axis().label("Value"))
        .legend(legend().scale("series").title("Series"))
        .layer(labels().at(2., 12.).text("Peak").offset(8., 8.))
        .caption(caption("Two series, one shared engine"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    let output = Output::new(bytes)?;
    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/authoring");
    std::fs::create_dir_all(&artifacts)?;
    for (format, extension) in [
        (Format::Svg, "svg"),
        (Format::Pdf, "pdf"),
        (Format::Png, "png"),
    ] {
        let artifact =
            output.export(&plot, format, export_options(PageSize::points(700., 420.)?))?;
        artifact.save(artifacts.join(format!("native-primary.{extension}")))?;
    }
    if std::env::args().any(|arg| arg == "--headless") {
        return Ok(());
    }
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let input = ChartInput::from_plot(&plot, font).expect("native plot")
            .accessible_summary("Daily observations for Alpha and Beta. Arrow keys inspect observations and Space selects them.")
            .expect("bounded summary");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                    None,
                    gpui::size(px(950.), px(640.)),
                    cx,
                ))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Primary Authoring API".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    chart: cx.new(|cx| ChartView::new(input, cx)),
                })
            },
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
