//! FIX-GG07: actual renderer must paint portable segment/cap/join unions only once.
use chart_core::{
    grammar::AestheticUnits,
    prelude::*,
    scene::{Color, Primitive},
};
use chart_export::*;
#[test]
fn translucent_stroke_body_cap_and_bend_have_identical_png_alpha() {
    let data = Data::columns()
        .column("x", [1., 3., 3.])
        .column("y", [2., 2., 3.])
        .build()
        .unwrap();
    let plot = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            line()
                .color(Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                })
                .alpha(0.5)
                .linewidth(20.)
                .aesthetic_units(AestheticUnits::Points)
                .lineend(LineEnd::Round)
                .linejoin(LineJoin::Round),
        )
        .x_axis(x_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .build()
        .unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let frame = output
        .request(
            &plot,
            export_options(PageSize::points(400., 400.).unwrap()).dpi(144),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let paths = frame
        .scene()
        .items()
        .iter()
        .filter(|item| item.layer.is_some())
        .filter_map(|item| match &item.primitive {
            Primitive::ShapePath {
                anchors,
                fill: Some(fill),
                stroke: None,
                ..
            } => Some((anchors, *fill)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    let (anchors, fill) = paths[0];
    assert_eq!(fill.alpha, 128);
    assert_eq!(anchors.len(), 3);
    let bytes = frame.export(Format::Png).unwrap().bytes;
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels).unwrap();
    let sample = |x: f64, y: f64| {
        let offset = ((y * 2.).round() as usize * info.width as usize + (x * 2.).round() as usize)
            * info.color_type.samples();
        pixels[offset..offset + 3].to_vec()
    };
    let start = anchors[0];
    let bend = anchors[1];
    let expected = vec![255, 127, 127];
    for (x, y) in [
        ((start.x() + bend.x()) / 2., start.y()),
        (start.x() + 3., start.y() + 3.),
        (bend.x() - 3., bend.y() - 3.),
        (bend.x(), bend.y()),
    ] {
        assert_eq!(
            sample(x, y),
            expected,
            "overlapping contours at ({x},{y}) must share one alpha operation"
        );
    }
    assert!(
        frame
            .export(Format::Pdf)
            .unwrap()
            .bytes
            .starts_with(b"%PDF")
    );
    let svg = String::from_utf8(frame.export(Format::Svg).unwrap().bytes).unwrap();
    assert!(svg.contains("fill-rule=\"nonzero\""));
}
