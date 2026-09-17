//! Typed authoring and one staged, dependency-free grammar preparation engine.
//!
//! Output geometry is in declared calculation/data space, not destination pixels.
//! [`crate::layout`] maps this output into destination geometry with axes and text-aware margins.

//! Typed native inputs lower to the same explicit bin/rectangle recipe as field-based layers:
//!
//! ```
//! use chart_core::{ChartResult, DatasetId, FieldId, LayerId, Revision, RowKey,
//!                  SchemaVersion, SourceEpoch};
//! use chart_core::data::{DataLimits, TypedRows};
//! use chart_core::grammar::{BinSpec, ChartDefinition, CompileLimits, Compiler, Layer,
//!                           PreparedRows, TypedDataBuilder};
//! use chart_core::state::ChartState;
//! use chart_core::transaction::DataStore;
//!
//! # fn main() -> ChartResult<()> {
//! let data = DatasetId::new(1);
//! let x = FieldId::new(1);
//! let rows = TypedRows::snapshot(data, Revision::INITIAL,
//!     (1..=4).map(RowKey::new).collect(), vec![0.0, 0.5, 1.0, 2.0], 4)?;
//! let batch = TypedDataBuilder::new(rows.get()?, SchemaVersion::new(1))
//!     .float(x, "value", |value| Some(*value)).finish(DataLimits::default())?;
//! let store = DataStore::new(SourceEpoch::new(1), vec![(data, batch)], DataLimits::default())?;
//! let definition = ChartDefinition::new(Revision::new(1)).layer(
//!     Layer::histogram(LayerId::new(1), data, BinSpec::new(x, vec![0.0, 1.0, 2.0])));
//! let prepared = Compiler::new().prepare(&definition, &store.snapshot(),
//!     &ChartState::default(), CompileLimits::default())?;
//! let PreparedRows::Binned(bins) = prepared.layers()[0].table().rows() else {
//!     panic!("histogram has a generated bin schema");
//! };
//! assert_eq!(bins.iter().map(|bin| bin.count).collect::<Vec<_>>(), vec![2, 2]);
//! assert_eq!(prepared.layers()[0].marks().len(), 2);
//!
//! // Resolve using the destination's exact font and measurement service.
//! use chart_core::layout::{layout, LayoutRequest, LayoutStatus};
//! use chart_core::services::{ResourceDescriptor, ResourceKind, TextMeasurer,
//!                            TextMetrics, TextRequest, Units};
//! struct FixtureMetrics;
//! impl TextMeasurer for FixtureMetrics {
//!     fn measure(&self, text: TextRequest<'_>) -> ChartResult<TextMetrics> {
//!         // Demonstration only: real hosts must shape/paint this exact font and units.
//!         TextMetrics::new(text.text.chars().count() as f64 * 6., 9., 3.)
//!     }
//! }
//! let font = ResourceDescriptor { id: chart_core::ResourceId::new(1),
//!     revision: Revision::new(1), kind: ResourceKind::Font, byte_len: 123 };
//! let request = LayoutRequest::new(chart_core::Rect::new(0., 0., 400., 240.)?,
//!                                  Units::Points, font);
//! let result = layout(std::sync::Arc::new(prepared), &request, &FixtureMetrics)?;
//! assert_eq!(result.status(), LayoutStatus::Ready);
//! assert_eq!(result.targets().len(), result.scene().items().len());
//! # Ok(())
//! # }
//! ```

