//! CLR-05 integrated perceptual ramps, alpha overlap and grayscale presentation.
use chart_core::{
    interpolate::{FactoryKind, InterpolationFactory, Value},
    prelude::*,
    scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    theme::NamedTheme,
};
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let mut figures = vec![];
    for (name, background, foreground, preset) in [
        ("light", "#ffffff", "#202020", NamedTheme::Editorial),
        ("dark", "#18212a", "#f0f0f0", NamedTheme::Editorial),
        ("grayscale", "#ffffff", "#202020", NamedTheme::Grayscale),
    ] {
        let background = chart_core::color::Paint::from_css(background)?;
        let foreground = chart_core::color::Paint::from_css(foreground)?;
        let data = Data::columns()
            .column(
                "x",
                (0..=20).map(|i| f64::from(i) / 20.).collect::<Vec<_>>(),
            )
            .build()?;
        let mut figure = plot(data)
            .title(
                title(format!("Perceptual color / {name}")).style(text_style().color(foreground)),
            )
            .x_axis(x_axis().scale(scale_linear().domain(-0.05, 1.05)))
            .y_axis(
                y_axis()
                    .scale(scale_linear().domain(0.4, 3.6))
                    .visible(false),
            )
            .theme(
                theme().preset(preset).style(
                    style()
                        .background(background)
                        .panel(background)
                        .foreground(foreground),
                ),
            );
        for (label, factory, y) in [
            ("Lab", FactoryKind::Lab, 3.),
            ("HCL", FactoryKind::Hcl, 2.),
            ("Cubehelix", FactoryKind::Cubehelix, 1.),
        ] {
            let scale = ScaleConstructor::Linear.create(ScaleOptions {
                range: Some(vec![
                    Value::Text("rgba(255, 30, 30, 0.5)".into()),
                    Value::Text("rgba(30, 90, 255, 0.5)".into()),
                ]),
                factory: Some(InterpolationFactory::new(factory)),
                ..Default::default()
            })?;
            figure = figure
                .layer(
                    points()
                        .aes(aes().x("x").y(y).color("x").color_scale(label))
                        .size(16.),
                )
                .layer(
                    labels()
                        .id(label)
                        .at(0.04, y + 0.32)
                        .text(label)
                        .style(text_style().color(foreground)),
                )
                .scale(color_mapped(label, scale.mapped(ScaleTraining::Authored)?))
                .legend(legend().scale(label).title(label));
        }
        figure = figure.legend(legend().scale("HCL").title("HCL / alpha 0.5"));
        figures.push((name, figure.build()?));
    }
    Ok(figures)
}
