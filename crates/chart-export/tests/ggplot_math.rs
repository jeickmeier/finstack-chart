//! Supplied-font mathematical layout and portable consumers.
use chart_core::{
    Limits, ResourceId, Revision,
    services::{ResourceDescriptor, ResourceKind, Units},
    typography::{MathExpression, MathFonts, RichRun, ShapeRequest},
};
use chart_export::{FontResource, FontResources};
fn fonts() -> (FontResources, MathFonts, ResourceDescriptor) {
    let bytes: [&[u8]; 6] = [
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif.ttf"),
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Italic.ttf"),
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Bold.ttf"),
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-BoldItalic.ttf"),
        include_bytes!("../../../fixtures/math/fonts/DejaVuMathTeXGyre.ttf"),
        include_bytes!("../../../fixtures/math/fonts/DejaVuSans.ttf"),
    ];
    let descriptors = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| ResourceDescriptor {
            id: ResourceId::new(
                9007199254748100
                    + if i == 4 {
                        5
                    } else if i == 5 {
                        4
                    } else {
                        i as u64
                    },
            ),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: b.len() as u64,
        })
        .collect::<Vec<_>>();
    let resources = FontResources::new(
        bytes
            .iter()
            .zip(&descriptors)
            .map(|(b, d)| FontResource::new(*d, (*b).into()).unwrap())
            .collect(),
    )
    .unwrap();
    (
        resources,
        MathFonts {
            regular: Some(descriptors[0]),
            italic: Some(descriptors[1]),
            bold: Some(descriptors[2]),
            bold_italic: Some(descriptors[3]),
            symbol: Some(descriptors[4]),
        },
        descriptors[0],
    )
}
#[test]
fn fixed_font_source_metric_diagnostics() {
    let (resources, fonts, font) = fonts();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/plotmath-metrics.json"
    ))
    .unwrap();
    let mut maximum = [0_f64; 3];
    let mut worst = [String::new(), String::new(), String::new()];
    let mut cases = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let size = case["font_size"].as_f64().unwrap();
        let expression = MathExpression::parse(source, fonts.clone(), Limits::default()).unwrap();
        let mut run = RichRun::new(source);
        run.fallback.push(ResourceDescriptor {
            id: ResourceId::new(9007199254748104),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: include_bytes!("../../../fixtures/math/fonts/DejaVuSans.ttf").len() as u64,
        });
        let result = expression
            .measure(
                ShapeRequest {
                    run: &run,
                    default_font: &font,
                    font_size: size,
                    units: Units::Points,
                    limits: Limits::default(),
                },
                &resources,
            )
            .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        for (i, (actual, key)) in [
            (result.width(), "width"),
            (result.ascent(), "ascent"),
            (result.descent(), "descent"),
        ]
        .into_iter()
        .enumerate()
        {
            let expected = case["result"][key].as_f64().unwrap();
            let delta = (actual - expected).abs();
            if delta > maximum[i] {
                maximum[i] = delta;
                worst[i] = format!("{source}: actual {actual} expected {expected}");
            }
        }
        assert!(result.width().is_finite());
        cases += 1;
    }
    eprintln!(
        "{cases} supplied-font expressions shaped; source metric maximum absolute deltas {maximum:?}, worst {worst:?}; diagnostics only, not a source parity assertion"
    );
    assert_eq!(cases, 298);
}

