//! AXIS-01/FIX-19: checked providers preserve semantics independently from inversion.
use chart_core::{
    ChartResult, DiagnosticCode, GuideId, Limits, Rect, ResourceId, Revision, ScaleId,
    composition::ScaleValue,
    data::{TimeUnit, TimestampType},
    grammar::{
        Compiler, CustomScale, ExtensionDescriptor, ExtensionRegistry, OperationRef,
        ScaleProviderInput, ValueSpace,
    },
    layout::{AxisSide, LayoutRequest, ResolvedScale, layout},
    prelude::*,
    scales::{Bounds, CheckedPositionalScale, GuideTickArguments, PositionalScale, ProviderBand},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
struct Mapping {
    domain: Vec<ScaleValue>,
    band: Option<ProviderBand>,
    position: f64,
    ticks: Option<Vec<ScaleValue>>,
    inverse: Option<ScaleValue>,
    label: Option<String>,
}
impl Default for Mapping {
    fn default() -> Self {
        Self {
            domain: vec![ScaleValue::Number(0.), ScaleValue::Number(10.)],
            band: None,
            position: 20.,
            ticks: None,
            inverse: None,
            label: None,
        }
    }
}
impl PositionalScale for Mapping {
    fn domain(&self) -> &[ScaleValue] {
        &self.domain
    }
    fn range(&self) -> Bounds {
        Bounds::new(100., 500.).unwrap()
    }
    fn map(&self, _: &ScaleValue) -> ChartResult<Option<f64>> {
        Ok(Some(self.position))
    }
    fn band(&self) -> Option<ProviderBand> {
        self.band
    }
    fn ticks(&self, _: &GuideTickArguments, _: usize) -> ChartResult<Option<Vec<ScaleValue>>> {
        Ok(self.ticks.clone())
    }
    fn has_inverse(&self) -> bool {
        self.inverse.is_some()
    }
    fn invert(&self, _: f64) -> ChartResult<ScaleValue> {
        Ok(self.inverse.clone().unwrap())
    }
    fn format(
        &self,
        value: &ScaleValue,
        index: usize,
        values: &[ScaleValue],
        arguments: &GuideTickArguments,
    ) -> ChartResult<Option<String>> {
        assert_eq!(value, &values[index]);
        if arguments.specifier.as_deref() == Some("context") {
            assert_eq!(arguments.count, Some(2.5));
            return Ok(Some(format!("{index}/{}:{value:?}", values.len())));
        }
        Ok(self.label.clone())
    }
}
fn checked(mapping: Mapping, space: ValueSpace) -> ChartResult<CheckedPositionalScale> {
    CheckedPositionalScale::new(Arc::new(mapping), space, Limits::default(), 16)
}
#[test]
fn optional_inverse_domain_fallback_and_formatter_context_are_independent() {
    let scale = checked(Mapping::default(), ValueSpace::Data).unwrap();
    let args = GuideTickArguments {
        seconds: None,
        time_width: None,
        width: None,
        count: Some(2.5),
        specifier: Some("context".into()),
        interval: None,
    };
    let values = scale.ticks(&args, 2).unwrap();
    assert_eq!(
        values,
        vec![ScaleValue::Number(0.), ScaleValue::Number(10.)]
    );
    assert_eq!(
        scale.labels(&values, &args, 2).unwrap(),
        vec!["0/2:Number(0.0)", "1/2:Number(10.0)"]
    );
    assert_eq!(scale.map(&ScaleValue::Number(3.)).unwrap(), Some(20.));
    assert!(!scale.capabilities().numeric_inverse);
    assert_eq!(
        scale.invert(20.).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    let inverse = checked(
        Mapping {
            inverse: Some(ScaleValue::Number(7.)),
            ..Mapping::default()
        },
        ValueSpace::Data,
    )
    .unwrap();
    assert!(inverse.capabilities().numeric_inverse);
    assert_eq!(inverse.invert(20.).unwrap(), ScaleValue::Number(7.));
    assert!(inverse.invert(f64::NAN).is_err());
    let wrong = checked(
        Mapping {
            inverse: Some(ScaleValue::Category("wrong".into())),
            ..Mapping::default()
        },
        ValueSpace::Data,
    )
    .unwrap();
    assert_eq!(
        wrong.invert(20.).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
}
#[test]
fn exact_timestamps_categories_bands_and_empty_repeated_labels_survive_adapter() {
    let origin = 9_007_199_254_741_001;
    let value = ScaleValue::Timestamp {
        value: origin,
        unit: TimeUnit::Nanoseconds,
    };
    let space = ValueSpace::Timestamp {
        representation: TimestampType {
            unit: TimeUnit::Nanoseconds,
            timezone: "UTC".into(),
        },
        origin,
    };
    let scale = checked(
        Mapping {
            domain: vec![value.clone()],
            ..Mapping::default()
        },
        space,
    )
    .unwrap();
    assert_eq!(
        scale.ticks(&GuideTickArguments::default(), 1).unwrap(),
        vec![value.clone()]
    );
    assert_eq!(
        scale
            .labels(
                std::slice::from_ref(&value),
                &GuideTickArguments::default(),
                1
            )
            .unwrap(),
        vec![origin.to_string()]
    );
    assert_eq!(scale.map(&value).unwrap(), Some(20.));
    assert!(
        scale
            .map(&ScaleValue::Timestamp {
                value: origin,
                unit: TimeUnit::Milliseconds
            })
            .is_err()
    );
    let category = ScaleValue::Category("A".into());
    let mapping = Mapping {
        domain: vec![category.clone()],
        band: Some(ProviderBand {
            bandwidth: 7.,
            round: true,
        }),
        ..Mapping::default()
    };
    let scale = checked(
        mapping,
        ValueSpace::Categorical {
            categories: vec!["A".into()],
        },
    )
    .unwrap();
    assert!(scale.capabilities().category_lookup);
    assert_eq!(scale.map(&category).unwrap(), Some(23.5));
    assert_eq!(
        scale.band_extent(&category).unwrap(),
        Some(Bounds::new(20., 27.).unwrap())
    );
    assert_eq!(scale.guide_position(&category, 0.5).unwrap(), Some(23.));
    assert_eq!(scale.guide_position(&category, 4.).unwrap(), Some(20.));
    assert!(scale.map(&ScaleValue::Number(0.)).is_err());
    for label in ["", "repeat"] {
        let scale = checked(
            Mapping {
                label: Some(label.into()),
                ..Mapping::default()
            },
            ValueSpace::Data,
        )
        .unwrap();
        assert_eq!(
            scale
                .labels(scale.domain(), &GuideTickArguments::default(), 2)
                .unwrap(),
            vec![label, label]
        );
    }
}
#[test]
fn metadata_callback_outputs_and_aggregate_work_are_checked() {
    assert!(
        CheckedPositionalScale::new(
            Arc::new(Mapping::default()),
            ValueSpace::Data,
            Limits::default(),
            1
        )
        .is_err()
    );
    for width in [-1., f64::NAN, f64::INFINITY] {
        assert!(
            checked(
                Mapping {
                    band: Some(ProviderBand {
                        bandwidth: width,
                        round: false
                    }),
                    ..Mapping::default()
                },
                ValueSpace::Data
            )
            .is_err()
        );
    }
    assert!(
        checked(
            Mapping {
                domain: vec![ScaleValue::Number(f64::NAN)],
                ..Mapping::default()
            },
            ValueSpace::Data
        )
        .is_err()
    );
    let scale = checked(
        Mapping {
            position: f64::INFINITY,
            ..Mapping::default()
        },
        ValueSpace::Data,
    )
    .unwrap();
    assert_eq!(
        scale.map(&ScaleValue::Number(0.)).unwrap_err().code,
        DiagnosticCode::PrecisionLoss
    );
    for ticks in [
        vec![ScaleValue::Number(1.); 3],
        vec![ScaleValue::Number(f64::NAN)],
        vec![ScaleValue::Category("wrong".into())],
    ] {
        let scale = checked(
            Mapping {
                ticks: Some(ticks),
                ..Mapping::default()
            },
            ValueSpace::Data,
        )
        .unwrap();
        assert!(scale.ticks(&GuideTickArguments::default(), 2).is_err());
    }
    let limits = Limits {
        max_text_bytes: 3,
        ..Limits::default()
    };
    let scale = CheckedPositionalScale::new(
        Arc::new(Mapping {
            label: Some("xx".into()),
            ..Mapping::default()
        }),
        ValueSpace::Data,
        limits,
        2,
    )
    .unwrap();
    assert_eq!(
        scale
            .labels(scale.domain(), &GuideTickArguments::default(), 2)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let mapping = Mapping {
        domain: vec![
            ScaleValue::Category("ab".into()),
            ScaleValue::Category("cd".into()),
        ],
        ..Mapping::default()
    };
    assert!(
        CheckedPositionalScale::new(
            Arc::new(mapping),
            ValueSpace::Categorical { categories: vec![] },
            limits,
            2
        )
        .is_err()
    );
    let args = GuideTickArguments {
        count: Some(f64::NAN),
        ..GuideTickArguments::default()
    };
    assert!(scale.ticks(&args, 2).is_err());
}

struct Factory {
    name: String,
    portable: bool,
    calls: Arc<AtomicUsize>,
    descriptors: Arc<AtomicUsize>,
}
impl CustomScale for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        self.descriptors.fetch_add(1, Ordering::SeqCst);
        ExtensionDescriptor::batch(&self.name, Revision::new(1), self.portable)
    }
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()> {
        if parameters == &serde_json::json!({}) {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                DiagnosticCode::Validation,
                "No parameters accepted.",
                "Use an empty object.",
            ))
        }
    }
    fn resolve(&self, input: ScaleProviderInput<'_>) -> ChartResult<Arc<dyn PositionalScale>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(input.extent, Some(Bounds::new(-10., 10.).unwrap()));
        assert_eq!(input.space, &ValueSpace::Data);
        assert_eq!(input.range, Bounds::new(100., 500.).unwrap());
        assert!(input.window.is_none());
        Ok(Arc::new(Fold {
            domain: vec![
                ScaleValue::Number(0.),
                ScaleValue::Number(5.),
                ScaleValue::Number(10.),
            ],
        }))
    }
}
struct Fold {
    domain: Vec<ScaleValue>,
}
impl PositionalScale for Fold {
    fn domain(&self) -> &[ScaleValue] {
        &self.domain
    }
    fn range(&self) -> Bounds {
        Bounds::new(100., 500.).unwrap()
    }
    fn map(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
        let ScaleValue::Number(value) = value else {
            unreachable!()
        };
        Ok(Some(100. + value.abs() * 40.))
    }
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request(units: Units) -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        units,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn builder(registry: Arc<ExtensionRegistry>, name: &str) -> PlotBuilder {
    plot(
        Data::columns()
            .column("x", vec![-10., -2., 2., 10.])
            .column("y", vec![0., 1., 2., 3.])
            .build()
            .unwrap(),
    )
    .extensions(registry)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .coordinate_scale(scale_registered(
                name,
                Revision::new(1),
                serde_json::json!({}),
            ))
            .range(100., 500.),
    )
}
#[test]
fn shared_resolution_primary_wire_and_registry_snapshots_keep_one_scale_owner() {
    let calls = Arc::new(AtomicUsize::new(0));
    let descriptors = Arc::new(AtomicUsize::new(0));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale(Arc::new(Factory {
            name: "test.fold".into(),
            portable: true,
            calls: calls.clone(),
            descriptors: descriptors.clone(),
        }))
        .unwrap();
    let frozen = Arc::new(registry.clone());
    let one = builder(frozen.clone(), "test.fold").build().unwrap();
    let many = builder(frozen.clone(), "test.fold")
        .guide(axis_guide("top", "x").side(AxisSide::Top))
        .guide(
            axis_guide("lower", "x")
                .side(AxisSide::Bottom)
                .translate(0., 30.),
        )
        .build()
        .unwrap();
    let render = |p: &Plot| {
        let prepared = Compiler::with_extensions(p.extensions().clone())
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap();
        layout(Arc::new(prepared), &request(Units::LogicalPixels), &Metrics).unwrap()
    };
    calls.store(0, Ordering::SeqCst);
    let one_scene = render(&one);
    let single = calls.swap(0, Ordering::SeqCst);
    let many_scene = render(&many);
    assert!(single > 0);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        single,
        "Extra guides must not retrain scales per layout iteration"
    );
    let axis = &many_scene.axes()[&ScaleId::new(0)];
    assert!(matches!(axis.scale, ResolvedScale::Provider(_)));
    assert_eq!(
        axis.map_value(&ScaleValue::Number(-2.)).unwrap(),
        Some(180.)
    );
    assert_eq!(axis.map_value(&ScaleValue::Number(2.)).unwrap(), Some(180.));
    assert!(axis.invert_value(180.).is_err());
    assert_eq!(one_scene.axes()[&ScaleId::new(0)].ticks, axis.ticks);
    let guide = many.guide("top").unwrap();
    assert_eq!(
        many_scene.guides()[&guide.id()].ticks,
        many_scene.guides()[&GuideId::new(0)].ticks
    );
    let wire = many.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
        10
    );
    assert_eq!(
        Plot::from_json(&wire).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    assert_eq!(
        Plot::from_json_with_extensions(&wire, frozen.clone())
            .unwrap()
            .to_json()
            .unwrap(),
        wire
    );
    assert_eq!(descriptors.load(Ordering::SeqCst), 1);
    registry
        .register_scale(Arc::new(Factory {
            name: "test.later".into(),
            portable: true,
            calls,
            descriptors,
        }))
        .unwrap();
    assert!(
        frozen
            .scale_descriptor(&OperationRef::new("test.later", Revision::new(1)))
            .is_err()
    );
    assert!(
        registry
            .scale_descriptor(&OperationRef::new("test.later", Revision::new(1)))
            .is_ok()
    );
    assert_eq!(render(&many).axes()[&ScaleId::new(0)].ticks, axis.ticks);
}
#[test]
fn registrations_and_native_only_publication_fail_explicitly() {
    let factory = || {
        Arc::new(Factory {
            name: "test.native".into(),
            portable: false,
            calls: Arc::new(AtomicUsize::new(0)),
            descriptors: Arc::new(AtomicUsize::new(0)),
        })
    };
    let mut registry = ExtensionRegistry::new();
    registry.register_scale(factory()).unwrap();
    assert_eq!(
        registry.register_scale(factory()).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    let registry = Arc::new(registry);
    assert!(builder(registry.clone(), "missing").build().is_err());
    let p = builder(registry.clone(), "test.native").build().unwrap();
    assert_eq!(
        p.to_json().unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    let prepared = Arc::new(
        Compiler::with_extensions(registry)
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap(),
    );
    assert!(layout(prepared.clone(), &request(Units::LogicalPixels), &Metrics).is_ok());
    assert_eq!(
        layout(prepared, &request(Units::Points), &Metrics)
            .unwrap_err()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    let bad = p
        .edit()
        .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
        .axis(x_axis().scale(scale_registered(
            "test.native",
            Revision::new(1),
            serde_json::json!({}),
        )))
        .build();
    assert_eq!(bad.unwrap_err().code, DiagnosticCode::UnsupportedCapability);
}

#[test]
fn prepared_layer_coordinates_recover_exact_timestamp_and_category_values() {
    struct Semantic(Vec<ScaleValue>);
    impl PositionalScale for Semantic {
        fn domain(&self) -> &[ScaleValue] {
            &self.0
        }
        fn range(&self) -> Bounds {
            Bounds::new(100., 500.).unwrap()
        }
        fn map(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
            Ok(self
                .0
                .iter()
                .position(|v| v == value)
                .map(|i| 100. + i as f64 * 40.))
        }
    }
    let origin = 9_007_199_254_741_001;
    let time_space = |origin| ValueSpace::Timestamp {
        origin,
        representation: TimestampType {
            unit: TimeUnit::Nanoseconds,
            timezone: "UTC".into(),
        },
    };
    let time = ScaleValue::Timestamp {
        value: origin + 2,
        unit: TimeUnit::Nanoseconds,
    };
    let categories = ValueSpace::Categorical {
        categories: vec!["B".into(), "A".into()],
    };
    for (domain, space, layer_space, value, expected) in [
        (
            vec![time],
            time_space(origin),
            time_space(origin + 1),
            1.,
            100.,
        ),
        (
            vec![
                ScaleValue::Category("A".into()),
                ScaleValue::Category("B".into()),
            ],
            ValueSpace::Categorical {
                categories: vec!["A".into(), "B".into()],
            },
            categories,
            0.,
            140.,
        ),
    ] {
        let scale = CheckedPositionalScale::new(
            Arc::new(Semantic(domain)),
            space.clone(),
            Limits::default(),
            2,
        )
        .unwrap();
        let axis = chart_core::layout::ResolvedAxis {
            spec: chart_core::layout::AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
            space,
            scale: ResolvedScale::Provider(scale),
            ticks: vec![],
        };
        assert_eq!(axis.map(value, &layer_space).unwrap(), Some(expected));
        assert!(axis.map(0.5, &layer_space).is_err());
        assert!(axis.map(f64::INFINITY, &layer_space).is_err());
    }
}

#[test]
fn categorical_minor_capability_rejects_nonfinite_output_and_clips_to_range() {
    assert_eq!(
        checked(Mapping::default(), ValueSpace::Data)
            .unwrap()
            .category_minor(1.)
            .unwrap_err()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    struct Minor;
    impl PositionalScale for Minor {
        fn domain(&self) -> &[ScaleValue] {
            &[]
        }
        fn range(&self) -> Bounds {
            Bounds::new(100., 0.).unwrap()
        }
        fn map(&self, _: &ScaleValue) -> ChartResult<Option<f64>> {
            Ok(None)
        }
        fn category_minor(&self, value: f64) -> ChartResult<Option<f64>> {
            Ok(Some(value))
        }
    }
    let scale = CheckedPositionalScale::new(
        Arc::new(Minor),
        ValueSpace::NullableCategorical { categories: vec![] },
        Limits::default(),
        16,
    )
    .unwrap();
    for value in [0., 20., 100.] {
        assert_eq!(scale.category_minor(value).unwrap(), Some(value));
    }
    for value in [-1., 101.] {
        assert_eq!(scale.category_minor(value).unwrap(), None);
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            scale.category_minor(value).unwrap_err().code,
            DiagnosticCode::PrecisionLoss
        );
    }
}
