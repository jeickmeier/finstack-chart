//! WP-06 FIX-07 scale portions. Expected values are independent of the implementation.
use chart_core::DiagnosticCode;
use chart_core::data::TimeUnit;
use chart_core::grammar::Extent;
use chart_core::scales::*;
fn bounds(a: f64, b: f64) -> Bounds {
    Bounds::new(a, b).unwrap()
}
fn near(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() <= tolerance,
        "{a} != {b}, tolerance {tolerance}"
    );
}

#[test]
fn fix07_empty_constant_descending_and_explicit_domains_are_locked() {
    let resolve = |data, options| {
        LinearScale::resolve(data, options, bounds(0., 100.), None, OutsidePolicy::Extend).unwrap()
    };
    let empty = resolve(None, ContinuousDomain::default());
    assert_eq!(empty.domain(), bounds(0., 1.));
    assert_eq!(empty.map(0.5).unwrap(), Some(50.));
    for (constant, a, b) in [(0., -1., 1.), (10., 9.5, 10.5), (-100., -105., -95.)] {
        let scale = resolve(
            Some(Extent {
                minimum: constant,
                maximum: constant,
            }),
            ContinuousDomain::default(),
        );
        assert_eq!(scale.domain(), bounds(a, b));
        near(scale.map(constant).unwrap().unwrap(), 50., 1e-12);
    }
    let explicit = ContinuousDomain {
        explicit: Some(bounds(5., -5.)),
        baseline: Baseline::Value(100.),
        padding: 0.5,
        nice: true,
        ..Default::default()
    };
    let scale = resolve(
        Some(Extent {
            minimum: -1000.,
            maximum: 1000.,
        }),
        explicit,
    );
    assert_eq!(scale.domain(), bounds(5., -5.));
    assert_eq!(scale.map(5.).unwrap(), Some(0.));
    assert_eq!(scale.map(-5.).unwrap(), Some(100.));
    for v in [-5., -2.5, 0., 2.5, 5.] {
        near(
            scale.invert(scale.map(v).unwrap().unwrap()).unwrap(),
            v,
            2e-15,
        );
    }
    assert_eq!(
        ScaleKind::from_name("unknown").unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    assert!(Bounds::new(f64::NAN, 1.).is_err());
    assert!(
        LinearScale::resolve(
            None,
            ContinuousDomain::explicit(bounds(f64::MAX, f64::MAX)),
            bounds(0., 1.),
            None,
            OutsidePolicy::Extend
        )
        .is_err()
    );
}

#[test]
fn baseline_padding_and_nice_precede_viewport_and_are_independent_of_range() {
    let options = ContinuousDomain {
        baseline: Baseline::Zero,
        padding: 0.1,
        nice: true,
        ..Default::default()
    };
    let a = LinearScale::resolve(
        Some(Extent {
            minimum: 2.,
            maximum: 3.,
        }),
        options,
        bounds(0., 200.),
        Some(bounds(2., 3.)),
        OutsidePolicy::Extend,
    )
    .unwrap();
    let b = LinearScale::resolve(
        Some(Extent {
            minimum: 2.,
            maximum: 3.,
        }),
        options,
        bounds(200., 0.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(a.domain(), bounds(-1., 4.));
    assert_eq!(a.domain(), b.domain());
    assert_eq!(a.viewport(), bounds(2., 3.));
    assert_eq!(a.map(2.).unwrap(), Some(0.));
    assert_eq!(a.map(3.).unwrap(), Some(200.));
    assert_eq!(b.map(-1.).unwrap(), Some(200.));
    assert_eq!(b.map(4.).unwrap(), Some(0.));
    assert!(
        LinearScale::resolve(
            None,
            ContinuousDomain {
                padding: f64::NAN,
                ..Default::default()
            },
            bounds(0., 1.),
            None,
            OutsidePolicy::Extend
        )
        .is_err()
    );
}

#[test]
fn robust_projection_handles_huge_domains_without_intermediate_overflow() {
    let scale = LinearScale::resolve(
        None,
        ContinuousDomain::explicit(bounds(-f64::MAX, f64::MAX)),
        bounds(0., 100.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(scale.map(-f64::MAX).unwrap(), Some(0.));
    assert_eq!(scale.map(0.).unwrap(), Some(50.));
    assert_eq!(scale.map(f64::MAX).unwrap(), Some(100.));
    assert_eq!(scale.invert(50.).unwrap(), 0.);
    let narrow = LinearScale::resolve(
        None,
        ContinuousDomain::explicit(bounds(1e16, 1e16 + 4.)),
        bounds(0., 100.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(narrow.map(1e16 + 2.).unwrap(), Some(50.));
    assert_eq!(narrow.invert(50.).unwrap(), 1e16 + 2.);
    for outside in [
        OutsidePolicy::Extend,
        OutsidePolicy::Clamp,
        OutsidePolicy::Omit,
    ] {
        let scale = LinearScale::resolve(
            None,
            ContinuousDomain::explicit(bounds(0., 1.)),
            bounds(0., 100.),
            None,
            outside,
        )
        .unwrap();
        assert_eq!(
            scale.map(2.).unwrap(),
            match outside {
                OutsidePolicy::Extend => Some(200.),
                OutsidePolicy::Clamp => Some(100.),
                OutsidePolicy::Omit => None,
            }
        );
        assert!(scale.map(f64::INFINITY).is_err());
        assert!(scale.invert(f64::NAN).is_err());
    }
}

#[test]
fn numeric_ticks_are_bounded_ordered_unique_and_locale_independent() {
    let scale = LinearScale::resolve(
        None,
        ContinuousDomain::explicit(bounds(-1., 1.)),
        bounds(0., 100.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    let ticks = scale.ticks(5, 16).unwrap();
    assert_eq!(
        ticks.iter().map(|t| t.value).collect::<Vec<_>>(),
        vec![-1., -0.5, 0., 0.5, 1.]
    );
    assert_eq!(
        ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
        vec!["−1", "−0.5", "0", "0.5", "1"]
    );
    assert!(scale.ticks(128, 1).is_err());
    assert!(scale.ticks(1, 16).is_err());
    let reversed = LinearScale::resolve(
        None,
        ContinuousDomain::explicit(bounds(1., -1.)),
        bounds(0., 100.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(
        reversed
            .ticks(5, 16)
            .unwrap()
            .iter()
            .map(|t| t.value)
            .collect::<Vec<_>>(),
        vec![1., 0.5, 0., -0.5, -1.]
    );
    assert_eq!(format_number(-0.), "0");
}

#[test]
fn categorical_explicit_order_centers_extents_gaps_and_lookup_are_distinct_from_inverse() {
    let observed = vec!["z".into(), "a".into(), "b".into()];
    let options = BandOptions {
        domain: None,
        inner_padding: 0.,
        outer_padding: 0.,
    };
    let scale = BandScale::resolve(&observed, &options, bounds(0., 300.)).unwrap();
    assert_eq!(scale.domain(), observed);
    assert_eq!(scale.center("z").unwrap(), Some(50.));
    assert_eq!(scale.center("b").unwrap(), Some(250.));
    assert_eq!(scale.extent("a").unwrap(), Some(bounds(100., 200.)));
    assert_eq!(scale.category_at(175.).unwrap(), Some("a"));
    assert_eq!(scale.category_at(300.).unwrap(), Some("b"));
    assert!(!scale.capabilities().numeric_inverse);
    assert!(scale.capabilities().category_lookup);
    assert_eq!(scale.center("missing").unwrap(), None);
    let explicit = BandScale::resolve(
        &observed,
        &BandOptions {
            domain: Some(vec!["b".into(), "z".into()]),
            ..options
        },
        bounds(200., 0.),
    )
    .unwrap();
    assert_eq!(explicit.center("b").unwrap(), Some(150.));
    assert_eq!(explicit.center("a").unwrap(), None);
    assert_eq!(explicit.extent("z").unwrap(), Some(bounds(100., 0.)));
    let gaps = BandScale::resolve(
        &observed,
        &BandOptions {
            inner_padding: 0.5,
            outer_padding: 0.25,
            domain: None,
        },
        bounds(0., 300.),
    )
    .unwrap();
    assert_eq!(gaps.extent("z").unwrap(), Some(bounds(25., 75.)));
    assert_eq!(gaps.category_at(100.).unwrap(), None);
    assert!(
        BandScale::resolve(
            &observed,
            &BandOptions {
                domain: Some(vec!["a".into(), "a".into()]),
                ..Default::default()
            },
            bounds(0., 1.)
        )
        .is_err()
    );
    assert!(
        BandScale::resolve(&[], &BandOptions::default(), bounds(0., 100.))
            .unwrap()
            .domain()
            .is_empty()
    );
}

#[test]
fn utc_calendar_matches_independent_unix_date_fixtures_and_rejects_invalid_dates() {
    // Values verified independently using Python datetime with timezone.utc.
    for (year, month, day, seconds) in [
        (1, 1, 1, -62_135_596_800),
        (1900, 3, 1, -2_203_891_200),
        (2000, 2, 29, 951_782_400),
        (2024, 2, 29, 1_709_164_800),
        (2024, 3, 1, 1_709_251_200),
        (9999, 12, 31, 253_402_214_400),
    ] {
        let expected = UtcDateTime {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
        };
        assert_eq!(expected.unix_seconds().unwrap(), seconds);
        assert_eq!(UtcDateTime::from_unix_seconds(seconds).unwrap(), expected);
    }
    assert_eq!(
        UtcDateTime::from_unix_seconds(-1).unwrap(),
        UtcDateTime {
            year: 1969,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59
        }
    );
    for (year, month, day) in [
        (1900, 2, 29),
        (2023, 2, 29),
        (2024, 4, 31),
        (0, 1, 1),
        (2024, 13, 1),
    ] {
        assert!(
            UtcDateTime {
                year,
                month,
                day,
                hour: 0,
                minute: 0,
                second: 0
            }
            .unix_seconds()
            .is_err()
        );
    }
}

#[test]
fn utc_ticks_follow_month_year_leap_day_and_monday_boundaries() {
    let scale = UtcScale::new(
        TimeBounds {
            start: 1_700_006_400,
            end: 1_709_424_000,
        },
        None,
        TimeUnit::Seconds,
        bounds(0., 400.),
        OutsidePolicy::Extend,
    )
    .unwrap();
    let months = scale.ticks(UtcInterval::Months(1), 16).unwrap();
    assert_eq!(
        months.iter().map(|t| t.value).collect::<Vec<_>>(),
        vec![1_701_388_800, 1_704_067_200, 1_706_745_600, 1_709_251_200]
    );
    assert_eq!(
        months.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
        vec!["2023-12", "2024-01", "2024-02", "2024-03"]
    );
    assert_eq!(
        scale.ticks(UtcInterval::Years(1), 16).unwrap(),
        vec![UtcTick {
            value: 1_704_067_200,
            label: "2024".into()
        }]
    );
    let leap = UtcScale::new(
        TimeBounds {
            start: 1_709_078_400,
            end: 1_709_251_200,
        },
        None,
        TimeUnit::Seconds,
        bounds(0., 200.),
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(
        leap.ticks(UtcInterval::Days(1), 8)
            .unwrap()
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        vec!["2024-02-28", "2024-02-29", "2024-03-01"]
    );
    let weeks = UtcScale::new(
        TimeBounds {
            start: -345_600,
            end: 950_400,
        },
        None,
        TimeUnit::Seconds,
        bounds(0., 100.),
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(
        weeks
            .ticks(UtcInterval::Weeks(1), 8)
            .unwrap()
            .iter()
            .map(|t| t.value)
            .collect::<Vec<_>>(),
        vec![-259_200, 345_600, 950_400]
    );
    assert!(scale.ticks(UtcInterval::Months(0), 8).is_err());
    assert!(scale.ticks(UtcInterval::Seconds(1), 2).is_err());
}

#[test]
fn fix07_large_origin_time_projection_keeps_integer_precision_and_labels() {
    let origin = 9_000_000_000_000_000_000_i64;
    let scale = UtcScale::new(
        TimeBounds {
            start: origin,
            end: origin + 1000,
        },
        None,
        TimeUnit::Nanoseconds,
        bounds(0., 100.),
        OutsidePolicy::Extend,
    )
    .unwrap();
    for offset in [0, 17, 257, 999, 1000] {
        let position = scale.map(origin + offset).unwrap().unwrap();
        near(position, offset as f64 / 10., 2e-14);
        assert!((scale.invert(position).unwrap() - (origin + offset)).abs() <= 1);
    }
    assert_eq!(scale.origin(), origin);
    assert_eq!(
        scale
            .ticks(UtcInterval::Ticks(250), 8)
            .unwrap()
            .iter()
            .map(|t| t.value - origin)
            .collect::<Vec<_>>(),
        vec![0, 250, 500, 750, 1000]
    );
    assert!(
        scale.ticks(UtcInterval::Ticks(250), 8).unwrap()[1]
            .label
            .ends_with(".000000250Z")
    );
    assert_eq!(
        UtcScale::new(
            TimeBounds {
                start: 0,
                end: (1_i64 << 53) + 1
            },
            None,
            TimeUnit::Nanoseconds,
            bounds(0., 100.),
            OutsidePolicy::Extend
        )
        .unwrap_err()
        .code,
        DiagnosticCode::PrecisionLoss
    );
    let before_epoch = UtcScale::new(
        TimeBounds { start: -2, end: 2 },
        None,
        TimeUnit::Milliseconds,
        bounds(0., 100.),
        OutsidePolicy::Extend,
    )
    .unwrap();
    assert_eq!(
        before_epoch.ticks(UtcInterval::Ticks(1), 8).unwrap()[1].label,
        "1969-12-31 23:59:59.999Z"
    );
}

#[test]
fn decimal_ticks_do_not_accumulate_label_noise_and_time_omission_precedes_precision_conversion() {
    let scale = LinearScale::resolve(
        None,
        ContinuousDomain::explicit(bounds(0., 1.)),
        bounds(0., 100.),
        None,
        OutsidePolicy::Extend,
    )
    .unwrap();
    let ticks = scale.ticks(11, 16).unwrap();
    assert_eq!(ticks[3].value, 0.3);
    assert_eq!(ticks[3].label, "0.3");
    for outside in [OutsidePolicy::Clamp, OutsidePolicy::Omit] {
        let scale = UtcScale::new(
            TimeBounds { start: 0, end: 100 },
            None,
            TimeUnit::Nanoseconds,
            bounds(0., 100.),
            outside,
        )
        .unwrap();
        assert_eq!(
            scale.map(i64::MAX).unwrap(),
            if outside == OutsidePolicy::Omit {
                None
            } else {
                Some(100.)
            }
        );
    }
}
