//! WP-11 independent scale expectations, FIX-07.
use chart_core::data::TimeUnit;
use chart_core::grammar::Extent;
use chart_core::scales::*;
use chart_core::scene::Color;
use chart_core::{Revision, ScaleId};
fn b(a: f64, z: f64) -> Bounds {
    Bounds::new(a, z).unwrap()
}
fn near(a: f64, z: f64) {
    assert!((a - z).abs() <= 1e-12 * z.abs().max(1.), "{a} != {z}");
}
#[test]
fn logarithmic_policies_roundtrips_and_invalids() {
    let tr = ScaleTransform::Log { base: 10. };
    let scale = NonlinearScale::resolve(
        Some(Extent {
            minimum: 1.,
            maximum: 1000.,
        }),
        ContinuousDomain::default(),
        tr,
        b(0., 300.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    for (x, p) in [(1., 0.), (10., 100.), (100., 200.), (1000., 300.)] {
        near(scale.map(x).unwrap().unwrap(), p);
        near(scale.invert(p).unwrap(), x);
    }
    assert_eq!(
        scale
            .ticks(4, 32)
            .unwrap()
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        vec!["1", "10", "100", "1000"]
    );
    assert_eq!(scale.map(0.).unwrap(), None);
    assert_eq!(scale.map(-1.).unwrap(), None);
    let explicit = ContinuousDomain {
        explicit: Some(b(100., 1.)),
        baseline: Baseline::Zero,
        padding: 0.4,
        nice: true,
        ..Default::default()
    };
    let reversed = NonlinearScale::resolve(
        Some(Extent {
            minimum: 7.,
            maximum: 7.,
        }),
        explicit,
        tr,
        b(0., 200.),
        Some(b(10., 1.)),
        OutsidePolicy::Clamp,
    )
    .unwrap();
    assert_eq!(reversed.domain(), b(100., 1.));
    near(reversed.map(10.).unwrap().unwrap(), 0.);
    near(reversed.map(1.).unwrap().unwrap(), 200.);
    near(reversed.map(100.).unwrap().unwrap(), 0.);
    let empty = NonlinearScale::resolve(
        None,
        ContinuousDomain::default(),
        tr,
        b(0., 1.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    near(empty.domain().start(), 1.);
    near(empty.domain().end(), 10.);
    let c = NonlinearScale::resolve(
        Some(Extent {
            minimum: 100.,
            maximum: 100.,
        }),
        ContinuousDomain::default(),
        tr,
        b(0., 1.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    near(c.map(100.).unwrap().unwrap(), 0.5);
    for base in [1., 0., -1., f64::INFINITY, f64::NAN] {
        assert!(ScaleTransform::Log { base }.validate().is_err());
    }
    assert!(
        NonlinearScale::resolve(
            None,
            ContinuousDomain::explicit(b(0., 10.)),
            tr,
            b(0., 1.),
            None,
            OutsidePolicy::Extend
        )
        .is_err()
    );
    assert!(
        NonlinearScale::resolve(
            None,
            ContinuousDomain {
                baseline: Baseline::Zero,
                ..Default::default()
            },
            tr,
            b(0., 1.),
            None,
            OutsidePolicy::Extend
        )
        .is_err()
    );
}
#[test]
fn symlog_signed_roundtrip_extreme_ratio() {
    let tr = ScaleTransform::Symlog { threshold: 2. };
    for (x, y) in [(-6., -4_f64.ln()), (0., 0.), (6., 4_f64.ln())] {
        near(tr.forward(x).unwrap().unwrap(), y);
        near(tr.inverse(y).unwrap(), x);
    }
    let scale = NonlinearScale::resolve(
        None,
        ContinuousDomain::explicit(b(-6., 6.)),
        tr,
        b(300., 0.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    near(scale.map(0.).unwrap().unwrap(), 150.);
    let tr = ScaleTransform::Symlog { threshold: 1e-300 };
    for x in [-1e200, -1., 0., 1., 1e200] {
        near(tr.inverse(tr.forward(x).unwrap().unwrap()).unwrap(), x);
    }
    assert!(ScaleTransform::Symlog { threshold: 0. }.validate().is_err());
}
#[test]
fn points_and_colors_have_declared_missing_and_domain_policy() {
    let labels = vec!["b".into(), "a".into(), "c".into()];
    let points = PointScale::resolve(&labels, &PointOptions::default(), b(0., 300.)).unwrap();
    for (s, p) in [("b", 50.), ("a", 150.), ("c", 250.)] {
        near(points.center(s).unwrap().unwrap(), p);
        assert_eq!(points.category_at(p).unwrap(), Some(s));
    }
    assert!(!points.capabilities().numeric_inverse);
    assert_eq!(points.center("unknown").unwrap(), None);
    assert_eq!(points.category_at(100.).unwrap(), Some("b"));
    assert!(
        PointScale::resolve(
            &labels,
            &PointOptions {
                padding: f64::MAX,
                domain: None
            },
            b(0., 1.)
        )
        .is_err()
    );
    let black = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    let white = Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };
    let null = Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 0,
    };
    let colors = ColorScale::Continuous {
        domain: b(0., 10.),
        palette: vec![black, white],
        clamp: true,
        missing: null,
    };
    assert_eq!(
        colors.numeric(Some(5.)).unwrap(),
        Color {
            red: 128,
            green: 128,
            blue: 128,
            alpha: 255
        }
    );
    assert_eq!(colors.numeric(Some(f64::MAX)).unwrap(), white);
    assert_eq!(colors.numeric(None).unwrap(), null);
    let ordinal = ColorScale::Discrete {
        domain: Some(labels.clone()),
        palette: vec![black, white],
        missing: null,
    };
    assert_eq!(ordinal.categorical(Some("c"), &[]).unwrap(), black);
    assert_eq!(ordinal.categorical(Some("x"), &[]).unwrap(), null);
    assert_eq!(
        ordinal.legend(ScaleId::new(8), &[]).unwrap().entries.len(),
        3
    );
}
#[test]
fn supplied_sessions_compress_gaps_and_retain_exact_large_timestamps() {
    let origin = 9_000_000_000_000_000_000_i64;
    let calendar = SessionCalendar {
        id: "fixture-only".into(),
        revision: Revision::INITIAL,
        unit: TimeUnit::Nanoseconds,
        sessions: vec![
            TimeBounds {
                start: origin,
                end: origin + 10,
            },
            TimeBounds {
                start: origin + 20,
                end: origin + 30,
            },
        ],
        closed: ClosedSessionPolicy::Omit,
    };
    let scale =
        SessionScale::new(calendar.clone(), b(0., 200.), None, OutsidePolicy::Extend).unwrap();
    assert_eq!(scale.map(origin + 5).unwrap(), Some(50.));
    assert_eq!(scale.map(origin + 15).unwrap(), None);
    assert_eq!(scale.map(origin + 20).unwrap(), Some(100.));
    assert_eq!(scale.invert(100.).unwrap(), origin + 20);
    assert_eq!(scale.invert(200.).unwrap(), origin + 30);
    let nearest = SessionScale::new(
        SessionCalendar {
            closed: ClosedSessionPolicy::Nearest,
            ..calendar.clone()
        },
        b(0., 200.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(nearest.map(origin + 15).unwrap(), Some(100.));
    let strict = SessionScale::new(
        SessionCalendar {
            closed: ClosedSessionPolicy::Error,
            ..calendar
        },
        b(0., 200.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert!(strict.map(origin + 15).is_err());
}
