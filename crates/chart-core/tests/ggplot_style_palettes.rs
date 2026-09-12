//! FIX-GG04: pinned scales palette capacities must not cycle overflow categories.
use chart_core::{
    interpolate::{Number, Value},
    scales::*,
};
#[test]
fn discrete_style_palettes_match_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/style-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 18);
    for case in cases {
        let n = case["n"].as_u64().unwrap() as usize;
        let keys: Vec<_> = (0..n)
            .map(|i| ScaleKey::Text(format!("level{i:02}")))
            .collect();
        let palette = match case["kind"].as_str().unwrap() {
            "solid" => GgplotDiscretePalette::Shape { solid: true },
            "hollow" => GgplotDiscretePalette::Shape { solid: false },
            "linetype" => GgplotDiscretePalette::LineType,
            _ => unreachable!(),
        };
        let spec = MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
            .with_ggplot(GgplotScalePolicy::Discrete {
                empty_population: false,
                limits: None,
                levels: None,
                drop: true,
                na_translate: false,
                palette,
            })
            .unwrap()
            .trained_keys(&keys)
            .unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        let scale = MappedScale::new(serde_json::from_str(&json).unwrap()).unwrap();
        let expected = case["values"].as_array().unwrap();
        assert_eq!(expected.len(), n);
        for (key, expected) in keys.iter().zip(expected) {
            let expected = if expected.is_null() {
                Value::Missing
            } else if let Some(v) = expected.as_f64() {
                Value::Number(Number(v))
            } else {
                Value::Text(expected.as_str().unwrap().into())
            };
            assert_eq!(
                scale.category(Some(key)).unwrap(),
                expected,
                "{case} {key:?}"
            );
        }
        assert_eq!(
            scale
                .category(Some(&ScaleKey::Text("unknown".into())))
                .unwrap(),
            Value::Missing
        );
    }
}

#[test]
fn reference_linetypes_reach_primary_rule_styles() {
    use chart_core::{
        grammar::{Compiler, LineType, ValueAesthetic},
        prelude::*,
        state::ChartState,
    };
    let labels: Vec<_> = (0..14).map(|i| format!("level{i:02}")).collect();
    let data = Data::columns()
        .column("x", (0..14).map(f64::from).collect::<Vec<_>>())
        .column("kind", categorical(labels.iter().map(String::as_str)))
        .build()
        .unwrap();
    let scale = MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
        .with_ggplot(GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: None,
            levels: None,
            drop: true,
            na_translate: false,
            palette: GgplotDiscretePalette::LineType,
        })
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(0.).x2("x").y2(1.))
        .layer(rule().value_scale(ValueAesthetic::LineType, "kind", scale))
        .build()
        .unwrap();
    let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let prepared = Compiler::new()
        .prepare(
            restored.definition(),
            &restored.source(),
            &ChartState::default(),
            restored.compile_limits(),
        )
        .unwrap();
    let marks = prepared.layers()[0].marks();
    assert_eq!(marks.len(), 14);
    assert_eq!(marks[0].style.line_type, Some(LineType::Solid));
    assert_eq!(marks[1].style.line_type, Some(LineType::Custom(0x22)));
    assert_eq!(marks[12].style.line_type, Some(LineType::Custom(0xF1)));
    assert_eq!(marks[13].style.line_type, Some(LineType::Blank));
}

