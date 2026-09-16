//! GG08 native and publication authors use shared row text and raster semantics.
use chart_core::{
    grammar::{
        AestheticUnits, AnnotationContent, RasterAnnotation, RowAnnotation, StatField, TextGeom,
        TextSizeUnit,
    },
    prelude::*,
    scene::Color,
};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", vec![1., 2., 3., 4.])
        .column("label", vec!["Alpha", "béta", "", "Delta"])
        .build()?;
    let mut b = plot(data).aes(aes().x("x").y(1.));
    let options = TextGeom {
        size: 14.,
        units: TextSizeUnit::Points,
        angle: if mode == 1 { 30. } else { 0. },
        fill: if mode == 1 {
            Some(Color {
                red: 230,
                green: 240,
                blue: 255,
                alpha: 255,
            })
        } else {
            None
        },
        check_overlap: mode == 2,
        ..Default::default()
    };
    if mode < 3 {
        let layer = points().text_geom(options).text_label("label");
        b = b.layer(if mode == 2 {
            layer.aes(aes().x(1.).y(1.))
        } else {
            layer
        });
    } else if mode == 3 {
        let data = Data::columns()
            .column("group", vec!["A", "A", "B"])
            .build()?;
        b = plot(data).layer(
            points()
                .stat(count().group("group"))
                .after_stat(stat_aes().x(StatField::Group).y(StatField::Count))
                .text_geom(options)
                .text_stat_label(StatField::Count),
        );
    } else {
        let data = Data::columns().column("x", vec![1.]).build()?;
        let content = if mode == 6 {
            AnnotationContent::Vector {
                geometry: chart_core::path::PathGeometry::from_commands(
                    vec![
                        chart_core::path::Command::MoveTo([-80., -60.]),
                        chart_core::path::Command::LineTo([80., -60.]),
                        chart_core::path::Command::LineTo([0., 60.]),
                        chart_core::path::Command::Close,
                    ],
                    100,
                )?,
                fill: Some(Color {
                    red: 20,
                    green: 90,
                    blue: 120,
                    alpha: 255,
                }),
                stroke: None,
            }
        } else {
            AnnotationContent::Raster {
                raster: RasterAnnotation {
                    width: 2,
                    height: 2,
                    pixels: vec![
                        Color {
                            red: 255,
                            green: 0,
                            blue: 0,
                            alpha: 255,
                        },
                        Color {
                            red: 0,
                            green: 255,
                            blue: 0,
                            alpha: 255,
                        },
                        Color {
                            red: 0,
                            green: 0,
                            blue: 255,
                            alpha: 255,
                        },
                        Color {
                            red: 255,
                            green: 255,
                            blue: 0,
                            alpha: 100,
                        },
                    ],
                },
                bounds: [-100., -60., 200., 120.],
                interpolate: mode == 5,
            }
        };
        b = plot(data)
            .aes(aes().x("x").y(1.))
            .layer(points().annotation(RowAnnotation {
                units: AestheticUnits::Points,
                content,
            }));
    }
    let x = if mode < 3 {
        x_axis().scale(scale_linear().domain(0., 5.))
    } else {
        x_axis()
    };
    b.x_axis(x.visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 3.)).visible(false))
        .build()
}
