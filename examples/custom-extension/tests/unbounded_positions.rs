//! FIX-GG04: exact Cartesian coordinate views over unbounded positional populations.
use chart_core::{grammar::ScaleOob, interpolate::Number, prelude::*, scales::*};
fn number(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        _ => f64::NAN,
    })
}
#[test]
fn exact_unbounded_point_views_match_reference() {
    check_views(false, false);
}
#[test]
fn authored_unbounded_point_views_match_reference() {
    check_views(true, false);
}
#[test]
fn authored_partial_limits_match_reference() {
    check_views(true, true);
}
fn check_views(authored: bool, partial: bool) {
    let fixture: serde_json::Value = serde_json::from_str(if partial {
        include_str!("../../../fixtures/parity/ggplot2/positional-authored-limits.json")
    } else {
        include_str!("../../../fixtures/parity/ggplot2/positional-unbounded-viewports-exact.json")
    })
    .unwrap();
    let registry = chart_extension_example::registry().unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for c in fixture["cases"].as_array().unwrap() {
        let binned = c["kind"] == "binned";
        // A fixed function is not an authored vector on an untrained scale.
        // The full zero-row vector contract belongs to authored numeric limits.
        if !authored && !binned && c["population"] == "empty" {
            continue;
        }
        // Numeric callbacks preserve a returned NaN; an authored vector fills it
        // from the population. Keep that separate argument contract explicit.
        if !authored
            && !binned
            && matches!(c["transform"].as_str(), Some("sqrt" | "log10"))
            && matches!(c["control"].as_str(), Some("both" | "lower" | "negative"))
        {
            continue;
        }
        let transform = match c["transform"].as_str().unwrap() {
            "sqrt" => Some(ScaleTransform::Sqrt),
            "reverse" => Some(ScaleTransform::Reverse),
            "log10" => Some(ScaleTransform::Log { base: 10. }),
            _ => None,
        };
        let limits = c["limits"].as_array().unwrap();
        let limits = [Number(number(&limits[0])), Number(number(&limits[1]))];
        let scale = if binned {
            let mut spec = GgplotBinnedPosition {
                transform: transform.clone(),
                ..Default::default()
            };
            spec.bins.limits = Some(limits.map(Some));
            scale_binned(spec)
        } else {
            match transform {
                Some(ScaleTransform::Sqrt) => scale_sqrt(),
                Some(ScaleTransform::Reverse) => scale_reverse(),
                Some(ScaleTransform::Log { base }) => scale_log(base),
                _ => scale_linear(),
            }
        };
        let view = c["viewport"].as_array().unwrap();
        // coord_cartesian takes the range of transformed endpoints. The core
        // viewport is directional; express that reference range explicitly.
        let mut view = [number(&view[0]), number(&view[1])];
        view.sort_by(f64::total_cmp);
        if transform == Some(ScaleTransform::Reverse) {
            view.swap(0, 1);
        }
        let mut axis = x_axis()
            .scale(scale)
            .range(100., 540.)
            .viewport(view[0], view[1])
            .guide_geometry(Some(chart_core::layout::GuideGeometry {
                labels: Some(chart_core::layout::GuideLabelPolicy::Preserve),
                ..Default::default()
            }))
            .oob(match c["oob"].as_str().unwrap() {
                "squish" => ScaleOob::Squish,
                "keep" => ScaleOob::Keep,
                _ => ScaleOob::Censor,
            });
        if !binned && authored {
            axis = axis.numeric_limits(Some(limits.map(Some)));
        } else if !binned {
            let mut operation = chart_extension_example::numeric_limits::operation("fixed");
            let mut callback_limits = limits;
            // Authored limits sort after the reverse transform. Function results
            // retain their authored order, so return that equivalent order.
            if transform == Some(ScaleTransform::Reverse) {
                callback_limits.swap(0, 1);
            }
            operation.parameters["bounds"] = serde_json::to_value(callback_limits).unwrap();
            axis = axis.limits_function(operation);
        }
        let data = Data::columns()
            .column(
                "x",
                c["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| (!v.is_null()).then(|| number(v)))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let actual = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(registry.clone())
            .aes(aes().x("x").y(1.))
            .layer(points())
            .x_axis(axis)
            .build()
            .and_then(|p| p.chart()?.prepare())
            .and_then(frame);
        if c["result"].get("error").is_some() {
            assert!(actual.is_err(), "reference rejection accepted: {c}");
            rejected += 1;
            continue;
        }
        let actual = actual.unwrap_or_else(|e| panic!("{e:?}: {c}"));
        let points: Vec<_> = actual
            .scene()
            .items()
            .iter()
            .filter_map(|item| match item.primitive {
                chart_core::scene::Primitive::Point { center, .. } => {
                    Some((center.x() - 100.) / 440.)
                }
                _ => None,
            })
            .collect();
        let expected: Vec<_> = c["result"]["point_positions"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .filter(|v| v.is_finite())
            .collect();
        assert_eq!(points.len(), expected.len(), "{points:?}: {c}");
        assert!(
            points
                .iter()
                .zip(&expected)
                .all(|(a, b)| (a - b).abs() <= 3e-12),
            "{points:?} != {expected:?}: {c}"
        );
        let axis = &actual.axes()[&chart_core::ScaleId::new(0)];
        let mut expected_ticks: Vec<_> = c["result"]["positions"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["result"]["labels"].as_array().unwrap())
            .filter_map(|(p, l)| {
                let p = number(p);
                p.is_finite().then(|| (p, l.as_str().unwrap_or_default()))
            })
            .collect();
        // Compare label/position relationships independently of guide iteration order.
        expected_ticks.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(
            axis.ticks.len(),
            expected_ticks.len(),
            "ticks {:?}: {c}",
            axis.ticks
        );
        let mut actual_ticks: Vec<_> = axis.ticks.iter().collect();
        actual_ticks.sort_by(|a, b| a.position.total_cmp(&b.position));
        for (tick, (position, label)) in actual_ticks.into_iter().zip(expected_ticks) {
            assert_eq!(tick.label, label, "{c}");
            assert!(
                ((tick.position - 100.) / 440. - position).abs() <= 3e-12,
                "{tick:?}: {c}"
            );
        }
        accepted += 1;
    }
    println!("{accepted} accepted, {rejected} rejected");
    assert_eq!(
        (accepted, rejected),
        if partial {
            (648, 0)
        } else if authored {
            (1368, 360)
        } else {
            (1044, 306)
        }
    );
}
#[test]
fn authored_limits_wire_and_profile_contract() {
    let make = |profile| {
        plot(
            Data::columns()
                .column("x", vec![1., 4., 9.])
                .build()
                .unwrap(),
        )
        .profile(profile)
        .aes(aes().x("x").y(1.))
        .layer(points())
        .x_axis(x_axis().numeric_limits(Some([None, Some(Number(5.))])))
        .build()
    };
    let p = make(Profile::Ggplot2_4_0_3).unwrap();
    let wire = p.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(value["version"], 31);
    let restored = Plot::from_json(&wire).unwrap();
    assert_eq!(restored.to_json().unwrap(), wire);
    assert_eq!(
        p.chart().unwrap().prepare().unwrap().positional_limits(),
        restored
            .chart()
            .unwrap()
            .prepare()
            .unwrap()
            .positional_limits()
    );
    value["version"] = 30.into();
    assert_eq!(
        Plot::from_json(&value.to_string()).unwrap_err().code,
        chart_core::DiagnosticCode::UnsupportedCapability
    );
    let legacy = make(Profile::LibraryV1).and_then(|p| p.chart()?.prepare());
    assert_eq!(
        legacy.unwrap_err().code,
        chart_core::DiagnosticCode::UnsupportedCapability
    );
}
fn frame(
    prepared: std::sync::Arc<chart_core::grammar::PreparedChart>,
) -> chart_core::ChartResult<chart_core::layout::LaidOutChart> {
    use chart_core::{services::*, *};
    let mut request = layout::LayoutRequest::new(
        Rect::new(0., 0., 640., 320.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.axes = prepared.definition().axes.clone();
    layout::layout(prepared, &request, &Metrics)
}

struct Metrics;
impl chart_core::services::TextMeasurer for Metrics {
    fn measure(
        &self,
        r: chart_core::services::TextRequest<'_>,
    ) -> chart_core::ChartResult<chart_core::services::TextMetrics> {
        chart_core::services::TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
