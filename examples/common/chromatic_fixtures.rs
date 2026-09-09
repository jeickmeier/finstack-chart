//! FIX-21 named categorical/Brewer/lookup/diverging/cyclic chart compositions.
use chart_core::{
    interpolate::{InterpolationSpec, Number, Value},
    prelude::*,
    scales::{chromatic::*, *},
    theme::NamedTheme,
};
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let mut figures = vec![];
    for (name, id, size) in [
        ("categorical", SchemeId::Category10, None),
        ("brewer", SchemeId::Blues, Some(5)),
    ] {
        let scale = ScaleConstructor::Ordinal.create(ScaleOptions {
            range: Some(vec![Value::Text("black".into())]),
            ..Default::default()
        })?;
        let data = Data::columns()
            .column("x", (0..10).map(f64::from).collect::<Vec<_>>())
            .column("group", categorical((0..10).map(|i| format!("C{i}"))))
            .build()?;
        let p = plot(data)
            .aes(aes().x("x").y(1.).color("group").color_scale("catalog"))
            .layer(points().size(13.))
            .scale(
                color_mapped("catalog", scale.mapped(ScaleTraining::Eligible)?)
                    .palette_scheme(SchemeSpec::new(id, size, false)),
            )
            .legend(legend().scale("catalog").title(if size.is_some() {
                "Blues / exact k = 5"
            } else {
                "Category10"
            }))
            .y_axis(y_axis().visible(false))
            .title(title(format!("Named palette / {name}")))
            .theme(theme().preset(NamedTheme::Editorial))
            .build()?;
        figures.push((name, p));
    }
    for (name, id, domain, constructor) in [
        (
            "lookup",
            InterpolatorId::Viridis,
            vec![0., 1.],
            ScaleConstructor::Sequential,
        ),
        (
            "diverging",
            InterpolatorId::RdBu,
            vec![-10., 0., 100.],
            ScaleConstructor::Diverging,
        ),
        (
            "cyclic",
            InterpolatorId::Rainbow,
            vec![0., 1.],
            ScaleConstructor::Sequential,
        ),
    ] {
        let values = if name == "diverging" {
            vec![-10., -8., -6., -4., -2., 0., 20., 40., 60., 80., 100.]
        } else {
            (0..=32).map(|i| f64::from(i) / 32.).collect()
        };
        let scale = constructor.create(ScaleOptions {
            domain: Some(
                domain
                    .into_iter()
                    .map(|v| ScaleInput::Number(Number(v)))
                    .collect(),
            ),
            interpolator: Some(InterpolationSpec::Chromatic {
                spec: ChromaticSpec { id, reverse: false },
            }),
            ..Default::default()
        })?;
        let p = plot(Data::columns().column("x", values).build()?)
            .aes(aes().x("x").y(1.).color("x").color_scale("catalog"))
            .layer(points().size(13.))
            .scale(color_mapped(
                "catalog",
                scale.mapped(ScaleTraining::Authored)?,
            ))
            .legend(legend().scale("catalog").title(id.name()))
            .y_axis(y_axis().visible(false))
            .title(title(format!("Named ramp / {name}")))
            .theme(theme().preset(NamedTheme::Editorial))
            .build()?;
        figures.push((name, p));
    }
    Ok(figures)
}