mod after_scale;
mod ggplot_position;
mod stack_position;
pub use after_scale::{AfterScaleAesthetic, AfterScaleRead, ThemeRead};
pub use ggplot_position::{
    DodgePreserve, GgplotDodgeSpec, GgplotStackSpec, JitterDodgeSpec, NudgeSpec,
};
mod colors;
mod style_channels;
pub use style_channels::{AestheticUnits, LineType, ValueAesthetic};
pub(crate) use style_channels::{reference_linewidth, reference_point_radius};
mod numeric_aesthetics;
mod radial_shapes;
mod shape_encoding;
pub use numeric_aesthetics::{NumericAesthetic, NumericEncoding};
pub use radial_shapes::RadialParameters;
mod compiler;
mod encode;
mod incremental_bins;
mod position_stage;
pub use incremental_bins::StatUpdateMetrics;
pub(crate) mod coordinate_policy;
mod geography;
mod geography_coordinate;
pub use geography_coordinate::{
    GeoAxisLabel, GeoGraticule, GeoLimitsMethod, GeoProjectionSelection, GeographicCoordinate,
};
mod geography_crs;
pub use geography_crs::GeoTransform;
mod geography_geometry;
mod geography_mapproj;
mod geography_mapproj_complex;
mod geography_projection;
mod geography_statistics;
pub use geography::{
    GeoAxisOrder, GeoCrs, GeoFeature, GeoFeatureCollection, GeoGeometry, GeoLayerSpec, GeoLimits,
    GeoOperation, GeoPolygon, GeoPosition,
};
pub use geography_projection::{MapprojMethod, MapprojProjection};
mod definition;
mod expression;
mod expression_stage;
mod extensions;
pub use coordinate_policy::{
    CartesianCoordinate, CoordinateClip, CoordinateLimits, CoordinateReverse, CoordinateSpec,
    RadialAxisPlacement, RadialCoordinate, RadialMode, RadialReverse, ThetaAxis,
    TransformedCoordinate,
};
pub(crate) mod facet_policy;
mod facets;
pub use facet_policy::{
    FacetAxes, FacetDirection, FacetLabelOperation, FacetLabeller, FacetPolicy, FacetSpace,
    FacetStripPosition, FacetSwitch,
};
mod geometry_extensions;
pub(crate) mod orientation;
pub(crate) mod positions;
mod shape_extensions;
pub use orientation::Orientation;
pub(crate) mod guide_extensions;
pub(crate) mod hierarchy;
pub use hierarchy::{
    HierarchyAggregation, HierarchyOrder, HierarchyProjection, HierarchyRadius, HierarchyRecipe,
    HierarchySource, PreparedHierarchy,
};
pub(crate) mod hierarchy_extensions;
pub use hierarchy_extensions::{CustomHierarchyOperation, HierarchyScalar};
pub(crate) mod interpolation_extensions;
mod palette_theme;
pub use interpolation_extensions::{CustomInterpolationFactory, InterpolationInput};
mod prepared;
mod scale_extensions;
pub use guide_extensions::{
    CustomGuideFormatter, FacetLabelContext, GuideFormatInput, GuideLabelsInput,
    GuideTemporalContext,
};
mod scale_stage;
mod semantics;
mod statistical_types;
mod statistics;
mod stats;
mod typed;

pub(crate) use colors::guide_key_population;
pub use colors::{ColorEncoding, ColorInput, PaintAesthetic};
pub use compiler::Compiler;
pub use definition::{
    BinAes, BinField, BinNumeric, BinSpec, ChartDefinition, ClipPolicy, CompileLimits, DataRef,
    Geom, Grouping, Layer, LineOrder, Mappings, Numeric, NumericTransform, OperationRef,
    OutlierPolicy, Position, ScaleBindings, SourceAes, SourceFilter, StatParameters, StatSpace,
    Statistic, Style, TransformDefinition,
};
pub use expression::{
    Expression, ExpressionBinary, ExpressionLimits, ExpressionNode, ExpressionReduce,
    ExpressionType, ExpressionUnary, ExpressionValue, SourceRead,
};
pub use extensions::{
    CustomStat, CustomStatInput, CustomStatOutput, ExtensionDescriptor, ExtensionParameters,
    ExtensionRegistry, extension_input_space,
};
pub use facets::{
    EmptyPanels, FacetLayout, FacetScales, FacetSpec, FacetTarget, PanelKey, PreparedPanel,
    StatScope,
};
pub(crate) use geometry_extensions::native_parameter_size;
pub use geometry_extensions::{
    CandleColors, CustomGeom, CustomGeomInput, CustomMark, GeometryExtension, GeometryInteraction,
    HitGeometry, SelectionPolicy, SemanticValue, validate_native_paint,
};
pub use prepared::{
    BinnedRow, DomainContributions, Extent, GeneratedField, GeneratedKind, GroupNumber, GroupValue,
    OperationRecord, OutputSchema, PopulationCounts, PreparationMetrics, PreparedChart,
    PreparedGeometry, PreparedLayer, PreparedMark, PreparedRows, PreparedTable, PreparedTarget,
    SourceRow, ValueSpace,
};
pub use scale_extensions::{CustomScale, ScaleProviderInput};
pub use scale_stage::{ScaleOob, ScaleProjection, TimestampProjection};
pub use semantics::{
    ExecutionSemantics, GroupPolicy, LayerGrammar, Profile, ScaleStage, TransformGrammar,
};
pub use shape_extensions::{CustomShape, ShapeFamily, ShapeOperation, ShapeProtocol};
pub use statistical_types::{
    AutoBinSpec, CountSpec, DodgeSpec, IncrementalCapabilities, JitterSpec, JitterUnits, OlsSpec,
    ShapeStackSpec, StackSpec, StatAes, StatColumn, StatField, StatNumeric, StatOutlier, StatValue,
    StatisticalRow, SummarySpec,
};
mod ggplot_stats;
pub use ggplot_stats::{BinClosure, BinStatistics, GgplotBinOptions};
pub use typed::TypedDataBuilder;

use crate::{Diagnostic, DiagnosticCode};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the declared mapping, operation, scope or budget before preparing again.",
    )
}

