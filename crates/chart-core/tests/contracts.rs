//! WP-02 contract tests, including the malformed-input portion of FIX-18 only.

use std::cell::Cell;
use std::rc::Rc;

use chart_core::scene::{Color, PathCommand, Primitive, Scene, SceneItem, Stroke};
use chart_core::services::{
    ResourceDescriptor, ResourceKind, ResourceProvider, TextMeasurer, TextMetrics, TextRequest,
    Units, measure_text, resolve_resource,
};
use chart_core::{
    ChartResult, DatasetId, Diagnostic, DiagnosticCode, FieldId, LayerId, Limits, Point, Rect,
    ResourceId, Revision, RowKey, SceneStamp, Severity,
};

const INK: Color = Color {
    red: 25,
    green: 50,
    blue: 75,
    alpha: 255,
};

fn point(x: f64, y: f64) -> Point {
    Point::new(x, y).unwrap()
}
fn bounds() -> Rect {
    Rect::new(0.0, 0.0, 100.0, 80.0).unwrap()
}
fn stamp() -> SceneStamp {
    SceneStamp {
        definition: Revision::new(2),
        store: Revision::new(9),
        layout: Revision::new(4),
        state: Revision::new(8),
        viewport: Revision::new(7),
    }
}
fn font() -> ResourceDescriptor {
    ResourceDescriptor {
        id: ResourceId::new(5),
        revision: Revision::new(3),
        kind: ResourceKind::Font,
        byte_len: 3,
    }
}
fn item(primitive: Primitive) -> SceneItem {
    SceneItem {
        layer: Some(LayerId::new(11)),
        clip: None,
        primitive,
    }
}
fn text(value: &str) -> SceneItem {
    item(Primitive::Text {
        origin: point(5.0, 20.0),
        text: value.into(),
        font: font().id,
        font_size: 12.0,
        color: INK,
    })
}
fn scene(
    items: &[SceneItem],
    resources: &[ResourceDescriptor],
    limits: Limits,
) -> ChartResult<Scene> {
    Scene::new(stamp(), Units::Points, bounds(), items, resources, limits)
}
fn path(commands: Vec<PathCommand>) -> SceneItem {
    item(Primitive::Path {
        commands,
        stroke: Stroke {
            color: INK,
            width: 1.5,
        },
    })
}

#[test]
fn identities_and_revisions_preserve_all_64_bits() {
    for value in [0, (1_u64 << 53) + 1, u64::MAX] {
        assert_eq!(DatasetId::new(value).get(), value);
        assert_eq!(FieldId::new(value).get(), value);
        assert_eq!(LayerId::new(value).get(), value);
        assert_eq!(ResourceId::new(value).get(), value);
        assert_eq!(RowKey::new(value).get(), value);
        assert_eq!(Revision::new(value).get(), value);
    }
    assert_ne!(RowKey::new(1_u64 << 53), RowKey::new((1_u64 << 53) + 1));
}

#[test]
fn revision_exhaustion_is_an_error_without_wrap_or_mutation() {
    assert_eq!(Revision::INITIAL.checked_next().unwrap(), Revision::new(1));
    assert_eq!(
        Revision::new(u64::MAX - 1).checked_next().unwrap().get(),
        u64::MAX
    );
    let last = Revision::new(u64::MAX);
    let error = last.checked_next().unwrap_err();
    assert_eq!(last.get(), u64::MAX);
    assert_eq!(error.code.as_str(), "CHART_REVISION_OVERFLOW");
    assert_eq!(error.severity, Severity::Error);
    assert!(!error.correction.is_empty());
}

#[test]
fn points_reject_nonfinite_coordinates_on_both_axes() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for result in [Point::new(bad, 0.0), Point::new(0.0, bad)] {
            let error = result.unwrap_err();
            assert_eq!(error.code, DiagnosticCode::NumericalDomain);
            assert!(error.to_string().starts_with("CHART_NUMERICAL_DOMAIN:"));
        }
    }
    assert_eq!(point(f64::MAX, -f64::MAX).x(), f64::MAX);
}

