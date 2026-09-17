//! Geographic resource joins and shared geometry emission; all targets remain source targets.
use super::{compiler::EncodedRow, *};
use crate::{ChartResult, DiagnosticCode, Point, data::DatasetSnapshot, scene::FillRule};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn resolve(
    layer: &Layer,
    data: &DatasetSnapshot,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<()> {
    let Some(spec) = &layer.geography else {
        return Ok(());
    };
    spec.collection.validate(GeoLimits {
        features: limits.max_prepared_rows,
        positions: limits.max_vertices,
        depth: 32,
    })?;
    super::stats::validate_group(data, &Grouping::Field(spec.join))?;
    if !matches!(layer.statistic.parameters, StatParameters::Identity)
        || !matches!(layer.position, Position::Identity)
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Geographic features require identity statistics and positions; use their explicit geometry-derived operations.",
        ));
    }
    let ids: BTreeMap<_, _> = spec
        .collection
        .features
        .iter()
        .enumerate()
        .map(|(i, f)| (&f.id, i))
        .collect();
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    for row in rows {
        row.geo_feature = row
            .key
            .and_then(|key| index.get(&key))
            .and_then(|source| super::stats::group_value(*source, &Grouping::Field(spec.join)))
            .and_then(|value| ids.get(&value).copied());
    }
    Ok(())
}
fn geometry_parts(
    geometry: &GeoGeometry,
    transform: &GeoTransform,
    parts: &mut Vec<PreparedGeometry>,
) -> ChartResult<()> {
    let point = |p: GeoPosition| -> ChartResult<Option<Point>> {
        match transform.transform(p) {
            Ok(p) => Point::new(p[0], p[1]).map(Some),
            Err(e) if e.code == DiagnosticCode::NumericalDomain => Ok(None),
            Err(e) => Err(e),
        }
    };
    let points = |values: &[GeoPosition]| -> ChartResult<Vec<Point>> {
        values
            .iter()
            .map(|p| point(*p))
            .collect::<ChartResult<Vec<_>>>()
            .map(|p| p.into_iter().flatten().collect())
    };
    match geometry {
        GeoGeometry::Empty => {}
        GeoGeometry::Point(p) => {
            if let Some(p) = point(*p)? {
                parts.push(PreparedGeometry::Point(p));
            }
        }
        GeoGeometry::MultiPoint(values) => {
            let transformed = values
                .iter()
                .map(|p| point(*p))
                .collect::<ChartResult<Vec<_>>>()?;
            // GDAL's multipoint transformation is atomic; line/ring vertices use partial reprojection.
            if transformed.iter().all(Option::is_some) {
                parts.extend(
                    transformed
                        .into_iter()
                        .flatten()
                        .map(PreparedGeometry::Point),
                );
            }
        }
        GeoGeometry::LineString(values) => {
            let points = points(values)?;
            if !points.is_empty() {
                parts.push(PreparedGeometry::LineRun(points));
            }
        }
        GeoGeometry::MultiLineString(lines) => {
            for line in lines {
                geometry_parts(&GeoGeometry::LineString(line.clone()), transform, parts)?;
            }
        }
        GeoGeometry::Polygon(rings) => {
            if !rings.is_empty() {
                let contours = rings
                    .iter()
                    .map(|ring| points(ring))
                    .collect::<ChartResult<Vec<_>>>()?;
                if contours[0].is_empty() {
                    return Ok(());
                }
                let contours = contours
                    .into_iter()
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<_>>();
                let anchor = contours[0][0];
                parts.push(PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(
                    PreparedSurface::Polygon {
                        contours,
                        anchors: vec![anchor],
                        rule: FillRule::EvenOdd,
                    },
                ))));
            }
        }
        GeoGeometry::MultiPolygon(polygons) => {
            for polygon in polygons {
                geometry_parts(&GeoGeometry::Polygon(polygon.clone()), transform, parts)?;
            }
        }
        GeoGeometry::Collection(geometries) => {
            for geometry in geometries {
                geometry_parts(geometry, transform, parts)?;
            }
        }
    }
    Ok(())
}
fn vertices(geometry: &PreparedGeometry) -> Vec<Point> {
    match geometry {
        PreparedGeometry::Point(p) => vec![*p],
        PreparedGeometry::LineRun(p) => p.clone(),
        PreparedGeometry::Recipe(recipe) => match recipe.as_ref() {
            PreparedRecipe::Surface(PreparedSurface::Polygon { contours, .. }) => {
                contours.iter().flatten().copied().collect()
            }
            _ => vec![],
        },
        _ => vec![],
    }
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    used: &mut usize,
    coordinate: Option<&GeographicCoordinate>,
) -> ChartResult<bool> {
    let Some(spec) = &layer.geography else {
        return Ok(false);
    };
    let calculation_crs = coordinate
        .map(GeographicCoordinate::source_crs)
        .unwrap_or(&GeoCrs::Wgs84);
    for row in rows {
        let Some(index) = row.geo_feature else {
            continue;
        };
        let feature = &spec.collection.features[index];
        let mut parts = vec![];
        let label = matches!(
            spec.operation,
            GeoOperation::PointOnSurface | GeoOperation::Centroid
        );
        let destination = if label {
            coordinate
                .and_then(|c| {
                    if let GeoProjectionSelection::Crs(crs) = &c.projection {
                        Some(crs)
                    } else {
                        None
                    }
                })
                .unwrap_or(calculation_crs)
        } else {
            calculation_crs
        };
        let transform = GeoTransform::new(
            feature.crs.as_ref().unwrap_or(&spec.collection.crs),
            destination,
            spec.collection.axis_order,
        )?;
        geometry_parts(&feature.geometry, &transform, &mut parts)?;
        let mut training = vec![];
        if label && destination != calculation_crs {
            let transform = GeoTransform::new(
                feature.crs.as_ref().unwrap_or(&spec.collection.crs),
                calculation_crs,
                spec.collection.axis_order,
            )?;
            geometry_parts(&feature.geometry, &transform, &mut training)?;
        }
        let training = if training.is_empty() {
            &parts
        } else {
            &training
        };
        for geometry in training {
            let points = vertices(geometry);
            super::compiler::charge(used, points.len(), "geographic training vertices")?;
            super::compiler::include_geometry(&mut prepared.domains, geometry);
            prepared.geography_vertices.extend(points);
        }
        if label {
            let inverse = GeoTransform::new(destination, calculation_crs, GeoAxisOrder::XY)?;
            parts = super::geography_statistics::anchor(&parts, spec.operation)?
                .map(|p| {
                    inverse
                        .transform([p.x(), p.y()])
                        .and_then(|p| Point::new(p[0], p[1]))
                        .map(PreparedGeometry::Point)
                })
                .transpose()?
                .into_iter()
                .collect();
        }
        for geometry in parts {
            let outputs = match spec.operation {
                GeoOperation::Geometry => vec![geometry],
                GeoOperation::Coordinates => vertices(&geometry)
                    .into_iter()
                    .map(PreparedGeometry::Point)
                    .collect(),
                GeoOperation::Borders => match geometry {
                    PreparedGeometry::Recipe(recipe) => match *recipe {
                        PreparedRecipe::Surface(PreparedSurface::Polygon { contours, .. }) => {
                            contours
                                .into_iter()
                                .map(PreparedGeometry::LineRun)
                                .collect()
                        }
                        _ => vec![],
                    },
                    other => vec![other],
                },
                GeoOperation::PointOnSurface | GeoOperation::Centroid => vec![geometry],
            };
            for geometry in outputs {
                let count = vertices(&geometry).len();
                super::compiler::charge(used, count, "geographic vertices")?;
                super::compiler::include_geometry(&mut prepared.domains, &geometry);
                let mut style = super::compiler::row_style(layer, row)?;
                let polygon = matches!(geometry, PreparedGeometry::Recipe(_));
                let point = matches!(geometry, PreparedGeometry::Point(_));
                let base = if spec.default_color && row.color.is_none() {
                    crate::scene::Color {
                        red: if polygon { 89 } else { 0 },
                        green: if polygon { 89 } else { 0 },
                        blue: if polygon { 89 } else { 0 },
                        alpha: 255,
                    }
                    .into()
                } else {
                    row.color.unwrap_or(layer.style.color)
                };
                let opacity = |paint| super::numeric_aesthetics::apply_opacity(paint, row.opacity);
                let alpha = |paint| {
                    super::numeric_aesthetics::apply_alpha(
                        opacity(paint),
                        layer.style.alpha.or(row.alpha),
                    )
                };
                style.color = if polygon { opacity(base) } else { alpha(base) };
                if polygon {
                    style.fill = Some(alpha(
                        layer.style.fill.or(row.fill).unwrap_or(
                            crate::scene::Color {
                                red: 229,
                                green: 229,
                                blue: 229,
                                alpha: 255,
                            }
                            .into(),
                        ),
                    ));
                    style.stroke = Some(opacity(layer.style.stroke.or(row.stroke).unwrap_or(base)));
                }
                if !point {
                    if spec.default_line_width && row.stroke_width.is_none() {
                        style.stroke_width = if polygon { 0.2 } else { 0.5 };
                    }
                    if style.units.is_none() {
                        style.stroke_width = super::reference_linewidth(
                            style.stroke_width,
                            crate::services::Units::Points,
                        );
                        style.units = Some(AestheticUnits::Points);
                    }
                    style.line_join = style.line_join.or(Some(LineJoin::Round));
                }
                let targets = vec![
                    row.target.clone();
                    if matches!(geometry, PreparedGeometry::LineRun(_)) {
                        count
                    } else {
                        1
                    }
                ];
                Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                    geometry,
                    style,
                    group: feature.id.clone(),
                    targets,
                    aesthetics: row.values.clone(),
                });
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_sf_partial_reprojection_preserves_geometry_family_policy() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/geography-controls.json"
        ))
        .unwrap();
        let geometries = [
            GeoGeometry::Point([0., 100.]),
            GeoGeometry::LineString(vec![[0., 0.], [0., 100.], [10., 10.]]),
            GeoGeometry::Polygon(vec![vec![[0., 0.], [0., 100.], [10., 10.], [0., 0.]]]),
            GeoGeometry::MultiPoint(vec![[0., 100.], [0., 0.]]),
        ];
        let transform =
            GeoTransform::new(&GeoCrs::Wgs84, &GeoCrs::WebMercator, GeoAxisOrder::XY).unwrap();
        for (index, geometry) in geometries.iter().enumerate() {
            let mut parts = vec![];
            geometry_parts(geometry, &transform, &mut parts).unwrap();
            let points = parts.iter().flat_map(vertices).collect::<Vec<_>>();
            let expected = &fixture["partial_transform"]["coordinates"][index];
            if expected.is_null() {
                assert!(points.is_empty(), "{index}");
            } else {
                let expected: Vec<[f64; 2]> = serde_json::from_value(expected.clone()).unwrap();
                assert_eq!(points.len(), expected.len());
                for (p, e) in points.iter().zip(expected) {
                    assert!((p.x() - e[0]).abs() < 1e-7);
                    assert!((p.y() - e[1]).abs() < 1e-7);
                }
            }
        }
    }
}
