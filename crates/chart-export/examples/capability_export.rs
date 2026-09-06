//! WP-03 headless SVG/PDF/PNG experiment; no native event loop or public exporter API.

#[path = "../../../fixtures/capability/support.rs"]
mod fixture;

use base64::Engine as _;
use resvg::{tiny_skia, usvg};
use std::{fs, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/wp-03".into()),
    );
    fs::create_dir_all(&output)?;
    let start = Instant::now();
    let tree = fixture::tree()?;
    println!(
        "cold font load + parse + shape: {:.3} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    let mut css = String::from("<style>");
    for (family, bytes) in fixture::FONTS {
        css.push_str(&format!("@font-face{{font-family:'{family}';src:url(data:font/ttf;base64,{}) format('truetype');}}", base64::engine::general_purpose::STANDARD.encode(bytes)));
    }
    css.push_str("</style>");
    let root_end = fixture::SVG.find('>').ok_or("missing SVG root")? + 1;
    let mut text_svg = fixture::SVG.to_string();
    text_svg.insert_str(root_end, &css);
    fs::write(output.join("capability-text.svg"), text_svg)?;
    let outline = tree.to_string(&usvg::WriteOptions::default());
    let (_, body) = outline.split_once('>').ok_or("missing outlined SVG root")?;
    fs::write(
        output.join("capability-outline.svg"),
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"180mm\" height=\"120mm\" viewBox=\"0 0 {} {}\">{body}",
            tree.size().width(),
            tree.size().height()
        ),
    )?;
    for embed_text in [true, false] {
        let start = Instant::now();
        let pdf = svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions {
                embed_text,
                pdfa: true,
                ..Default::default()
            },
            svg2pdf::PageOptions { dpi: 72.0 },
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            output.join(if embed_text {
                "capability-text.pdf"
            } else {
                "capability-outline.pdf"
            }),
            pdf,
        )?;
        println!(
            "PDF embed_text={embed_text}, convert + write: {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    for dpi in [300_u32, 600] {
        let start = Instant::now();
        let width = (180.0_f64 / 25.4 * f64::from(dpi)).round() as u32;
        let height = (120.0_f64 / 25.4 * f64::from(dpi)).round() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("PNG allocation failed")?;
        // Pixel rounding can extend a fractional pixel beyond the vector page.
        // Keep its declared white background opaque through that last pixel.
        pixmap.fill(tiny_skia::Color::WHITE);
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(dpi as f32 / 72.0, dpi as f32 / 72.0),
            &mut pixmap.as_mut(),
        );
        let file = fs::File::create(output.join(format!("capability-{dpi}.png")))?;
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let ppm = (f64::from(dpi) / 0.0254).round() as u32;
        encoder.set_pixel_dims(Some(png::PixelDimensions {
            xppu: ppm,
            yppu: ppm,
            unit: png::Unit::Meter,
        }));
        // White opaque page means premultiplied RGB equals straight RGB here.
        encoder.write_header()?.write_image_data(pixmap.data())?;
        println!(
            "PNG {dpi} DPI {width}x{height}, allocate + render + encode + write: {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    let missing = fixture::validate_font(fixture::FONTS[0].1, "\u{10ffff}")
        .expect_err("negative glyph probe must fail");
    println!("negative resource 0 / revision 0: {missing}");
    let mut samples = Vec::new();
    for sample in 0..40 {
        let start = Instant::now();
        std::hint::black_box(
            svg2pdf::to_pdf(&tree, Default::default(), Default::default())
                .map_err(|e| e.to_string())?,
        );
        if sample >= 10 {
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "warm prepared-tree PDF conversion only, warmup=10 n=30 p50={:.3} p95={:.3} ms",
        samples[14], samples[28]
    );
    Ok(())
}
