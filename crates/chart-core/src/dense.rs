//! Optional destination-only reduction. Exact preparation and inspection remain retained.
mod candles;
mod lines;
use crate::grammar::{Geom, PanelKey};
use crate::layout::LaidOutChart;
use crate::scene::{PathCommand, Primitive, Scene, SceneItem};
use crate::{ChartResult, Diagnostic, DiagnosticCode, FieldId, LayerId, Limits, Rect};
pub use candles::{CandleBucket, CandleSample, candle_buckets};
pub use lines::line_envelope;
use std::{collections::BTreeMap, sync::Arc};

/// Explicit screen-density policy, measured in destination units (logical pixels natively).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DensityOptions {
    /// None retains exact line paths; otherwise first/min/max/last per horizontal bucket.
    pub line_bucket_width: Option<f64>,
    /// None retains supplied candles; otherwise aggregate in horizontal display buckets.
    pub candle_bucket_width: Option<f64>,
    /// Optional supplied numeric volume fields by OHLC layer; absent volumes stay absent.
    #[serde(default)]
    pub candle_volume: BTreeMap<LayerId, FieldId>,
    /// Maximum horizontal buckets per panel, 1..65536.
    pub max_columns: usize,
}
impl Default for DensityOptions {
    fn default() -> Self {
        Self {
            line_bucket_width: Some(1.),
            candle_bucket_width: Some(3.),
            candle_volume: BTreeMap::new(),
            max_columns: 4096,
        }
    }
}
impl DensityOptions {
    /// Validate finite positive bucket widths and bounded catalog capacities.
    pub fn validate(&self) -> ChartResult<()> {
        if !(1..=65_536).contains(&self.max_columns)
            || self.candle_volume.len() > 256
            || [self.line_bucket_width, self.candle_bucket_width]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || v <= 0.)
        {
            return Err(error(
                "Density widths must be positive finite values, with 1..65536 columns and at most 256 volume bindings.",
            ));
        }
        Ok(())
    }
}
/// Counts distinguish retained source, exact semantic preparation and reduced paint.
#[derive(serde::Serialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct DensityMetrics {
    /// Unique retained datasets summed once; this includes rows outside the viewport.
    pub source_rows: usize,
    /// Exact prepared layer rows, including intentional repeated/broadcast uses.
    pub prepared_rows: usize,
    /// Exact input mark vertices before paint reduction.
    pub raw_vertices: usize,
    /// Reduced mark vertices; guide/furniture text is excluded.
    pub rendered_vertices: usize,
    /// Raw samples represented by reduced line paths and candle buckets.
    pub represented_samples: usize,
    /// Original paint item count, including guides.
    pub raw_items: usize,
    /// Reduced paint item count, including guides.
    pub rendered_items: usize,
    /// Reduced monotonic straight line runs.
    pub line_runs: usize,
    /// Nonmonotonic/curved line paths retained exactly.
    pub line_fallbacks: usize,
    /// Rendered aggregate candle buckets.
    pub candle_buckets: usize,
    /// Candle layers deliberately retained exactly due to incompatible mapping/extension/inset.
    pub candle_fallbacks: usize,
}
/// A reduced paint scene beside its exact immutable inspection/publication source.
pub struct DenseChart {
    source: Arc<LaidOutChart>,
    scene: Scene,
    metrics: DensityMetrics,
    candles: Vec<CandleBucket>,
}
impl DenseChart {
    /// Reduce only supported paint geometry. All exact targets/index inputs remain in source.
    pub fn prepare(
        source: Arc<LaidOutChart>,
        options: &DensityOptions,
        limits: Limits,
    ) -> ChartResult<Self> {
        options.validate()?;
        let raw = source.scene();
        let mut metrics = DensityMetrics {
            source_rows: source
                .prepared()
                .source()
                .get()?
                .datasets()
                .map(|d| d.len())
                .sum(),
            prepared_rows: source
                .prepared()
                .layers()
                .iter()
                .map(|l| l.table().rows().len())
                .sum(),
            raw_items: raw.items().len(),
            ..Default::default()
        };
        let mut replacements: BTreeMap<(Option<PanelKey>, LayerId), Vec<SceneItem>> =
            BTreeMap::new();
        let mut candles = vec![];
        if let Some(width) = options.candle_bucket_width
            && source.insets().is_empty()
        {
            if source.panels().is_empty() {
                candles::prepare(
                    &source,
                    None,
                    width,
                    options,
                    &mut replacements,
                    &mut candles,
                    &mut metrics,
                )?;
            } else {
                for panel in source.panels() {
                    candles::prepare(
                        &panel.chart,
                        Some(panel.key.clone()),
                        width,
                        options,
                        &mut replacements,
                        &mut candles,
                        &mut metrics,
                    )?;
                }
            }
        }
        if options.candle_bucket_width.is_some() && !source.insets().is_empty() {
            metrics.candle_fallbacks += source
                .prepared()
                .layers()
                .iter()
                .filter(|l| {
                    l.visible()
                        && source
                            .prepared()
                            .definition()
                            .layers
                            .iter()
                            .any(|a| a.id == l.id() && matches!(a.geom, Geom::Ohlc { .. }))
                })
                .count();
        }
        let mut items = vec![];
        let mut emitted = std::collections::BTreeSet::new();
        for (i, item) in raw.items().iter().enumerate() {
            if item.layer.is_some() {
                metrics.raw_vertices += vertices(&item.primitive);
            }
            if let Some(layer) = item.layer {
                let key = (source.item_panels()[i].clone(), layer);
                if let Some(reduced) = replacements.get(&key) {
                    if emitted.insert(key) {
                        items.extend(reduced.iter().cloned());
                    }
                    continue;
                }
            }
            let mut next = item.clone();
            let line = item.layer.is_some_and(|id| {
                source.prepared().definition().layers.iter().any(|l| {
                    l.id == id
                        && matches!(l.geom, Geom::Line { .. })
                        && l.geometry_extension.is_none()
                })
            });
            if line && let Some(width) = options.line_bucket_width {
                match &item.primitive {
                    Primitive::Path { commands, stroke } => {
                        if let Some((commands, samples, runs)) = lines::reduce_path(
                            commands,
                            item.clip.unwrap_or(raw.bounds()),
                            width,
                            options.max_columns,
                        )? {
                            next.primitive = Primitive::Path {
                                commands,
                                stroke: *stroke,
                            };
                            metrics.represented_samples += samples;
                            metrics.line_runs += runs;
                        } else {
                            metrics.line_fallbacks += 1;
                        }
                    }
                    _ => metrics.line_fallbacks += 1,
                }
            }
            items.push(next);
        }
        metrics.rendered_items = items.len();
        metrics.rendered_vertices = items
            .iter()
            .filter(|i| i.layer.is_some())
            .map(|i| vertices(&i.primitive))
            .sum();
        let scene = Scene::new(
            raw.stamp(),
            raw.units(),
            raw.bounds(),
            &items,
            raw.resources(),
            limits,
        )?;
        Ok(Self {
            source,
            scene,
            metrics,
            candles,
        })
    }
    /// Exact source scene for inspection, source lookup and fresh publication preparation.
    pub fn source(&self) -> &Arc<LaidOutChart> {
        &self.source
    }
    /// Destination-only paint scene; it deliberately does not claim representative row targets.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    /// Work/representation counts, independent of timing or RSS.
    pub fn metrics(&self) -> &DensityMetrics {
        &self.metrics
    }
    /// Exact declared bucket values and complete represented source targets.
    pub fn candles(&self) -> &[CandleBucket] {
        &self.candles
    }
}
fn vertices(p: &Primitive) -> usize {
    match p {
        Primitive::Path { commands, .. }
        | Primitive::FilledPath { commands, .. }
        | Primitive::DashedPath { commands, .. } => commands
            .iter()
            .map(|c| match c {
                PathCommand::MoveTo(_) | PathCommand::LineTo(_) => 1,
                PathCommand::QuadraticTo(..) => 2,
                PathCommand::CubicTo(..) => 3,
                PathCommand::Close => 0,
            })
            .sum(),
        Primitive::Point { .. } | Primitive::Symbol { .. } => 1,
        Primitive::Rule { .. } => 2,
        Primitive::Rectangle { .. } | Primitive::GradientRectangle { .. } => 4,
        _ => 0,
    }
}
pub(crate) fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use explicit finite destination buckets and retained exact source geometry.",
    )
}
pub(crate) fn column(x: f64, view: Rect, width: f64, max: usize) -> ChartResult<i64> {
    let count = (view.width() / width).ceil();
    if !count.is_finite() || count > max as f64 {
        return Err(error(
            "Density policy exceeds its declared horizontal column budget.",
        ));
    }
    Ok(((x - view.origin().x()) / width).floor().clamp(-1., count) as i64)
}
