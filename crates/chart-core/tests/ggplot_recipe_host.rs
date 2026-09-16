//! GG07 recipe adapters preserve owned mappings and capability versions.
use chart_core::{
    grammar::{BuiltinRecipe, IntervalKind, IntervalRecipe, RecipeAesthetic},
    plot::host::{Component, Draft},
    prelude::*,
};

#[test]
fn recipe_host_fields_expressions_and_stat_channels_roundtrip() {
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [3., 4.])
        .column("lo", [2., 3.])
        .column("hi", [4., 5.])
        .build()
        .unwrap();
    let aes = Component::new("aes", "[]")
        .unwrap()
        .set("x", r#"["x"]"#)
        .unwrap()
        .set("y", r#"["y"]"#)
        .unwrap();
    let base = Component::new("errorbar", "[]").unwrap();
    let layer = base
        .recipe_value_field(RecipeAesthetic::Lower, data.field("lo").unwrap())
        .unwrap()
        .recipe_value_expression(
            RecipeAesthetic::Upper,
            &Component::source_expression(data.field("hi").unwrap()),
        )
        .unwrap();
    let plot = Draft::new(&data)
        .with("aes", &aes)
        .unwrap()
        .with("layer", &layer)
        .unwrap()
        .build()
        .unwrap();
    let wire = plot.to_json().unwrap();
    assert_eq!(plot.definition().wire_version(), 74);
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    assert!(
        !plot.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .is_empty()
    );
    let mut lower: serde_json::Value = serde_json::from_str(&wire).unwrap();
    lower["version"] = 73.into();
    assert!(Plot::from_json(&lower.to_string()).is_err());
    let other = Data::columns().column("lo", [0.]).build().unwrap();
    let invalid = base
        .recipe_value_field(RecipeAesthetic::Lower, other.field("lo").unwrap())
        .unwrap();
    assert!(
        Draft::new(&data)
            .with("aes", &aes)
            .unwrap()
            .with("layer", &invalid)
            .unwrap()
            .build()
            .is_err()
    );
}

#[test]
fn every_recipe_capability_requires_new_wire_even_without_a_recipe_selector() {
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [3., 4.])
        .build()
        .unwrap();
    for layer in [
        points().recipe(BuiltinRecipe::Interval(IntervalRecipe {
            kind: IntervalKind::LineRange,
            width: None,
            ..Default::default()
        })),
        points().recipe_value(RecipeAesthetic::Width, 0.5),
        line().lineend(LineEnd::Round),
        line().linejoin(LineJoin::Bevel),
        points().stat(count().sum_count().x("x").y("y")),
        points().stat(count().ggplot_count().count_partition("x")),
    ] {
        let result = plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build()
            .unwrap();
        assert_eq!(result.definition().wire_version(), 74);
        assert_eq!(
            Plot::from_json(&result.to_json().unwrap())
                .unwrap()
                .definition()
                .wire_version(),
            74
        );
    }
}

#[test]
fn implicit_count_partitions_evaluated_expression_values_without_overpartitioning() {
    use chart_core::{
        grammar::{CountRecipe, ExpressionUnary, NumericAesthetic, PreparedRows, StatField},
        scales::{GgplotNumericIdentity, MappedScaleSpec, ScaleFunctionSpec, ScaleTraining},
    };
    let data = Data::columns()
        .column("x", [1., 1., 1.])
        .column("y", [2., 2., 2.])
        .column("v", [-1., 1., 2.])
        .build()
        .unwrap();
    let mut scale = MappedScaleSpec::authored(ScaleFunctionSpec::GgplotNumericIdentity(
        GgplotNumericIdentity::default(),
    ));
    scale.training = ScaleTraining::Eligible;
    let original = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .recipe(BuiltinRecipe::Count(CountRecipe::default()))
                .numeric_scale(
                    NumericAesthetic::Size,
                    source_expr("v").unary(ExpressionUnary::Abs),
                    scale,
                ),
        )
        .build()
        .unwrap();
    for plot in [
        &original,
        &Plot::from_json(&original.to_json().unwrap()).unwrap(),
    ] {
        let prepared = plot.chart().unwrap().prepare().unwrap();
        let layer = &prepared.layers()[0];
        let PreparedRows::Statistical(rows) = layer.table().rows() else {
            panic!("expected StatSum")
        };
        assert_eq!(
            rows.len(),
            2,
            "abs(-1) and abs(1) must share one count population"
        );
        let mut actual = rows
            .iter()
            .zip(layer.marks())
            .map(|(row, mark)| {
                (
                    mark.style.radius,
                    row.value(&StatField::WeightedCount).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(actual, [(1., 2.), (2., 1.)]);
    }
}
