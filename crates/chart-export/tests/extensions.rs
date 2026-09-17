//! FIX-17 actual scene/export/inspection integration using an external-style public API crate.
use chart_core::{grammar::*, inspection::*, layout::*, portable::*, services::*, state::*, *};
use chart_export::*;
use chart_extension_example::*;
use std::sync::Arc;
fn resources() -> (FontResources, PublicationProfile) {
    let bytes: Arc<[u8]> =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf")
            .as_slice()
            .into();
    let desc = ResourceDescriptor {
        id: ResourceId::new(1),
        revision: Revision::new(1),
        kind: ResourceKind::Font,
        byte_len: bytes.len() as u64,
    };
    let fonts = FontResources::new(vec![FontResource::new(desc, bytes).unwrap()]).unwrap();
    let page = PageSize::points(360., 240.).unwrap();
    let profile = PublicationProfile::new(page, desc).unwrap();
    (fonts, profile)
}
#[test]
fn registered_geometry_shares_axes_explicit_guides_hit_values_and_keyboard_order() {
    let prepared = Arc::new(prepare(false).unwrap());
    let (fonts, profile) = resources();
    let laid = Arc::new(layout(prepared, &profile.layout, &fonts).unwrap());
    assert_eq!(laid.interactions().len(), 2);
    let labels = laid.axes()[&ScaleId::new(0)]
        .ticks
        .iter()
        .map(|t| t.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["Lower", "Boundary", "Upper"]);
    let mut inspector = Inspector::new(laid.clone(), 4., 8).unwrap();
    let stamp = laid.scene().stamp();
    let first = laid.interactions().values().next().unwrap();
    let at = first.hit.anchor().unwrap();
    inspector
        .dispatch(
            stamp,
            InspectionAction::Hover(Some(at)),
            InputOrigin::Pointer,
        )
        .unwrap();
    assert_eq!(inspector.hits().len(), 1);
    assert_eq!(
        inspector.hits()[0].values[0],
        ("Count".into(), SemanticValue::Unsigned(3))
    );
    assert_eq!(inspector.hits()[0].selection, SelectionPolicy::AtomicTarget);
    inspector
        .dispatch(stamp, InspectionAction::Clear, InputOrigin::Programmatic)
        .unwrap();
    inspector
        .dispatch(
            stamp,
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard,
        )
        .unwrap();
    assert!(inspector.hits()[0].position.x() > at.x());
    // A clipped outer corner is outside the supplied chamfer polygon despite its rectangle bounds.
    let b = first.hit.bounds().unwrap();
    let outside = Point::new(
        b.origin().x() + b.width() * 0.005,
        b.origin().y() + b.height() * 0.005,
    )
    .unwrap();
    inspector
        .dispatch(
            stamp,
            InspectionAction::Hover(Some(outside)),
            InputOrigin::Pointer,
        )
        .unwrap();
    assert!(!first.hit.contains(outside));
    assert!(
        inspector
            .hits()
            .iter()
            .all(|hit| hit.layer != LayerId::new(1))
    );
    // The builtin point layer intentionally shares the same generated boundary.
    assert!(
        inspector
            .hits()
            .iter()
            .any(|hit| hit.layer == LayerId::new(2))
    );
}
#[test]
fn portable_vector_output_and_native_only_export_fail_explicitly() {
    let (fonts, profile) = resources();
    let f = FigureRequest::new(
        definition(false),
        store().unwrap().snapshot(),
        ChartState::default(),
        fonts.clone(),
        profile.clone(),
        InteractionCapture::ALL,
    )
    .unwrap()
    .with_extensions(registry().unwrap())
    .prepare()
    .unwrap();
    let svg = String::from_utf8(f.export(Format::Svg).unwrap().bytes).unwrap();
    assert!(!svg.contains("<image"));
    assert!(svg.contains("Boundary"));
    assert!(f.export(Format::Pdf).unwrap().bytes.starts_with(b"%PDF-"));
    assert!(f.export(Format::Png).unwrap().bytes.starts_with(b"\x89PNG"));
    let error = FigureRequest::new(
        definition(true),
        store().unwrap().snapshot(),
        ChartState::default(),
        fonts,
        profile,
        InteractionCapture::ALL,
    )
    .unwrap()
    .with_extensions(registry().unwrap())
    .prepare()
    .err()
    .unwrap();
    assert_eq!(error.code, DiagnosticCode::UnsupportedCapability);
    assert!(error.message.contains(PAINTER));
}
#[test]
fn custom_native_and_unknown_operation_descriptors_reject_portable_execution() {
    let snapshot = store().unwrap().snapshot();
    let dataset = snapshot.get().unwrap().dataset(DatasetId::new(1)).unwrap();
    let data = DataEnvelope {
        version: 1,
        epoch: SourceEpoch::new(1),
        datasets: vec![DatasetWire {
            id: DatasetId::new(1),
            batch: BatchWire::from_batch(dataset.chunks()[0].batch()),
        }],
    };
    for native in [false, true] {
        let chart = ChartEnvelope {
            version: 1,
            definition: definition(native),
        };
        let json = encode(&chart).unwrap();
        let input = encode(&data).unwrap();
        let result = Session::with_extensions(&json, &input, registry().unwrap());
        if native {
            assert_eq!(
                result.err().unwrap().code,
                DiagnosticCode::UnsupportedCapability
            );
        } else {
            let mut s = result.unwrap();
            assert!(s.semantics_json().unwrap().contains("Custom"));
            assert_eq!(
                Session::new(&json, &input).err().unwrap().code,
                DiagnosticCode::UnsupportedCapability
            );
        }
    }
}

#[test]
fn custom_coordinates_use_resolved_scales_and_reject_wrong_guide_types() {
    use chart_core::composition::ScaleValue;
    let (fonts, profile) = resources();
    let laid = layout(Arc::new(prepare(false).unwrap()), &profile.layout, &fonts).unwrap();
    let x = &laid.axes()[&ScaleId::new(0)];
    let y = &laid.axes()[&ScaleId::new(1)];
    let clip = Rect::new(0., 0., 360., 240.).unwrap();
    let coordinates = Cartesian::new(x, y, clip).unwrap();
    assert!(coordinates.capabilities().inverse);
    assert!(coordinates.capabilities().rectangular_clip);
    assert!(!coordinates.capabilities().path_subdivision);
    assert_eq!(coordinates.clip(), clip);
    let p = coordinates
        .project(&ScaleValue::Number(0.75), &ScaleValue::Number(0.25))
        .unwrap()
        .unwrap();
    let (ScaleValue::Number(a), ScaleValue::Number(b)) = coordinates.inverse(p).unwrap() else {
        panic!("numeric inverse")
    };
    assert!((a - 0.75).abs() < 1e-12 && (b - 0.25).abs() < 1e-12);
    assert_eq!(
        coordinates
            .project(
                &ScaleValue::Category("Lower".into()),
                &ScaleValue::Number(0.)
            )
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
    let mut d = definition(false);
    d.axes[0].guide_ticks.as_mut().unwrap()[0].value = ScaleValue::Timestamp {
        value: 0,
        unit: chart_core::data::TimeUnit::Seconds,
    };
    let prepared = Compiler::with_extensions(registry().unwrap())
        .prepare(
            &d,
            &store().unwrap().snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(
        layout(Arc::new(prepared), &profile.layout, &fonts)
            .unwrap_err()
            .code,
        DiagnosticCode::SchemaConflict
    );
}
