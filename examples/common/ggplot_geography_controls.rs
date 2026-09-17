//! GG15 independent typed geography authors over shared source fields.
use chart_core::{grammar::*, prelude::*};
fn ring(x: f64, y: f64, w: f64, h: f64) -> Vec<GeoPosition> {
    vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h], [x, y]]
}
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let mut geometry = GeoGeometry::MultiPolygon(vec![
        vec![ring(-20., 10., 30., 40.), ring(-10., 20., 10., 15.)],
        vec![ring(20., 15., 15., 20.)],
    ]);
    if mode == 12 {
        geometry = GeoGeometry::Polygon(vec![
            vec![
                [170., 10.],
                [-170., 10.],
                [-170., 40.],
                [170., 40.],
                [170., 10.],
            ],
            vec![
                [160., 20.],
                [-160., 20.],
                [-160., 30.],
                [160., 30.],
                [160., 20.],
            ],
        ]);
    }
    let mut collection = GeoFeatureCollection {
        features: vec![GeoFeature {
            id: GroupValue::Text("land".into()),
            geometry,
            crs: None,
        }],
        crs: GeoCrs::Wgs84,
        axis_order: GeoAxisOrder::XY,
    };
    if mode == 3 {
        collection.axis_order = GeoAxisOrder::YX;
        if let GeoGeometry::MultiPolygon(polygons) = &mut collection.features[0].geometry {
            for p in polygons.iter_mut().flatten().flatten() {
                p.swap(0, 1);
            }
        }
    }
    if mode == 16 {
        collection.features.push(GeoFeature {
            id: GroupValue::Text("city".into()),
            geometry: GeoGeometry::Point([1113194.9079327357, 5621521.486192066]),
            crs: Some(GeoCrs::WebMercator),
        });
    }
    let data = Data::columns()
        .column(
            "id",
            ["land", if mode == 16 { "city" } else { "unmatched" }],
        )
        .column("value", [1., 2.])
        .column("x", [-30., 45.])
        .column("y", [5., 60.])
        .build()?;
    let mut layer = points().geography(collection, "id");
    if let Some(operation) = match mode {
        4 => Some(GeoOperation::Centroid),
        5 | 14 | 15 => Some(GeoOperation::PointOnSurface),
        6 => Some(GeoOperation::Borders),
        7 => Some(GeoOperation::Coordinates),
        _ => None,
    } {
        layer = layer.geography_operation(operation);
    }
    if mode == 14 || mode == 15 {
        layer = layer
            .text_geom(TextGeom {
                fill: if mode == 15 {
                    Some(chart_core::scene::Color {
                        red: 255,
                        green: 255,
                        blue: 220,
                        alpha: 255,
                    })
                } else {
                    None
                },
                ..Default::default()
            })
            .text_label("id");
    }
    if mode == 11 {
        layer = layer
            .fill(chart_core::color::parse_r("#40a0d0")?)
            .color(chart_core::color::parse_r("#603030")?)
            .linewidth(0.8)
            .alpha(0.5);
    }
    let projection = match mode {
        1 | 4 | 5 | 10 | 11 | 12 | 13 | 14 | 15 => GeoProjectionSelection::Crs(GeoCrs::WebMercator),
        2 => GeoProjectionSelection::Crs(GeoCrs::Proj(
            "+proj=utm +zone=31 +datum=WGS84 +units=m +no_defs".into(),
        )),
        8 => GeoProjectionSelection::Mapproj(MapprojProjection::resolve(
            MapprojMethod::Mollweide,
            vec![],
            Some([90., 0., 0.]),
            [-20., 35.],
        )?),
        9 => GeoProjectionSelection::Mapproj(MapprojProjection::resolve(
            MapprojMethod::Tetra,
            vec![],
            Some([90., 0., 0.]),
            [-20., 35.],
        )?),
        _ => GeoProjectionSelection::Crs(GeoCrs::Wgs84),
    };
    let mut coordinate = GeographicCoordinate {
        projection,
        default_crs: Some(GeoCrs::Wgs84),
        ..Default::default()
    };
    if mode == 10 {
        coordinate.limits_method = GeoLimitsMethod::GeometryBounds;
        coordinate.view.xlim = Some([
            Some(chart_core::composition::ScaleValue::Number(-5.)),
            Some(chart_core::composition::ScaleValue::Number(5.)),
        ]);
    }
    if mode == 17 {
        coordinate.graticule.longitude = Some(vec![-20., 0., 20.]);
        coordinate.graticule.latitude = Some(vec![10., 30., 50.]);
        coordinate.graticule.label_axes = [
            GeoAxisLabel::Longitude,
            GeoAxisLabel::Latitude,
            GeoAxisLabel::Longitude,
            GeoAxisLabel::Latitude,
        ];
    }
    if mode == 18 {
        coordinate.graticule.datum = None;
    }
    let mut builder = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(layer)
        .coordinate(CoordinateSpec::Geographic(coordinate));
    if mode == 13 {
        builder = builder.layer(
            line()
                .aes(aes().x("x").y("y"))
                .color(chart_core::color::parse_r("red")?),
        );
    }
    if mode >= 17 {
        builder = builder.theme(
            theme().reference_preset(chart_core::theme::ThemePreset::Grey, Default::default())?,
        );
    }
    builder.build()
}
