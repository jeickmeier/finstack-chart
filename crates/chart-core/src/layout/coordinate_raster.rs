//! Bounded inverse image sampling and pixel-exact source-cell inspection coverage.
use super::{LayoutRequest, coordinate_map::CoordinateMap};
use crate::grammar::{CoordinateClip, RasterAnnotation};
use crate::scene::{Color, Primitive, RasterHitRegion};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
fn error(code: DiagnosticCode, message: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use an invertible coordinate map or increase the explicit raster work budget.",
    )
}
fn charge(remaining: &mut usize, n: usize) -> ChartResult<()> {
    *remaining = remaining.checked_sub(n).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Coordinate image work budget exceeded.",
        )
    })?;
    Ok(())
}
fn inside(r: Rect, p: Point) -> bool {
    p.x() >= r.origin().x() && p.x() <= r.max_x() && p.y() >= r.origin().y() && p.y() <= r.max_y()
}
fn intersection(a: Rect, b: Rect) -> ChartResult<Option<Rect>> {
    let x = a.origin().x().max(b.origin().x());
    let y = a.origin().y().max(b.origin().y());
    let right = a.max_x().min(b.max_x());
    let bottom = a.max_y().min(b.max_y());
    if right <= x || bottom <= y {
        Ok(None)
    } else {
        Rect::new(x, y, right - x, bottom - y).map(Some)
    }
}
fn pixel_index(raster: &RasterAnnotation, u: f64, v: f64) -> usize {
    let x = (u.floor().max(0.) as usize).min(raster.width - 1);
    let y = (v.floor().max(0.) as usize).min(raster.height - 1);
    y * raster.width + x
}
/// Pixel-center bilinear interpolation in premultiplied sRGB, with clamped edge pixels.
fn sample(raster: &RasterAnnotation, u: f64, v: f64, linear: bool) -> Color {
    if !linear {
        return raster.pixels[pixel_index(raster, u, v)];
    }
    let x = u - 0.5;
    let y = v - 0.5;
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let mut color = [0.; 4];
    for (dx, dy, w) in [
        (0., 0., (1. - fx) * (1. - fy)),
        (1., 0., fx * (1. - fy)),
        (0., 1., (1. - fx) * fy),
        (1., 1., fx * fy),
    ] {
        let c = raster.pixels[pixel_index(raster, x0 + dx, y0 + dy)];
        let a = f64::from(c.alpha) / 255.;
        color[0] += f64::from(c.red) * a * w;
        color[1] += f64::from(c.green) * a * w;
        color[2] += f64::from(c.blue) * a * w;
        color[3] += a * w;
    }
    if color[3] == 0. {
        return Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        };
    }
    let channel = |i: usize| (color[i] / color[3]).round().clamp(0., 255.) as u8;
    Color {
        red: channel(0),
        green: channel(1),
        blue: channel(2),
        alpha: (color[3] * 255.).round().clamp(0., 255.) as u8,
    }
}
/// Warp one existing image. Returned coverage indexes the unchanged original target array.
/// Resolution is at least two samples per destination unit and one per device pixel.
/// Device scale selects the density; it does not multiply an existing density twice.
pub(super) fn project(
    primitive: &Primitive,
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    if matches!(
        primitive,
        Primitive::GradientRectangle { .. } | Primitive::SampledGradientRectangle { .. }
    ) {
        let image = gradient_image(primitive, request, remaining)?;
        return project(&image, map, request, remaining);
    }
    let Primitive::RasterImage {
        bounds: source,
        raster,
        interpolate,
        cells,
        ..
    } = primitive
    else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Coordinate raster sampler requires an image.",
        ));
    };
    let source_count = raster
        .width
        .checked_mul(raster.height)
        .filter(|n| *n == raster.pixels.len() && *n > 0)
        .ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Coordinate image dimensions are invalid.",
            )
        })?;
    charge(remaining, source_count)?;
    let corners = [
        Point::new(source.origin().x(), source.origin().y())?,
        Point::new(source.max_x(), source.origin().y())?,
        Point::new(source.max_x(), source.max_y())?,
        Point::new(source.origin().x(), source.max_y())?,
    ];
    let Some(bounds) = map.hull(&corners)? else {
        return Ok(vec![]);
    };
    let Some(bounds) = intersection(bounds, request.figure_bounds.unwrap_or(request.bounds))?
    else {
        return Ok(vec![]);
    };
    let bounds = if map.clip() == CoordinateClip::On {
        let Some(b) = intersection(bounds, map.plot)? else {
            return Ok(vec![]);
        };
        b
    } else {
        bounds
    };
    let scale = sample_density(request);
    if !scale.is_finite() || scale <= 0. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Coordinate raster scale must be finite and positive.",
        ));
    }
    let width = (bounds.width() * scale).ceil();
    let height = (bounds.height() * scale).ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width < 1.
        || height < 1.
        || width > remaining.saturating_add(1) as f64
        || height > remaining.saturating_add(1) as f64
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Coordinate raster dimensions exceed the work budget.",
        ));
    }
    let (width, height) = (width as usize, height as usize);
    let count = width.checked_mul(height).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Coordinate raster allocation overflow.",
        )
    })?;
    charge(remaining, count)?;
    if count > request.limits.max_path_commands {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Coordinate raster exceeds the scene pixel budget.",
        ));
    }
    // Original raster cells are grid-aligned. Build the ownership lookup once rather than
    // searching every cell for every destination sample.
    let mut owners = vec![if cells.is_empty() { Some(0) } else { None }; source_count];
    for (target, cell) in cells.iter().enumerate() {
        charge(remaining, 1)?;
        let Some(cell) = intersection(*cell, *source)? else {
            continue;
        };
        let xa =
            (((cell.origin().x() - source.origin().x()) / source.width() * raster.width as f64)
                .floor()
                .max(0.) as usize)
                .min(raster.width);
        let xb = (((cell.max_x() - source.origin().x()) / source.width() * raster.width as f64)
            .ceil()
            .max(0.) as usize)
            .min(raster.width);
        let ya = (((cell.origin().y() - source.origin().y()) / source.height()
            * raster.height as f64)
            .floor()
            .max(0.) as usize)
            .min(raster.height);
        let yb = (((cell.max_y() - source.origin().y()) / source.height() * raster.height as f64)
            .ceil()
            .max(0.) as usize)
            .min(raster.height);
        charge(remaining, (xb - xa).saturating_mul(yb - ya))?;
        for y in ya..yb {
            for x in xa..xb {
                let p = Point::new(
                    source.origin().x() + (x as f64 + 0.5) / raster.width as f64 * source.width(),
                    source.origin().y() + (y as f64 + 0.5) / raster.height as f64 * source.height(),
                )?;
                if inside(cell, p) {
                    owners[y * raster.width + x] = Some(target);
                }
            }
        }
    }
    let transparent = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };
    let mut pixels = vec![transparent; count];
    let mut hits = vec![];
    for y in 0..height {
        let mut run: Option<(usize, usize)> = None;
        for x in 0..=width {
            let mut target = None;
            if x < width {
                let p = Point::new(
                    bounds.origin().x() + (x as f64 + 0.5) / width as f64 * bounds.width(),
                    bounds.origin().y() + (y as f64 + 0.5) / height as f64 * bounds.height(),
                )?;
                if map.clip() != CoordinateClip::On || map.contains(p) {
                    let inverse = match map.inverse_in(p, *source, 8)? {
                        Some(values) => values,
                        // Angular position at the exact center has no unique source value.
                        // Its single sampling cell remains transparent; public inverse stays unavailable.
                        None if matches!(map.spec, crate::grammar::CoordinateSpec::Radial(_)) => {
                            vec![]
                        }
                        None => {
                            return Err(error(
                                DiagnosticCode::UnsupportedCapability,
                                "Image coordinate map has no inverse.",
                            ));
                        }
                    };
                    let mut sources = inverse.into_iter().filter(|p| {
                        p.x() >= source.origin().x()
                            && p.x() < source.max_x()
                            && p.y() >= source.origin().y()
                            && p.y() < source.max_y()
                    });
                    if let Some(p) = sources.next() {
                        if sources.next().is_some() {
                            return Err(error(
                                DiagnosticCode::UnsupportedCapability,
                                "Image coordinate map has multiple overlapping inverse branches.",
                            ));
                        }
                        let u =
                            (p.x() - source.origin().x()) / source.width() * raster.width as f64;
                        let v =
                            (p.y() - source.origin().y()) / source.height() * raster.height as f64;
                        pixels[y * width + x] = sample(raster, u, v, *interpolate);
                        target = owners[pixel_index(raster, u, v)];
                    }
                }
            }
            if run.as_ref().map(|r| r.1) != target {
                if let Some((start, target_index)) = run.take() {
                    charge(remaining, 5)?;
                    if count.saturating_add((hits.len() + 1).saturating_mul(5))
                        > request.limits.max_path_commands
                    {
                        return Err(error(
                            DiagnosticCode::ResourceLimit,
                            "Coordinate raster hit coverage exceeds scene budget.",
                        ));
                    }
                    let left = bounds.origin().x() + start as f64 / width as f64 * bounds.width();
                    let right = if x == width {
                        bounds.max_x()
                    } else {
                        bounds.origin().x() + x as f64 / width as f64 * bounds.width()
                    };
                    let top = bounds.origin().y() + y as f64 / height as f64 * bounds.height();
                    let bottom = if y + 1 == height {
                        bounds.max_y()
                    } else {
                        bounds.origin().y() + (y + 1) as f64 / height as f64 * bounds.height()
                    };
                    hits.push(RasterHitRegion {
                        bounds: Rect::new(left, top, right - left, bottom - top)?,
                        target_index,
                    });
                }
                if let Some(target) = target {
                    run = Some((x, target));
                }
            }
        }
    }
    if hits.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![Primitive::RasterImage {
        bounds,
        raster: RasterAnnotation {
            width,
            height,
            pixels,
        },
        interpolate: false,
        cells: vec![],
        hits,
    }])
}

