//! FIX-S06: independent D3 contexts plus area/topology/default/ownership checks.
use chart_core::{
    ChartResult,
    path::{Command, Path, PathRequest},
    shape::{SYMBOLS, SYMBOLS_FILL, SYMBOLS_STROKE, ShapeLimits, Symbol, SymbolDraw, SymbolKind},
};
use serde_json::{Value, json};
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 2e-12 * b.abs().max(1.), "{a} != {b}");
}
fn compare(a: &Value, b: &Value) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => near(a.as_f64().unwrap(), b.as_f64().unwrap()),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                compare(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            for (k, a) in a {
                compare(a, &b[k]);
            }
        }
        _ => assert_eq!(a, b),
    }
}
#[test]
fn every_symbol_size_matches_independent_reference_and_palette_aliases() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/symbol.json")).unwrap();
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 156);
    for case in corpus["cases"].as_array().unwrap() {
        let symbol = Symbol::new()
            .kind(serde_json::from_value(case["kind"].clone()).unwrap())
            .size(case["size"].as_f64().unwrap());
        if case["size"].as_f64().unwrap() < 0. {
            assert!(symbol.generate().is_err());
            continue;
        }
        let actual = symbol
            .generate()
            .unwrap_or_else(|e| panic!("{}: {e:?}", case["id"]));
        let expected = PathRequest {
            version: 1,
            digits: Some(3.),
            limits: Default::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        }
        .build()
        .unwrap();
        compare(
            &serde_json::to_value(actual.geometry()).unwrap(),
            &serde_json::to_value(expected.geometry()).unwrap(),
        );
        for digits in [0, 3] {
            assert_eq!(
                symbol
                    .clone()
                    .digits(Some(f64::from(digits)))
                    .unwrap()
                    .generate()
                    .unwrap()
                    .to_svg()
                    .unwrap(),
                case["svg"][digits.to_string()].as_str().unwrap(),
                "{} digits={digits}",
                case["id"]
            );
        }
        assert_eq!(
            symbol.clone().generate().unwrap().geometry(),
            actual.geometry()
        );
    }
    assert_eq!(json!(SYMBOLS_FILL), corpus["palettes"]["fill"]);
    assert_eq!(json!(SYMBOLS_STROKE), corpus["palettes"]["stroke"]);
    assert_eq!(SYMBOLS, SYMBOLS_FILL);
    assert_eq!(SymbolKind::X, SymbolKind::Times);
    assert_eq!(
        serde_json::from_value::<SymbolKind>(json!("X")).unwrap(),
        SymbolKind::Times
    );
    assert_eq!(
        Symbol::new().generate().unwrap().to_svg().unwrap(),
        corpus["default_svg"].as_str().unwrap()
    );
}
fn polygon_area(path: &Path) -> f64 {
    let mut points = Vec::new();
    let mut current = [0., 0.];
    for c in path.geometry().commands() {
        match c {
            Command::MoveTo(p) | Command::LineTo(p) => current = *p,
            Command::Horizontal(dx) => current[0] += dx,
            Command::Vertical(dy) => current[1] += dy,
            Command::Close => continue,
            _ => panic!("polygon contains non-polygon geometry"),
        }
        points.push(current);
    }
    (0..points.len())
        .map(|i| {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .abs()
        / 2.
}

#[test]
fn filled_area_and_stroked_topology_are_distinct() {
    for size in [1., 64., 256.] {
        for kind in SYMBOLS_FILL {
            let p = Symbol::new().kind(kind).size(size).generate().unwrap();
            if kind == SymbolKind::Circle {
                let r = (size / std::f64::consts::PI).sqrt();
                assert!(
                    p.geometry()
                        .commands()
                        .iter()
                        .any(|c| matches!(c,Command::Arc{radius,..} if (*radius-r).abs()<1e-12))
                );
            } else {
                near(polygon_area(&p), size);
            }
        }
    }
    for (kind, subpaths) in [
        (SymbolKind::Plus, 2),
        (SymbolKind::Times, 2),
        (SymbolKind::Asterisk, 3),
    ] {
        let p = Symbol::new().kind(kind).generate().unwrap();
        assert_eq!(
            p.geometry()
                .commands()
                .iter()
                .filter(|c| matches!(c, Command::MoveTo(_)))
                .count(),
            subpaths
        );
        assert!(
            !p.geometry()
                .commands()
                .iter()
                .any(|c| matches!(c, Command::Close))
        );
        assert!(kind.is_open());
    }
    let p = Symbol::new()
        .kind(SymbolKind::Square)
        .size(64.)
        .generate()
        .unwrap();
    assert_eq!(p.geometry().commands()[0], Command::MoveTo([-4., -4.]));
    let p = Symbol::new()
        .kind(SymbolKind::Triangle)
        .size(64.)
        .generate()
        .unwrap();
    let points: Vec<_> = p
        .geometry()
        .commands()
        .iter()
        .filter_map(|c| match c {
            Command::MoveTo(p) | Command::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let lengths: Vec<_> = (0..3)
        .map(|i| {
            (points[i][0] - points[(i + 1) % 3][0]).hypot(points[i][1] - points[(i + 1) % 3][1])
        })
        .collect();
    near(lengths[0], lengths[1]);
    near(lengths[1], lengths[2]);
}
#[test]
fn standalone_draw_native_accessor_and_bounds_share_one_engine() {
    struct Custom;
    impl SymbolDraw for Custom {
        fn draw(&self, p: &mut Path, size: f64) -> ChartResult<()> {
            p.move_to(0., 0.)?;
            p.line_to(size.sqrt(), 0.)
        }
    }
    let g = Symbol::new();
    let p = g
        .generate_by(&(SymbolKind::Star, 100.), |d| Ok(*d))
        .unwrap();
    assert_eq!(
        p.geometry(),
        g.clone()
            .kind(SymbolKind::Star)
            .size(100.)
            .generate()
            .unwrap()
            .geometry()
    );
    let mut direct = Path::new();
    SymbolKind::Star.draw(&mut direct, 100.).unwrap();
    assert_eq!(p.geometry(), direct.geometry());
    assert_eq!(
        g.generate_with(&Custom, 9.).unwrap().geometry().commands(),
        &[Command::MoveTo([0., 0.]), Command::LineTo([3., 0.])]
    );
    for size in [-1., f64::NAN, f64::INFINITY] {
        assert!(g.clone().size(size).generate().is_err());
    }
    assert!(g.clone().digits(Some(-1.)).is_err());
    assert!(
        g.clone()
            .limits(ShapeLimits {
                max_points: 0,
                ..Default::default()
            })
            .generate()
            .is_err()
    );
    assert!(
        g.clone()
            .limits(ShapeLimits {
                path: chart_core::path::PathLimits {
                    max_operations: 1,
                    ..Default::default()
                },
                ..Default::default()
            })
            .generate()
            .is_err()
    );
    assert!(serde_json::from_value::<Symbol>(json!({"unknown":1})).is_err());
    assert_eq!(
        g.clone()
            .digits(None)
            .unwrap()
            .generate()
            .unwrap()
            .geometry(),
        g.generate().unwrap().geometry()
    );
}
