//! WP-16 reproducible index-versus-scan comparison; not the WP-22 release workload gate.
use chart_core::{
    data::*, grammar::*, inspection::*, layout::*, scales::*, services::*, state::*,
    transaction::DataStore, *,
};
use std::{sync::Arc, time::Instant};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut results = vec![];
    for line in [false, true] {
        let n = 100_000;
        let d = DatasetId::new(1);
        let x = FieldId::new(1);
        let y = FieldId::new(2);
        let rows = TypedRows::snapshot(
            d,
            Revision::INITIAL,
            (0..n).map(|i| RowKey::new(i as u64)).collect(),
            (0..n).collect(),
            n,
        )?;
        let batch = TypedDataBuilder::new(rows.get()?, SchemaVersion::new(1))
            .float(x, "x", |i| {
                Some(if line { *i as f64 } else { (*i % 1000) as f64 })
            })
            .float(y, "y", |i| Some((*i / 1000) as f64))
            .finish(DataLimits::default())?;
        let store = DataStore::new(SourceEpoch::new(1), vec![(d, batch)], DataLimits::default())?;
        let def = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
            LayerId::new(1),
            d,
            if line { Geom::line() } else { Geom::Point },
            SourceAes::new().x(x).y(y),
        ));
        let mut compiler = Compiler::new();
        let prepared = Arc::new(compiler.prepare(
            &def,
            &store.snapshot(),
            &ChartState::default(),
            CompileLimits::default(),
        )?);
        let mut r = LayoutRequest::new(
            Rect::new(0., 0., 1000., 500.)?,
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        r.padding = 0.;
        for a in &mut r.axes {
            a.visible = false;
            a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(
                0.,
                if a.side.horizontal() {
                    if line { n as f64 } else { 1000. }
                } else {
                    100.
                },
            )?));
        }
        let chart = Arc::new(layout(prepared.clone(), &r, &Metrics)?);
        let started = Instant::now();
        let inspector = Inspector::new(chart.clone(), 3., 16)?;
        let build_ns = started.elapsed().as_nanos();
        let points: Vec<_> = (0..512)
            .map(|i| Point::new(f64::from((i * 127) % 1000), f64::from((i * 43) % 500)).unwrap())
            .collect();
        let started = Instant::now();
        let indexed: Vec<_> = points
            .iter()
            .map(|p| inspector.query(*p, InspectionMode::Auto))
            .collect();
        let index_ns = started.elapsed().as_nanos();
        let started = Instant::now();
        let scanned: Vec<_> = points
            .iter()
            .map(|p| inspector.query_scan(*p, InspectionMode::Auto))
            .collect();
        let scan_ns = started.elapsed().as_nanos();
        for (a, b) in indexed.iter().zip(&scanned) {
            assert_eq!(a.hits, b.hits);
        }
        assert!(Arc::ptr_eq(&prepared, inspector.presented().prepared()));
        let hits = indexed
            .iter()
            .find(|r| !r.hits.is_empty())
            .unwrap()
            .hits
            .iter()
            .map(|h| MarkTarget::from_inspected(h, SourceEpoch::new(1)))
            .collect::<Vec<_>>();
        let mut reducer = ActionReducer::default();
        let started = Instant::now();
        reducer.present(chart);
        let target_index_ns = started.elapsed().as_nanos();
        let started = Instant::now();
        for _ in 0..512 {
            let request = reducer.request(
                &def,
                ChartAction::SetHover(hits.clone()),
                ActionOrigin::Pointer,
            );
            reducer.dispatch(&def, request)?;
        }
        let reducer_ns = started.elapsed().as_nanos();
        results.push(serde_json::json!({"geometry":if line{"line"}else{"scatter"},"candidates":inspector.candidate_count(),"queries":points.len(),"index_build_ns":build_ns,"index_query_ns":index_ns,"scan_query_ns":scan_ns,"indexed_examined":indexed.iter().map(|r|r.examined).sum::<usize>(),"scan_examined":scanned.iter().map(|r|r.examined).sum::<usize>(),"target_index_build_ns":target_index_ns,"reducer_hover_ns":reducer_ns,"exact_hits_match":true,"prepared_arc_unchanged":true}));
    }
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
