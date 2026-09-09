//! SP-07: shared native/publication examples, with independently authored host counterparts.
use chart_core::{
    Revision,
    data::TimeUnit,
    interpolate::{Number, Value},
    layout::AxisSide,
    prelude::*,
    scales::*,
    theme::NamedTheme,
    typography::NumericFormat,
};
use std::sync::Arc;

fn values(v: &[&str]) -> Vec<Value> {
    v.iter().map(|v| Value::Text((*v).into())).collect()
}
fn input(v: &[f64]) -> Vec<ScaleInput> {
    v.iter().map(|v| ScaleInput::Number(Number(*v))).collect()
}
pub fn zone() -> CalendarZone {
    CalendarZone::Local(Arc::new(TimeZoneRules {
        version: 1,
        zone: "America/New_York".into(),
        revision: Revision::new(42),
        tzdata: "2025c".into(),
        coverage: TimeBounds {
            start: 1_672_531_200_000,
            end: 1_735_689_600_000,
        },
        initial_offset_seconds: -18000,
        transitions: vec![
            TimeZoneTransition {
                at_millis: 1_678_604_400_000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1_699_164_000_000,
                offset_seconds: -18000,
            },
            TimeZoneTransition {
                at_millis: 1_710_054_000_000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1_730_613_600_000,
                offset_seconds: -18000,
            },
        ],
    }))
}
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let mut result = vec![];
    let mut x = NumericScaleSpec::d3(NumericFamily::Linear);
    x.domain = [0., 10., 100.].map(Number).to_vec();
    x.range = [0., 50., 100.].map(Number).to_vec();
    let mut y = NumericScaleSpec::d3(NumericFamily::Pow { exponent: 0.5 });
    y.domain = [0., 16.].map(Number).to_vec();
    result.push((
        "numeric",
        plot(
            Data::columns()
                .name("numeric")
                .column("x", [0., 5., 10., 55., 100.])
                .column("y", [1., 4., 9., 16., 9.])
                .build()?,
        )
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points().size(4.))
        .x_axis(x_axis().scale(scale_numeric(x)))
        .y_axis(y_axis().scale(scale_numeric(y)))
        .axis(
            x_axis()
                .name("secondary")
                .side(AxisSide::Top)
                .secondary("x", 0.1, 0.)
                .numeric_format(NumericFormat {
                    specifier: ".1f".into(),
                    locale: Default::default(),
                }),
        )
        .title(title("Piecewise x, square-root y"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?,
    ));
    let ordinal = ScaleConstructor::Ordinal.create(ScaleOptions {
        range: Some(values(&["#28587b", "#c77c35"])),
        ..Default::default()
    })?;
    result.push((
        "categorical",
        plot(
            Data::columns()
                .name("categories")
                .column("category", categorical(["A", "B", "C", "A", "B", "C"]))
                .column(
                    "region",
                    categorical(["North", "North", "North", "South", "South", "South"]),
                )
                .column("value", [2., 5., 8., 20., 35., 50.])
                .build()?,
        )
        .aes(
            aes()
                .x("category")
                .y("value")
                .color("region")
                .color_scale("regions"),
        )
        .layer(points().size(7.))
        .x_axis(x_axis().scale(scale_band_d3(BandSpec {
            padding_inner: 0.3,
            padding_outer: 0.15,
            align: 0.2,
            round: true,
            ..Default::default()
        })))
        .scale(color_mapped(
            "regions",
            ordinal.mapped(ScaleTraining::Eligible)?,
        ))
        .legend(legend().scale("regions").title("Region"))
        .facet(facet_wrap("region").columns(2).free_y(true))
        .title(title("Bands with free y facets"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?,
    ));
    let quantile = ScaleConstructor::Quantile.create(ScaleOptions {
        range: Some(values(&["#abc9d8", "#4385a5", "#163c55"])),
        ..Default::default()
    })?;
    let size = ScaleConstructor::Threshold.create(ScaleOptions {
        domain: Some(vec![
            ScaleInput::Key(ScaleKey::Number(Number(5.))),
            ScaleInput::Key(ScaleKey::Number(Number(10.))),
        ]),
        range: Some([3., 6., 9.].map(Value::number).to_vec()),
        ..Default::default()
    })?;
    result.push((
        "classifier",
        plot(
            Data::columns()
                .name("distribution")
                .column("x", [0., 1., 2., 3., 4., 5., 6., 7., 8.])
                .column("value", [0., 0., 1., 2., 3., 5., 8., 13., 21.])
                .build()?,
        )
        .aes(
            aes()
                .x("x")
                .y("value")
                .color("value")
                .color_scale("quantiles"),
        )
        .layer(points().numeric_scale(
            NumericAesthetic::Size,
            "value",
            size.mapped(ScaleTraining::Authored)?,
        ))
        .scale(color_mapped(
            "quantiles",
            quantile.mapped(ScaleTraining::Eligible)?,
        ))
        .legend(legend().scale("quantiles").title("Sample terciles"))
        .title(title("Exact quantiles and threshold size"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?,
    ));
    let diverging = ScaleConstructor::Diverging.create(ScaleOptions {
        domain: Some(input(&[-10., 0., 100.])),
        range: Some(values(&["#bb4a43", "#f8f5ee", "#28587b"])),
        clamp: Some(true),
        ..Default::default()
    })?;
    let opacity = ScaleConstructor::Linear.create(ScaleOptions {
        domain: Some(input(&[-10., 100.])),
        range: Some([0.4, 1.].map(Value::number).to_vec()),
        clamp: Some(true),
        ..Default::default()
    })?;
    result.push((
        "diverging",
        plot(
            Data::columns()
                .name("diverging")
                .column("x", [-10., -5., 0., 25., 50., 75., 100.])
                .column("y", [1., 2., 3., 2., 1., 2., 3.])
                .build()?,
        )
        .aes(aes().x("x").y("y").color("x").color_scale("asymmetric"))
        .layer(points().size(9.).numeric_scale(
            NumericAesthetic::Opacity,
            "x",
            opacity.mapped(ScaleTraining::Authored)?,
        ))
        .scale(color_mapped(
            "asymmetric",
            diverging.mapped(ScaleTraining::Authored)?,
        ))
        .legend(legend().scale("asymmetric").title("Center = 0"))
        .title(title("An asymmetric diverging guide"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?,
    ));
    let start = 1_730_606_400_000;
    let time = TimeScaleSpec {
        domain: vec![start, start + 5 * 3_600_000],
        zone: zone(),
        ..Default::default()
    };
    result.push((
        "local-time",
        plot(
            Data::columns()
                .name("local-time")
                .column(
                    "time",
                    timestamps(
                        (0..=5).map(|i| start + i * 3_600_000).collect(),
                        TimeUnit::Milliseconds,
                        "UTC",
                    ),
                )
                .column("value", [1., 2., 1.5, 3., 2., 4.])
                .build()?,
        )
        .aes(aes().x("time").y("value"))
        .layer(line())
        .layer(points().size(3.))
        .x_axis(
            x_axis()
                .scale(
                    scale_calendar(time)
                        .calendar_interval(CalendarInterval::new(CalendarUnit::Hour)),
                )
                .time_format(TimeFormat {
                    pattern: Some("%H:%M %Z".into()),
                    ..Default::default()
                }),
        )
        .y_axis(y_axis().visible(false))
        .title(title("One elapsed hour through the fold"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?,
    ));
    Ok(result)
}
