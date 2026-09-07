//! STM-05: independent extrema/order/gap and supplied OHLC/volume expectations.
use chart_core::{
    dense::*,
    provenance::{SourceRef, Target},
    *,
};
fn target(k: u64) -> Target {
    Target::Source(SourceRef {
        dataset: DatasetId::new(1),
        key: RowKey::new(k),
    })
}
#[test]
fn envelope_preserves_chronological_endpoints_and_extrema_per_bucket() {
    let view = Rect::new(0., 0., 3., 20.).unwrap();
    let ys = [5., 9., 1., 6., 3., 8., 2., 4., 7., 0., 10., 6.];
    let points: Vec<_> = ys
        .iter()
        .enumerate()
        .map(|(i, y)| Point::new(i as f64 / 4., *y).unwrap())
        .collect();
    let chosen = line_envelope(&points, view, 1., 4).unwrap().unwrap();
    assert_eq!(chosen, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    let many: Vec<_> = (0..1000)
        .map(|i| {
            Point::new(
                i as f64 / 100.,
                if i == 333 { 99. } else { (i % 17) as f64 },
            )
            .unwrap()
        })
        .collect();
    let chosen = line_envelope(&many, Rect::new(0., 0., 10., 100.).unwrap(), 1., 10)
        .unwrap()
        .unwrap();
    assert!(chosen.len() <= 40);
    assert_eq!(chosen.first(), Some(&0));
    assert_eq!(chosen.last(), Some(&999));
    assert!(chosen.contains(&333));
    assert!(chosen.windows(2).all(|w| w[0] < w[1]));
    for bucket in 0..10 {
        let rows = &many[bucket * 100..(bucket + 1) * 100];
        let min = rows.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min);
        let max = rows.iter().map(|p| p.y()).fold(f64::NEG_INFINITY, f64::max);
        let retained: Vec<_> = chosen
            .iter()
            .copied()
            .filter(|i| *i / 100 == bucket)
            .map(|i| many[i].y())
            .collect();
        assert!(retained.contains(&min) && retained.contains(&max));
    }
    let mut reversed = many.clone();
    reversed.reverse();
    let reversed_indices = line_envelope(&reversed, Rect::new(0., 0., 10., 100.).unwrap(), 1., 10)
        .unwrap()
        .unwrap();
    assert!(reversed_indices.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(reversed_indices.first(), Some(&0));
    assert_eq!(reversed_indices.last(), Some(&999));
    let turning = [
        Point::new(0., 0.).unwrap(),
        Point::new(2., 4.).unwrap(),
        Point::new(1., 3.).unwrap(),
    ];
    assert!(line_envelope(&turning, view, 1., 4).unwrap().is_none());
    assert!(line_envelope(&many, view, 0., 4).is_err());
    assert!(line_envelope(&many, view, 0.1, 4).is_err());
}
fn candle(key: u64, x: f64, display: f64, prices: [f64; 4], volume: Option<f64>) -> CandleSample {
    CandleSample {
        x,
        display_x: display,
        open: prices[0],
        high: prices[1],
        low: prices[2],
        close: prices[3],
        volume,
        target: target(key),
    }
}
#[test]
fn candle_buckets_use_chronological_first_open_high_low_last_close_and_valid_volume() {
    let data = vec![
        candle(3, 3., 0.8, [15., 18., 12., 14.], Some(9.)),
        candle(1, 1., 0.1, [10., 16., 8., 15.], Some(2.)),
        candle(2, 2., 0.5, [15., 20., 7., 15.], None),
        candle(4, 4., 1.4, [20., 21., 19., 20.], None),
        candle(5, 5., -1., [0., 0., 0., 0.], Some(500.)),
    ];
    let b = candle_buckets(&data, Rect::new(0., 0., 2., 100.).unwrap(), 1., 2).unwrap();
    assert_eq!(b.len(), 2);
    assert_eq!(
        (
            b[0].open,
            b[0].high,
            b[0].low,
            b[0].close,
            b[0].volume,
            b[0].valid_volumes
        ),
        (10., 20., 7., 14., Some(11.), 2)
    );
    assert_eq!(b[0].targets, vec![target(1), target(2), target(3)]);
    assert_eq!((b[0].first_x, b[0].last_x), (1., 3.));
    assert_eq!(b[1].volume, None);
    let cancel = vec![
        candle(1, 1., 0.1, [1., 1., 1., 1.], Some(1e16)),
        candle(2, 2., 0.2, [1., 1., 1., 1.], Some(1.)),
        candle(3, 3., 0.3, [1., 1., 1., 1.], Some(-1e16)),
    ];
    assert_eq!(
        candle_buckets(&cancel, Rect::new(0., 0., 2., 1.).unwrap(), 1., 2).unwrap()[0].volume,
        Some(1.)
    );
    let bad = [candle(1, 1., 0.1, [2., 1., 0., 0.], None)];
    assert!(candle_buckets(&bad, Rect::new(0., 0., 2., 1.).unwrap(), 1., 2).is_err());
}

use chart_core::{
    data::*, grammar::*, layout::*, services::*, state::ChartState, transaction::DataStore,
};
use std::sync::Arc;
struct Measure;
impl TextMeasurer for Measure {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn laid_out(rows: Vec<(f64, Option<f64>)>) -> Arc<LaidOutChart> {
    let typed = TypedRows::snapshot(
        DatasetId::new(1),
        Revision::new(1),
        (1..=rows.len() as u64).map(RowKey::new).collect(),
        rows,
        10_000,
    )
    .unwrap();
    let batch = TypedDataBuilder::new(typed.get().unwrap(), SchemaVersion::new(1))
        .float(FieldId::new(1), "x", |r| Some(r.0))
        .float(FieldId::new(2), "y", |r| r.1)
        .finish(DataLimits::default())
        .unwrap();
    let store = DataStore::new(
        SourceEpoch::new(1),
        vec![(DatasetId::new(1), batch)],
        DataLimits::default(),
    )
    .unwrap();
    let definition = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        DatasetId::new(1),
        Geom::Line {
            order: LineOrder::X,
            connect_gaps: false,
        },
        SourceAes::new().x(FieldId::new(1)).y(FieldId::new(2)),
    ));
    let prepared = Compiler::new()
        .prepare(
            &definition,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    let request = LayoutRequest::new(
        Rect::new(0., 0., 300., 200.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::new(1),
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    Arc::new(layout(Arc::new(prepared), &request, &Measure).unwrap())
}
#[test]
fn reduced_scene_preserves_gap_boundaries_and_exact_raw_lookup_for_discarded_vertices() {
    use chart_core::scene::{PathCommand, Primitive};
    let raw = laid_out(
        (0..1000)
            .map(|i| {
                (
                    i as f64,
                    if i == 500 {
                        None
                    } else {
                        Some(if i == 333 { 100. } else { (i % 7) as f64 })
                    },
                )
            })
            .collect(),
    );
    let before = raw.scene().items().to_vec();
    let dense = DenseChart::prepare(
        raw.clone(),
        &DensityOptions {
            line_bucket_width: Some(10.),
            ..Default::default()
        },
        Limits::default(),
    )
    .unwrap();
    assert!(Arc::ptr_eq(dense.source(), &raw));
    assert_eq!(raw.scene().items(), before);
    assert_eq!(dense.metrics().source_rows, 1000);
    assert_eq!(dense.metrics().prepared_rows, 1000);
    assert_eq!(dense.metrics().represented_samples, 999);
    assert_eq!(dense.metrics().line_runs, 2);
    assert!(dense.metrics().rendered_vertices < 150);
    let paths: Vec<_> = dense
        .scene()
        .items()
        .iter()
        .filter_map(|i| {
            if i.layer.is_some() {
                if let Primitive::Path { commands, .. } = &i.primitive {
                    Some(commands)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|p| matches!(p[0], PathCommand::MoveTo(_))));
    let inspector =
        chart_core::inspection::Inspector::new(dense.source().clone(), 10., 32).unwrap();
    assert_eq!(inspector.semantic_targets().count(), 999);
    let discarded = inspector
        .semantic_targets()
        .find(|h| h.target == target(124))
        .unwrap();
    let descriptor = inspector
        .describe_target(discarded, &ChartState::default())
        .unwrap();
    assert!(descriptor.cells.iter().any(|c| c.value == "123"));
    assert_eq!(
        raw.prepared()
            .source()
            .get()
            .unwrap()
            .dataset(DatasetId::new(1))
            .unwrap()
            .lookup_entries(),
        1000
    );
}