#[test]
fn rectangles_validate_extents_and_arithmetic_not_only_inputs() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Rect::new(0.0, 0.0, bad, 1.0).is_err());
        assert!(Rect::new(0.0, 0.0, 1.0, bad).is_err());
    }
    assert!(Rect::new(f64::MAX, 0.0, f64::MAX, 1.0).is_err());
    assert!(Rect::new(0.0, f64::MAX, 1.0, f64::MAX).is_err());
    let empty = Rect::new(-5.0, 7.0, 0.0, 0.0).unwrap();
    assert_eq!((empty.max_x(), empty.max_y()), (-5.0, 7.0));
    let regular = Rect::new(-5.0, 7.0, 8.0, 4.0).unwrap();
    assert_eq!((regular.max_x(), regular.max_y()), (3.0, 11.0));
}

#[test]
fn scene_preserves_order_clips_units_and_stamps() {
    let clip = Rect::new(2.0, 3.0, 20.0, 10.0).unwrap();
    let mut first = item(Primitive::Rectangle {
        bounds: clip,
        fill: INK,
    });
    first.layer = None; // Decoration remains without a source/layer target.
    first.clip = Some(clip);
    let second = item(Primitive::Rule {
        from: point(1.0, 2.0),
        to: point(8.0, 9.0),
        stroke: Stroke {
            color: INK,
            width: 2.0,
        },
    });
    let inputs = [first, second];
    let output = scene(&inputs, &[], Limits::default()).unwrap();
    assert_eq!(output.items(), inputs);
    assert_eq!(output.bounds(), bounds());
    assert_eq!(output.units(), Units::Points);
    assert_eq!(output.stamp(), stamp());
    assert_eq!(output.items()[0].layer, None);
}

#[test]
fn invalid_replacement_and_input_mutation_leave_owned_scene_unchanged() {
    let mut inputs = [text("original")];
    let mut resources = [font()];
    let saved = scene(&inputs, &resources, Limits::default()).unwrap();
    if let Primitive::Text {
        text, font_size, ..
    } = &mut inputs[0].primitive
    {
        text.clear();
        *font_size = f64::NAN;
    }
    resources[0].revision = Revision::new(99);
    assert!(scene(&inputs, &resources, Limits::default()).is_err());
    assert_eq!(saved.items(), &[text("original")]);
    assert_eq!(saved.resources(), &[font()]);
    assert_eq!(saved.stamp(), stamp());
}

#[test]
fn missing_font_diagnostic_identifies_resource_layer_and_scene() {
    let error = scene(&[text("missing")], &[], Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::MissingResource);
    assert_eq!(error.context.resource, Some(font().id));
    assert_eq!(error.context.layer, Some(LayerId::new(11)));
    assert_eq!(error.context.stamp, Some(stamp()));
    assert_eq!(error.severity, Severity::Error);
    assert!(!error.message.is_empty());
    assert!(!error.correction.is_empty());
}

#[test]
fn scene_rejects_two_revisions_of_one_resource_identity() {
    let another = ResourceDescriptor {
        revision: Revision::new(4),
        ..font()
    };
    let error = scene(&[], &[font(), another], Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::SchemaConflict);
    assert_eq!(error.context.resource, Some(font().id));
}

#[test]
fn image_resource_cannot_stand_in_for_a_font() {
    let image = ResourceDescriptor {
        kind: ResourceKind::Image,
        ..font()
    };
    let error = scene(&[text("wrong kind")], &[image], Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::UnsupportedCapability);
    assert_eq!(error.context.resource_revision, Some(font().revision));
}

#[test]
fn count_limits_fail_before_primitive_validation() {
    let limits = Limits {
        max_items: 0,
        ..Limits::default()
    };
    // Otherwise this item fails for an absent font. The count guard runs first.
    let error = scene(&[text("missing")], &[], limits).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::ResourceLimit);
    assert_eq!(error.context.stamp, Some(stamp()));
    let limits = Limits {
        max_resources: 0,
        ..Limits::default()
    };
    assert_eq!(
        scene(&[], &[font()], limits).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert!(
        scene(
            &[],
            &[],
            Limits {
                max_items: 0,
                max_resources: 0,
                ..limits
            }
        )
        .is_ok()
    );
}

