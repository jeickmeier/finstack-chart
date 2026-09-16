//! FIX-GG07: actual PNG and SVG consumers distinguish explicit compound fill rules.
#[path = "../../../examples/common/ggplot_surface_recipes.rs"]
mod fixtures;
use chart_export::*;
#[test]
fn exported_polygon_holes_and_raster_sampling_reach_real_renderers() {
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    for mode in 0..4 {
        let plot = fixtures::author(mode).unwrap();
        let frame = output
            .request(
                &plot,
                export_options(PageSize::points(600., 360.).unwrap()).dpi(144),
            )
            .unwrap()
            .prepare()
            .unwrap();
        let svg = String::from_utf8(frame.export(Format::Svg).unwrap().bytes).unwrap();
        assert!(svg.contains(if mode < 2 {
            "fill-rule=\"evenodd\""
        } else {
            "fill-rule=\"nonzero\""
        }));
        let bytes = frame.export(Format::Png).unwrap().bytes;
        let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND);
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut pixels).unwrap();
        let offset =
            ((info.height / 2 * info.width + info.width / 2) as usize) * info.color_type.samples();
        let center = &pixels[offset..offset + 3];
        if mode == 2 {
            assert_eq!(center, [51, 51, 51]);
        } else {
            assert_eq!(center, [255, 255, 255]);
        }
        assert!(
            frame
                .export(Format::Pdf)
                .unwrap()
                .bytes
                .starts_with(b"%PDF")
        );
    }
    for mode in [6, 7] {
        let plot = fixtures::author(mode).unwrap();
        let frame = output
            .request(&plot, export_options(PageSize::points(600., 360.).unwrap()))
            .unwrap()
            .prepare()
            .unwrap();
        let svg = String::from_utf8(frame.export(Format::Svg).unwrap().bytes).unwrap();
        assert!(svg.contains(if mode == 6 {
            "image-rendering=\"optimizeSpeed\""
        } else {
            "image-rendering=\"optimizeQuality\""
        }));
    }
}