#[path = "../../../examples/common/ggplot_math_controls.rs"]
mod authors;
fn output_fonts() -> (chart_export::Output, MathFonts, ResourceDescriptor) {
    let mut output = chart_export::Output::new(
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif.ttf").as_slice(),
    )
    .unwrap();
    let fonts = MathFonts {
        regular: Some(output.primary_font()),
        italic: Some(
            output
                .register_font(
                    include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Italic.ttf")
                        .as_slice(),
                )
                .unwrap(),
        ),
        bold: Some(
            output
                .register_font(
                    include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Bold.ttf").as_slice(),
                )
                .unwrap(),
        ),
        bold_italic: Some(
            output
                .register_font(
                    include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-BoldItalic.ttf")
                        .as_slice(),
                )
                .unwrap(),
        ),
        symbol: Some(
            output
                .register_font(
                    include_bytes!("../../../fixtures/math/fonts/DejaVuMathTeXGyre.ttf").as_slice(),
                )
                .unwrap(),
        ),
    };
    let fallback = output
        .register_font(include_bytes!("../../../fixtures/math/fonts/DejaVuSans.ttf").as_slice())
        .unwrap();
    (output, fonts, fallback)
}
#[test]
fn mathematical_consumers_replay_and_animated_rules_retain_portable_geometry() {
    use chart_export::{Format, PageSize, TextMode, export_options};
    let (output, fonts, fallback) = output_fonts();
    let options = export_options(PageSize::points(600., 360.).unwrap());
    let a = authors::author(0, fonts.clone(), fallback).unwrap();
    let b = authors::author(1, fonts, fallback).unwrap();
    let a = output
        .request(&a, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let b = output
        .request(&b, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let plan = b.guide_transition(&a).unwrap();
    for t in [0., 0.5, 1.] {
        let frame = plan.sample(t).unwrap();
        let svg = frame.export(Format::Svg).unwrap();
        assert!(svg.bytes.starts_with(b"<"));
    }
    for mode in [TextMode::Preserve, TextMode::Outline] {
        let (output, fonts, fallback) = output_fonts();
        for case in 0..authors::CASES {
            let original = authors::author(case, fonts.clone(), fallback).unwrap();
            let replay =
                chart_core::prelude::Plot::from_json(&original.to_json().unwrap()).unwrap();
            let original = output
                .request(&original, options.clone().text(mode))
                .unwrap()
                .prepare()
                .unwrap();
            let replay = output
                .request(&replay, options.clone().text(mode))
                .unwrap()
                .prepare()
                .unwrap();
            assert_eq!(original.scene_json().unwrap(), replay.scene_json().unwrap());
            for format in [Format::Svg, Format::Pdf, Format::Png] {
                assert!(!original.export(format).unwrap().bytes.is_empty());
            }
        }
    }
}

#[test]
fn cairo_atom_metrics_quantify_device_rounding_separately_from_math_layout() {
    let (resources, fonts, regular) = fonts();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/plotmath-metrics.json"
    ))
    .unwrap();
    let mut maximum = [0_f64; 3];
    let mut count = 0;
    for atom in fixture["metric_controls"].as_array().unwrap() {
        let text = atom["text"].as_str().unwrap();
        if !text.is_ascii() && atom["font_family"] == "DejaVu Serif" {
            continue;
        }
        count += 1;
        let face = if atom["font_family"] == "DejaVu Serif" {
            regular
        } else {
            fonts.symbol.unwrap()
        };
        let source = serde_json::to_string(text).unwrap();
        let expression = MathExpression::parse(
            source.clone(),
            MathFonts {
                regular: Some(face),
                ..Default::default()
            },
            Limits::default(),
        )
        .unwrap();
        let run = RichRun::new(&source);
        let metrics = expression
            .measure(
                ShapeRequest {
                    run: &run,
                    default_font: &face,
                    font_size: atom["font_size"].as_f64().unwrap(),
                    units: Units::Points,
                    limits: Limits::default(),
                },
                &resources,
            )
            .unwrap();
        for (i, (actual, key)) in [
            (metrics.width(), "width"),
            (metrics.ascent(), "ascent"),
            (metrics.descent(), "descent"),
        ]
        .into_iter()
        .enumerate()
        {
            maximum[i] = maximum[i].max((actual - atom[key].as_f64().unwrap()).abs());
        }
    }
    eprintln!("Independent ordinary glyph Cairo/design-unit metric deltas: {maximum:?}");
    assert_eq!(count, 168);
}

#[test]
fn worst_width_difference_is_exactly_explained_by_independent_atom_metrics() {
    let (resources, fonts, font) = fonts();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/plotmath-metrics.json"
    ))
    .unwrap();
    let measure = |source: &str, faces: MathFonts| {
        let expression = MathExpression::parse(source, faces, Limits::default()).unwrap();
        let run = RichRun::new(source);
        expression
            .measure(
                ShapeRequest {
                    run: &run,
                    default_font: &font,
                    font_size: 24.,
                    units: Units::Points,
                    limits: Limits::default(),
                },
                &resources,
            )
            .unwrap()
            .width()
    };
    let mut actual_sum = 0.;
    let mut reference_sum = 0.;
    for (text, family, multiplier) in [
        ("x", "DejaVu Serif", 1.),
        ("y", "DejaVu Serif", 1.),
        ("z", "DejaVu Serif", 1.),
        (", ", "DejaVu Math TeX Gyre", 2.),
    ] {
        let reference = fixture["metric_controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["text"] == text && a["font_family"] == family && a["font_size"] == 24)
            .unwrap();
        reference_sum += multiplier * reference["width"].as_f64().unwrap();
        let mut faces = fonts.clone();
        if family == "DejaVu Math TeX Gyre" {
            faces.regular = faces.symbol;
        }
        actual_sum += multiplier * measure(&serde_json::to_string(text).unwrap(), faces);
    }
    let reference = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["source"] == "list(x, y, z)" && a["font_size"] == 24)
        .unwrap()["result"]["width"]
        .as_f64()
        .unwrap();
    let actual = measure("list(x,y,z)", fonts);
    assert!((reference - reference_sum).abs() < 1e-12);
    assert!((actual - actual_sum).abs() < 1e-12);
    assert!(((reference - actual) - (reference_sum - actual_sum)).abs() < 1e-12);
}