#[test]
fn text_budget_counts_total_utf8_bytes_not_characters_or_per_item_length() {
    let inputs = [text("é"), text("é")]; // Two bytes each, four bytes total.
    assert!(
        scene(
            &inputs,
            &[font()],
            Limits {
                max_text_bytes: 4,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    let error = scene(
        &inputs,
        &[font()],
        Limits {
            max_text_bytes: 3,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, DiagnosticCode::ResourceLimit);
    assert_eq!(error.context.layer, Some(LayerId::new(11)));
}

#[test]
fn path_budget_is_aggregate_and_precedes_segment_scanning() {
    let a = path(vec![
        PathCommand::MoveTo(point(0.0, 0.0)),
        PathCommand::LineTo(point(1.0, 1.0)),
    ]);
    let inputs = [a.clone(), a];
    assert!(
        scene(
            &inputs,
            &[],
            Limits {
                max_path_commands: 4,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    assert_eq!(
        scene(
            &inputs,
            &[],
            Limits {
                max_path_commands: 3,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
    let malformed = path(vec![PathCommand::Close]);
    assert_eq!(
        scene(
            &[malformed],
            &[],
            Limits {
                max_path_commands: 0,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .code,
        DiagnosticCode::ResourceLimit
    );
}

#[test]
fn resource_budgets_reject_oversized_and_overflowing_totals() {
    let limits = Limits {
        max_resource_bytes: 2,
        ..Limits::default()
    };
    assert_eq!(
        scene(&[], &[font()], limits).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    let a = ResourceDescriptor {
        byte_len: u64::MAX,
        ..font()
    };
    let b = ResourceDescriptor {
        id: ResourceId::new(6),
        byte_len: 1,
        ..font()
    };
    let limits = Limits {
        max_resource_bytes: u64::MAX,
        max_total_resource_bytes: u64::MAX,
        ..Limits::default()
    };
    assert!(scene(&[], &[a], limits).is_ok()); // Descriptors allocate no resource bytes.
    assert_eq!(
        scene(&[], &[a, b], limits).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
}

#[test]
fn invalid_sizes_and_overflowing_point_extents_are_rejected() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let point_item = item(Primitive::Point {
            center: point(0.0, 0.0),
            radius: bad,
            fill: INK,
        });
        let rule = item(Primitive::Rule {
            from: point(0.0, 0.0),
            to: point(1.0, 1.0),
            stroke: Stroke {
                color: INK,
                width: bad,
            },
        });
        for input in [point_item, rule] {
            assert_eq!(
                scene(&[input], &[], Limits::default()).unwrap_err().code,
                DiagnosticCode::NumericalDomain
            );
        }
    }
    let huge = item(Primitive::Point {
        center: point(0.0, 0.0),
        radius: f64::MAX,
        fill: INK,
    });
    assert!(scene(&[huge], &[], Limits::default()).is_err());
}

#[test]
fn numeric_paths_preserve_segments_and_reject_invalid_subpath_order() {
    let p = point(0.0, 0.0);
    let commands = vec![
        PathCommand::MoveTo(p),
        PathCommand::QuadraticTo(point(1.0, 2.0), point(3.0, 4.0)),
        PathCommand::Close,
        PathCommand::MoveTo(p),
        PathCommand::CubicTo(p, point(5.0, 6.0), point(7.0, 8.0)),
    ];
    let input = path(commands);
    assert_eq!(
        scene(std::slice::from_ref(&input), &[], Limits::default())
            .unwrap()
            .items(),
        &[input]
    );
    for invalid in [
        vec![],
        vec![PathCommand::Close],
        vec![PathCommand::LineTo(p)],
        vec![PathCommand::MoveTo(p)],
        vec![
            PathCommand::MoveTo(p),
            PathCommand::LineTo(p),
            PathCommand::Close,
            PathCommand::LineTo(p),
        ],
    ] {
        let error = scene(&[path(invalid)], &[], Limits::default()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Validation);
        assert_eq!(error.context.layer, Some(LayerId::new(11)));
    }
}

// Deliberately single-threaded host: Rc<Cell<_>> does not implement Send/Sync.
struct BytesHost {
    calls: Rc<Cell<usize>>,
    bytes: &'static [u8],
    missing: bool,
}
impl ResourceProvider for BytesHost {
    fn resolve(&self, resource: &ResourceDescriptor) -> ChartResult<&[u8]> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(resource.id, font().id);
        assert_eq!(resource.revision, font().revision);
        if self.missing {
            return Err(Diagnostic::error(
                DiagnosticCode::MissingResource,
                "Revision unavailable.",
                "Supply this exact revision.",
            ));
        }
        Ok(self.bytes)
    }
}

#[test]
fn resource_limits_are_checked_before_calling_a_non_send_host() {
    let host = BytesHost {
        calls: Rc::new(Cell::new(0)),
        bytes: &[1, 2, 3],
        missing: false,
    };
    let error = resolve_resource(
        &host,
        &font(),
        Limits {
            max_resource_bytes: 2,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert_eq!(host.calls.get(), 0);
    assert_eq!(error.code, DiagnosticCode::ResourceLimit);
    assert_eq!(error.context.resource_revision, Some(font().revision));
    let empty = ResourceDescriptor {
        byte_len: 0,
        ..font()
    };
    assert_eq!(
        resolve_resource(&host, &empty, Limits::default())
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidResource
    );
    assert_eq!(host.calls.get(), 0);
}

#[test]
fn resource_resolution_borrows_exact_bytes_and_checks_length() {
    let mut host = BytesHost {
        calls: Rc::new(Cell::new(0)),
        bytes: &[1, 2, 3],
        missing: false,
    };
    let bytes = resolve_resource(&host, &font(), Limits::default()).unwrap();
    assert_eq!(bytes, &[1, 2, 3]);
    assert!(std::ptr::eq(bytes.as_ptr(), host.bytes.as_ptr()));
    host.bytes = &[1, 2];
    let error = resolve_resource(&host, &font(), Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::InvalidResource);
    assert_eq!(error.context.resource, Some(font().id));
    assert_eq!(host.calls.get(), 2);
}

#[test]
fn host_failures_preserve_codes_and_gain_requested_resource_context() {
    let host = BytesHost {
        calls: Rc::new(Cell::new(0)),
        bytes: &[],
        missing: true,
    };
    let error = resolve_resource(&host, &font(), Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::MissingResource);
    assert_eq!(error.context.resource, Some(font().id));
    assert_eq!(error.context.resource_revision, Some(font().revision));
}

struct MetricsHost {
    calls: Rc<Cell<usize>>,
    invalid: bool,
}
impl TextMeasurer for MetricsHost {
    fn measure(&self, request: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(request.text, "A−B");
        assert_eq!(request.font, &font());
        assert_eq!(request.font_size, 12.0);
        assert_eq!(request.units, Units::Points);
        TextMetrics::new(if self.invalid { f64::NAN } else { 24.0 }, 9.0, 3.0)
    }
}

#[test]
fn measurement_receives_exact_destination_and_returns_finite_baseline_metrics() {
    let host = MetricsHost {
        calls: Rc::new(Cell::new(0)),
        invalid: false,
    };
    let resource = font();
    let request = TextRequest {
        text: "A−B",
        font: &resource,
        font_size: 12.0,
        units: Units::Points,
    };
    let metrics = measure_text(&host, request, Limits::default()).unwrap();
    assert_eq!(
        (
            metrics.width(),
            metrics.ascent(),
            metrics.descent(),
            metrics.height()
        ),
        (24.0, 9.0, 3.0, 12.0)
    );
    assert_eq!(host.calls.get(), 1);
    let bad_host = MetricsHost {
        calls: Rc::new(Cell::new(0)),
        invalid: true,
    };
    let error = measure_text(&bad_host, request, Limits::default()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::NumericalDomain);
    assert_eq!(error.context.resource_revision, Some(resource.revision));
}

#[test]
fn measurement_preflight_rejects_invalid_sizes_and_text_before_host_calls() {
    let host = MetricsHost {
        calls: Rc::new(Cell::new(0)),
        invalid: false,
    };
    let resource = font();
    let request = TextRequest {
        text: "A−B",
        font: &resource,
        font_size: 12.0,
        units: Units::Points,
    };
    let error = measure_text(
        &host,
        request,
        Limits {
            max_text_bytes: 4,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, DiagnosticCode::ResourceLimit); // A + three-byte minus + B = 5.
    for bad in [0.0, -12.0, f64::NAN, f64::INFINITY] {
        let error = measure_text(
            &host,
            TextRequest {
                font_size: bad,
                ..request
            },
            Limits::default(),
        )
        .unwrap_err();
        assert_eq!(error.code, DiagnosticCode::NumericalDomain);
    }
    assert_eq!(host.calls.get(), 0);
}

#[test]
fn text_metric_construction_rejects_nonfinite_negative_and_overflowing_values() {
    for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(TextMetrics::new(bad, 1.0, 1.0).is_err());
        assert!(TextMetrics::new(1.0, bad, 1.0).is_err());
        assert!(TextMetrics::new(1.0, 1.0, bad).is_err());
    }
    assert!(TextMetrics::new(1.0, f64::MAX, f64::MAX).is_err());
    assert_eq!(TextMetrics::new(0.0, 0.0, 0.0).unwrap().height(), 0.0);
}
