//! ITP-07 standalone wire/consumer contracts, independent of chart envelopes.
use chart_core::{DiagnosticCode, interpolate::*};
#[test]
fn every_descriptor_family_round_trips_and_keeps_owned_samples() {
    let n = Value::number;
    let text = |s: &str| Value::Text(s.into());
    let specs = vec![
        InterpolationSpec::Between {
            factory: InterpolationFactory::new(FactoryKind::Value),
            a: Value::Array(vec![n(0.), text("2px")]),
            b: Value::Array(vec![n(10.), text("6px"), Value::Null]),
        },
        InterpolationSpec::Between {
            factory: InterpolationFactory::new(FactoryKind::Number),
            a: n(f64::NAN),
            b: n(10.),
        },
        InterpolationSpec::Basis {
            values: vec![Number(0.), Number(8.), Number(-2.)],
            closed: false,
        },
        InterpolationSpec::RgbBasis {
            values: vec![text("red"), text("blue"), text("white")],
            closed: true,
        },
        InterpolationSpec::Discrete {
            values: vec![Value::Missing, Value::Null],
        },
        InterpolationSpec::Piecewise {
            factory: InterpolationFactory::new(FactoryKind::Rgb)
                .with_gamma(2.2)
                .unwrap(),
            values: vec![text("red"), text("blue"), text("white")],
        },
        InterpolationSpec::Transform {
            a: [1., 0., 0., 1., 0., 0.],
            b: [2., 0., 0., 3., 10., 20.],
            syntax: TransformSyntax::Css,
        },
        InterpolationSpec::TransformText {
            a: "rotate(350)".into(),
            b: "rotate(10)".into(),
            syntax: TransformSyntax::Svg,
        },
        InterpolationSpec::Zoom {
            a: ZoomView::new([0., 0., 10.]).unwrap(),
            b: ZoomView::new([0., 0., 1.]).unwrap(),
            rho: None,
        },
    ];
    for spec in specs {
        let f = Interpolator::new(spec).unwrap();
        let json = f.to_json().unwrap();
        let copy = Interpolator::from_json(&json).unwrap();
        assert_eq!(f.spec(), copy.spec());
        assert_eq!(copy.to_json().unwrap(), json);
        let held = f.sample(0.3).unwrap();
        for t in [1., 0., 0.5, 0.25] {
            assert_eq!(f.sample(t).unwrap(), copy.sample(t).unwrap());
        }
        assert_eq!(held, f.sample(0.3).unwrap());
        assert_eq!(f.duration_ms(), copy.duration_ms());
        let samples = f.quantize(3).unwrap();
        assert_eq!(samples[0], f.sample(0.).unwrap());
        assert_eq!(samples[2], f.sample(1.).unwrap());
    }
}
#[test]
fn color_consumers_keep_floating_channels_and_portable_validation_is_strict() {
    let f = InterpolationFactory::new(FactoryKind::Value)
        .between(Value::Text("red".into()), Value::Text("blue".into()))
        .unwrap();
    assert_eq!(f.sample_color(0.5).unwrap().rgb().r, 127.5);
    assert_eq!(
        f.sample(0.5).unwrap(),
        Value::Text("rgb(128, 0, 128)".into())
    );
    let f = Interpolator::new(InterpolationSpec::Piecewise {
        factory: InterpolationFactory::new(FactoryKind::Rgb),
        values: vec![
            Value::Text("red".into()),
            Value::Text("blue".into()),
            Value::Text("white".into()),
        ],
    })
    .unwrap();
    assert_eq!(f.sample_color(0.25).unwrap().rgb().r, 127.5);
    let f = InterpolationFactory::new(FactoryKind::Number)
        .between(Value::number(0.), Value::number(1.))
        .unwrap();
    assert!(f.sample_color(0.5).is_err());
    assert!(f.sample_transform(0.5).is_err());
    assert_eq!(f.duration_ms(), None);
    for json in [
        r#"{"version":1,"spec":{"operation":"Unknown"}}"#,
        r#"{"version":1,"extra":0,"spec":{"operation":"Discrete","values":[{"kind":"Null"}]}}"#,
        r#"{"version":1,"spec":{"operation":"Basis","closed":false,"values":[null,1]}}"#,
        r#"{"version":1,"spec":{"operation":"Between","factory":{"kind":"Number","gamma":2},"a":{"kind":"Number","value":1},"b":{"kind":"Number","value":2}}}"#,
    ] {
        assert!(Interpolator::from_json(json).is_err(), "{json}");
    }
    let e = Interpolator::from_json(
        r#"{"version":2,"spec":{"operation":"Discrete","values":[{"kind":"Null"}]}}"#,
    )
    .unwrap_err();
    assert_eq!(e.code, DiagnosticCode::UnsupportedCapability);
    let large = Value::Text("x".repeat(MAX_VALUE_BYTES / 2));
    let f = Interpolator::new(InterpolationSpec::Discrete {
        values: vec![large],
    })
    .unwrap();
    assert!(f.quantize(3).is_err(), "aggregate sampled text bound");
    assert!(
        InterpolationFactory::new(FactoryKind::Number)
            .with_gamma(1.)
            .is_err()
    );
}
