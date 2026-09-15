//! SP-04 mapped color/numeric styles, real guide intervals and retained quantile updates.
use chart_core::{
    interpolate::*,
    prelude::*,
    scales::*,
    scene::Color,
    state::{ChartAction, Viewport},
};
use chart_export::*;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color {
        red: r,
        green: g,
        blue: b,
        alpha: a,
    }
}
fn colors(values: &[&str]) -> Vec<Value> {
    values.iter().map(|v| Value::Text((*v).into())).collect()
}
fn mapped_diverging(output: ScaleRangeFunction) -> MappedScaleSpec {
    MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Diverging {
            family: NumericFamily::Linear,
            domain: [Number(-10.), Number(0.), Number(100.)],
            clamp: true,
        },
        output,
        unknown: Value::Missing,
    }))
}
#[test]
fn named_mappings_use_independent_midpoint_radius_opacity_and_width() {
    let color = mapped_diverging(ScaleRangeFunction::Interpolate(
        InterpolationSpec::Piecewise {
            factory: InterpolationFactory::new(FactoryKind::Value),
            values: colors(&["red", "white", "blue"]),
        },
    ));
    let size = MappedScaleSpec::authored(ScaleFunctionSpec::Threshold(ThresholdSpec {
        domain: vec![ScaleKey::Number(Number(0.))],
        range: vec![Value::number(2.), Value::number(6.)],
        unknown: None,
    }));
    let opacity = mapped_diverging(ScaleRangeFunction::Identity);
    let width = MappedScaleSpec::authored(ScaleFunctionSpec::Continuous(ContinuousScaleSpec {
        family: NumericFamily::Linear,
        domain: vec![Number(-10.), Number(100.)],
        range: vec![Value::number(1.), Value::number(12.)],
        factory: InterpolationFactory::new(FactoryKind::Number),
        clamp: true,
        unknown: Value::Missing,
    }));
    let p = plot(
        Data::columns()
            .column("x", [0., 1., 2.])
            .column("v", [-10., 0., 100.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("v").color("v").color_scale("diverging"))
    .scale(color_mapped("diverging", color))
    .layer(
        points()
            .numeric_scale(NumericAesthetic::Size, "v", size)
            .numeric_scale(NumericAesthetic::Opacity, "v", opacity)
            .numeric_scale(NumericAesthetic::StrokeWidth, "v", width),
    )
    .build()
    .unwrap();
    let json = p.to_json().unwrap();
    let mut wire: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(wire["version"], 5);
    let restored = Plot::from_json(&json).unwrap();
    assert_eq!(restored.to_json().unwrap(), json);
    wire["version"] = 4.into();
    assert!(Plot::from_json(&wire.to_string()).is_err());
    let f = Output::new(FONT)
        .unwrap()
        .request(&p, export_options(PageSize::points(500., 400.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let layer = &f.layout().prepared().layers()[0];
    assert_eq!(layer.marks().len(), 3);
    assert_eq!(
        layer
            .marks()
            .iter()
            .map(|m| m.style.color)
            .collect::<Vec<_>>(),
        [
            rgba(255, 0, 0, 0),
            rgba(255, 255, 255, 128),
            rgba(0, 0, 255, 255)
        ]
    );
    assert_eq!(
        layer
            .marks()
            .iter()
            .map(|m| m.style.radius)
            .collect::<Vec<_>>(),
        [2., 6., 6.]
    );
    assert_eq!(
        layer
            .marks()
            .iter()
            .map(|m| m.style.stroke_width)
            .collect::<Vec<_>>(),
        [1., 2., 12.]
    );
    assert_eq!(layer.color_legend().unwrap().midpoint, Some(Number(0.)));
    assert_eq!(
        layer.color_legend().unwrap().entries[1].1,
        rgba(255, 255, 255, 255)
    );
    assert!(!f.export(Format::Svg).unwrap().bytes.is_empty());
}
fn data(name: &str, values: [f64; 2]) -> Data {
    Data::columns()
        .name(name)
        .column("x", [0., 1.])
        .column("v", values)
        .build()
        .unwrap()
}
fn quantile_plot(a: Data, b: Data) -> Plot {
    let scale = MappedScaleSpec {
        colorbar_options: None,
        palette_theme_aesthetics: vec![],
        oob_function: None,
        rescaler_function: None,
        palette_fallback_indices: vec![],
        missing_paint_is_na: false,
        palette_function: None,
        resolved_numeric_limits: None,
        resolved_discrete_limits_null: false,
        trained_transformed_bounds: None,
        limits_function: None,
        breaks_function: None,
        guide: None,
        ggplot: None,
        catalog: None,
        function: ScaleFunctionSpec::Classifier(ClassifierSpec {
            domain: ClassifierDomain::Quantile(vec![]),
            range: colors(&["red", "blue"]),
            unknown: None,
        }),
        training: ScaleTraining::Eligible,
    };
    plot(a)
        .aes(aes().x("x").y("v").color("v").color_scale("q"))
        .scale(color_mapped("q", scale))
        .layer(points())
        .layer(points().data(b))
        .build()
        .unwrap()
}
fn paint_snapshot(f: &FigureSnapshot) -> Vec<Vec<Color>> {
    f.layout()
        .prepared()
        .layers()
        .iter()
        .map(|l| l.marks().iter().map(|m| m.style.color).collect())
        .collect()
}
#[test]
fn corrected_shared_quantile_population_matches_fresh_batch_and_ignores_zoom() {
    let p = quantile_plot(data("a", [0., 10.]), data("b", [20., 100.]));
    let mut chart = Chart::new(p).unwrap();
    let output = Output::new(FONT).unwrap();
    let options =
        export_options(PageSize::points(500., 400.).unwrap()).basis(CaptureBasis::Current);
    let old = output
        .live_request(&chart, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let old_svg = old.export(Format::Svg).unwrap().bytes;
    let red = rgba(255, 0, 0, 255);
    let blue = rgba(0, 0, 255, 255);
    assert_eq!(paint_snapshot(&old), vec![vec![red, red], vec![blue, blue]]);
    let guide = old.layout().prepared().layers()[0].color_legend().unwrap();
    assert_eq!(
        guide.intervals[0].upper,
        Some(ScaleKey::Number(Number(15.)))
    );
    assert_eq!(
        guide,
        old.layout().prepared().layers()[1].color_legend().unwrap()
    );
    let transaction = chart
        .transaction()
        .unwrap()
        .replace("a", data("a", [0., 90.]))
        .build()
        .unwrap();
    chart.apply_transaction(transaction).unwrap();
    let current = output
        .live_request(&chart, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let fresh = output
        .request(
            &quantile_plot(data("a", [0., 90.]), data("b", [20., 100.])),
            options.clone(),
        )
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(paint_snapshot(&current), paint_snapshot(&fresh));
    assert_eq!(
        paint_snapshot(&current),
        vec![vec![red, blue], vec![red, blue]]
    );
    let guide = current.layout().prepared().layers()[0]
        .color_legend()
        .unwrap();
    assert_eq!(
        guide.intervals[0].upper,
        Some(ScaleKey::Number(Number(55.)))
    );
    chart
        .act(ChartAction::SetViewport(Viewport {
            x: Some((0., 0.5)),
            y: None,
        }))
        .unwrap();
    let zoom = output
        .live_request(&chart, options)
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(paint_snapshot(&zoom), paint_snapshot(&current));
    assert_eq!(
        zoom.layout().prepared().layers()[0].color_legend(),
        Some(guide)
    );
    assert_eq!(old.export(Format::Svg).unwrap().bytes, old_svg);
}
#[test]
fn post_stat_training_and_exact_integer_threshold_keys_reach_the_chart() {
    let quantile = |range| MappedScaleSpec {
        colorbar_options: None,
        palette_theme_aesthetics: vec![],
        oob_function: None,
        rescaler_function: None,
        palette_fallback_indices: vec![],
        missing_paint_is_na: false,
        palette_function: None,
        resolved_numeric_limits: None,
        resolved_discrete_limits_null: false,
        trained_transformed_bounds: None,
        limits_function: None,
        breaks_function: None,
        guide: None,
        ggplot: None,
        catalog: None,
        function: ScaleFunctionSpec::Classifier(ClassifierSpec {
            domain: ClassifierDomain::Quantile(vec![]),
            range,
            unknown: None,
        }),
        training: ScaleTraining::Eligible,
    };
    let p = plot(
        Data::columns()
            .column("v", [0., 100., 10., 20.])
            .column("g", ["A", "A", "B", "C"])
            .build()
            .unwrap(),
    )
    .transform(transform("means", summary().x("v").group("g")))
    .scale(color_mapped("means", quantile(colors(&["red", "blue"]))))
    .layer(
        points()
            .from_transform("means")
            .after_stat(
                stat_aes()
                    .x(StatField::Group)
                    .y(StatField::Mean)
                    .color(StatField::Mean)
                    .color_scale("means"),
            )
            .numeric_scale(
                NumericAesthetic::Size,
                StatField::Mean,
                quantile(vec![Value::number(2.), Value::number(8.)]),
            ),
    )
    .build()
    .unwrap();
    let f = Output::new(FONT)
        .unwrap()
        .request(&p, export_options(PageSize::points(500., 400.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let layer = &f.layout().prepared().layers()[0];
    assert_eq!(
        layer.color_legend().unwrap().intervals[0].upper,
        Some(ScaleKey::Number(Number(20.)))
    );
    assert_eq!(
        layer
            .marks()
            .iter()
            .map(|m| m.style.radius)
            .collect::<Vec<_>>(),
        [8., 2., 8.]
    );
    let cut = u64::MAX - 1;
    let scale = MappedScaleSpec::authored(ScaleFunctionSpec::Threshold(ThresholdSpec {
        domain: vec![ScaleKey::Unsigned(cut)],
        range: colors(&["red", "blue"]),
        unknown: None,
    }));
    let p = plot(
        Data::columns()
            .column("x", [0., 1., 2.])
            .column("v", [cut - 1, cut, cut + 1])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y(1.).color("v").color_scale("exact"))
    .scale(color_mapped("exact", scale))
    .layer(points())
    .build()
    .unwrap();
    let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let f = Output::new(FONT)
        .unwrap()
        .request(&p, export_options(PageSize::points(500., 400.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(
        paint_snapshot(&f),
        vec![vec![
            rgba(255, 0, 0, 255),
            rgba(0, 0, 255, 255),
            rgba(0, 0, 255, 255)
        ]]
    );
    assert_eq!(
        f.layout().prepared().layers()[0]
            .color_legend()
            .unwrap()
            .intervals[0]
            .upper,
        Some(ScaleKey::Unsigned(cut))
    );
}
#[test]
fn opacity_multiplies_floating_alpha_before_the_single_paint_boundary() {
    let floating =
        chart_core::color::ColorValue::from(chart_core::color::rgb(100.4, 50.4, 25.4).opacity(0.7));
    for lane in 0..4 {
        let opacity =
            MappedScaleSpec::authored(ScaleFunctionSpec::Continuous(ContinuousScaleSpec {
                family: NumericFamily::Linear,
                domain: vec![Number(0.), Number(1.)],
                range: vec![Value::number(0.5), Value::number(0.5)],
                factory: InterpolationFactory::new(FactoryKind::Number),
                clamp: true,
                unknown: Value::Missing,
            }));
        let d = Data::columns()
            .column("x", [0.5])
            .column("g", ["a"])
            .build()
            .unwrap();
        let mut p = plot(d).aes(aes().x("x").y(1.));
        if lane > 0 {
            p = p.aes(
                aes()
                    .x("x")
                    .y(1.)
                    .color(if lane == 1 { "g" } else { "x" })
                    .color_scale("c"),
            );
            p = if lane == 1 {
                p.scale(color_discrete("c").palette(vec![floating]))
            } else {
                let function = if lane == 2 {
                    ScaleFunctionSpec::Classifier(ClassifierSpec {
                        domain: ClassifierDomain::Quantize([Number(0.), Number(1.)]),
                        range: vec![Value::Color(floating)],
                        unknown: None,
                    })
                } else {
                    ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                        normalization: NormalizationSpec::sequential(NumericFamily::Linear),
                        output: ScaleRangeFunction::Interpolate(InterpolationSpec::Between {
                            factory: InterpolationFactory::new(FactoryKind::Rgb),
                            a: Value::Color(floating),
                            b: Value::Color(floating),
                        }),
                        unknown: Value::Missing,
                    })
                };
                p.scale(color_mapped("c", MappedScaleSpec::authored(function)))
            };
        }
        let p = p
            .layer(
                points()
                    .color(floating)
                    .numeric_scale(NumericAesthetic::Opacity, "x", opacity),
            )
            .build()
            .unwrap();
        let f = Output::new(FONT)
            .unwrap()
            .request(&p, export_options(PageSize::points(500., 400.).unwrap()))
            .unwrap()
            .prepare()
            .unwrap();
        assert_eq!(
            f.layout().prepared().layers()[0].marks()[0].style.color,
            rgba(100, 50, 25, 89),
            "lane {lane}"
        );
    }
}
