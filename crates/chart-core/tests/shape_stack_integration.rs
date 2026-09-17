//! FIX-S07/09: tidy adaptation, sparse topology, actual targets and legacy isolation.
use chart_core::plot::{shape_area, shape_stack};
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{Compiler, GroupValue, PreparedChart, PreparedGeometry},
    layout::{AxisScale, LayoutRequest, layout},
    plot::ColumnData,
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    shape::{StackMissing, StackOffset, StackOrder, StackSeries},
    state::ChartState,
};
use serde::Deserialize;
use std::sync::Arc;
const KEY: u64 = 9_007_199_254_741_001;
#[derive(Deserialize)]
struct Case {
    id: String,
    matrix: Vec<Vec<Option<f64>>>,
    keys: Vec<String>,
    order: StackOrder,
    offset: StackOffset,
    missing: StackMissing,
    series: Vec<StackSeries<serde_json::Value>>,
}
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}
fn prepare(p: &Plot) -> ChartResult<PreparedChart> {
    Compiler::new().prepare(
        p.definition(),
        &p.source(),
        &ChartState::default(),
        p.compile_limits(),
    )
}
fn near(a: f64, b: f64, id: &str) {
    assert!((a - b).abs() <= 2e-12 * b.abs().max(1.), "{id}: {a} != {b}");
}
fn make(c: &Case, area: bool, reverse: bool) -> Plot {
    let mut rows: Vec<_> = c
        .matrix
        .iter()
        .enumerate()
        .flat_map(|(sample, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(group, value)| value.map(|v| (sample, group, v)))
        })
        .collect();
    if reverse {
        rows.reverse();
    }
    let data = Data::columns()
        .column("x", rows.iter().map(|r| r.0 as f64).collect::<Vec<_>>())
        .column("y", rows.iter().map(|r| r.2).collect::<Vec<_>>())
        .column("group", rows.iter().map(|r| r.1 as i64).collect::<Vec<_>>())
        .keys(rows.iter().map(|r| KEY + (r.0 * c.keys.len() + r.1) as u64))
        .build()
        .unwrap();
    let layer = if area { shape_area() } else { bars() };
    plot(data)
        .aes(aes().x("x").x2("x").y("y").y2(0.).group("group"))
        .layer(
            layer.position(
                shape_stack(
                    (0..c.keys.len())
                        .map(|g| GroupValue::Int(g as i64))
                        .collect(),
                )
                .stack_order(c.order.clone())
                .stack_offset(c.offset)
                .stack_missing(c.missing),
            ),
        )
        .build()
        .unwrap()
}
#[test]
fn all_reference_layouts_flow_through_tidy_bars_and_areas_in_either_row_order() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/shapes/stack.json")).unwrap();
    let mut checked = 0;
    for c in corpus.cases.iter().filter(|c| !c.keys.is_empty()) {
        for area in [false, true] {
            for reverse in [false, true] {
                let p = make(c, area, reverse);
                let prepared = prepare(&p).unwrap_or_else(|e| panic!("{}: {e}", c.id));
                assert_eq!(p.definition().wire_version(), 7);
                let mut observed = vec![];
                for mark in prepared.layers()[0].marks() {
                    let GroupValue::Int(group) = mark.group else {
                        panic!("integer group");
                    };
                    match &mark.geometry {
                        PreparedGeometry::Bar { from, to, .. } => {
                            let sample = from.x() as usize;
                            let expected = &c.series[group as usize].points[sample];
                            near(from.y(), expected.y1.0, &c.id);
                            near(to.y(), expected.y0.0, &c.id);
                        }
                        PreparedGeometry::StackBandRun {
                            lower,
                            upper,
                            sources,
                        } => {
                            assert_eq!(lower.len(), sources.len());
                            assert_eq!(lower.len(), upper.len());
                            for ((lo, hi), source) in lower.iter().zip(upper).zip(sources) {
                                let sample = lo.x() as usize;
                                let expected = &c.series[group as usize].points[sample];
                                near(lo.y(), expected.y0.0, &c.id);
                                near(hi.y(), expected.y1.0, &c.id);
                                assert_eq!(
                                    source.is_some(),
                                    c.matrix[sample][group as usize].is_some()
                                );
                                if let Some(i) = source {
                                    assert!(*i < mark.targets.len());
                                }
                            }
                        }
                        _ => panic!("unexpected geometry"),
                    }
                    for target in &mark.targets {
                        let chart_core::provenance::Target::Source(source) = target else {
                            panic!("source");
                        };
                        observed.push(source.key.get());
                    }
                }
                observed.sort_unstable();
                let mut expected: Vec<_> = c
                    .matrix
                    .iter()
                    .enumerate()
                    .flat_map(|(i, row)| {
                        row.iter().enumerate().filter_map(move |(g, v)| {
                            v.map(|_| KEY + (i * c.keys.len() + g) as u64)
                        })
                    })
                    .collect();
                expected.sort_unstable();
                assert_eq!(observed, expected, "{} area={area}", c.id);
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 1620);
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(p: &Plot) -> chart_core::layout::LaidOutChart {
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.padding = 0.;
    for axis in &mut request.axes {
        axis.visible = false;
        axis.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 4.).unwrap()));
    }
    layout(Arc::new(prepare(p).unwrap()), &request, &Metrics).unwrap()
}
#[test]
fn sparse_zero_boundaries_have_no_synthetic_observation_and_gaps_do_not_bridge() {
    let mut c = Case {
        id: "sparse".into(),
        matrix: vec![
            vec![Some(2.), Some(1.)],
            vec![None, Some(1.)],
            vec![Some(2.), Some(1.)],
        ],
        keys: vec!["a".into(), "b".into()],
        order: StackOrder::None,
        offset: StackOffset::None,
        missing: StackMissing::Zero,
        series: vec![],
    };
    let p = make(&c, true, false);
    let f = frame(&p);
    let shapes: Vec<_> = f
        .scene()
        .items()
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            if let Primitive::ShapePath {
                geometry, anchors, ..
            } = &item.primitive
            {
                Some((i, geometry, anchors))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(shapes.len(), 2);
    assert_eq!(
        shapes[0].2,
        &[
            Point::new(0., 100.).unwrap(),
            Point::new(200., 100.).unwrap()
        ]
    );
    assert_eq!(f.targets()[shapes[0].0].len(), 2);
    assert!(
        shapes[0]
            .1
            .commands()
            .iter()
            .any(|c| matches!(c, chart_core::path::Command::LineTo([100., 200.])))
    );
    c.missing = StackMissing::Gap;
    let p = make(&c, true, false);
    let prepared = prepare(&p).unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 3);
    let f = frame(&p);
    assert_eq!(f.targets().iter().map(Vec::len).sum::<usize>(), 5);
    c.missing = StackMissing::Error;
    assert!(prepare(&make(&c, true, false)).is_err());
}
#[test]
fn duplicate_cells_baselines_catalogs_and_missing_payloads_are_explicit() {
    let data = Data::columns()
        .column("x", vec![0., 1., 2.])
        .column(
            "y",
            ColumnData::from(vec![2., 99., 2.]).validity(vec![true, false, true]),
        )
        .column("g", vec![0_i64, 0, 0])
        .keys([KEY, KEY + 1, KEY + 2])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").x2("x").y("y").y2(0.).group("g"))
        .layer(
            shape_area()
                .position(shape_stack(vec![GroupValue::Int(0)]).stack_missing(StackMissing::Zero)),
        )
        .build()
        .unwrap();
    let prepared = prepare(&p).unwrap();
    assert_eq!(prepared.layers()[0].marks()[0].targets.len(), 2);
    let mut definition = p.definition().clone();
    let mut layer = definition.layers[0].clone();
    if let chart_core::grammar::Mappings::Source(a) = &mut layer.mappings {
        a.x = Some(chart_core::grammar::Numeric::Literal(0.));
        a.x2 = Some(chart_core::grammar::Numeric::Literal(0.));
    }
    definition.layers[0] = layer;
    assert!(
        Compiler::new()
            .prepare(
                &definition,
                &p.source(),
                &ChartState::default(),
                p.compile_limits()
            )
            .is_err()
    );
    let mut definition = p.definition().clone();
    if let chart_core::grammar::Position::ShapeStack(s) = &mut definition.layers[0].position {
        s.groups = vec![GroupValue::Int(9)];
    }
    assert!(
        Compiler::new()
            .prepare(
                &definition,
                &p.source(),
                &ChartState::default(),
                p.compile_limits()
            )
            .is_err()
    );
    let mut definition = p.definition().clone();
    if let chart_core::grammar::Mappings::Source(a) = &mut definition.layers[0].mappings {
        a.y2 = Some(chart_core::grammar::Numeric::Literal(1.));
    }
    assert!(
        Compiler::new()
            .prepare(
                &definition,
                &p.source(),
                &ChartState::default(),
                p.compile_limits()
            )
            .is_err()
    );
    let mut definition = p.definition().clone();
    definition.layers[0].geom = chart_core::grammar::Geom::Point;
    assert!(
        Compiler::new()
            .prepare(
                &definition,
                &p.source(),
                &ChartState::default(),
                p.compile_limits()
            )
            .is_err()
    );
}
