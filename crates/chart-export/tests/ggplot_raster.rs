//! FIX-GG08: the declared nearest/smooth mode must reach the actual PNG renderer.
#[path = "../../../examples/common/ggplot_text_marks.rs"]
mod fixtures;
use chart_export::*;
#[test]
fn nearest_and_interpolated_pixels_reach_svg_png_and_pdf() {
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let mut samples = Vec::new();
    for mode in [4, 5] {
        let plot = fixtures::author(mode).unwrap();
        let frame = output
            .request(
                &plot,
                export_options(PageSize::points(600., 360.).unwrap()).dpi(144),
            )
            .unwrap()
            .prepare()
            .unwrap();
        let scene: serde_json::Value = serde_json::from_str(&frame.scene_json().unwrap()).unwrap();
        let bounds = &scene["items"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|i| i["primitive"].get("RasterImage"))
            .unwrap()["bounds"];
        let x = (bounds["origin"]["x"].as_f64().unwrap()
            + bounds["width"].as_f64().unwrap() * 0.45)
            * 2.;
        let y = (bounds["origin"]["y"].as_f64().unwrap()
            + bounds["height"].as_f64().unwrap() * 0.2)
            * 2.;
        let png = frame.export(Format::Png).unwrap().bytes;
        let mut decoder = png::Decoder::new(std::io::Cursor::new(png));
        decoder.set_transformations(png::Transformations::EXPAND);
        let mut reader = decoder.read_info().unwrap();
        let mut bytes = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut bytes).unwrap();
        let channels = info.color_type.samples();
        let offset = (y.floor() as usize * info.width as usize + x.floor() as usize) * channels;
        samples.push(bytes[offset..offset + 3].to_vec());
        let svg = String::from_utf8(frame.export(Format::Svg).unwrap().bytes).unwrap();
        assert!(svg.contains(if mode == 4 {
            "image-rendering=\"optimizeSpeed\""
        } else {
            "image-rendering=\"optimizeQuality\""
        }));
        assert!(
            frame
                .export(Format::Pdf)
                .unwrap()
                .bytes
                .starts_with(b"%PDF")
        );
    }
    assert_eq!(samples[0], [255, 0, 0]);
    assert!(
        samples[1][0] > 0 && samples[1][1] > 0,
        "linear interpolation must mix adjacent red and green pixels"
    );
}
