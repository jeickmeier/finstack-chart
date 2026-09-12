//! Offline comparison with real D3 transition scheduling/joins and SVG transforms.
use chart_core::{
    ChartResult,
    composition::ScaleValue,
    layout::{
        AxisSide, GuideDomain, GuideTransitionFrame, GuideTransitionPlan, GuideTransitionTick,
    },
    scales::{BandScale, BandSpec, Bounds, NumericScale},
};
use serde_json::Value;

enum Mapping {
    Numeric(NumericScale),
    Band(BandScale),
}
impl Mapping {
    fn new(spec: &Value) -> Self {
        let range = spec
            .get("range")
            .map(|v| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()])
            .unwrap_or([0., 100.]);
        if spec["band"] == true {
            Self::Band(
                BandScale::resolve_d3(
                    &[],
                    &BandSpec {
                        domain: Some(
                            spec["domain"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|s| s.as_str().unwrap().into())
                                .collect(),
                        ),
                        padding_inner: 0.2,
                        padding_outer: 0.2,
                        align: 0.5,
                        round: spec["round"] == true,
                    },
                    Bounds::new(range[0], range[1]).unwrap(),
                )
                .unwrap(),
            )
        } else {
            let domain = spec
                .get("domain")
                .map(|v| vec![v[0].as_f64().unwrap(), v[1].as_f64().unwrap()])
                .unwrap_or(vec![0., 1.]);
            Self::Numeric(
                NumericScale::linear()
                    .with_domain(domain)
                    .unwrap()
                    .with_range(range)
                    .unwrap(),
            )
        }
    }
    fn raw(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
        match (self, value) {
            (Self::Numeric(s), ScaleValue::Number(n)) => s.map_finite(*n),
            (Self::Band(s), ScaleValue::Category(c)) => s.start(c),
            _ => panic!("invalid fixture"),
        }
    }
    fn position_at(&self, value: &ScaleValue, offset: f64) -> ChartResult<Option<f64>> {
        let position = match (self, value) {
            (Self::Band(s), ScaleValue::Category(c)) => s.guide_position(c, offset)?,
            _ => self.raw(value)?,
        };
        Ok(position.map(|p| p + offset))
    }
}
fn value(v: &Value) -> ScaleValue {
    if let Some(s) = v.as_str() {
        ScaleValue::Category(s.into())
    } else {
        ScaleValue::Number(v.as_f64().unwrap())
    }
}
fn numbers(s: &str) -> Vec<f64> {
    s.split(|c: char| c.is_alphabetic() || c == ',' || c == '(' || c == ')' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}
fn frame(spec: &Value, labels: &Value) -> GuideTransitionFrame {
    let side = match spec["side"].as_str().unwrap_or("Bottom") {
        "Top" => AxisSide::Top,
        "Right" => AxisSide::Right,
        "Left" => AxisSide::Left,
        _ => AxisSide::Bottom,
    };
    let horizontal = matches!(side, AxisSide::Top | AxisSide::Bottom);
    let sign = if matches!(side, AxisSide::Top | AxisSide::Left) {
        -1.
    } else {
        1.
    };
    let inner = spec["tickSizeInner"].as_f64().unwrap_or(6.);
    let outer = spec["tickSizeOuter"].as_f64().unwrap_or(6.);
    let padding = spec["tickPadding"].as_f64().unwrap_or(3.);
    let range = spec
        .get("range")
        .map(|v| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()])
        .unwrap_or([0., 100.]);
    let pixel_offset = spec["offset"].as_f64().unwrap_or(0.5);
    let numbers = match (horizontal, outer != 0.) {
        (true, true) => vec![
            range[0] + pixel_offset,
            sign * outer,
            pixel_offset,
            range[1] + pixel_offset,
            sign * outer,
        ],
        (true, false) => vec![
            range[0] + pixel_offset,
            pixel_offset,
            range[1] + pixel_offset,
        ],
        (false, true) => vec![
            sign * outer,
            range[0] + pixel_offset,
            pixel_offset,
            range[1] + pixel_offset,
            sign * outer,
        ],
        (false, false) => vec![
            pixel_offset,
            range[0] + pixel_offset,
            range[1] + pixel_offset,
        ],
    };
    let mapping = Mapping::new(spec);
    let ticks: Vec<_> = spec["values"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let value = value(v);
            GuideTransitionTick {
                identity: i,
                position: mapping.position_at(&value, pixel_offset).unwrap().unwrap(),
                value,
                index: i,
                // Tick formatting is certified separately by AX02. Here the captured
                // target label is input, and its immediate timing is the assertion.
                label: labels[i]["text"]["label"].as_str().unwrap().into(),
                opacity: 1.,
                line_end: sign * inner,
                label_offset: sign * (inner.max(0.) + padding),
            }
        })
        .collect();
    GuideTransitionFrame {
        side,
        translation: [0., 0.],
        domain: GuideDomain {
            horizontal,
            caps: outer != 0.,
            numbers,
        },
        next_identity: ticks.len(),
        ticks,
    }
}
fn close(a: f64, b: f64, context: &str) {
    assert!((a - b).abs() < 1e-9, "{context}: {a} != {b}");
}
fn compare(frame: &GuideTransitionFrame, expected: &Value, id: &str) {
    let domain = numbers(expected["domain"].as_str().unwrap());
    assert_eq!(frame.domain.numbers.len(), domain.len(), "{id}");
    for (&a, b) in frame.domain.numbers.iter().zip(domain) {
        close(a, b, id);
    }
    assert_eq!(
        frame.ticks.len(),
        expected["ticks"].as_array().unwrap().len(),
        "{id}"
    );
    for (actual, expected) in frame
        .ticks
        .iter()
        .zip(expected["ticks"].as_array().unwrap())
    {
        assert_eq!(
            actual.identity,
            expected["identity"].as_u64().unwrap() as usize,
            "{id}"
        );
        assert_eq!(actual.value, value(&expected["value"]), "{id}");
        assert_eq!(
            actual.label,
            expected["text"]["label"].as_str().unwrap(),
            "{id}"
        );
        let position = numbers(expected["attributes"]["transform"].as_str().unwrap());
        close(
            actual.position,
            position[usize::from(!frame.domain.horizontal)],
            id,
        );
        close(
            actual.opacity,
            expected["attributes"]["opacity"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
            id,
        );
        let attr = if frame.domain.horizontal { "y2" } else { "x2" };
        close(
            actual.line_end,
            expected["line"][attr].as_str().unwrap().parse().unwrap(),
            id,
        );
        let attr = if frame.domain.horizontal { "y" } else { "x" };
        close(
            actual.label_offset,
            expected["text"]["attributes"][attr]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
            id,
        );
    }
}
#[test]
fn pinned_transition_start_mid_end_and_interruption() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/axes/transitions.json")).unwrap();
    let mut samples = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let states = case["states"].as_array().unwrap();
        let before = frame(&case["before"], &states[0]["ticks"]);
        compare(&before, &states[0], id);
        let label_inputs: Value = case["after"]["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|_| serde_json::json!({"text":{"label":""}}))
            .collect::<Vec<_>>()
            .into();
        let after = frame(&case["after"], &label_inputs);
        // Labels in target selection order, including reordered duplicates.
        let mut after = after;
        for (i, tick) in after.ticks.iter_mut().enumerate() {
            tick.label = case["after_labels"][i].as_str().unwrap().into();
        }
        let old = Mapping::new(&case["before"]);
        let new = Mapping::new(&case["after"]);
        let mut plan = GuideTransitionPlan::new(
            before,
            after,
            |v| {
                Ok(old
                    .position_at(v, case["before"]["offset"].as_f64().unwrap_or(0.5))?
                    .map(|p| {
                        p - case["before"]["offset"].as_f64().unwrap_or(0.5)
                            + case["after"]["offset"].as_f64().unwrap_or(0.5)
                    }))
            },
            |v| new.raw(v),
            |v| new.position_at(v, case["after"]["offset"].as_f64().unwrap_or(0.5)),
            chart_core::Limits {
                max_items: 1000,
                ..Default::default()
            },
        )
        .unwrap();
        compare(&plan.sample(0.).unwrap(), &states[1], id);
        samples += 1;
        let mid = plan.sample(0.5).unwrap();
        compare(&mid, &states[2], id);
        samples += 1;
        if let Some(interrupt) = case.get("interrupt") {
            let target = frame(interrupt, &states.last().unwrap()["ticks"]);
            let mapping = Mapping::new(interrupt);
            plan = GuideTransitionPlan::new(
                mid,
                target,
                |v| new.position_at(v, case["after"]["offset"].as_f64().unwrap_or(0.5)),
                |v| mapping.raw(v),
                |v| mapping.position_at(v, 0.5),
                chart_core::Limits {
                    max_items: 1000,
                    ..Default::default()
                },
            )
            .unwrap();
            compare(&plan.sample(0.).unwrap(), &states[3], id);
            compare(&plan.sample(0.5).unwrap(), &states[4], id);
            samples += 2;
        }
        compare(&plan.sample(1.).unwrap(), states.last().unwrap(), id);
        samples += 1;
    }
    assert_eq!(samples, fixture["cases"].as_array().unwrap().len() * 3 + 2);
}

