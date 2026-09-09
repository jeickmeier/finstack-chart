//! FIX-P01–05: offline reference and independent geometry/atomicity expectations.
use chart_core::path::{Command, Path, PathLimits, PathOp, PathSink, Precision, path, path_round};
use chart_core::{ChartResult, Diagnostic, DiagnosticCode};

#[derive(serde::Deserialize)]
struct Corpus {
    schema_version: u32,
    cases: Vec<Case>,
}
#[derive(serde::Deserialize)]
struct Case {
    id: String,
    digits: Option<f64>,
    operations: Vec<PathOp>,
    observations: Vec<Observation>,
}
#[derive(serde::Deserialize)]
struct Observation {
    svg: String,
    current: [Option<f64>; 2],
    start: [Option<f64>; 2],
    error: Option<String>,
}

#[derive(Debug, PartialEq)]
enum Token {
    Op(char),
    Number(f64),
}
fn tokens(svg: &str) -> Vec<Token> {
    let bytes = svg.as_bytes();
    let mut i = 0;
    let mut out = vec![];
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == ',' || c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() {
            out.push(Token::Op(c));
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len()
            && (bytes[i].is_ascii_digit()
                || matches!(bytes[i], b'.' | b'e' | b'E')
                || (matches!(bytes[i], b'+' | b'-') && matches!(bytes[i - 1], b'e' | b'E')))
        {
            i += 1;
        }
        out.push(Token::Number(svg[start..i].parse().unwrap()));
    }
    out
}
fn near(actual: f64, expected: f64, context: &str) {
    // Unitless path-coordinate/trigonometric arithmetic: 2e-12 relative or absolute.
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs().max(1.),
        "{context}: {actual} != {expected}"
    );
}
fn compare_point(actual: Option<[f64; 2]>, expected: [Option<f64>; 2], context: &str) {
    match (actual, expected) {
        (None, [None, None]) => (),
        (Some(a), [Some(x), Some(y)]) => {
            near(a[0], x, context);
            near(a[1], y, context);
        }
        pair => panic!("{context}: state topology {pair:?}"),
    }
}
#[test]
fn all_pinned_sequences_match_offline_at_every_operation() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-path/cases.json")).unwrap();
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.cases.len(), 86);
    for case in corpus.cases {
        let mut builder = Path::with_digits(case.digits).unwrap();
        assert_eq!(case.operations.len(), case.observations.len());
        for (operation, expected) in case.operations.iter().zip(case.observations) {
            let before = builder.clone();
            let result = builder.apply(operation);
            assert_eq!(
                result.is_err(),
                expected.error.is_some(),
                "{} {operation:?}: {result:?}",
                case.id
            );
            if result.is_err() {
                assert_eq!(builder, before, "{}: non-atomic failure", case.id);
            }
            let actual = builder.to_svg().unwrap();
            if case.id.starts_with("round-") && !case.id.starts_with("round-small")
                || case.id == "unrounded-magnitudes"
            {
                assert_eq!(
                    actual, expected.svg,
                    "{}: canonical numeric spelling",
                    case.id
                );
            } else {
                let (a, e) = (tokens(&actual), tokens(&expected.svg));
                assert_eq!(
                    a.len(),
                    e.len(),
                    "{}: {actual} != {}",
                    case.id,
                    expected.svg
                );
                for (a, e) in a.iter().zip(e.iter()) {
                    match (a, e) {
                        (Token::Number(a), Token::Number(e)) => near(*a, *e, &case.id),
                        _ => assert_eq!(a, e, "{}: command topology", case.id),
                    }
                }
            }
            compare_point(builder.current_point(), expected.current, &case.id);
            compare_point(builder.start_point(), expected.start, &case.id);
        }
    }
}
#[test]
fn independent_tangency_rectangles_and_precision() {
    let mut p = path();
    p.move_to(0., 0.).unwrap();
    p.arc_to(10., 0., 10., 10., 2.).unwrap();
    let geometry = p.geometry();
    let [
        Command::MoveTo([0., 0.]),
        Command::LineTo(first),
        Command::Arc {
            radius,
            large: false,
            clockwise: true,
            to: last,
        },
    ] = geometry.commands()
    else {
        panic!("right-angle tangent topology")
    };
    near(first[0], 8., "first tangent");
    near(first[1], 0., "first tangent");
    near(last[0], 10., "second tangent");
    near(last[1], 2., "second tangent");
    near(*radius, 2., "radius");
    // Center (8,2): both endpoints have radius 2 and their tangents align with the lines.
    near((first[0] - 8.).hypot(first[1] - 2.), 2., "first radius");
    near((last[0] - 8.).hypot(last[1] - 2.), 2., "second radius");
    p.rect(10., 20., -5., 4.).unwrap();
    assert_eq!(p.current_point(), Some([10., 20.]));
    assert!(p.to_svg().unwrap().ends_with("M10,20h-5v4h5Z"));
    assert_eq!(geometry.commands().len(), 3, "prior result mutated");
    p.geometry().validate_for_scene().unwrap();
    let mut rounded = path_round();
    rounded.move_to(1.23456, 2.34567).unwrap();
    assert_eq!(rounded.to_svg().unwrap(), "M1.235,2.346");
    let exact = rounded.geometry();
    assert_eq!(
        exact.to_svg(Precision::Unrounded, 100).unwrap(),
        "M1.23456,2.34567"
    );
    assert_eq!(exact.commands(), rounded.geometry().commands());
    let mut ties = Path::with_digits(Some(0.)).unwrap();
    ties.move_to(1.5, -1.5).unwrap();
    ties.line_to(-0., 0.).unwrap();
    assert_eq!(ties.to_svg().unwrap(), "M2,-1L0,0");
    for digits in [-1., f64::NAN, f64::INFINITY] {
        assert!(Path::with_digits(Some(digits)).is_err());
    }
    assert_eq!(
        Precision::from_digits(15.99).unwrap(),
        Precision::Digits(15)
    );
    assert_eq!(Precision::from_digits(16.).unwrap(), Precision::Unrounded);
}
#[test]
fn rejected_arguments_budgets_batches_and_external_sink_errors() {
    let mut p = path();
    p.move_to(1., 2.).unwrap();
    let original = p.clone();
    let bad = [
        PathOp::MoveTo([f64::NAN, 0.]),
        PathOp::LineTo([0., f64::INFINITY]),
        PathOp::QuadraticCurveTo([0., 0., f64::NEG_INFINITY, 0.]),
        PathOp::BezierCurveTo([0., 0., 0., f64::NAN, 0., 0.]),
        PathOp::ArcTo([0., 0., 1., 1., -1.]),
        PathOp::Arc {
            x: 0.,
            y: 0.,
            r: 1.,
            a0: f64::NAN,
            a1: 1.,
            anticlockwise: false,
        },
        PathOp::Rect([f64::MAX, 0., f64::MAX, 1.]),
        PathOp::ArcTo([1e200, 0., 0., 1e200, 1.]),
    ];
    for operation in bad {
        assert!(p.apply(&operation).is_err());
        assert_eq!(p, original);
    }
    assert!(
        p.apply_batch(&[
            PathOp::LineTo([3., 4.]),
            PathOp::ArcTo([0., 0., 1., 1., -1.])
        ])
        .is_err()
    );
    assert_eq!(p, original);
    let ops = [
        PathOp::LineTo([3., 4.]),
        PathOp::ClosePath,
        PathOp::LineTo([5., 6.]),
    ];
    let mut batch = p.clone();
    batch.apply_batch(&ops).unwrap();
    for op in &ops {
        p.apply(op).unwrap();
    }
    assert_eq!(p, batch);
    p.geometry().validate_for_scene().unwrap();
    let mut tiny = Path::with_options(
        Precision::Unrounded,
        PathLimits {
            max_commands: 1,
            ..PathLimits::default()
        },
    )
    .unwrap();
    tiny.move_to(0., 0.).unwrap();
    let before = tiny.clone();
    assert!(tiny.rect(0., 0., 1., 1.).is_err());
    assert_eq!(tiny, before);
    let mut one = Path::with_options(
        Precision::Unrounded,
        PathLimits {
            max_operations: 1,
            ..PathLimits::default()
        },
    )
    .unwrap();
    one.close_path().unwrap();
    assert_eq!(one.operation_count(), 1);
    assert!(one.close_path().is_err());
    assert_eq!(
        p.geometry()
            .to_svg(Precision::Unrounded, 2)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let mut bare = path();
    bare.line_to(1., 2.).unwrap();
    assert_eq!(bare.to_svg().unwrap(), "L1,2");
    assert!(bare.geometry().validate_for_scene().is_err());
    struct Sink(Vec<Command>);
    impl PathSink for Sink {
        fn command(&mut self, c: &Command) -> ChartResult<()> {
            if self.0.len() == 2 {
                return Err(Diagnostic::error(
                    DiagnosticCode::Cancelled,
                    "sink rejected",
                    "stop replay",
                ));
            }
            self.0.push(*c);
            Ok(())
        }
    }
    let mut sink = Sink(vec![]);
    assert!(p.replay(&mut sink).is_err());
    assert_eq!(sink.0.len(), 2);
    let mut untouched = Sink(vec![]);
    assert!(p.geometry().replay(&mut untouched, 1).is_err());
    assert!(untouched.0.is_empty());
}
