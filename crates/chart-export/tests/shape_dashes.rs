//! FIX-S09: curved and filled vector strokes retain dash styling in immutable output.
use chart_core::plot::shape_line;
use chart_core::{
    composition::Anchor,
    path::Path,
    prelude::*,
    scene::{Color, Primitive, Stroke},
    shape::CurveSpec,
};
use chart_export::*;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
#[test]
fn curved_svg_and_pdf_png_retain_dash_pattern_fill_and_snapshot() {
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0., 1.])
        .build()
        .unwrap();
    let mut path = Path::default();
    path.arc([0., 0.], 20., 0., std::f64::consts::TAU, false)
        .unwrap();
    path.move_to(10., 0.).unwrap();
    path.arc([0., 0.], 10., 0., std::f64::consts::TAU, true)
        .unwrap();
    let black = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    let yellow = Color {
        red: 230,
        green: 200,
        blue: 30,
        alpha: 255,
    };
    let make = |pattern| {
        plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(shape_line().curve(CurveSpec::BumpX))
            .layer(
                vector_path("ring", path.geometry())
                    .anchor(Anchor::Output { x: 150., y: 80. })
                    .fill(Some(yellow))
                    .stroke(Some(Stroke {
                        color: black,
                        width: 2.,
                    })),
            )
            .theme(theme().style(style().dashes(pattern)))
            .build()
            .unwrap()
    };
    let output = Output::new(FONT).unwrap();
    let options = export_options(PageSize::points(300., 160.).unwrap()).dpi(72);
    let solid = output
        .request(&make(vec![]), options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let retained = output.request(&make(vec![5., 3.]), options).unwrap();
    let dashed = retained.prepare().unwrap();
    assert_eq!(dashed.scene().wire_version(), 4);
    let vector = dashed
        .scene()
        .items()
        .iter()
        .find_map(|i| {
            if let Primitive::VectorPath {
                geometry,
                fill,
                stroke,
                dashes,
            } = &i.primitive
            {
                Some((geometry, fill, stroke, dashes))
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(*vector.1, Some(yellow));
    assert_eq!(
        *vector.2,
        Some(Stroke {
            color: black,
            width: 2.
        })
    );
    assert_eq!(vector.3, &vec![5., 3.]);
    assert_eq!(vector.0.commands().len(), path.geometry().commands().len());
    let svg = String::from_utf8(dashed.export(Format::Svg).unwrap().bytes).unwrap();
    assert_eq!(svg.matches("stroke-dasharray=\"5,3\"").count(), 2);
    assert!(svg.contains("fill-rule=\"nonzero\""));
    for format in [Format::Svg, Format::Pdf, Format::Png] {
        let bytes = dashed.export(format).unwrap().bytes;
        assert_ne!(bytes, solid.export(format).unwrap().bytes);
        assert_eq!(
            bytes,
            retained.prepare().unwrap().export(format).unwrap().bytes
        );
    }
}
