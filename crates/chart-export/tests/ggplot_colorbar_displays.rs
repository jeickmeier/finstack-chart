//! FIX-GG05: adjacent opaque cells cannot expose the page through antialias seams.
#[path = "../../../examples/common/ggplot_colorbar_fixtures.rs"]
mod fixtures;
use chart_core::{interpolate::Number, prelude::*, scales::*, scene::Primitive};
use chart_export::*;
#[test]
fn dense_constant_rectangle_bar_has_no_page_colored_seams() {
    let descriptor = fixtures::scale("asymmetric", false)
        .with_ggplot(GgplotScalePolicy::Continuous {
            limits: Some([Some(Number(2.)), Some(Number(2.))]),
            empty_population: false,
            nonfinite_population: false,
            oob: GgplotOob::Censor,
        })
        .unwrap()
        .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
            breaks: Some(vec![Number(2.)]),
            ..Default::default()
        }))
        .unwrap()
        .with_colorbar_options(GgplotColorbarOptions {
            display: GgplotColorbarDisplay::Rectangles,
            ..Default::default()
        });
    let plot = fixtures::author("asymmetric", "color", "single", false)
        .edit()
        .scale(color_mapped("v", descriptor))
        .build()
        .unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let snapshot = output
        .request(
            &plot,
            export_options(PageSize::points(600., 360.).unwrap()).dpi(144),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let bars: Vec<_> = snapshot
        .scene()
        .items()
        .iter()
        .filter_map(|item| {
            if item.layer.is_some() || item.clip.is_none() {
                return None;
            }
            match item.primitive {
                Primitive::Rectangle { bounds, .. }
                | Primitive::SampledGradientRectangle { bounds, .. }
                    if (bounds.width() - 18.).abs() < 1e-10 =>
                {
                    Some(bounds)
                }
                _ => None,
            }
        })
        .collect();
    let first = bars.first().expect("bar paint");
    let last = bars.last().unwrap();
    let x = ((first.origin().x() + first.width() / 2.) * 2.).floor() as usize;
    let top = (first.origin().y() * 2.).ceil() as usize + 2;
    let bottom = (last.max_y() * 2.).floor() as usize - 2;
    let png = snapshot.export(Format::Png).unwrap().bytes;
    let mut reader = png::Decoder::new(png.as_slice()).read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels).unwrap();
    let channels = info.color_type.samples();
    assert!(bottom - top > 200);
    for y in top..bottom {
        let i = (y * info.width as usize + x) * channels;
        assert_eq!(&pixels[i..i + 3], &[255, 183, 160], "page seam at y={y}");
    }
}