#[test]
fn reference_shape_palette_reaches_points_and_constant_override() {
    use chart_core::{
        grammar::{Compiler, PreparedGeometry, ValueAesthetic},
        prelude::*,
        shape::{Symbol, SymbolKind},
        state::ChartState,
    };
    let labels = ["a", "b", "c", "d", "e", "f", "g"];
    let data = Data::columns()
        .column("x", vec![1.; 7])
        .column("kind", categorical(labels))
        .build()
        .unwrap();
    let scale = MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
        .with_ggplot(GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: None,
            levels: None,
            drop: true,
            na_translate: false,
            palette: GgplotDiscretePalette::Shape { solid: true },
        })
        .unwrap();
    let base = points()
        .radius(3.)
        .value_scale(ValueAesthetic::Shape, "kind", scale);
    for constant in [false, true] {
        let layer = if constant {
            base.clone()
                .aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.)))
        } else {
            base.clone()
        };
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(layer)
            .build()
            .unwrap();
        let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let prepared = Compiler::new()
            .prepare(
                restored.definition(),
                &restored.source(),
                &ChartState::default(),
                restored.compile_limits(),
            )
            .unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), if constant { 7 } else { 6 });
        for (i, mark) in marks.iter().enumerate() {
            let code = if constant {
                21
            } else {
                [16, 17, 15, 3, 7, 8][i]
            };
            let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
                panic!("point shape did not reach shared geometry")
            };
            let expected = Symbol::new()
                .kind(SymbolKind::Ggplot(code))
                .size(9. * std::f64::consts::PI)
                .generate()
                .unwrap()
                .geometry();
            assert_eq!(*geometry, expected);
            if constant {
                assert_eq!(mark.style.fill.unwrap().alpha, 0);
            }
        }
    }
}

#[test]
fn automatic_style_mappings_share_population_and_survive_layer_edits() {
    use chart_core::{
        grammar::{Compiler, ValueAesthetic as V},
        prelude::*,
        state::ChartState,
    };
    let d = Data::columns()
        .column("x", [1., 2., 3.])
        .column("kind", categorical(["b", "a", "b"]))
        .build()
        .unwrap();
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.).shape("kind"))
        .layer(points().name("first"))
        .layer(points().name("second"))
        .build()
        .unwrap();
    let a = &p.definition().layers[0].value_scales[&V::Shape];
    let b = &p.definition().layers[1].value_scales[&V::Shape];
    assert_eq!(a.id, b.id);
    let edited = p
        .edit()
        .layer("first", points().radius(5.))
        .build()
        .unwrap();
    assert_eq!(
        edited.definition().layers[0].value_scales[&V::Shape].id,
        a.id
    );
    let prepared = Compiler::new()
        .prepare(
            edited.definition(),
            &edited.source(),
            &ChartState::default(),
            edited.compile_limits(),
        )
        .unwrap();
    assert_eq!(
        prepared.layers()[0].marks()[0].aesthetics[&V::Shape],
        Value::Number(Number(17.))
    );
    assert_eq!(
        prepared.layers()[0].marks()[1].aesthetics[&V::Shape],
        Value::Number(Number(16.))
    );
    assert!(
        plot(d.clone())
            .aes(aes().x("x").y(1.).shape("kind"))
            .layer(points())
            .build()
            .is_err()
    );
    assert!(
        plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.).shape("x"))
            .layer(points())
            .build()
            .is_err()
    );
}

#[test]
fn automatic_alpha_and_linewidth_use_independent_reference_scales() {
    use chart_core::{
        grammar::{Compiler, NumericAesthetic as N},
        prelude::*,
        state::ChartState,
    };
    let d = Data::columns().column("x", [1., 3.]).build().unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(0.).x2("x").y2(1.).alpha("x").linewidth("x"))
        .layer(rule().name("rules"))
        .build()
        .unwrap();
    let scales = &p.definition().layers[0].numeric_scales;
    assert_ne!(scales[&N::Alpha].id, scales[&N::StrokeWidth].id);
    let edited = p
        .edit()
        .layer("rules", rule().color(rgb(255, 0, 0)))
        .build()
        .unwrap();
    assert_eq!(edited.definition().layers[0].numeric_scales, *scales);
    for (plot, expected_alpha, expected_width) in [
        (edited, [26, 255], [1., 6.]),
        (
            p.edit()
                .layer("rules", rule().alpha(0.25).linewidth(2.))
                .build()
                .unwrap(),
            [64, 64],
            [2., 2.],
        ),
    ] {
        let restored = Plot::from_json(&plot.to_json().unwrap()).unwrap();
        let prepared = Compiler::new()
            .prepare(
                restored.definition(),
                &restored.source(),
                &ChartState::default(),
                restored.compile_limits(),
            )
            .unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), 2);
        for i in 0..2 {
            assert_eq!(marks[i].style.color.alpha, expected_alpha[i]);
            assert_eq!(marks[i].style.stroke_width, expected_width[i]);
        }
    }
}
