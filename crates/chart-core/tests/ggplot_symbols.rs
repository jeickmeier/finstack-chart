//! All 26 R point glyphs at four sizes, using R 4.6.1's own PDF-device records.
use chart_core::{
    path::{Command, PathGeometry},
    shape::{Symbol, SymbolKind, SymbolPaint},
};
use serde_json::Value;
#[derive(Debug)]
enum Primitive {
    Circle([f64; 2], f64),
    Line(Vec<[f64; 2]>),
    Polygon(Vec<[f64; 2]>),
}
fn primitives(path: &PathGeometry) -> Vec<Primitive> {
    let mut out = vec![];
    let mut points = vec![];
    let mut circle = None;
    let mut current = [0., 0.];
    fn flush(
        out: &mut Vec<Primitive>,
        points: &mut Vec<[f64; 2]>,
        circle: &mut Option<([f64; 2], f64)>,
        closed: bool,
    ) {
        if let Some((center, radius)) = circle.take() {
            out.push(Primitive::Circle(center, radius));
            points.clear();
        } else if !points.is_empty() {
            out.push(if closed {
                Primitive::Polygon(std::mem::take(points))
            } else {
                Primitive::Line(std::mem::take(points))
            });
        }
    }
    for command in path.commands() {
        match *command {
            Command::MoveTo(p) => {
                flush(&mut out, &mut points, &mut circle, false);
                current = p;
                points.push(p);
            }
            Command::LineTo(p) => {
                current = p;
                points.push(p);
            }
            Command::Horizontal(d) => {
                current[0] += d;
                points.push(current);
            }
            Command::Vertical(d) => {
                current[1] += d;
                points.push(current);
            }
            Command::Arc { radius, to, .. } => {
                if circle.is_none() {
                    circle = Some((
                        [(current[0] + to[0]) / 2., (current[1] + to[1]) / 2.],
                        radius,
                    ));
                }
                current = to;
            }
            Command::Close => flush(&mut out, &mut points, &mut circle, true),
            _ => panic!("Point glyph unexpectedly uses polynomial curves"),
        }
    }
    flush(&mut out, &mut points, &mut circle, false);
    out
}
fn near(a: [f64; 2], b: [f64; 2], tol: f64) -> bool {
    (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol
}
fn point(value: &Value) -> [f64; 2] {
    [value[0].as_f64().unwrap(), value[1].as_f64().unwrap()]
}
fn equivalent(actual: &Primitive, expected: &Value, tol: f64) -> bool {
    match actual {
        Primitive::Circle(center, radius) => {
            expected["kind"] == "circle"
                && near(*center, point(&expected["center"]), tol)
                && (radius - expected["radius"].as_f64().unwrap()).abs() <= tol
        }
        Primitive::Line(points) | Primitive::Polygon(points) => {
            let closed = matches!(actual, Primitive::Polygon(_));
            if expected["kind"] != if closed { "polygon" } else { "line" } {
                return false;
            }
            let expected: Vec<_> = expected["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(point)
                .collect();
            if points.len() != expected.len() {
                return false;
            }
            (0..if closed { points.len() } else { 1 }).any(|shift| {
                points
                    .iter()
                    .enumerate()
                    .all(|(i, p)| near(*p, expected[(i + shift) % points.len()], tol))
                    || points.iter().enumerate().all(|(i, p)| {
                        near(
                            *p,
                            expected[(points.len() - 1 - i + shift) % points.len()],
                            tol,
                        )
                    })
            })
        }
    }
}
#[test]
fn every_reference_point_shape_retains_topology_extents_and_paint_mode() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/aesthetics.json"
    ))
    .unwrap();
    let cases = corpus["symbols"].as_array().unwrap();
    assert_eq!(cases.len(), 104);
    let tol = corpus["symbol_coordinate_tolerance_points"]
        .as_f64()
        .unwrap();
    for case in cases {
        let code = case["pch"].as_u64().unwrap() as u8;
        let size = case["device_size"].as_f64().unwrap();
        let kind = SymbolKind::Ggplot(code);
        let paint = SymbolPaint::Auto.resolve(kind).unwrap();
        let path = Symbol::new()
            .kind(kind)
            .size(std::f64::consts::PI * (0.375 * size).powi(2))
            .generate()
            .unwrap();
        let mut actual = primitives(&path.geometry());
        let expected = case["primitives"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "shape {code}, size {size}");
        for expected in expected {
            let at = actual
                .iter()
                .position(|a| equivalent(a, expected, tol))
                .unwrap_or_else(|| {
                    panic!("shape {code}, size {size}: {actual:?} has no {expected}")
                });
            actual.remove(at);
            assert_eq!(
                paint.fills(),
                expected["fill"].as_bool().unwrap(),
                "shape {code}"
            );
            assert_eq!(
                paint.strokes(),
                expected["stroke"].as_bool().unwrap(),
                "shape {code}"
            );
        }
    }
    for code in [26, 31, 255] {
        assert!(
            Symbol::new()
                .kind(SymbolKind::Ggplot(code))
                .generate()
                .is_err()
        );
        assert!(SymbolPaint::Auto.resolve(SymbolKind::Ggplot(code)).is_err());
    }
}
