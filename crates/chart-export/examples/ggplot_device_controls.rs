//! GG17 actual raster devices from one immutable supplied-font scene.
use chart_core::prelude::*;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory =
        std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&directory)?;
    let data = Data::columns()
        .column("x", [0., 1., 2.])
        .column("y", [1., 3., 2.])
        .build()?;
    let plot = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(line())
        .layer(points())
        .build()?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for dpi in [72, 144] {
        let frame = output
            .request(
                &plot,
                export_options(PageSize::points(360., 240.)?)
                    .dpi(dpi)
                    .raster_device(RasterDeviceOptions {
                        tiff_compression: TiffCompression::Lzw,
                        tiff_predictor: true,
                        ..Default::default()
                    }),
            )?
            .prepare()?;
        for (format, extension) in [
            (Format::Jpeg, "jpeg"),
            (Format::Tiff, "tiff"),
            (Format::Bmp, "bmp"),
            (Format::Png, "png"),
            (Format::PostScript, "ps"),
            (Format::Eps, "eps"),
            (Format::PicTeX, "tex"),
            (Format::Emf, "emf"),
        ] {
            frame
                .export(format)?
                .save(directory.join(format!("device-{dpi}.{extension}")))?;
        }
        for (compression, name) in [
            (TiffCompression::Jpeg, "jpeg"),
            (TiffCompression::Lzma, "lzma"),
            (TiffCompression::Zstd, "zstd"),
            (TiffCompression::Webp, "webp"),
        ] {
            output
                .request(
                    &plot,
                    export_options(PageSize::points(360., 240.)?)
                        .dpi(dpi)
                        .raster_device(RasterDeviceOptions {
                            tiff_compression: compression,
                            ..Default::default()
                        }),
                )?
                .prepare()?
                .export(Format::Tiff)?
                .save(directory.join(format!("codec-{dpi}-{name}.tiff")))?;
        }
        let mut pages = FigurePages::new(frame.clone());
        pages.push(frame.clone())?;
        drop(frame);
        for (format, extension) in [
            (Format::PostScript, "ps"),
            (Format::Pdf, "pdf"),
            (Format::Tiff, "tiff"),
        ] {
            std::fs::write(
                directory.join(format!("pages-{dpi}.{extension}")),
                pages.export(format)?,
            )?;
        }
    }
    println!("PASS 30 real device publications at 72/144 DPI.");
    Ok(())
}
