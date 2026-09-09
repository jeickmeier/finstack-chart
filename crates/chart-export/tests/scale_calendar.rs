//! SP-06 supplied timezone geometry, retained axes and publication use one revision.
use chart_core::{Revision, composition::ScaleValue, data::TimeUnit, prelude::*, scales::*};
use chart_export::*;
use std::sync::Arc;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn zone() -> CalendarZone {
    CalendarZone::Local(Arc::new(TimeZoneRules {
        version: 1,
        zone: "America/New_York".into(),
        revision: Revision::new(42),
        tzdata: "2025c".into(),
        coverage: TimeBounds {
            start: 1_672_531_200_000,
            end: 1_735_689_600_000,
        },
        initial_offset_seconds: -18000,
        transitions: vec![
            TimeZoneTransition {
                at_millis: 1_678_604_400_000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1_699_164_000_000,
                offset_seconds: -18000,
            },
            TimeZoneTransition {
                at_millis: 1_710_054_000_000,
                offset_seconds: -14400,
            },
            TimeZoneTransition {
                at_millis: 1_730_613_600_000,
                offset_seconds: -18000,
            },
        ],
    }))
}
fn figure(start: i64) -> Plot {
    let spec = TimeScaleSpec {
        domain: vec![start, start + 5 * 3_600_000],
        zone: zone(),
        ..Default::default()
    };
    plot(
        Data::columns()
            .column(
                "time",
                timestamps(
                    (0..=5).map(|i| start + i * 3_600_000).collect(),
                    TimeUnit::Milliseconds,
                    "UTC",
                ),
            )
            .column("value", [1., 2., 1.5, 3., 2., 4.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("time").y("value"))
    .layer(line())
    .layer(points().size(3.))
    .x_axis(
        x_axis()
            .scale(
                scale_calendar(spec).calendar_interval(CalendarInterval::new(CalendarUnit::Hour)),
            )
            .time_format(TimeFormat {
                pattern: Some("%H:%M %Z".into()),
                ..Default::default()
            }),
    )
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap()
}
#[test]
fn fold_gap_labels_and_geometry_share_revision_in_v5_publication() {
    let output = Output::new(FONT).unwrap();
    for (name, start, expected) in [
        (
            "spring",
            1_710_046_800_000,
            [
                "00:00 -0500",
                "01:00 -0500",
                "03:00 -0400",
                "04:00 -0400",
                "05:00 -0400",
                "06:00 -0400",
            ],
        ),
        (
            "fall",
            1_730_606_400_000,
            [
                "00:00 -0400",
                "01:00 -0400",
                "01:00 -0500",
                "02:00 -0500",
                "03:00 -0500",
                "04:00 -0500",
            ],
        ),
    ] {
        let p = figure(start);
        let wire = p.to_json().unwrap();
        assert_eq!(p.definition().wire_version(), 5);
        let p = Plot::from_json(&wire).unwrap();
        assert_eq!(p.to_json().unwrap(), wire);
        let mut wrong: serde_json::Value = serde_json::from_str(&wire).unwrap();
        wrong["version"] = 4.into();
        assert!(Plot::from_json(&wrong.to_string()).is_err());
        let request = output
            .request(&p, export_options(PageSize::points(900., 300.).unwrap()))
            .unwrap();
        let f = request.prepare().unwrap();
        let axis = f
            .layout()
            .axes()
            .values()
            .find(|a| matches!(a.scale, chart_core::layout::ResolvedScale::Calendar(_)))
            .unwrap();
        let chart_core::layout::ResolvedScale::Calendar(scale) = &axis.scale else {
            unreachable!()
        };
        assert_eq!(scale.calendar().zone(), &zone());
        assert_eq!(
            axis.ticks
                .iter()
                .map(|t| t.label.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        for i in 0..=5 {
            let value = ScaleValue::Timestamp {
                value: start + i * 3_600_000,
                unit: TimeUnit::Milliseconds,
            };
            let position = axis.map_value(&value).unwrap().unwrap();
            assert_eq!(axis.invert_value(position).unwrap(), value);
            let expected = scale.range().start()
                + (scale.range().end() - scale.range().start()) * i as f64 / 5.;
            assert!((position - expected).abs() < 1e-10);
        }
        let svg = f.export(Format::Svg).unwrap().bytes;
        assert_eq!(
            request
                .prepare()
                .unwrap()
                .export(Format::Svg)
                .unwrap()
                .bytes,
            svg
        );
        if let Some(dir) = std::env::var_os("CHART_TIME_ARTIFACTS") {
            let dir = std::path::Path::new(&dir);
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join(format!("{name}.plot.json")), wire).unwrap();
            for (extension, format) in [
                ("svg", Format::Svg),
                ("pdf", Format::Pdf),
                ("png", Format::Png),
            ] {
                std::fs::write(
                    dir.join(format!("{name}.{extension}")),
                    f.export(format).unwrap().bytes,
                )
                .unwrap();
            }
        }
    }
}
#[test]
fn calendar_axis_preserves_piecewise_knots_units_and_visible_windows() {
    use chart_core::{interpolate::Value, layout::ResolvedScale};
    let start = 1_700_000_000_000_000_001;
    let spec = TimeScaleSpec {
        unit: TimeUnit::Nanoseconds,
        domain: vec![start, start + 2, start + 10],
        range: vec![Value::number(0.), Value::number(50.), Value::number(100.)],
        ..Default::default()
    };
    let axis = x_axis().scale(scale_calendar(spec.clone())).visible(false);
    let id = axis.handle().unwrap().id();
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(
                    vec![start, start + 2, start + 10],
                    TimeUnit::Nanoseconds,
                    "UTC",
                ),
            )
            .column("y", [1., 2., 3.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(axis)
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap();
    let f = Output::new(FONT)
        .unwrap()
        .request(&p, export_options(PageSize::points(500., 300.).unwrap()))
        .unwrap()
        .prepare()
        .unwrap();
    let ResolvedScale::Calendar(s) = &f.layout().axes()[&id].scale else {
        panic!("calendar")
    };
    let middle = (s.range().start() + s.range().end()) / 2.;
    assert!((s.map(start + 2).unwrap().unwrap() - middle).abs() < 1e-10);
    assert_eq!(s.invert(middle).unwrap(), start + 2);
    let view = TimeAxisScale::resolve(
        spec,
        Bounds::new(0., 100.).unwrap(),
        Some(TimeBounds {
            start: start + 2,
            end: start + 10,
        }),
        OutsidePolicy::Omit,
    )
    .unwrap();
    assert_eq!(view.map(start).unwrap(), None);
    assert_eq!(view.map(start + 2).unwrap(), Some(0.));
    assert_eq!(view.invert(50.).unwrap(), start + 6);
}