mod legends;
pub use legends::{
    CustomGuidePath, CustomLegend, KeyGlyph, KeyOverrides, LayerLegend, LegendAesthetic,
    LegendOptions, LegendPosition,
};
mod symbols;
pub use symbols::{SymbolEncoding, SymbolLegend, SymbolLegendEntry, SymbolSizeGuide};

mod scale_limit_extensions;
pub use scale_limit_extensions::{CustomScaleLimits, ScaleLimitsInput, ScaleLimitsOperation};

pub(crate) mod scale_break_extensions;
pub use scale_break_extensions::{
    CustomScaleBreaks, ScaleBreaksInput, ScaleBreaksOperation, ScaleBreaksOutput,
};

pub(crate) mod scale_palette_extensions;
pub use scale_palette_extensions::{
    CustomScalePalette, ScalePaletteDomain, ScalePaletteInput, ScalePaletteOperation,
    ScalePaletteOutput,
};

pub(crate) mod scale_vector_extensions;
pub use scale_vector_extensions::{
    CustomScaleVector, ScaleVectorInput, ScaleVectorOperation, ScaleVectorStage,
};

mod positional_vectors;

pub(crate) mod transform_extensions;
pub use transform_extensions::{
    CustomTransformFactory, PointwiseTransform, PreparedTransform, TransformOperation,
    TransformSelection,
};

mod transform_resolution;

mod text_geom;
pub use text_geom::{TextFont, TextGeom, TextSizeUnit};

mod ggplot_summary;
pub use ggplot_summary::{
    GgplotCountOptions, GgplotSummaryOptions, SummaryBins, SummaryFunction, SummaryHelper,
    summary_values,
};

mod row_annotation;
pub use row_annotation::{AnnotationContent, RasterAnnotation, RowAnnotation};

mod ggplot_bin_training;

mod recipe_types;
pub use recipe_types::{
    ArrowEnds, ArrowSpec, BuiltinRecipe, IntervalKind, IntervalPoint, IntervalRecipe,
    IntervalStroke, PreparedRecipe, RecipeAesthetic, ReferenceKind, ReferenceRecipe, StepDirection,
};
mod recipe_emit;
mod recipe_intervals;
mod recipe_marks;
mod recipe_surfaces;
pub use recipe_marks::{
    ColumnRecipe, CountRecipe, CurveRecipe, PreparedMarkRecipe, RugRecipe, SpokeRecipe,
};
pub use recipe_surfaces::{PolygonRecipe, PreparedSurface, RasterRecipe, TileRecipe};

mod stroke_controls;
pub use stroke_controls::{LineEnd, LineJoin};

pub(crate) mod quantile_regression;

mod recipe_distributions;
pub use recipe_distributions::{
    AreaOutline, BoxplotRecipe, DensityRecipe, DotAxis, DotStack, DotplotRecipe,
    PreparedDistribution, SourceBoxOutliers, ViolinRecipe,
};

pub(crate) mod distributions;
pub use distributions::{Bandwidth, DensityControls, DensityKernel};
mod analytic_types;
pub use analytic_types::{
    AnalyticFunction, Connection, DistributionKind, DistributionSpec, DotBinMethod,
    FunctionArgument, UnivariateKind, UnivariateSpec, ViolinScale,
};
mod analytic_functions;
pub use analytic_functions::CustomAnalyticFunction;
mod distribution_stage;
mod univariate_kernels;
mod univariate_stage;

mod model_types;
pub use model_types::{
    LoessFamily, LoessSurface, ModelFamily, ModelMethod, ModelOptions, ModelQuantileSolver,
    ModelSpec, ModelTerm,
};
mod model_gam;
mod model_glm;
mod model_linear;
mod model_loess;
mod model_predict;
mod model_quantile_fn;
mod model_stage;
mod model_symmetric;

mod model_extensions;
pub use model_extensions::{CustomModel, ModelEstimate, ModelInput};

mod spatial;
mod spatial_types;
pub use spatial_types::{ContourLevels, DensityContour, EllipseKind, SpatialKind, SpatialSpec};

mod recipe_models;

mod spatial_stage;

mod key_extensions;
pub use key_extensions::{CustomKeyGlyph, KeyGlyphInput, KeyGlyphOutput, KeyGlyphSelection};

pub(crate) use extensions::parameter_size as extension_parameter_size;

mod guide_drawing;
pub use guide_drawing::{CustomGuideDrawing, GuideDrawingInput, GuideDrawingSelection};

mod facet_extensions;
pub(crate) mod registry;
pub use facet_extensions::{CustomFacet, FacetPlanInput, FacetSelection};

mod coordinate_extensions;
pub use coordinate_extensions::{
    CoordinateSelection, CoordinateTrainInput, CustomCoordinate, TrainedCoordinate,
};
