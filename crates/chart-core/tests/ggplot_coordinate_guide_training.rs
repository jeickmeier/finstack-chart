//! Coordinate view selection must use the existing guide owner over the new view.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, ScaleId,
    grammar::{CartesianCoordinate, CoordinateSpec, TransformedCoordinate},
    layout::{AxisScale, AxisSide, AxisSpec, LayoutRequest, layout},
    prelude::*,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(transformed: bool, secondary: bool) -> chart_core::layout::LaidOutChart {
    let d = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0., 1.])
        .build()
        .unwrap();
    let view = CartesianCoordinate {
        xlim: Some([
            Some(ScaleValue::Number(0.42)),
            Some(ScaleValue::Number(0.48)),
        ]),
        expand: [false; 4],
        ..Default::default()
    };
    let spec = if transformed {
        CoordinateSpec::Transformed(TransformedCoordinate {
            x: chart_core::scales::GgplotTransform::Sqrt,
            view,
            ..Default::default()
        })
    } else {
        CoordinateSpec::Cartesian(view)
    };
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(points().aes(aes().x("x").y("y")))
        .coordinate(spec)
        .build()
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    if secondary {
        let mut a = AxisSpec::new(ScaleId::new(2), AxisSide::Top);
        a.scale = AxisScale::Secondary {
            source: ScaleId::new(0),
            factor: 100.,
            offset: 0.,
            transform: None,
        };
        request.axes.push(a);
    }
    layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap()
}
#[test]
fn zoom_reselects_numeric_major_minor_and_secondary_values_without_retraining_marks() {
    for transformed in [false, true] {
        let f = frame(transformed, true);
        let plot = f.plot().unwrap();
        let axis = &f.axes()[&ScaleId::new(0)];
        let chart_core::layout::ResolvedScale::Linear(s) = &axis.scale else {
            panic!("linear scale")
        };
        assert_eq!(s.domain().minimum(), 0.);
        assert_eq!(s.domain().maximum(), 1.);
        for guide in f.guides().values().filter(|g| g.spec.side.horizontal()) {
            let factor = if guide.spec.scale == ScaleId::new(2) {
                100.
            } else {
                1.
            };
            assert!(guide.ticks.len() >= 3, "{:?}", guide.ticks);
            for tick in &guide.ticks {
                let ScaleValue::Number(n) = tick.value else {
                    panic!("number")
                };
                assert!(
                    n >= 0.42 * factor - 1e-10 && n <= 0.48 * factor + 1e-10,
                    "{n}"
                );
                assert!(
                    tick.position >= plot.origin().x() - 1e-6
                        && tick.position <= plot.max_x() + 1e-6,
                    "{}",
                    tick.position
                );
            }
            if factor == 1. {
                assert!(!guide.minor_ticks.is_empty());
                for tick in &guide.minor_ticks {
                    let Some(ScaleValue::Number(n)) = tick.value else {
                        panic!()
                    };
                    assert!((0.42..=0.48).contains(&n));
                }
            }
        }
    }
}
#[test]
fn temporal_coordinate_zoom_preserves_typed_tick_identity() {
    use chart_core::data::TimeUnit;
    let origin = 1_577_836_800_i64;
    let data = Data::columns()
        .column(
            "date",
            timestamps(vec![origin, origin + 30 * 86400], TimeUnit::Seconds, "UTC"),
        )
        .column("y", [0., 1.])
        .build()
        .unwrap();
    let view = CartesianCoordinate {
        xlim: Some([
            Some(ScaleValue::Timestamp {
                value: origin + 10 * 86400,
                unit: TimeUnit::Seconds,
            }),
            Some(ScaleValue::Timestamp {
                value: origin + 12 * 86400,
                unit: TimeUnit::Seconds,
            }),
        ]),
        expand: [false; 4],
        ..Default::default()
    };
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            points().aes(
                aes()
                    .x(Mapping::Timestamp {
                        field: "date".into(),
                        origin,
                    })
                    .y("y"),
            ),
        )
        .coordinate(CoordinateSpec::Cartesian(view))
        .build()
        .unwrap();
    let request = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    let f = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap();
    let guide = f
        .guides()
        .values()
        .find(|g| g.spec.side == AxisSide::Bottom)
        .unwrap();
    assert!(guide.ticks.len() >= 2);
    for tick in &guide.ticks {
        let ScaleValue::Timestamp { value, unit } = tick.value else {
            panic!("typed timestamp")
        };
        assert_eq!(unit, TimeUnit::Seconds);
        assert!((origin + 10 * 86400..=origin + 12 * 86400).contains(&value));
    }
}