fn laid_out(
    domain: f64,
    values: &[f64],
    tick_labels: &[&str],
    range: [f64; 2],
    translation: [f64; 2],
) -> std::sync::Arc<chart_core::layout::LaidOutChart> {
    use chart_core::{
        Rect, ResourceId, Revision,
        grammar::Compiler,
        layout::{GuideFormatter, GuideProfile, LayoutRequest, layout},
        prelude::*,
        services::{
            ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
        },
        state::ChartState,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(
                r.text.len() as f64 * r.font_size * 0.5,
                r.font_size * 0.8,
                r.font_size * 0.2,
            )
        }
    }
    let p = plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_linear().domain(0., domain))
            .range(range[0], range[1])
            .visible(false),
    )
    .guide(
        axis_guide("animated", "x")
            .side(AxisSide::Bottom)
            .guide_profile(GuideProfile::D3_3_0_0)
            .translate(translation[0], translation[1])
            .tick_values(Some(
                values.iter().copied().map(ScaleValue::Number).collect(),
            ))
            .tick_format(Some(GuideFormatter::Labels(
                tick_labels.iter().map(|s| (*s).into()).collect(),
            ))),
    )
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap();
    let mut definition = p.definition().clone();
    definition.guides[0].id = chart_core::GuideId::new(10);
    let prepared = std::sync::Arc::new(
        Compiler::default()
            .prepare(
                &definition,
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap(),
    );
    let r = LayoutRequest::new(
        Rect::new(0., 0., 400., 250.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    std::sync::Arc::new(layout(prepared, &r, &Metrics).unwrap())
}
#[test]
fn displayed_scene_keeps_exit_labels_interruption_and_final_identity_without_relayout() {
    use chart_core::{
        Limits,
        layout::LayoutGuideTransition,
        scene::{GuideRole, Primitive},
    };
    let before = laid_out(1., &[0., 0.5, 1.], &["a", "b", "c"], [20., 120.], [0., 0.]);
    let after = laid_out(2., &[0., 1., 2.], &["x", "y", "z"], [20., 120.], [10., 3.]);
    let plan = LayoutGuideTransition::new(before, after.clone(), Limits::default()).unwrap();
    let mid = plan.sample(0.5).unwrap();
    let presentation = mid.guide_presentation();
    let frame = &presentation[0].frame;
    assert_eq!(
        frame
            .ticks
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        ["x", "b", "y", "z"]
    );
    let snapshots = mid.guide_snapshots();
    assert_eq!(
        snapshots
            .iter()
            .find(|s| s.spec.id == presentation[0].guide)
            .unwrap()
            .ticks
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        ["x", "b", "y", "z"]
    );
    let mut seen = 0;
    for item in mid.scene().items() {
        if let Some(c) = &item.guide
            && c.role == GuideRole::Line
        {
            let a = c.animation.unwrap();
            let tick = frame
                .ticks
                .iter()
                .find(|t| t.identity == a.identity)
                .unwrap();
            close(a.opacity, tick.opacity, "scene opacity");
            let Primitive::Rule { from, to, .. } = item.primitive else {
                panic!("line")
            };
            close(
                from.x(),
                tick.position + frame.translation[0],
                "painted position",
            );
            close(from.y(), frame.translation[1], "painted baseline");
            close(to.y() - from.y(), tick.line_end, "painted line");
            seen += 1;
        }
    }
    assert_eq!(seen, 4);
    let data = |c: &chart_core::layout::LaidOutChart| {
        c.scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(data(&mid), data(&after));
    assert_eq!(mid.scene().wire_version(), 15);
    let final_frame = plan.sample(1.).unwrap();
    assert_eq!(final_frame.scene().items(), after.scene().items());
    assert_eq!(
        final_frame.guide_presentation()[0]
            .frame
            .ticks
            .iter()
            .map(|t| t.identity)
            .collect::<Vec<_>>(),
        [0, 2, 3]
    );
    let next = laid_out(4., &[0., 2., 4.], &["u", "v", "w"], [10., 210.], [0., 0.]);
    let interrupted = LayoutGuideTransition::new(mid, next, Limits::default())
        .unwrap()
        .sample(0.)
        .unwrap();
    let interrupted = interrupted.guide_presentation();
    let entered = interrupted[0]
        .frame
        .ticks
        .iter()
        .find(|t| t.value == ScaleValue::Number(4.))
        .unwrap();
    // Prior TARGET scale [0,2] -> [20,120], rather than displayed tick interpolation.
    close(entered.position, 220.5, "interrupted enter target mapping");
    assert!(plan.sample(f64::NAN).is_err());
    assert!(
        LayoutGuideTransition::new(
            after.clone(),
            after,
            Limits {
                max_items: 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
}
