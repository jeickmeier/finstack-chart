use super::{AesBuilder, Data, Mapping, error};
use crate::{ChartResult, DiagnosticCode, Revision, grammar::*};

#[derive(Clone)]
enum Kind {
    Identity,
    Bin {
        bins: usize,
        edges: Option<Vec<f64>>,
        outliers: OutlierPolicy,
    },
    Count {
        required: Vec<Mapping>,
    },
    Summary {
        quantiles: Vec<f64>,
        empty_sum_zero: bool,
    },
    Fit,
    Custom(OperationRef, serde_json::Value),
}
/// Statistical component using the existing kernels and source/generated-stage contracts.
#[derive(Clone)]
pub struct StatBuilder {
    kind: Kind,
    x: Option<Mapping>,
    y: Option<Mapping>,
    group: Option<Mapping>,
    all_groups: bool,
    space: StatSpace,
    y_space: StatSpace,
    parameter_fields: std::collections::BTreeMap<String, Mapping>,
    failure: Option<crate::Diagnostic>,
}
fn stat(kind: Kind) -> StatBuilder {
    StatBuilder {
        kind,
        x: None,
        y: None,
        group: None,
        all_groups: false,
        space: StatSpace::Data,
        y_space: StatSpace::Data,
        parameter_fields: Default::default(),
        failure: None,
    }
}
/// Preserve source or existing generated rows.
pub fn identity_stat() -> StatBuilder {
    stat(Kind::Identity)
}
/// Equal-width bins by default; explicit edges retain the existing closure/outlier policy.
pub fn bin() -> StatBuilder {
    stat(Kind::Bin {
        bins: 30,
        edges: None,
        outliers: OutlierPolicy::Exclude,
    })
}
/// Count rows satisfying the optional required numeric inputs.
pub fn count() -> StatBuilder {
    stat(Kind::Count { required: vec![] })
}
/// Count/min/max/mean/sum and median over finite source values by default.
pub fn summary() -> StatBuilder {
    stat(Kind::Summary {
        quantiles: vec![0.5],
        empty_sum_zero: false,
    })
}
/// Intercept OLS with two fitted endpoints; does not imply LOESS, GAM or confidence bands.
pub fn fit() -> StatBuilder {
    stat(Kind::Fit)
}
/// Select an explicitly registered statistic and declarative parameters.
pub fn custom_stat(
    id: impl Into<String>,
    version: Revision,
    parameters: serde_json::Value,
) -> StatBuilder {
    stat(Kind::Custom(OperationRef::new(id, version), parameters))
}
impl StatBuilder {
    pub(super) fn explicit_grouping(&self, data: &Data) -> ChartResult<Option<Grouping>> {
        if self.all_groups {
            Ok(Some(Grouping::All))
        } else {
            self.group.as_ref().map(|g| g.grouping(data)).transpose()
        }
    }
    /// Bind one custom-statistic numeric parameter to a checked source mapping at build.
    /// The parameter object receives the canonical Numeric value; opaque IDs are never guessed.
    pub fn field_parameter(mut self, name: impl Into<String>, mapping: impl Into<Mapping>) -> Self {
        let name = name.into();
        if !matches!(self.kind, Kind::Custom(..)) {
            return self.invalid("Field parameters require a registered custom statistic.");
        }
        if let Err(e) = super::validate_name(&name) {
            self.failure = Some(e);
        }
        self.parameter_fields.insert(name, mapping.into());
        self
    }
    /// Override the source input/predictor; otherwise inherit x from the layer mappings.
    pub fn x(mut self, x: impl Into<Mapping>) -> Self {
        self.x = Some(x.into());
        self
    }
    /// Override the OLS response; otherwise inherit y from the layer mappings.
    pub fn y(mut self, y: impl Into<Mapping>) -> Self {
        self.y = Some(y.into());
        self
    }
    /// Override population grouping independently of visual encodings.
    pub fn group(mut self, group: impl Into<Mapping>) -> Self {
        self.group = Some(group.into());
        self.all_groups = false;
        self
    }
    /// Explicitly use one statistical population, overriding inherited source grouping.
    pub fn group_all(mut self) -> Self {
        self.group = None;
        self.all_groups = true;
        self
    }
    /// Set the number of equal-width bins.
    pub fn bins(mut self, count: usize) -> Self {
        if let Kind::Bin { bins, .. } = &mut self.kind {
            *bins = count;
        } else {
            self = self.invalid("Bin count requires the bin statistic.");
        }
        self
    }
    /// Set explicit bin edges.
    pub fn breaks(mut self, values: Vec<f64>) -> Self {
        if let Kind::Bin { edges, .. } = &mut self.kind {
            *edges = Some(values);
        } else {
            self = self.invalid("Breaks require the bin statistic.");
        }
        self
    }
    /// Configure explicit-bin outlier handling.
    pub fn outliers(mut self, policy: OutlierPolicy) -> Self {
        if let Kind::Bin { outliers, .. } = &mut self.kind {
            *outliers = policy;
        } else {
            self = self.invalid("Outlier policy requires the bin statistic.");
        }
        self
    }
    /// Require finite values in these fields when counting.
    pub fn required(mut self, values: impl IntoIterator<Item = impl Into<Mapping>>) -> Self {
        if let Kind::Count { required } = &mut self.kind {
            *required = values.into_iter().map(Into::into).collect();
        } else {
            self = self.invalid("Required fields configure the count statistic.");
        }
        self
    }
    /// Set exact summary quantile probabilities in authored order.
    pub fn quantiles(mut self, values: Vec<f64>) -> Self {
        if let Kind::Summary { quantiles, .. } = &mut self.kind {
            *quantiles = values;
        } else {
            self = self.invalid("Quantiles require the summary statistic.");
        }
        self
    }
    /// Return zero for an empty summary sum only when explicitly requested.
    pub fn empty_sum_zero(mut self, enabled: bool) -> Self {
        if let Kind::Summary { empty_sum_zero, .. } = &mut self.kind {
            *empty_sum_zero = enabled;
        } else {
            self = self.invalid("Empty-sum policy requires the summary statistic.");
        }
        self
    }
    /// Apply an explicit affine pre-stat transform, independent of axis transforms.
    pub fn transform(mut self, factor: f64, offset: f64) -> Self {
        self.space = StatSpace::Transformed(NumericTransform::affine(factor, offset));
        self
    }
    /// Apply an explicit affine OLS response transform.
    pub fn y_transform(mut self, factor: f64, offset: f64) -> Self {
        self.y_space = StatSpace::Transformed(NumericTransform::affine(factor, offset));
        self
    }
    fn invalid(mut self, message: &str) -> Self {
        self.failure = Some(error(DiagnosticCode::UnsupportedCapability, message));
        self
    }
    pub(super) fn lower(&self, data: &Data, aes: &AesBuilder) -> ChartResult<Statistic> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if (self.x.is_some()
            && !matches!(
                self.kind,
                Kind::Bin { .. } | Kind::Summary { .. } | Kind::Fit
            ))
            || (self.y.is_some() && !matches!(self.kind, Kind::Fit))
            || (self.space != StatSpace::Data
                && !matches!(
                    self.kind,
                    Kind::Bin { .. } | Kind::Summary { .. } | Kind::Fit | Kind::Custom(..)
                ))
            || (self.y_space != StatSpace::Data && !matches!(self.kind, Kind::Fit))
            || (matches!(self.kind, Kind::Identity) && (self.group.is_some() || self.all_groups))
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Statistic options do not apply to the selected operation.",
            ));
        }
        if matches!(self.kind, Kind::Bin { edges: None, outliers, .. } if outliers != OutlierPolicy::Exclude)
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Outlier handling requires explicit bin edges.",
            ));
        }
        let group = self
            .explicit_grouping(data)?
            .or(aes.resolved_grouping(data)?)
            .unwrap_or(Grouping::All);
        let x = || {
            self.x
                .as_ref()
                .or(aes.x.as_ref())
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Statistic requires an x/input field.",
                    )
                })?
                .resolve(data)
        };
        Ok(match &self.kind {
            Kind::Identity => Statistic::identity(),
            Kind::Bin {
                bins,
                edges,
                outliers,
            } => {
                if let Some(edges) = edges {
                    Statistic::bin(BinSpec {
                        input: x()?,
                        edges: edges.clone(),
                        outliers: *outliers,
                        grouping: group,
                        space: self.space.clone(),
                    })
                } else {
                    Statistic::auto_bin(AutoBinSpec {
                        input: x()?,
                        bins: *bins,
                        grouping: group,
                        space: self.space.clone(),
                    })
                }
            }
            Kind::Count { required } => Statistic::count(CountSpec {
                required: required
                    .iter()
                    .map(|m| m.resolve(data))
                    .collect::<ChartResult<_>>()?,
                grouping: group,
            }),
            Kind::Summary {
                quantiles,
                empty_sum_zero,
            } => Statistic::summary(SummarySpec {
                input: x()?,
                grouping: group,
                quantiles: quantiles.clone(),
                empty_sum_zero: *empty_sum_zero,
                space: self.space.clone(),
            }),
            Kind::Fit => Statistic::ols(OlsSpec {
                x: x()?,
                y: self
                    .y
                    .as_ref()
                    .or(aes.y.as_ref())
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "OLS requires a y response field.",
                        )
                    })?
                    .resolve(data)?,
                grouping: group,
                x_space: self.space.clone(),
                y_space: self.y_space.clone(),
            }),
            Kind::Custom(op, params) => {
                let mut values = params.clone();
                if !self.parameter_fields.is_empty() {
                    let object = values.as_object_mut().ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "Field parameters require a custom parameter object.",
                        )
                    })?;
                    for (name, mapping) in &self.parameter_fields {
                        object.insert(
                            name.clone(),
                            serde_json::to_value(mapping.resolve(data)?)
                                .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?,
                        );
                    }
                }
                Statistic::custom(
                    op.clone(),
                    ExtensionParameters {
                        values,
                        grouping: group,
                        space: self.space.clone(),
                    },
                )
            }
        })
    }
    pub(super) fn default_mappings(&self, geom: Geom) -> ChartResult<Option<Mappings>> {
        let mut aes = match self.kind {
            Kind::Identity => return Ok(None),
            Kind::Bin { .. } => return Ok(Some(Mappings::Binned(BinAes::histogram()))),
            Kind::Fit => StatAes::new(StatField::X, StatField::Y),
            Kind::Count { .. } => StatAes::new(StatField::Group, StatField::Count),
            Kind::Summary { .. } => StatAes::new(StatField::Group, StatField::Mean),
            Kind::Custom(_, _) => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Custom statistics require explicit generated aes mappings.",
                ));
            }
        };
        if matches!(geom, Geom::Bar { .. }) {
            aes.y2 = Some(StatNumeric::Literal(0.));
        }
        Ok(Some(Mappings::Statistical(aes)))
    }
}
/// Generated statistical mappings; source names/handles cannot enter this stage.
#[derive(Clone, Debug)]
pub struct StatAesBuilder {
    pub(super) aes: StatAes,
    pub(super) color: Option<ColorInput>,
    pub(super) color_scale: Option<String>,
}
/// Begin generated mappings; override X/Y to the statistic's declared output fields.
pub fn stat_aes() -> StatAesBuilder {
    StatAesBuilder {
        aes: StatAes::new(StatField::X, StatField::Y),
        color: None,
        color_scale: None,
    }
}
impl StatAesBuilder {
    /// Map one generated numeric value to an explicitly named continuous color scale.
    pub fn color(mut self, field: StatField) -> Self {
        self.color = Some(ColorInput::Statistical(field));
        self
    }
    /// Select an authored color scale independently of the generated value.
    pub fn color_scale(mut self, name: impl Into<String>) -> Self {
        self.color_scale = Some(name.into());
        self
    }
    /// Map stable generated groups to an explicitly named discrete scale.
    pub fn color_group(mut self, scale: impl Into<String>) -> Self {
        self.color = Some(ColorInput::Group);
        self.color_scale = Some(scale.into());
        self
    }
    /// Generated x field or literal.
    pub fn x(mut self, value: impl Into<StatNumeric>) -> Self {
        self.aes.x = value.into();
        self
    }
    /// Generated y field or literal.
    pub fn y(mut self, value: impl Into<StatNumeric>) -> Self {
        self.aes.y = value.into();
        self
    }
    /// Generated second horizontal endpoint.
    pub fn x2(mut self, value: impl Into<StatNumeric>) -> Self {
        self.aes.x2 = Some(value.into());
        self
    }
    /// Generated second vertical endpoint/baseline.
    pub fn y2(mut self, value: impl Into<StatNumeric>) -> Self {
        self.aes.y2 = Some(value.into());
        self
    }
    /// Generated size in baseline destination units.
    pub fn size(mut self, value: impl Into<StatNumeric>) -> Self {
        self.aes.size = Some(value.into());
        self
    }
}
/// Generated histogram mappings, separate from generic statistical and source fields.
#[derive(Clone, Debug)]
pub struct BinAesBuilder {
    pub(super) aes: BinAes,
    pub(super) color_scale: Option<String>,
}
/// Begin generated histogram endpoint/count mappings.
pub fn bin_aes() -> BinAesBuilder {
    BinAesBuilder {
        aes: BinAes::histogram(),
        color_scale: None,
    }
}
impl BinAesBuilder {
    /// Map stable bin groups to an explicitly named discrete color scale.
    pub fn color_group(mut self, scale: impl Into<String>) -> Self {
        self.color_scale = Some(scale.into());
        self
    }
    /// Generated x coordinate.
    pub fn x(mut self, value: impl Into<BinNumeric>) -> Self {
        self.aes.x = value.into();
        self
    }
    /// Generated y coordinate.
    pub fn y(mut self, value: impl Into<BinNumeric>) -> Self {
        self.aes.y = value.into();
        self
    }
    /// Generated second x endpoint.
    pub fn x2(mut self, value: impl Into<BinNumeric>) -> Self {
        self.aes.x2 = Some(value.into());
        self
    }
    /// Generated second y endpoint.
    pub fn y2(mut self, value: impl Into<BinNumeric>) -> Self {
        self.aes.y2 = Some(value.into());
        self
    }
    /// Generated size.
    pub fn size(mut self, value: impl Into<BinNumeric>) -> Self {
        self.aes.size = Some(value.into());
        self
    }
}
/// Source population filter, separate from display-domain/viewport controls.
#[derive(Clone, Debug)]
pub struct FilterBuilder {
    value: Mapping,
    minimum: Option<f64>,
    maximum: Option<f64>,
}
/// Filter one source numeric input before statistics.
pub fn filter(value: impl Into<Mapping>) -> FilterBuilder {
    FilterBuilder {
        value: value.into(),
        minimum: None,
        maximum: None,
    }
}
impl FilterBuilder {
    pub(super) fn expression(mut self, value: crate::grammar::Expression<Mapping>) -> Self {
        self.value = value.into();
        self
    }
    /// Inclusive lower bound.
    pub fn minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }
    /// Inclusive upper bound.
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }
    pub(super) fn lower(&self, data: &Data) -> ChartResult<SourceFilter> {
        Ok(SourceFilter {
            value: self.value.resolve(data)?,
            minimum: self.minimum,
            maximum: self.maximum,
        })
    }
}
/// Position component using the existing position kernel.
#[derive(Clone, Debug)]
pub struct PositionBuilder {
    pub(super) value: Position,
    failure: Option<crate::Diagnostic>,
}
/// Stack by exact ordered group values; absent/new groups follow the existing validation policy.
pub fn stack(order: Vec<GroupValue>) -> PositionBuilder {
    PositionBuilder {
        value: Position::Stack(StackSpec {
            order,
            normalize: false,
        }),
        failure: None,
    }
}
/// Reference stack with explicit tidy groups, input rank order and missing gaps.
pub fn shape_stack(groups: Vec<GroupValue>) -> PositionBuilder {
    PositionBuilder {
        value: Position::ShapeStack(ShapeStackSpec {
            groups,
            order: crate::shape::StackOrder::None,
            offset: crate::shape::StackOffset::None,
            missing: crate::shape::StackMissing::Gap,
        }),
        failure: None,
    }
}
/// Dodge into exact group slots occupying the full categorical band by default.
pub fn dodge(order: Vec<GroupValue>) -> PositionBuilder {
    PositionBuilder {
        value: Position::Dodge(DodgeSpec { order, width: 1. }),
        failure: None,
    }
}
/// Deterministic keyed jitter with explicit seed and zero initial displacement.
pub fn jitter(seed: u64) -> PositionBuilder {
    PositionBuilder {
        value: Position::Jitter(JitterSpec {
            seed,
            x: 0.,
            y: 0.,
            units: JitterUnits::Data,
        }),
        failure: None,
    }
}
impl PositionBuilder {
    pub(super) fn lower(&self) -> ChartResult<Position> {
        self.failure.clone().map_or(Ok(self.value.clone()), Err)
    }
    /// Reference stacking rank policy, including explicit catalog permutations.
    pub fn stack_order(mut self, order: crate::shape::StackOrder) -> Self {
        if let Position::ShapeStack(s) = &mut self.value {
            s.order = order;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Stack order requires shape_stack.",
            ));
        }
        self
    }
    /// Reference stack baseline/normalization policy.
    pub fn stack_offset(mut self, offset: crate::shape::StackOffset) -> Self {
        if let Position::ShapeStack(s) = &mut self.value {
            s.offset = offset;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Stack offset requires shape_stack.",
            ));
        }
        self
    }
    /// Explicit missing-cell policy for the tidy stack matrix.
    pub fn stack_missing(mut self, missing: crate::shape::StackMissing) -> Self {
        if let Position::ShapeStack(s) = &mut self.value {
            s.missing = missing;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Stack missing policy requires shape_stack.",
            ));
        }
        self
    }
    /// Normalize each sign side for a stack.
    pub fn normalize(mut self, normalize: bool) -> Self {
        if let Position::Stack(spec) = &mut self.value {
            spec.normalize = normalize;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Normalization requires stack.",
            ));
        }
        self
    }
    /// Set the band fraction used by dodge slots.
    pub fn width(mut self, width: f64) -> Self {
        if let Position::Dodge(spec) = &mut self.value {
            spec.width = width;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Width requires dodge.",
            ));
        }
        self
    }
    /// Set jitter half-widths in the selected units.
    pub fn displacement(mut self, x: f64, y: f64) -> Self {
        if let Position::Jitter(spec) = &mut self.value {
            spec.x = x;
            spec.y = y;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Displacement requires jitter.",
            ));
        }
        self
    }
    /// Set jitter calculation/display units.
    pub fn units(mut self, units: JitterUnits) -> Self {
        if let Position::Jitter(spec) = &mut self.value {
            spec.units = units;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Units require jitter.",
            ));
        }
        self
    }
}
