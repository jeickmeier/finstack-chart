//! Shared primary Rust authors for the bounded GG-05 default colorbar proof.
use chart_core::{
    color::Paint,
    interpolate::{InterpolationSpec, Number, Value},
    prelude::*,
    scales::*,
};
pub fn scale(palette: &str, hidden: bool) -> MappedScaleSpec {
    let (colors, values) = match palette {
        "ordinary" => (vec!["#000000", "#ffffff"], vec![0., 1.]),
        "asymmetric" => (vec!["#0000ff", "#ffffff", "#ff0000"], vec![0., 0.2, 1.]),
        "transparent" => (
            vec!["#0000ff20", "#ffffff80", "#ff0000e0"],
            vec![0., 0.2, 1.],
        ),
        "discontinuous" => (
            vec!["#ff0000", "#ff0000", "#0000ff", "#0000ff"],
            vec![0., 0.499, 0.5, 1.],
        ),
        _ => unreachable!(),
    };
    MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Sequential {
            family: NumericFamily::Linear,
            domain: [Number(-2.), Number(8.)],
            clamp: false,
        },
        output: ScaleRangeFunction::Interpolate(InterpolationSpec::GgplotPalette {
            spec: chromatic::ggplot::PaletteSpec::Gradient {
                colors: colors
                    .into_iter()
                    .map(|v| Paint::from_css(v).unwrap())
                    .collect(),
                values: Some(values.into_iter().map(Number).collect()),
            },
        }),
        unknown: Value::Missing,
    }))
    .with_ggplot(GgplotScalePolicy::Continuous {
        limits: Some([Some(Number(-2.)), Some(Number(8.))]),
        empty_population: false,
        nonfinite_population: false,
        oob: GgplotOob::Censor,
    })
    .unwrap()
    .with_guide(if hidden {
        GgplotScaleGuide::Hidden
    } else {
        GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
            breaks: Some([-2., 0., 3., 8.].map(Number).to_vec()),
            ..Default::default()
        })
    })
    .unwrap()
}
pub fn author(palette: &str, channel: &str, facet: &str, hidden: bool) -> Plot {
    author_values(
        palette,
        channel,
        facet,
        hidden,
        [-2., 0., 3., 8., -2., 0., 3., 8.],
    )
}
pub fn author_values(
    palette: &str,
    channel: &str,
    facet: &str,
    hidden: bool,
    values: [f64; 8],
) -> Plot {
    let data = Data::columns()
        .column("x", [1., 2., 3., 4., 1., 2., 3., 4.])
        .column("v", values)
        .column("f", categorical(["A", "A", "A", "A", "B", "B", "B", "B"]))
        .build()
        .unwrap();
    let aes = aes().x("x").y(1.);
    let aes = match channel {
        "color" => aes.color("v").color_scale("v"),
        "fill" => aes.fill("v").fill_scale("v"),
        "stroke" => aes.stroke("v").stroke_scale("v"),
        _ => unreachable!(),
    };
    let mut p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes)
        .scale(color_mapped("v", scale(palette, hidden)))
        .layer(points());
    if facet != "single" {
        p = p.facet(facet_wrap("f").collect_guides(facet == "collected"));
    }
    p.build().unwrap()
}
