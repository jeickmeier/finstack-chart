//! FIX-GG02: independent scalar semantics and bounded stage-typed expression graphs.
use chart_core::{DiagnosticCode, grammar::*};
fn numbers(expr: &Expression<usize>, inputs: &[Option<f64>]) -> Vec<Option<f64>> {
    expr.evaluate(
        inputs.len(),
        ExpressionLimits::default(),
        |_| Ok(ExpressionType::Number),
        |_, i| {
            inputs[i].map_or(
                ExpressionValue::Missing(ExpressionType::Number),
                ExpressionValue::Number,
            )
        },
    )
    .unwrap()
    .iter()
    .map(ExpressionValue::number)
    .collect()
}
#[test]
fn arithmetic_reductions_missingness_and_snapshot_reads() {
    let x = Expression::read(0);
    let ratio = x.clone() / x.clone().reduce(ExpressionReduce::Sum, true);
    assert_eq!(
        numbers(&ratio, &[Some(3.), Some(2.), Some(2.)]),
        vec![Some(3. / 7.), Some(2. / 7.), Some(2. / 7.)]
    );
    assert_eq!(
        numbers(&(x.clone() * 2. + 1.), &[Some(2.), None, Some(-3.)]),
        vec![Some(5.), None, Some(-5.)]
    );
    assert_eq!(
        numbers(
            &x.clone().unary(ExpressionUnary::Log10),
            &[Some(10.), Some(0.), Some(-1.)]
        ),
        vec![Some(1.), None, None]
    );
    assert_eq!(
        numbers(
            &x.clone().reduce(ExpressionReduce::Sum, false),
            &[Some(1.), None]
        ),
        vec![None, None]
    );
    assert_eq!(
        numbers(
            &x.clone().reduce(ExpressionReduce::Sum, true),
            &[Some(1.), None]
        ),
        vec![Some(1.), Some(1.)]
    );
    assert_eq!(
        numbers(
            &x.clone().reduce(ExpressionReduce::Sum, true),
            &[Some(1e16), Some(1.), Some(-1e16)]
        ),
        vec![Some(1.); 3]
    );
    assert_eq!(numbers(&(x / 0.), &[Some(1.)]), vec![None]);
}
#[test]
fn invalid_types_cycles_and_work_are_rejected_even_without_rows() {
    let mut expr = Expression::<usize> {
        nodes: vec![ExpressionNode::Unary {
            op: ExpressionUnary::Abs,
            input: 0,
        }],
        output: 0,
    };
    assert_eq!(
        expr.validate(ExpressionLimits::default(), |_| Ok(ExpressionType::Number))
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
    expr.nodes = vec![
        ExpressionNode::Literal(ExpressionValue::Text("text".into())),
        ExpressionNode::Unary {
            op: ExpressionUnary::Abs,
            input: 0,
        },
    ];
    expr.output = 1;
    assert!(
        expr.evaluate(
            0,
            ExpressionLimits::default(),
            |_| Ok(ExpressionType::Number),
            |_, _| unreachable!()
        )
        .is_err()
    );
    let expr = Expression::<usize>::constant(2.);
    let limits = ExpressionLimits {
        max_cells: 2,
        ..Default::default()
    };
    assert_eq!(
        expr.evaluate(
            3,
            limits,
            |_| Ok(ExpressionType::Number),
            |_, _| unreachable!()
        )
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
    let text = Expression::<usize> {
        nodes: vec![ExpressionNode::Literal(ExpressionValue::Text(
            "abcd".into(),
        ))],
        output: 0,
    };
    assert_eq!(
        text.evaluate(
            2,
            ExpressionLimits {
                max_text_bytes: 5,
                ..Default::default()
            },
            |_| Ok(ExpressionType::Text),
            |_, _| unreachable!()
        )
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
}
#[test]
fn typed_missing_booleans_and_text_have_explicit_semantics() {
    use ExpressionNode as N;
    use ExpressionValue as V;
    let expr = Expression::<usize> {
        nodes: vec![
            N::Literal(V::Missing(ExpressionType::Boolean)),
            N::Literal(V::Boolean(false)),
            N::Binary {
                op: ExpressionBinary::And,
                left: 0,
                right: 1,
            },
        ],
        output: 2,
    };
    assert_eq!(
        expr.evaluate(
            1,
            ExpressionLimits::default(),
            |_| Ok(ExpressionType::Number),
            |_, _| unreachable!()
        )
        .unwrap(),
        vec![V::Boolean(false)]
    );
    let json = serde_json::to_string(&expr).unwrap();
    assert_eq!(
        serde_json::from_str::<Expression<usize>>(&json).unwrap(),
        expr
    );
}

#[test]
fn means_do_not_overflow_or_underflow_from_an_intermediate_sum() {
    let mean = Expression::read(0).reduce(ExpressionReduce::Mean, false);
    assert_eq!(
        numbers(&mean, &[Some(f64::MAX), Some(f64::MAX)]),
        vec![Some(f64::MAX); 2]
    );
    let small = f64::from_bits(1);
    assert_eq!(
        numbers(&mean, &[Some(small), Some(small)]),
        vec![Some(small); 2]
    );
}
