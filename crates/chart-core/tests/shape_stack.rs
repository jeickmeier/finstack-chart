//! FIX-S07 independent complete order/offset corpus and native protocol bounds.
use chart_core::{
    ChartResult, Diagnostic, DiagnosticCode,
    shape::{
        Stack, StackLimits, StackMissing, StackOffset, StackOffsetting, StackOrder, StackOrdering,
    },
};
use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
struct Case {
    id: String,
    matrix: Vec<Vec<Option<f64>>>,
    keys: Vec<String>,
    order: StackOrder,
    offset: StackOffset,
    missing: StackMissing,
    series: Value,
}
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}
fn compare(a: &Value, b: &Value, path: &str) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a.as_f64().unwrap();
            let b = b.as_f64().unwrap();
            assert!(
                (a - b).abs() <= 2e-12 * b.abs().max(1.),
                "{path}: {a} != {b}"
            );
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}");
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                compare(a, b, &format!("{path}/{i}"));
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{path}"
            );
            for (k, a) in a {
                compare(a, &b[k], &format!("{path}/{k}"));
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
#[test]
fn every_order_offset_and_missing_policy_matches_independent_reference() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/shapes/stack.json")).unwrap();
    assert_eq!(corpus.cases.len(), 435);
    for c in corpus.cases {
        let s = Stack::new()
            .keys(c.keys)
            .order(c.order)
            .offset(c.offset)
            .missing(c.missing);
        let data:Vec<_>=c.matrix.iter().enumerate().map(|(i,values)|serde_json::json!({"id":(9007199254741001_u64+2*i as u64).to_string(),"values":values})).collect();
        let actual = s
            .generate_by(&data, |_, key, sample, _| Ok(c.matrix[sample][key]))
            .unwrap_or_else(|e| panic!("{}: {e}", c.id));
        compare(&serde_json::to_value(actual).unwrap(), &c.series, &c.id);
        let copy: Stack = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(copy, s);
    }
}
#[test]
fn signed_endpoints_keep_expand_diverging_and_legacy_distinct() {
    let base = Stack::new().keys(vec!["positive".into(), "negative".into()]);
    let data = vec![vec![Some(2.), Some(-1.)]];
    let expand = base
        .clone()
        .offset(StackOffset::Expand)
        .generate(&data)
        .unwrap();
    assert_eq!(
        (expand[0].points[0].y0.0, expand[0].points[0].y1.0),
        (0., 2.)
    );
    assert_eq!(
        (expand[1].points[0].y0.0, expand[1].points[0].y1.0),
        (2., 1.)
    );
    let diverging = base
        .clone()
        .offset(StackOffset::Diverging)
        .generate(&data)
        .unwrap();
    assert_eq!(
        (diverging[1].points[0].y0.0, diverging[1].points[0].y1.0),
        (-1., 0.)
    );
    let reverse = base.order(StackOrder::Reverse).generate(&data).unwrap();
    assert_eq!((&*reverse[0].key, reverse[0].index), ("positive", 1));
    assert_eq!((&*reverse[1].key, reverse[1].index), ("negative", 0));
}
struct Reversed;
impl StackOrdering for Reversed {
    fn order(&self, s: &[Vec<[f64; 2]>]) -> ChartResult<Vec<usize>> {
        Ok((0..s.len()).rev().collect())
    }
}
struct Shift;
impl StackOffsetting for Shift {
    fn offset(&self, s: &mut [Vec<[f64; 2]>], order: &[usize]) -> ChartResult<()> {
        StackOffset::None.offset(s, order)?;
        for s in s {
            for p in s {
                p[0] += 3.;
                p[1] += 3.;
            }
        }
        Ok(())
    }
}
struct Bad;
impl StackOrdering for Bad {
    fn order(&self, _: &[Vec<[f64; 2]>]) -> ChartResult<Vec<usize>> {
        Ok(vec![0, 0])
    }
}
#[test]
fn native_protocols_accessors_errors_and_budgets_are_checked() {
    let s = Stack::new().keys(vec!["a".into(), "b".into()]);
    let data = [9007199254741001_u64, 9007199254741003];
    let a = s
        .generate_with(
            &data,
            |datum, key, sample, all| {
                assert_eq!(all, data);
                assert_eq!(*datum, data[sample]);
                Ok(Some((key + sample + 1) as f64))
            },
            &Reversed,
            &Shift,
        )
        .unwrap();
    assert_eq!(a[0].index, 1);
    assert_eq!(a[0].points[0].data, data[0]);
    assert_eq!((a[1].points[0].y0.0, a[1].points[0].y1.0), (3., 5.));
    assert!(
        s.generate_with(&data, |_, _, _, _| Ok(Some(1.)), &Bad, &Shift)
            .is_err()
    );
    for order in [vec![0], vec![0, 0], vec![0, 2]] {
        assert!(
            s.clone()
                .order(StackOrder::Explicit(order))
                .generate(&[])
                .is_err()
        );
    }
    assert!(
        s.clone()
            .missing(StackMissing::Error)
            .generate(&[vec![None, Some(1.)]])
            .is_err()
    );
    assert!(s.generate(&[vec![Some(f64::INFINITY), Some(1.)]]).is_err());
    assert!(s.generate(&[vec![Some(1.)]]).is_err());
    assert!(
        s.clone()
            .offset(StackOffset::Expand)
            .generate(&[vec![Some(f64::MAX), Some(f64::MAX)]])
            .is_err()
    );
    assert!(
        s.clone()
            .limits(StackLimits {
                max_cells: 1,
                ..Default::default()
            })
            .generate(&[vec![Some(1.); 2]])
            .is_err()
    );
    assert!(
        s.clone()
            .offset(StackOffset::Wiggle)
            .limits(StackLimits {
                max_work: 3,
                ..Default::default()
            })
            .generate(&[vec![Some(1.); 2]])
            .is_err()
    );
    let error = s
        .generate_by(&data, |_, _, _, _| {
            Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "accessor failure",
                "repair input",
            ))
        })
        .unwrap_err();
    assert_eq!(error.message, "accessor failure");
    let v = s
        .value(Some(7.))
        .generate_by(&data, |_, _, _, _| panic!("constant skips accessor"))
        .unwrap();
    assert_eq!(v[1].points[0].y1.0, 14.);
}