struct GradientSamples {
    bounds: Rect,
    direction: crate::scene::GradientDirection,
    stops: Vec<(f64, Color)>,
    steps: bool,
}
impl GradientSamples {
    fn new(primitive: &Primitive, remaining: &mut usize) -> ChartResult<Self> {
        let count = match primitive {
            Primitive::GradientRectangle { .. } => 2,
            Primitive::SampledGradientRectangle { colors, mode, .. } => colors
                .len()
                .saturating_mul(if *mode == crate::scene::SampledGradientMode::Steps {
                    2
                } else {
                    1
                }),
            _ => 0,
        };
        charge(remaining, count)?;
        match primitive {
            Primitive::GradientRectangle { bounds, gradient } => Ok(Self {
                bounds: *bounds,
                direction: gradient.direction,
                stops: vec![(0., gradient.start), (1., gradient.end)],
                steps: false,
            }),
            Primitive::SampledGradientRectangle {
                bounds,
                direction,
                colors,
                mode,
            } if colors.len() >= 2 => Ok(Self {
                bounds: *bounds,
                direction: *direction,
                stops: mode.stops(colors).collect(),
                steps: *mode == crate::scene::SampledGradientMode::Steps,
            }),
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Expected a validated gradient rectangle.",
            )),
        }
    }
    fn at(&self, p: Point) -> Color {
        let t = match self.direction {
            crate::scene::GradientDirection::Horizontal => {
                (p.x() - self.bounds.origin().x()) / self.bounds.width()
            }
            crate::scene::GradientDirection::Vertical => {
                (p.y() - self.bounds.origin().y()) / self.bounds.height()
            }
        };
        let next = self.stops.partition_point(|(position, _)| *position <= t);
        if next == 0 {
            return self.stops[0].1;
        }
        if next == self.stops.len() {
            return self.stops[next - 1].1;
        }
        let (a, ca) = self.stops[next - 1];
        let (b, cb) = self.stops[next];
        let t = (t - a) / (b - a);
        let wa = (1. - t) * f64::from(ca.alpha);
        let wb = t * f64::from(cb.alpha);
        let alpha = wa + wb;
        if alpha == 0. {
            return Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            };
        }
        let channel = |a: u8, b: u8| {
            ((f64::from(a) * wa + f64::from(b) * wb) / alpha)
                .round()
                .clamp(0., 255.) as u8
        };
        Color {
            red: channel(ca.red, cb.red),
            green: channel(ca.green, cb.green),
            blue: channel(ca.blue, cb.blue),
            alpha: alpha.round().clamp(0., 255.) as u8,
        }
    }
}
fn sample_density(request: &LayoutRequest) -> f64 {
    request.device_scale.unwrap_or(1.).max(2.)
}
fn dimension(length: f64, request: &LayoutRequest, remaining: usize) -> ChartResult<usize> {
    let n = (length * sample_density(request)).ceil();
    if !n.is_finite() || n < 1. || n > remaining as f64 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Gradient raster resolution exceeds budget.",
        ));
    }
    Ok(n as usize)
}
fn gradient_image(
    primitive: &Primitive,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Primitive> {
    let gradient = GradientSamples::new(primitive, remaining)?;
    let horizontal = gradient.direction == crate::scene::GradientDirection::Horizontal;
    let n = dimension(
        if horizontal {
            gradient.bounds.width()
        } else {
            gradient.bounds.height()
        },
        request,
        *remaining,
    )?;
    charge(remaining, n)?;
    let pixels = (0..n)
        .map(|i| {
            let f = (i as f64 + 0.5) / n as f64;
            Point::new(
                gradient.bounds.origin().x()
                    + if horizontal {
                        f * gradient.bounds.width()
                    } else {
                        0.
                    },
                gradient.bounds.origin().y()
                    + if horizontal {
                        0.
                    } else {
                        f * gradient.bounds.height()
                    },
            )
            .map(|p| gradient.at(p))
        })
        .collect::<ChartResult<Vec<_>>>()?;
    Ok(Primitive::RasterImage {
        bounds: gradient.bounds,
        raster: RasterAnnotation {
            width: if horizontal { n } else { 1 },
            height: if horizontal { 1 } else { n },
            pixels,
        },
        interpolate: !gradient.steps,
        cells: vec![],
        hits: vec![],
    })
}
/// Clip destination-space panel gradients without applying the data coordinate transform.
pub(super) fn panel_gradient(
    primitive: &Primitive,
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<Primitive>> {
    let gradient = GradientSamples::new(primitive, remaining)?;
    let Some(bounds) = intersection(gradient.bounds, map.plot)? else {
        return Ok(vec![]);
    };
    let width = dimension(bounds.width(), request, *remaining)?;
    let height = dimension(bounds.height(), request, *remaining)?;
    let count = width.checked_mul(height).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Panel gradient raster allocation overflow.",
        )
    })?;
    charge(remaining, count)?;
    if count > request.limits.max_path_commands {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Panel gradient exceeds scene pixel budget.",
        ));
    }
    let mut pixels = Vec::with_capacity(count);
    for y in 0..height {
        for x in 0..width {
            let p = Point::new(
                bounds.origin().x() + (x as f64 + 0.5) / width as f64 * bounds.width(),
                bounds.origin().y() + (y as f64 + 0.5) / height as f64 * bounds.height(),
            )?;
            pixels.push(if map.contains(p) {
                gradient.at(p)
            } else {
                Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                }
            });
        }
    }
    Ok(vec![Primitive::RasterImage {
        bounds,
        raster: RasterAnnotation {
            width,
            height,
            pixels,
        },
        interpolate: false,
        cells: vec![],
        hits: vec![],
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{CartesianCoordinate, CoordinateSpec, RadialCoordinate, RadialMode};
    use crate::layout::coordinate_map::CoordinateDomain;
    fn request(bounds: Rect) -> LayoutRequest {
        LayoutRequest::new(
            bounds,
            crate::services::Units::Points,
            crate::services::ResourceDescriptor {
                id: crate::ResourceId::new(1),
                revision: crate::Revision::new(1),
                kind: crate::services::ResourceKind::Font,
                byte_len: 1,
            },
        )
    }
    fn map(spec: CoordinateSpec, bounds: Rect) -> CoordinateMap {
        CoordinateMap::new(
            spec,
            [
                CoordinateDomain {
                    domain: [0., 1.],
                    viewport: [0., 1.],
                    range: [bounds.origin().x(), bounds.max_x()],
                },
                CoordinateDomain {
                    domain: [0., 1.],
                    viewport: [0., 1.],
                    range: [bounds.max_y(), bounds.origin().y()],
                },
            ],
            [[0., 1.], [0., 1.]],
            bounds,
        )
        .unwrap()
    }
    fn image(bounds: Rect, interpolate: bool) -> Primitive {
        Primitive::RasterImage {
            bounds,
            cells: vec![
                Rect::new(
                    bounds.origin().x(),
                    bounds.origin().y(),
                    bounds.width() / 2.,
                    bounds.height(),
                )
                .unwrap(),
                Rect::new(
                    bounds.origin().x() + bounds.width() / 2.,
                    bounds.origin().y(),
                    bounds.width() / 2.,
                    bounds.height(),
                )
                .unwrap(),
            ],
            hits: vec![],
            raster: RasterAnnotation {
                width: 2,
                height: 1,
                pixels: vec![
                    Color {
                        red: 255,
                        green: 0,
                        blue: 0,
                        alpha: 255,
                    },
                    Color {
                        red: 0,
                        green: 0,
                        blue: 255,
                        alpha: 255,
                    },
                ],
            },
            interpolate,
        }
    }
    #[test]
    fn clipped_panel_gradient_preserves_steps_and_has_no_hit_regions() {
        let bounds = Rect::new(0., 0., 21., 21.).unwrap();
        let map = map(
            CoordinateSpec::Radial(RadialCoordinate {
                mode: crate::grammar::RadialMode::Radial,
                inner_radius: 0.5,
                clip: Some(CoordinateClip::On),
                expand: false,
                ..Default::default()
            }),
            bounds,
        );
        let red = Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 255,
        };
        let blue = Color {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 255,
        };
        let p = Primitive::SampledGradientRectangle {
            bounds,
            direction: crate::scene::GradientDirection::Horizontal,
            colors: vec![red, blue],
            mode: crate::scene::SampledGradientMode::Steps,
        };
        let out = panel_gradient(&p, &map, &request(bounds), &mut 10000).unwrap();
        let Primitive::RasterImage {
            raster,
            hits,
            cells,
            ..
        } = &out[0]
        else {
            panic!("raster")
        };
        assert!(hits.is_empty() && cells.is_empty());
        assert!(raster.pixels.iter().any(|c| c.alpha == 0));
        assert!(raster.pixels.contains(&red));
        assert!(raster.pixels.contains(&blue));
        assert!(
            raster
                .pixels
                .iter()
                .all(|c| c.alpha == 0 || *c == red || *c == blue)
        );
        assert!(panel_gradient(&p, &map, &request(bounds), &mut 3).is_err());
    }
    #[test]
    fn retina_panel_gradient_samples_each_device_pixel_once_within_default_budget() {
        let bounds = Rect::new(0., 0., 480., 320.).unwrap();
        let map = map(
            CoordinateSpec::Radial(RadialCoordinate {
                mode: crate::grammar::RadialMode::Radial,
                inner_radius: 0.4,
                clip: Some(CoordinateClip::On),
                expand: false,
                ..Default::default()
            }),
            bounds,
        );
        let p = Primitive::SampledGradientRectangle {
            bounds,
            direction: crate::scene::GradientDirection::Horizontal,
            colors: vec![
                Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
                Color {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha: 255,
                },
            ],
            mode: crate::scene::SampledGradientMode::Endpoints,
        };
        let mut request = request(bounds);
        request.device_scale = Some(2.);
        let mut remaining = request.limits.max_path_commands;
        let out = panel_gradient(&p, &map, &request, &mut remaining).unwrap();
        let Primitive::RasterImage { raster, .. } = &out[0] else {
            panic!("raster")
        };
        assert_eq!((raster.width, raster.height), (960, 640));
        assert!(raster.pixels.len() < request.limits.max_path_commands);
    }
    #[test]
    fn identity_pixels_and_original_cell_regions_are_aligned() {
        let bounds = Rect::new(0., 0., 20., 10.).unwrap();
        let map = map(
            CoordinateSpec::Cartesian(CartesianCoordinate::default()),
            bounds,
        );
        let out = project(&image(bounds, false), &map, &request(bounds), &mut 10000).unwrap();
        let Primitive::RasterImage {
            raster,
            hits,
            cells,
            interpolate,
            ..
        } = &out[0]
        else {
            panic!()
        };
        assert_eq!((raster.width, raster.height), (40, 20));
        assert!(cells.is_empty());
        assert!(!interpolate);
        assert_eq!(hits.len(), 40);
        for y in 0..20 {
            for x in 0..40 {
                let pixel = raster.pixels[y * 40 + x];
                assert_eq!(pixel.red, if x < 20 { 255 } else { 0 });
                assert_eq!(pixel.blue, if x < 20 { 0 } else { 255 });
            }
        }
        for h in hits {
            assert_eq!(h.target_index, usize::from(h.bounds.origin().x() >= 10.));
        }
    }
    #[test]
    fn bilinear_sampling_uses_premultiplied_alpha_and_clamped_edges() {
        let raster = RasterAnnotation {
            width: 2,
            height: 1,
            pixels: vec![
                Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
                Color {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha: 0,
                },
            ],
        };
        assert_eq!(
            sample(&raster, 1., 0.5, true),
            Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 128
            }
        );
        assert_eq!(sample(&raster, 0., 0.5, true), raster.pixels[0]);
        assert_eq!(sample(&raster, 1.9, 0.5, false), raster.pixels[1]);
    }
    #[test]
    fn radial_holes_are_transparent_and_excluded_from_target_regions() {
        let bounds = Rect::new(0., 0., 40., 40.).unwrap();
        let map = map(
            CoordinateSpec::Radial(RadialCoordinate {
                mode: RadialMode::Radial,
                clip: Some(CoordinateClip::On),
                inner_radius: 0.5,
                ..Default::default()
            }),
            bounds,
        );
        let out = project(&image(bounds, false), &map, &request(bounds), &mut 100000).unwrap();
        let Primitive::RasterImage {
            bounds,
            raster,
            hits,
            ..
        } = &out[0]
        else {
            panic!()
        };
        let center = Point::new(20., 20.).unwrap();
        assert!(!hits.iter().any(|h| inside(h.bounds, center)));
        assert_eq!(
            raster.pixels[pixel_index(
                raster,
                (center.x() - bounds.origin().x()) / bounds.width() * raster.width as f64,
                (center.y() - bounds.origin().y()) / bounds.height() * raster.height as f64
            )]
            .alpha,
            0
        );
        assert!(hits.iter().any(|h| h.target_index == 0));
        assert!(hits.iter().any(|h| h.target_index == 1));
        assert!(project(&image(*bounds, false), &map, &request(*bounds), &mut 2).is_err());
    }
}
