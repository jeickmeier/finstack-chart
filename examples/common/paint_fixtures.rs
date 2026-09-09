//! CLR-04: retained floating paint inputs shared by native and headless destinations.
use chart_core::{
    color::{self, ColorValue, Paint},
    prelude::*,
    scene::{GradientDirection, LinearGradient, Stroke},
    theme::{GeometryTheme, NamedTheme},
};
fn data() -> ChartResult<Data> {
    Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [1., 2., 1.5])
        .build()
}
fn ink() -> ColorValue {
    color::lab(28., 7., -18.).into()
}
fn heading(text: &str) -> TitleBuilder {
    title(text).style(text_style().color(ink()).size(1.15))
}
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let mut result = vec![];
    let single = Data::columns().column("one", [1.]).build()?;
    let mut swatches = plot(single)
        .title(heading("Color spaces at the paint boundary"))
        .x_axis(x_axis().scale(scale_linear().domain(0., 6.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 2.)).visible(false))
        .theme(
            theme()
                .preset(NamedTheme::Editorial)
                .style(style().gradient(LinearGradient {
                    direction: GradientDirection::Horizontal,
                    start: Paint::from(color::hsl(40., 0.4, 0.98)),
                    end: Paint::from(color::hsl(210., 0.3, 0.94)),
                })),
        );
    for (i, (name, value)) in [
        ("RGB", ColorValue::from(color::rgb(230.25, 55.75, 80.5))),
        ("HSL", color::hsl(155., 0.75, 0.38).into()),
        ("Lab", color::lab(65., 45., 50.).into()),
        ("HCL", color::hcl(270., 70., 60.).into()),
        ("Cubehelix", color::cubehelix(210., 1.3, 0.55).into()),
    ]
    .into_iter()
    .enumerate()
    {
        let x = (i + 1) as f64;
        swatches = swatches
            .layer(points().aes(aes().x(x).y(1.15)).color(value).size(19.))
            .layer(
                labels()
                    .id(name)
                    .at(x, 0.65)
                    .text(name)
                    .style(text_style().color(value)),
            );
    }
    result.push(("spaces", swatches.build()?));
    let ramp_data = Data::columns()
        .column("x", (0..=10).map(|i| i as f64 / 10.).collect::<Vec<_>>())
        .column("y", vec![1.; 11])
        .build()?;
    result.push((
        "continuous",
        plot(ramp_data)
            .aes(aes().x("x").y("y"))
            .layer(points().aes(aes().color("x").color_scale("ramp")).size(12.))
            .scale(
                color_continuous("ramp", 0., 1.)
                    .palette(vec![
                        Paint::from(color::lab(65., 70., 65.)),
                        Paint::from(color::hcl(240., 70., 65.)),
                    ])
                    .missing(color::hsl(0., 0., 0.5)),
            )
            .legend(legend().scale("ramp").title("Floating RGB"))
            .title(heading("One ramp for marks and guide stops"))
            .x_axis(x_axis().scale(scale_linear().domain(-0.1, 1.1)))
            .y_axis(y_axis().scale(scale_linear().domain(0., 2.)).visible(false))
            .theme(theme().preset(NamedTheme::Editorial))
            .build()?,
    ));
    let candles = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("open", [2., 4., 3., 5.])
        .column("close", [4., 3., 5., 4.])
        .column("low", [1., 2., 2., 3.])
        .column("high", [5., 5., 6., 6.])
        .build()?;
    result.push((
        "candles",
        plot(candles)
            .aes(aes().x("x").y("open").y2("close").low("low").high("high"))
            .layer(
                ohlc()
                    .width(12.)
                    .color(color::rgb(40.5, 45.5, 50.5))
                    .candle_colors(chart_core::grammar::CandleColors {
                        up: color::hcl(150., 55., 55.),
                        down: color::hcl(25., 65., 55.),
                    }),
            )
            .title(heading("Directional colors retain their space"))
            .x_axis(x_axis().scale(scale_linear().domain(0.5, 4.5)))
            .y_axis(y_axis().scale(scale_linear().domain(0., 7.)))
            .theme(
                theme()
                    .preset(NamedTheme::Editorial)
                    .geometry(GeometryTheme {
                        ink: Paint::from(ink()),
                        paper: Paint::from(color::gray(98.)),
                        accent: Paint::from(color::hcl(270., 50., 55.)),
                        point_size: 1.5,
                        line_width: 0.5,
                    }),
            )
            .build()?,
    ));
    let rich = |text: &str| rich_text(text).style(text_style().color(ink()));
    let mut path = path();
    path.move_to(0., 0.)?;
    path.line_to(30., 0.)?;
    path.line_to(15., 25.)?;
    path.close_path()?;
    result.push((
        "furniture",
        plot(data()?)
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .size(8.)
                    .style(style().mark(color::hsl(210., 0.7, 0.5))),
            )
            .layer(
                vector_path("triangle", path.geometry())
                    .fill(Some(color::lab(65., 50., 45.).opacity(0.55)))
                    .stroke(Some(Stroke {
                        color: color::hcl(20., 55., 45.),
                        width: 2.,
                    }))
                    .anchor(chart_core::composition::Anchor::Output { x: 310., y: 155. }),
            )
            .layer(
                labels()
                    .id("label")
                    .at(1.4, 1.5)
                    .text("Retained annotation")
                    .style(text_style().color(color::hcl(300., 40., 40.))),
            )
            .title(title("Rich text and authored paint").rich(rich("Rich text and authored paint")))
            .subtitle(
                subtitle("Shared color descriptors")
                    .style(text_style().color(color::hsl(210., 0.5, 0.35))),
            )
            .caption(
                caption("Caption / source / footnote")
                    .style(text_style().color(color::lab(40., 20., -20.))),
            )
            .source_note(
                source_note("Source: supplied values").style(text_style().color(color::gray(40.))),
            )
            .footnote(
                footnote("Final scene uses sRGB8")
                    .style(text_style().color(color::hcl(100., 20., 35.))),
            )
            .x_axis(
                x_axis()
                    .scale(scale_linear().domain(0.5, 3.5))
                    .rich_label(rich("Authored x")),
            )
            .y_axis(
                y_axis()
                    .scale(scale_linear().domain(0.5, 2.5))
                    .rich_label(rich("Authored y")),
            )
            .theme(
                theme().preset(NamedTheme::Editorial).style(
                    style()
                        .foreground(ink())
                        .annotation(ink())
                        .focus(color::hcl(90., 60., 70.))
                        .selection(color::hcl(300., 40., 60.))
                        .grid(color::gray(90.))
                        .panel(color::gray(99.)),
                ),
            )
            .build()?,
    ));
    Ok(result)
}
