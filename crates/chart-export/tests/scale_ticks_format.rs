//! SP-05: formatter descriptors execute through immutable publication and layout thinning.
use chart_core::{
    prelude::*,
    scales::*,
    typography::{NumericFormat, NumericLocale},
};
use chart_export::*;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn plot_with(family: NumericFamily, domain: [f64; 2], format: Option<NumericFormat>) -> Plot {
    let mut s = NumericScaleSpec::d3(family);
    s.domain = domain.map(Into::into).to_vec();
    let mut axis = x_axis().scale(scale_numeric(s));
    if let Some(f) = format {
        axis = axis.numeric_format(f);
    }
    plot(
        Data::columns()
            .column("x", domain)
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(axis)
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap()
}
#[test]
fn inferred_numeric_locale_survives_wire_and_retained_publication() {
    let locale = NumericLocale {
        decimal: ",".into(),
        thousands: Some(" ".into()),
        currency: [String::new(), " EUR".into()],
        ..Default::default()
    };
    let p = plot_with(
        NumericFamily::Linear,
        [1000., 2000.],
        Some(NumericFormat {
            specifier: "$,.2f".into(),
            locale,
        }),
    );
    let wire = p.to_json().unwrap();
    assert_eq!(p.definition().wire_version(), 5);
    let restored = Plot::from_json(&wire).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    let mut wrong: serde_json::Value = serde_json::from_str(&wire).unwrap();
    wrong["version"] = 4.into();
    assert!(Plot::from_json(&wrong.to_string()).is_err());
    let output = Output::new(FONT).unwrap();
    let request = output
        .request(
            &restored,
            export_options(PageSize::points(900., 300.).unwrap())
                .layout(layout_options().target_ticks(4)),
        )
        .unwrap();
    let f = request.prepare().unwrap();
    let labels: Vec<_> = f
        .layout()
        .axes()
        .values()
        .flat_map(|a| a.ticks.iter().map(|t| t.label.as_str()))
        .collect();
    assert_eq!(
        labels,
        vec![
            "1 000,00 EUR",
            "1 200,00 EUR",
            "1 400,00 EUR",
            "1 600,00 EUR",
            "1 800,00 EUR",
            "2 000,00 EUR"
        ]
    );
    let svg = f.export(Format::Svg).unwrap();
    assert_eq!(
        request
            .prepare()
            .unwrap()
            .export(Format::Svg)
            .unwrap()
            .bytes,
        svg.bytes
    );
    assert!(!f.export(Format::Pdf).unwrap().bytes.is_empty());
    assert!(!f.export(Format::Png).unwrap().bytes.is_empty());
}
#[test]
fn raw_log_candidates_survive_label_suppression_and_layout_thinning() {
    let p = plot_with(
        NumericFamily::Log { base: 10. },
        [1., 100.],
        Some(NumericFormat {
            specifier: ".2f".into(),
            locale: Default::default(),
        }),
    );
    let output = Output::new(FONT).unwrap();
    let render = |width| {
        output
            .request(
                &p,
                export_options(PageSize::points(width, 250.).unwrap())
                    .layout(layout_options().target_ticks(4)),
            )
            .unwrap()
            .prepare()
            .unwrap()
    };
    let wide = render(900.);
    let narrow = render(180.);
    let mut s = NumericScaleSpec::d3(NumericFamily::Log { base: 10. });
    s.domain = vec![1.0.into(), 100.0.into()];
    let standalone = NumericScale::new(s).unwrap();
    let raw = standalone.ticks(4., 100).unwrap();
    let formatter = standalone
        .tick_format(4., Some(".2f"), Default::default())
        .unwrap();
    assert_eq!(raw.len(), 19);
    assert!(raw.iter().any(|x| formatter.format(*x).is_empty()));
    let ticks = |f: &FigureSnapshot| {
        f.layout()
            .axes()
            .values()
            .flat_map(|a| &a.ticks)
            .cloned()
            .collect::<Vec<_>>()
    };
    let a = ticks(&wide);
    let b = ticks(&narrow);
    assert_eq!(
        a.iter().filter(|t| t.label.is_empty()).count(),
        raw.iter()
            .filter(|x| formatter.format(**x).is_empty())
            .count()
    );
    assert!(
        b.iter().filter(|t| !t.label.is_empty()).count()
            < a.iter().filter(|t| !t.label.is_empty()).count()
    );
    assert_eq!(standalone.ticks(4., 100).unwrap(), raw);
    assert!(!wide.export(Format::Svg).unwrap().bytes.is_empty());
    assert!(!narrow.export(Format::Svg).unwrap().bytes.is_empty());
    if let Some(dir) = std::env::var_os("CHART_SCALE_FORMAT_ARTIFACTS") {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, figure) in [("log-wide", wide), ("log-narrow", narrow)] {
            for (extension, format) in [
                ("svg", Format::Svg),
                ("pdf", Format::Pdf),
                ("png", Format::Png),
            ] {
                std::fs::write(
                    std::path::Path::new(&dir).join(format!("{name}.{extension}")),
                    figure.export(format).unwrap().bytes,
                )
                .unwrap();
            }
        }
    }
}
#[test]
fn invalid_formatter_rejected_before_empty_axis_or_category_publication() {
    let legacy_mapping = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [1., 2.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().numeric_format(NumericFormat {
        specifier: ".2f".into(),
        locale: Default::default(),
    }))
    .build()
    .unwrap();
    assert_eq!(
        legacy_mapping.definition().wire_version(),
        5,
        "The formatter alone requires v5."
    );
    let invalid = plot(
        Data::columns()
            .column("x", Vec::<f64>::new())
            .column("y", Vec::<f64>::new())
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().numeric_format(NumericFormat {
        specifier: ".f".into(),
        locale: Default::default(),
    }))
    .build();
    assert!(invalid.is_err());
    let p = plot(
        Data::columns()
            .column("x", categorical(["a", "b"]))
            .column("y", [1., 2.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().numeric_format(NumericFormat {
        specifier: ".2f".into(),
        locale: Default::default(),
    }))
    .build()
    .unwrap();
    assert!(
        Output::new(FONT)
            .unwrap()
            .request(&p, export_options(PageSize::points(400., 300.).unwrap()))
            .and_then(|r| r.prepare())
            .is_err()
    );
}
