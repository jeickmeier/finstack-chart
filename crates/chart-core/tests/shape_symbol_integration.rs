//! FIX-S06/09: area-size symbols, explicit type domains, guides and source targets.
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{Compiler, NumericAesthetic as A, PreparedGeometry},
    inspection::{InputOrigin, InspectionAction, InspectionMode, Inspector},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    path::Affine,
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::{Symbol as Generator, SymbolKind as S, SymbolPaint},
    state::ChartState,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(plot: &Plot, log: bool) -> Arc<LaidOutChart> {
    let prepared = Compiler::new()
        .prepare(
            plot.definition(),
            &plot.source(),
            &ChartState::default(),
            plot.compile_limits(),
        )
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.padding = 0.;
    for a in &mut request.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 4.).unwrap()));
    }
    if log {
        request.axes[1].scale = AxisScale::Nonlinear {
            domain: ContinuousDomain::explicit(Bounds::new(1., 10000.).unwrap()),
            transform: chart_core::scales::ScaleTransform::Log { base: 10. },
        };
    }
    Arc::new(layout(Arc::new(prepared), &request, &Metrics).unwrap())
}

fn prepared(p: &Plot) -> chart_core::grammar::PreparedChart {
    Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap()
}
#[test]
fn display_area_is_independent_of_log_center_and_legacy_point_radius() {
    let d = Data::columns()
        .column("x", [2.])
        .column("y", [100.])
        .column("area", [std::f64::consts::PI * 25.])
        .keys([9_007_199_254_741_001])
        .build()
        .unwrap();
    let p = plot(d.clone())
        .aes(aes().x("x").y("y"))
        .layer(shape_symbol().shape_value(A::AreaSize, d.field("area").unwrap()))
        .build()
        .unwrap();
    let f = frame(&p, true);
    let (index, shape) = f
        .scene()
        .items()
        .iter()
        .enumerate()
        .find_map(|(i, v)| {
            if let Primitive::ShapePath {
                geometry,
                anchors,
                fill,
                stroke,
                ..
            } = &v.primitive
            {
                Some((i, (geometry, anchors, fill, stroke)))
            } else {
                None
            }
        })
        .unwrap();
    assert!(shape.2.is_some() && shape.3.is_none());
    assert_eq!(shape.1, &[Point::new(200., 100.).unwrap()]);
    let expected = Generator::new()
        .size(std::f64::consts::PI * 25.)
        .generate()
        .unwrap()
        .geometry()
        .transformed(
            Affine::new([1., 0., 0., 1., 200., 100.]).unwrap(),
            0.01,
            100,
        )
        .unwrap();
    assert_eq!(*shape.0, expected);
    assert_eq!(f.targets()[index].len(), 1);
    let mut inspector = Inspector::new(f.clone(), 0.001, 128).unwrap();
    assert_eq!(
        inspector
            .query(Point::new(204., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    assert!(
        inspector
            .query(Point::new(206., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    inspector
        .dispatch(
            f.scene().stamp(),
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard,
        )
        .unwrap();
    assert_eq!(inspector.hits()[0].target, f.targets()[index][0]);
    let legacy = plot(d)
        .aes(aes().x("x").y("y").size(5.))
        .layer(points())
        .build()
        .unwrap();
    assert!(
        frame(&legacy, true)
            .scene()
            .items()
            .iter()
            .any(|i| matches!(i.primitive, Primitive::Point { radius: 5., .. }))
    );
    assert_eq!(p.definition().wire_version(), 7);
    assert!(Plot::from_json(&p.to_json().unwrap()).is_ok());
}
#[test]
fn mapped_types_and_size_guides_retain_exact_resolved_geometry() {
    let d = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., 2., 2.])
        .column("category", ["a", "b", "c"])
        .column("area", [16., 64., 256.])
        .build()
        .unwrap();
    let p = plot(d.clone())
        .aes(aes().x("x").y("y"))
        .layer(
            shape_symbol()
                .symbol_types(
                    d.field("category").unwrap(),
                    vec!["a".into(), "b".into(), "c".into()],
                    vec![S::Circle, S::Square, S::Plus],
                )
                .shape_value(A::AreaSize, "area")
                .symbol_title("Kind")
                .symbol_size_guide("Area", vec![16., 64., 256.]),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let layer = &pre.layers()[0];
    assert_eq!(layer.marks().len(), 3);
    for ((mark, kind), size) in layer
        .marks()
        .iter()
        .zip([S::Circle, S::Square, S::Plus])
        .zip([16., 64., 256.])
    {
        let PreparedGeometry::ShapePath {
            geometry, paint, ..
        } = &mark.geometry
        else {
            panic!()
        };
        assert_eq!(
            *geometry,
            Generator::new()
                .kind(kind)
                .size(size)
                .generate()
                .unwrap()
                .geometry()
        );
        assert_eq!(
            *paint,
            if kind == S::Plus {
                SymbolPaint::Stroke
            } else {
                SymbolPaint::Fill
            }
        );
    }
    let guides = layer.symbol_legends();
    assert_eq!(guides.len(), 2);
    assert_eq!(
        guides[0].entries.iter().map(|e| e.kind).collect::<Vec<_>>(),
        [S::Circle, S::Square, S::Plus]
    );
    assert_eq!(
        guides[1].entries.iter().map(|e| e.size).collect::<Vec<_>>(),
        [16., 64., 256.]
    );
    let f = frame(&p, false);
    let glyphs: Vec<_> = f
        .scene()
        .items()
        .iter()
        .filter_map(|i| {
            if let Primitive::VectorPath {
                geometry,
                fill,
                stroke,
                ..
            } = &i.primitive
            {
                Some((geometry, fill, stroke))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(glyphs.len(), 6);
    assert!(glyphs[2].1.is_none() && glyphs[2].2.is_some());
    assert!(
        f.scene()
            .items()
            .iter()
            .any(|i| matches!(&i.primitive,Primitive::Text{text,..} if text=="Kind"))
    );
    let wire = p.to_json().unwrap();
    let restored = Plot::from_json(&wire).unwrap();
    assert_eq!(prepared(&restored).layers()[0].symbol_legends(), guides);
}
#[test]
fn open_strokes_and_zero_size_do_not_create_filled_or_invisible_targets() {
    let d = Data::columns()
        .column("x", [2.])
        .column("y", [2.])
        .build()
        .unwrap();
    let p = plot(d.clone())
        .aes(aes().x("x").y("y"))
        .layer(shape_symbol().symbol_kind(S::Plus).symbol_size(256.))
        .build()
        .unwrap();
    let f = frame(&p, false);
    let inspector = Inspector::new(f, 0.001, 128).unwrap();
    assert_eq!(
        inspector
            .query(Point::new(210., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .len(),
        1
    );
    assert!(
        inspector
            .query(Point::new(207., 107.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
    let p = plot(d)
        .aes(aes().x("x").y("y"))
        .layer(shape_symbol().symbol_kind(S::Star).symbol_size(0.))
        .build()
        .unwrap();
    let f = frame(&p, false);
    assert!(
        !f.scene()
            .items()
            .iter()
            .any(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
    );
    assert!(
        Inspector::new(f, 0.001, 128)
            .unwrap()
            .query(Point::new(200., 100.).unwrap(), InspectionMode::Containment)
            .hits
            .is_empty()
    );
}
#[test]
fn explicit_symbol_catalogs_validate_and_preserve_unknown_policy() {
    let d = Data::columns()
        .column("x", [1., 2.])
        .column("y", [2., 2.])
        .column("category", ["a", "missing"])
        .build()
        .unwrap();
    let make = |layer| {
        plot(d.clone())
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build()
    };
    let p =
        make(shape_symbol().symbol_types("category", vec!["a".into()], vec![S::Cross])).unwrap();
    assert_eq!(prepared(&p).layers()[0].marks().len(), 1);
    let p = make(
        shape_symbol()
            .symbol_types("category", vec!["a".into()], vec![S::Cross])
            .symbol_missing(Some(S::Times)),
    )
    .unwrap();
    assert_eq!(prepared(&p).layers()[0].marks().len(), 2);
    for layer in [
        shape_symbol().symbol_size(-1.),
        shape_symbol()
            .symbol_kind(S::Plus)
            .symbol_paint(SymbolPaint::Fill),
        shape_symbol().symbol_types("category", vec!["a".into()], vec![]),
        shape_symbol().symbol_types("category", vec!["a".into(), "a".into()], vec![S::Circle]),
        shape_symbol().shape_value(A::Size, 20.),
        shape_symbol().symbol_size_guide("Invalid", vec![2.]),
        points().shape_value(A::AreaSize, 2.),
    ] {
        match make(layer) {
            Err(_) => {}
            Ok(p) => assert!(
                Compiler::new()
                    .prepare(
                        p.definition(),
                        &p.source(),
                        &ChartState::default(),
                        p.compile_limits()
                    )
                    .is_err()
            ),
        }
    }
}

#[test]
fn size_guide_samples_use_the_actual_mapping_instead_of_input_units() {
    use chart_core::{
        interpolate::{Number, Value},
        scales::{ScaleConstructor, ScaleInput, ScaleOptions, ScaleTraining},
    };
    let scale = ScaleConstructor::Linear
        .create(ScaleOptions {
            domain: Some([0., 2.].map(|n| ScaleInput::Number(Number(n))).to_vec()),
            range: Some([16., 256.].map(Value::number).to_vec()),
            ..Default::default()
        })
        .unwrap();
    let d = Data::columns()
        .column("area", [0., 1., 2.])
        .build()
        .unwrap();
    let p = plot(d)
        .aes(aes().x(2.).y(2.))
        .layer(
            shape_symbol()
                .numeric_scale(
                    A::AreaSize,
                    "area",
                    scale.mapped(ScaleTraining::Authored).unwrap(),
                )
                .symbol_size_guide("Input", vec![0., 1., 2.]),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let layer = &pre.layers()[0];
    assert_eq!(
        layer.symbol_legends()[0]
            .entries
            .iter()
            .map(|e| e.size)
            .collect::<Vec<_>>(),
        [16., 136., 256.]
    );
    for (mark, size) in layer.marks().iter().zip([16., 136., 256.]) {
        let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
            panic!()
        };
        assert_eq!(
            *geometry,
            Generator::new().size(size).generate().unwrap().geometry()
        );
    }
}