#[test]
fn vertical_accent_and_integral_differences_decompose_into_signed_atoms() {
    let (resources, fonts, font) = fonts();
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/plotmath-vertical-controls.json"
    ))
    .unwrap();
    let compounds: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/plotmath-metrics.json"
    ))
    .unwrap();
    let regular = ttf_parser::Face::parse(
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif.ttf"),
        0,
    )
    .unwrap();
    let italic = ttf_parser::Face::parse(
        include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Italic.ttf"),
        0,
    )
    .unwrap();
    let symbol = ttf_parser::Face::parse(
        include_bytes!("../../../fixtures/math/fonts/DejaVuMathTeXGyre.ttf"),
        0,
    )
    .unwrap();
    for size in [12., 24.] {
        let atom = |name: &str, key: &str| {
            source["atoms"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["text"] == name && a["base_size"] == size)
                .unwrap()[key]
                .as_f64()
                .unwrap()
        };
        let ink = |face: &ttf_parser::Face<'_>, ch: char, scale: f64| {
            let box_ = face
                .glyph_bounding_box(face.glyph_index(ch).unwrap())
                .unwrap();
            let unit = scale / f64::from(face.units_per_em());
            (f64::from(box_.y_max) * unit, -f64::from(box_.y_min) * unit)
        };
        let measure = |text: &str| {
            MathExpression::parse(text, fonts.clone(), Limits::default())
                .unwrap()
                .measure(
                    ShapeRequest {
                        run: &RichRun::new(text),
                        default_font: &font,
                        font_size: size,
                        units: Units::Points,
                        limits: Limits::default(),
                    },
                    &resources,
                )
                .unwrap()
        };
        // Accent ascent = body ascent + signed accent depth + gap + accent ascent.
        let expected_dot = atom("x", "ascent")
            + atom("dotmath", "descent")
            + 0.1 * atom("X", "ascent")
            + atom("dotmath", "ascent");
        let (xh, _) = ink(&italic, 'x', size);
        let (cap, _) = ink(&regular, 'X', size);
        let (dh, dd) = ink(&symbol, '⋅', size);
        let design_dot = xh + dd + 0.1 * cap + dh;
        // Integral lower half baseline = .99*lower ascent - axis; append centred subscript.
        let expected_integral = 0.99 * atom("integral-bottom", "ascent") - atom("+", "ascent") / 2.
            + atom("integral-bottom", "descent")
            + (atom("a", "ascent") + atom("a", "descent")) / 2.;
        let (lh, ld) = ink(&symbol, '⌡', size);
        let (axis, _) = ink(&regular, '+', size);
        let (ah, ad) = ink(&italic, 'a', size * 0.7);
        let design_integral = 0.99 * lh - axis / 2. + ld + (ah + ad) / 2.;
        for (text, key, expected, design, actual) in [
            (
                "dot(x)",
                "ascent",
                expected_dot,
                design_dot,
                measure("dot(x)").ascent(),
            ),
            (
                "integral(f(x)*dx, a, b)",
                "descent",
                expected_integral,
                design_integral,
                measure("integral(f(x)*dx,a,b)").descent(),
            ),
        ] {
            let reference = compounds["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["source"] == text && a["font_size"] == size)
                .unwrap()["result"][key]
                .as_f64()
                .unwrap();
            assert!(
                (reference - expected).abs() < 1e-12,
                "source {text} {size}: {reference} != {expected}"
            );
            assert!(
                (actual - design).abs() < 1e-10,
                "design {text} {size}: {actual} != {design}"
            );
            assert!(((reference - actual) - (expected - design)).abs() < 1e-10);
        }
    }
}
