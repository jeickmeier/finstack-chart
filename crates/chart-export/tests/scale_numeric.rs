//! SP-02 named numeric axes share the standalone kernel and strict v5 descriptors.
use chart_core::{composition::ScaleValue, prelude::*, scales::*, scene::Primitive};
use chart_export::*;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
#[test]
fn each_numeric_family_executes_through_named_axes_and_retained_publication() {
    let cases = [
        (
            NumericFamily::Linear,
            vec![0., 10., 100.],
            vec![0., 50., 100.],
            vec![0., 10., 100.],
            [0., 0.5, 1.],
        ),
        (
            NumericFamily::Pow { exponent: 2. },
            vec![0., 10.],
            vec![0., 1.],
            vec![0., 5., 10.],
            [0., 0.25, 1.],
        ),
        (
            NumericFamily::Pow { exponent: 0.5 },
            vec![0., 10.],
            vec![0., 1.],
            vec![0., 5., 10.],
            [0., 0.5_f64.sqrt(), 1.],
        ),
        (
            NumericFamily::Log { base: 10. },
            vec![-100., -1.],
            vec![0., 1.],
            vec![-100., -10., -1.],
            [0., 0.5, 1.],
        ),
        (
            NumericFamily::Symlog { constant: 1. },
            vec![-9., 9.],
            vec![0., 1.],
            vec![-9., 0., 9.],
            [0., 0.5, 1.],
        ),
        (
            NumericFamily::Radial,
            vec![0., 100.],
            vec![0., 10.],
            vec![0., 25., 100.],
            [0., 0.5, 1.],
        ),
        (
            NumericFamily::Identity,
            vec![0., 10.],
            vec![0., 10.],
            vec![0., 5., 10.],
            [0., 0.5, 1.],
        ),
        (
            NumericFamily::Linear,
            vec![3., 3.],
            vec![0., 1.],
            vec![2., 3., 4.],
            [0.5, 0.5, 0.5],
        ),
        (
            NumericFamily::Linear,
            vec![0., 0., 10.],
            vec![0., 50., 100.],
            vec![0., 5., 10.],
            [0.5, 0.75, 1.],
        ),
    ];
    for (family, domain, range, x, expected) in cases {
        let mut spec = NumericScaleSpec::d3(family.clone());
        spec.domain = domain.into_iter().map(Into::into).collect();
        spec.range = range.into_iter().map(Into::into).collect();
        let axis = x_axis()
            .name("numeric")
            .scale(scale_numeric(spec))
            .visible(false);
        let id = axis.handle().unwrap().id();
        let p = plot(
            Data::columns()
                .column("x", x.clone())
                .column("y", [0., 1., 2.])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .x_axis(x_axis().visible(false))
        .axis(axis)
        .layer(points().axes("numeric", "y"))
        .build()
        .unwrap();
        let wire = p.to_json().unwrap();
        assert_eq!(p.definition().wire_version(), 5);
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
        let mut old: serde_json::Value = serde_json::from_str(&wire).unwrap();
        old["version"] = 4.into();
        assert!(Plot::from_json(&old.to_string()).is_err());
        let request = Output::new(FONT)
            .unwrap()
            .request(&p, export_options(PageSize::points(400., 300.).unwrap()))
            .unwrap();
        let f = request.prepare().unwrap();
        let axis = &f.layout().axes()[&id];
        let chart_core::layout::ResolvedScale::Numeric(scale) = &axis.scale else {
            panic!("wrong numeric axis")
        };
        let positions: Vec<_> = f
            .scene()
            .items()
            .iter()
            .filter_map(|i| match i.primitive {
                Primitive::Point { center, .. } if i.layer.is_some() => Some(center.x()),
                _ => None,
            })
            .collect();
        assert_eq!(positions.len(), 3);
        for (i, t) in expected.into_iter().enumerate() {
            let expected = scale.range().start() * (1. - t) + scale.range().end() * t;
            assert!(
                (positions[i] - expected).abs() < 1e-9,
                "{family:?}: {} != {expected}",
                positions[i]
            );
            assert_eq!(
                axis.map_value(&ScaleValue::Number(x[i])).unwrap().unwrap(),
                positions[i]
            );
        }
        assert!(!f.export(Format::Svg).unwrap().bytes.is_empty());
    }
}
