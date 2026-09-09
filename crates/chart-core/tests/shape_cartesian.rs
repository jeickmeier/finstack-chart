//! FIX-S02/03: actual generator arithmetic against pinned independent shape contexts.
use chart_core::{
    ChartResult, DiagnosticCode,
    path::{Command, Path, PathLimits, PathRequest},
    shape::{
        Area, AreaBoundary, AreaPoint, Coordinate, CurveFactory, CurveProtocol, CurveSpec, Line,
        ShapeLimits,
    },
};
use serde_json::{Value, json};
fn curve(settings: &Value) -> CurveSpec {
    let mut value = json!({"kind":settings["curve"].as_str().unwrap().trim_start_matches("curve")});
    for parameter in ["alpha", "beta", "tension"] {
        if let Some(v) = settings.get(parameter) {
            value[parameter] = v.clone();
        }
    }
    serde_json::from_value(value).unwrap()
}
fn near(actual: f64, expected: f64, id: &str) {
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs().max(1.),
        "{id}: {actual} != {expected}"
    );
}
fn compare(actual: &Value, expected: &Value, id: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => near(a.as_f64().unwrap(), b.as_f64().unwrap(), id),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{id}");
            for (a, b) in a.iter().zip(b) {
                compare(a, b, id);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{id}"
            );
            for (k, a) in a {
                compare(a, &b[k], id);
            }
        }
        _ => assert_eq!(actual, expected, "{id}"),
    }
}
#[test]
fn complete_cartesian_seed_matrix_matches_actual_generators() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/cases.json")).unwrap();
    let mut count = 0;
    for case in corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["family"] == "line" || v["family"] == "area")
    {
        let id = case["id"].as_str().unwrap();
        let data: Vec<Vec<f64>> = serde_json::from_value(case["input"].clone()).unwrap();
        let spec = curve(&case["settings"]);
        let expected = PathRequest {
            version: 1,
            digits: Some(3.),
            limits: Default::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        }
        .build()
        .unwrap();
        let actual = if case["family"] == "line" {
            let mut line = Line::new().curve(spec).unwrap();
            if let Some(mask) = case["settings"].get("defined") {
                line = line.defined(serde_json::from_value::<Vec<bool>>(mask.clone()).unwrap());
            }
            line.generate(&data)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"))
        } else {
            Area::new()
                .curve(spec)
                .unwrap()
                .generate(&data)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"))
        };
        compare(
            &serde_json::to_value(actual.geometry()).unwrap(),
            &serde_json::to_value(expected.geometry()).unwrap(),
            id,
        );
        assert_eq!(
            actual.to_svg().unwrap(),
            case["svg_digits"]["3"].as_str().unwrap_or(""),
            "{id}"
        );
        count += 1;
    }
    assert_eq!(count, 268);
}
#[test]
fn ordering_gaps_independent_boundaries_and_helper_inheritance_are_explicit() {
    let data = [[2., 0.], [0., 1.], [1., 0.]];
    assert_eq!(
        Line::new().generate(&data).unwrap().to_svg().unwrap(),
        "M2,0L0,1L1,0"
    );
    assert_eq!(
        Line::new()
            .defined(vec![true, false, true])
            .generate(&data)
            .unwrap()
            .to_svg()
            .unwrap(),
        "M2,0ZM1,0Z"
    );
    let rows = [[0., 4., 3., 2.], [1., 5., 4., 3.]];
    let area = Area::new()
        .x0(Coordinate::Column(0))
        .y0(Coordinate::Column(1))
        .x1(Some(Coordinate::Column(2)))
        .y1(Some(Coordinate::Column(3)));
    assert_eq!(
        area.generate(&rows).unwrap().to_svg().unwrap(),
        "M3,2L4,3L1,5L0,4Z"
    );
    let area = Area::new()
        .x(Coordinate::Constant(1.2345))
        .y(Coordinate::Constant(2.3456))
        .digits(Some(0.))
        .unwrap();
    assert_eq!(
        area.boundary(AreaBoundary::X1)
            .generate(&data[..1])
            .unwrap()
            .to_svg()
            .unwrap(),
        "M0,2.346Z"
    );
    assert_eq!(
        area.boundary(AreaBoundary::Y1)
            .generate(&data[..1])
            .unwrap()
            .to_svg()
            .unwrap(),
        "M1.235,0Z"
    );
    assert_eq!(
        area.boundary(AreaBoundary::X0),
        area.boundary(AreaBoundary::Y0)
    );
    for curve in [
        CurveSpec::Linear,
        CurveSpec::Basis,
        CurveSpec::Natural,
        CurveSpec::CatmullRom { alpha: 0.5 },
    ] {
        let line = Line::new().curve(curve).unwrap();
        let original = line.generate(&data).unwrap().geometry();
        for digits in [None, Some(0.), Some(12.)] {
            assert_eq!(
                line.clone()
                    .digits(digits)
                    .unwrap()
                    .generate(&data)
                    .unwrap()
                    .geometry(),
                original
            );
        }
    }
}
#[test]
fn independent_monotone_and_natural_invariants_hold() {
    let data = [[0., 0.], [1., 2.], [2., 3.], [4., 3.5]];
    let path = Line::new()
        .curve(CurveSpec::MonotoneX)
        .unwrap()
        .generate(&data)
        .unwrap();
    let mut start = data[0];
    let mut segments = 0;
    for command in path.geometry().commands() {
        if let Command::CubicTo(v) = command {
            for i in 0..=100 {
                let t = i as f64 / 100.;
                let s = 1. - t;
                let y = s * s * s * start[1]
                    + 3. * s * s * t * v[1]
                    + 3. * s * t * t * v[3]
                    + t * t * t * v[5];
                assert!(y >= start[1] - 1e-13 && y <= v[5] + 1e-13);
            }
            start = [v[4], v[5]];
            segments += 1;
        }
    }
    assert_eq!(segments, data.len() - 1);
    let geometry = Line::new()
        .curve(CurveSpec::Natural)
        .unwrap()
        .generate(&data)
        .unwrap()
        .geometry();
    let Command::CubicTo(first) = geometry.commands()[1] else {
        panic!("first cubic")
    };
    let Command::CubicTo(last) = *geometry.commands().last().unwrap() else {
        panic!("last cubic")
    };
    for j in 0..2 {
        near(
            data[0][j] - 2. * first[j] + first[j + 2],
            0.,
            "natural initial curvature",
        );
        near(
            last[j] - 2. * last[j + 2] + last[j + 4],
            0.,
            "natural final curvature",
        );
    }
    for (catmull, cardinal) in [
        (
            CurveSpec::CatmullRom { alpha: 0. },
            CurveSpec::Cardinal { tension: 0. },
        ),
        (
            CurveSpec::CatmullRomOpen { alpha: 0. },
            CurveSpec::CardinalOpen { tension: 0. },
        ),
        (
            CurveSpec::CatmullRomClosed { alpha: 0. },
            CurveSpec::CardinalClosed { tension: 0. },
        ),
    ] {
        assert_eq!(
            Line::new()
                .curve(catmull)
                .unwrap()
                .generate(&data)
                .unwrap()
                .geometry(),
            Line::new()
                .curve(cardinal)
                .unwrap()
                .generate(&data)
                .unwrap()
                .geometry()
        );
    }
}
#[test]
fn input_errors_and_resource_bounds_never_return_partial_owned_geometry() {
    let data = [[0., 1.], [1., 2.], [2., 3.]];
    let calls = std::cell::Cell::new(0);
    let line = Line::new().limits(ShapeLimits {
        max_points: 2,
        ..Default::default()
    });
    assert_eq!(
        line.generate_by(&data, |d, _, _| {
            calls.set(calls.get() + 1);
            Ok(Some(*d))
        })
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(calls.get(), 0);
    assert!(Line::new().defined(vec![true]).generate(&data).is_err());
    assert!(Line::new().generate(&[[0., f64::NAN]]).is_err());
    assert!(
        Line::new()
            .defined(false)
            .generate(&[[f64::NAN, f64::INFINITY]])
            .unwrap()
            .geometry()
            .commands()
            .is_empty()
    );
    assert!(
        Line::new()
            .x(Coordinate::Column(7))
            .generate(&data)
            .is_err()
    );
    assert!(
        CurveSpec::Cardinal { tension: f64::NAN }
            .validate()
            .is_err()
    );
    for beta in [0., 0.85, 1.] {
        assert!(Area::new().curve(CurveSpec::Bundle { beta }).is_err());
    }
    assert!(
        Line::new()
            .limits(ShapeLimits {
                path: PathLimits {
                    max_commands: 2,
                    ..Default::default()
                },
                ..Default::default()
            })
            .generate(&data)
            .is_err()
    );
    assert!(serde_json::from_value::<Line>(json!({"unknown":true})).is_err());
    assert!(serde_json::from_value::<CurveSpec>(json!({"kind":"Basis","tension":0})).is_err());
    let mut path = Path::new();
    {
        let mut curve = CurveSpec::Linear.context(&mut path, 1).unwrap();
        assert!(curve.point(1., 2.).is_err());
        curve.line_start().unwrap();
        assert!(curve.point(f64::NAN, 0.).is_err());
        curve.point(1., 2.).unwrap();
        assert!(curve.point(2., 3.).is_err());
        curve.line_end().unwrap();
    }
    assert_eq!(path.to_svg().unwrap(), "M1,2Z");
}

struct SpyFactory(std::cell::RefCell<Vec<&'static str>>);
struct Spy<'a> {
    path: &'a mut Path,
    events: &'a std::cell::RefCell<Vec<&'static str>>,
    first: bool,
}
impl CurveProtocol for Spy<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.events.borrow_mut().push("areaStart");
        Ok(())
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.events.borrow_mut().push("areaEnd");
        Ok(())
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.events.borrow_mut().push("lineStart");
        self.first = true;
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        self.events.borrow_mut().push("lineEnd");
        Ok(())
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.events.borrow_mut().push("point");
        if self.first {
            self.first = false;
            self.path.move_to(x, y)
        } else {
            self.path.line_to(x, y)
        }
    }
}
impl CurveFactory for SpyFactory {
    fn supports_area(&self) -> bool {
        true
    }
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        _: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>> {
        Ok(Box::new(Spy {
            path,
            events: &self.0,
            first: true,
        }))
    }
}
#[test]
fn native_accessors_and_custom_protocol_receive_exact_order_and_area_lifecycle() {
    let data = [1, 2, 3, 4];
    let factory = SpyFactory(Default::default());
    let mut visited = vec![];
    let result = Area::new()
        .generate_with(&data, &factory, |datum, i, input| {
            visited.push((i, *datum, input.len()));
            Ok((i != 2).then_some(AreaPoint {
                lower: [i as f64, 0.],
                upper: [i as f64, *datum as f64],
            }))
        })
        .unwrap();
    assert_eq!(visited, vec![(0, 1, 4), (1, 2, 4), (2, 3, 4), (3, 4, 4)]);
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/cases.json")).unwrap();
    let expected = corpus["protocols"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["family"] == "area")
        .unwrap();
    assert_eq!(
        factory.0.borrow().as_slice(),
        expected["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v[0].as_str().unwrap())
            .collect::<Vec<_>>()
    );
    let reference = PathRequest {
        version: 1,
        digits: Some(3.),
        limits: Default::default(),
        operations: serde_json::from_value(expected["operations"].clone()).unwrap(),
    }
    .build()
    .unwrap();
    assert_eq!(result.geometry(), reference.geometry());
}

#[test]
fn extended_controls_helpers_and_irregular_matrix_matches_reference() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/shapes/cartesian.json")).unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let data: Vec<Vec<f64>> = serde_json::from_value(case["input"].clone()).unwrap();
        let result = if case["family"] == "line" {
            serde_json::from_value::<Line>(case["config"].clone())
                .unwrap()
                .generate(&data)
        } else {
            let area = serde_json::from_value::<Area>(case["config"].clone()).unwrap();
            if case["helper"].is_null() {
                area.generate(&data)
            } else {
                area.boundary(serde_json::from_value(case["helper"].clone()).unwrap())
                    .generate(&data)
            }
        };
        if case["finite"] == false {
            assert_eq!(
                result.unwrap_err().code,
                DiagnosticCode::NumericalDomain,
                "{id}"
            );
            continue;
        }
        let actual = result.unwrap_or_else(|e| panic!("{id}: {e:?}"));
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
            id,
        );
        assert_eq!(
            actual.to_svg().unwrap(),
            case["svg"].as_str().unwrap_or(""),
            "{id}"
        );
    }
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 561);
}

#[test]
fn generated_paths_keep_the_authored_edit_budget_after_curve_bounds_end() {
    for rows in [vec![], vec![vec![0., 1.]]] {
        let mut path = chart_core::shape::Line::new().generate(&rows).unwrap();
        let saved = path.clone();
        path.move_to(10., 20.).unwrap();
        assert_ne!(path.geometry(), saved.geometry());
        let mut area = chart_core::shape::Area::new().generate(&rows).unwrap();
        area.move_to(10., 20.).unwrap();
    }
    let mut limits = chart_core::shape::ShapeLimits::default();
    limits.path.max_commands = 1;
    let line = chart_core::shape::Line::new().limits(limits);
    let mut path = line.generate(&[] as &[Vec<f64>]).unwrap();
    path.move_to(0., 0.).unwrap();
    assert_eq!(
        path.line_to(1., 1.).unwrap_err().code,
        chart_core::DiagnosticCode::ResourceLimit
    );
}
