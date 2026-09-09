//! AXIS-01/FIX-19: independently identified guides retain shared positional meaning.
use chart_core::{
    ChartResult, DiagnosticCode, GuideId, Rect, ResourceId, Revision, ScaleId,
    composition::ScaleValue,
    grammar::{Compiler, PreparedChart},
    layout::{AxisScale, AxisSide, AxisSpec, CustomGuideTick, GuideSpec, LayoutRequest, layout},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use std::{cell::Cell, sync::Arc};
struct Metrics(Cell<usize>);
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.0.set(self.0.get() + 1);
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn prepared() -> Arc<PreparedChart> {
    let p = plot(
        Data::columns()
            .column("x", vec![0., 5., 10.])
            .column("y", vec![1., 2., 3.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .build()
    .unwrap();
    Arc::new(
        Compiler::new()
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap(),
    )
}
fn request() -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    r.axes[0].scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 10.).unwrap()));
    r.axes[0].range = Some(Bounds::new(100., 500.).unwrap());
    r.axes[1].range = Some(Bounds::new(300., 100.).unwrap());
    r
}
fn guide(id: u64, side: AxisSide, translation: [f64; 2]) -> GuideSpec {
    let mut g = GuideSpec::new(GuideId::new(id), ScaleId::new(0), side);
    g.translation = translation;
    g.guide_ticks = Some(vec![
        CustomGuideTick {
            value: ScaleValue::Number(2.),
            label: "two".into(),
        },
        CustomGuideTick {
            value: ScaleValue::Number(8.),
            label: "eight".into(),
        },
    ]);
    g
}
#[test]
fn one_scale_drives_top_bottom_and_two_translated_same_side_guides() {
    let mut r = request();
    r.guides = vec![
        guide(10, AxisSide::Top, [0., 0.]),
        guide(11, AxisSide::Bottom, [10., 30.]),
        guide(12, AxisSide::Bottom, [20., 60.]),
    ];
    let scene = layout(prepared(), &r, &Metrics(Cell::new(0))).unwrap();
    assert_eq!(scene.axes().len(), 2);
    assert_eq!(scene.guides().len(), 5);
    for (id, offset) in [(10, 0.), (11, 10.), (12, 20.)] {
        let g = &scene.guides()[&GuideId::new(id)];
        assert_eq!(g.spec.scale, ScaleId::new(0));
        assert_eq!(
            g.ticks.iter().map(|t| t.position).collect::<Vec<_>>(),
            vec![180. + offset, 420. + offset]
        );
        assert_eq!(
            g.ticks.iter().map(|t| t.value.clone()).collect::<Vec<_>>(),
            vec![ScaleValue::Number(2.), ScaleValue::Number(8.)]
        );
    }
    assert_eq!(
        scene.axes()[&ScaleId::new(0)]
            .map_value(&ScaleValue::Number(2.))
            .unwrap(),
        Some(180.)
    );
}
#[test]
fn replacing_one_guide_scale_preserves_its_identity_and_other_guides() {
    let mut r = request();
    let mut other = AxisSpec::new(ScaleId::new(9), AxisSide::Top);
    other.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 20.).unwrap()));
    other.range = r.axes[0].range;
    other.visible = false;
    r.axes.push(other);
    r.guides = vec![
        guide(10, AxisSide::Top, [0., 0.]),
        guide(11, AxisSide::Bottom, [0., 30.]),
    ];
    let before = layout(prepared(), &r, &Metrics(Cell::new(0))).unwrap();
    r.guides[0].scale = ScaleId::new(9);
    let after = layout(prepared(), &r, &Metrics(Cell::new(0))).unwrap();
    assert_eq!(
        before.guides()[&GuideId::new(10)].spec.id,
        after.guides()[&GuideId::new(10)].spec.id
    );
    assert_eq!(
        after.guides()[&GuideId::new(10)]
            .ticks
            .iter()
            .map(|t| t.position)
            .collect::<Vec<_>>(),
        vec![140., 260.]
    );
    assert_eq!(
        before.guides()[&GuideId::new(11)].ticks,
        after.guides()[&GuideId::new(11)].ticks
    );
    assert_eq!(
        before.axes()[&ScaleId::new(0)].ticks,
        after.axes()[&ScaleId::new(0)].ticks
    );
}
#[test]
fn invalid_guide_identity_orientation_reference_and_work_reject_before_measurement() {
    for failure in 0..5 {
        let mut r = request();
        let mut g = guide(10, AxisSide::Top, [0., 0.]);
        match failure {
            0 => g.id = GuideId::new(0),
            1 => g.side = AxisSide::Right,
            2 => g.scale = ScaleId::new(99),
            3 => g.translation = [f64::NAN, 0.],
            _ => {}
        }
        r.guides.push(g);
        if failure == 4 {
            r.guides = (10..73)
                .map(|id| guide(id, AxisSide::Top, [0., 0.]))
                .collect();
        }
        let metrics = Metrics(Cell::new(0));
        let err = layout(prepared(), &r, &metrics).unwrap_err();
        assert_eq!(metrics.0.get(), 0);
        if failure == 4 {
            assert_eq!(err.code, DiagnosticCode::ResourceLimit);
        }
    }
}
#[test]
fn legacy_axis_wire_fields_round_trip_and_unknown_fields_still_reject() {
    let axis = request().axes.remove(0);
    let mut wire = serde_json::to_value(&axis).unwrap();
    assert_eq!(wire["visible"], true);
    assert!(wire.get("guide").is_none());
    assert_eq!(
        serde_json::from_value::<AxisSpec>(wire.clone()).unwrap(),
        axis
    );
    wire["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<AxisSpec>(wire).is_err());
    let g = guide(10, AxisSide::Top, [10., 20.]);
    assert_eq!(
        serde_json::from_value::<GuideSpec>(serde_json::to_value(&g).unwrap()).unwrap(),
        g
    );
}

#[test]
fn primary_builders_names_edits_and_version_eight_round_trips_retain_ownership() {
    let top = axis_guide("top", "x")
        .side(AxisSide::Top)
        .ticks([(ScaleValue::Number(2.), "two".into())]);
    let top_id = top.handle().unwrap().id();
    let p = plot(
        Data::columns()
            .column("x", vec![0., 10.])
            .column("y", vec![1., 2.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_linear().domain(0., 10.))
            .range(100., 500.),
    )
    .axis(
        x_axis()
            .name("other")
            .side(AxisSide::Top)
            .visible(false)
            .scale(scale_linear().domain(0., 20.))
            .range(100., 500.),
    )
    .guide(top)
    .guide(
        axis_guide("second", "x")
            .side(AxisSide::Bottom)
            .translate(0., 30.),
    )
    .build()
    .unwrap();
    assert_eq!(p.guide("top").unwrap().id(), top_id);
    assert_eq!(p.guide_axis("top").unwrap(), p.axis("x").unwrap());
    assert_eq!(p.guide("x").unwrap(), p.axis("x").unwrap().guide());
    assert_eq!(p.definition().wire_version(), 8);
    let encoded = p.to_json().unwrap();
    let round = Plot::from_json(&encoded).unwrap();
    assert_eq!(
        p.named_guides().collect::<Vec<_>>(),
        round.named_guides().collect::<Vec<_>>()
    );
    assert_eq!(p.definition(), round.definition());
    let edited = p
        .edit()
        .guide(axis_guide("top", "other").side(AxisSide::Top))
        .build()
        .unwrap();
    assert_eq!(edited.guide("top").unwrap().id(), top_id);
    assert_eq!(edited.guide_axis("top").unwrap(), p.axis("other").unwrap());
    assert_eq!(edited.guide_axis("second").unwrap(), p.axis("x").unwrap());
    assert_eq!(p.guide_axis("top").unwrap(), p.axis("x").unwrap());
    let mut bad: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    bad["version"] = serde_json::json!(7);
    assert!(Plot::from_json(&bad.to_string()).is_err());
    bad["version"] = serde_json::json!(8);
    bad["guides"] = serde_json::json!({});
    assert!(Plot::from_json(&bad.to_string()).is_err());
}
