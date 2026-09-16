use super::{AesBuilder, Data, Mapping, error};
use crate::{ChartResult, DiagnosticCode, Revision, grammar::*};

#[derive(Clone)]
enum Kind {
    Distribution(DistributionKind),
    Univariate(UnivariateKind),
    Identity,
    Bin {
        bins: usize,
        edges: Option<Vec<f64>>,
        outliers: OutlierPolicy,
        options: Option<GgplotBinOptions>,
        weight: Option<Mapping>,
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
    analytic_input: Option<Mapping>,
    analytic_weight: Option<Mapping>,
    analytic_width: Option<f64>,
    kind: Kind,
    reference_count: bool,
    sum_count: bool,
    count_partitions: Vec<Mapping>,
    summary_helper: Option<SummaryHelper>,
    summary_bins: Option<SummaryBins>,
    count_weight: Option<Mapping>,
    count_width: Option<f64>,
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
        analytic_input: None,
        analytic_weight: None,
        analytic_width: None,
        kind,
        reference_count: false,
        sum_count: false,
        count_partitions: vec![],
        summary_helper: None,
        summary_bins: None,
        count_weight: None,
        count_width: None,
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
        options: None,
        weight: None,
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
/// Built-in distribution statistic with explicit estimator controls.
pub fn distribution_stat(kind: DistributionKind) -> StatBuilder {
    stat(Kind::Distribution(kind))
}
/// Built-in univariate statistic with explicit function or interpolation controls.
pub fn univariate_stat(kind: UnivariateKind) -> StatBuilder {
    stat(Kind::Univariate(kind))
}
/// Type-seven hinges, observed whiskers and source outlier membership.
pub fn boxplot_stat() -> StatBuilder {
    distribution_stat(DistributionKind::Boxplot {
        coefficient: 1.5,
        quantile_type: 7,
    })
}
/// Gaussian FFT density with the reference default bandwidth selector.
pub fn density_stat() -> StatBuilder {
    distribution_stat(DistributionKind::Density {
        trim: false,
        controls: DensityControls::default(),
    })
}
/// Trimmed area-normalized violin density and inserted quantile rows.
pub fn violin_stat() -> StatBuilder {
    distribution_stat(DistributionKind::Violin {
        controls: DensityControls::default(),
        trim: true,
        scale: ViolinScale::Area,
        quantiles: vec![0.25, 0.5, 0.75],
        drop: true,
    })
}
/// Successive dot-density bins along the x axis.
pub fn dotplot_stat() -> StatBuilder {
    distribution_stat(DistributionKind::Dotplot {
        bin_axis: DotAxis::X,
        method: DotBinMethod::DotDensity,
        bin_width: None,
        bin_positions_all: false,
        closed: BinClosure::Right,
        origin: None,
    })
}
/// Empirical distribution with reference infinite endpoint padding.
pub fn ecdf_stat() -> StatBuilder {
    univariate_stat(UnivariateKind::Ecdf { n: None, pad: true })
}
/// Theoretical normal quantiles against sorted sample values.
pub fn qq_stat() -> StatBuilder {
    univariate_stat(UnivariateKind::Qq {
        distribution: AnalyticFunction::default(),
        quantiles: None,
        line: false,
        probabilities: [0.25, 0.75],
        full_range: false,
    })
}
/// Reference line through the sample's first and third quartiles.
pub fn qq_line_stat() -> StatBuilder {
    univariate_stat(UnivariateKind::Qq {
        distribution: AnalyticFunction::default(),
        quantiles: None,
        line: true,
        probabilities: [0.25, 0.75],
        full_range: false,
    })
}
/// Sample one pure function at 101 transformed x coordinates.
pub fn function_stat(function: AnalyticFunction) -> StatBuilder {
    univariate_stat(UnivariateKind::Function {
        function,
        n: 101,
        range: None,
    })
}
/// Retain one source observation for each distinct mapped tuple.
pub fn unique_stat() -> StatBuilder {
    univariate_stat(UnivariateKind::Unique)
}
/// Connect sorted observations using a named step or relative coordinate matrix.
pub fn connect_stat(connection: Connection) -> StatBuilder {
    univariate_stat(UnivariateKind::Connect { connection })
}
/// Align group curves and zero crossings before area positioning.
pub fn align_stat() -> StatBuilder {
    univariate_stat(UnivariateKind::Align)
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
    /// Set the sample input independently of x/y geometry mappings.
    pub fn input(mut self, mapping: impl Into<Mapping>) -> Self {
        self.analytic_input = Some(mapping.into());
        self
    }
    /// Bind observation weights for an analytical statistic.
    pub fn weight(mut self, mapping: impl Into<Mapping>) -> Self {
        self.analytic_weight = Some(mapping.into());
        self
    }
    /// Set distribution group width in independent-axis data units.
    pub fn width(mut self, width: f64) -> Self {
        self.analytic_width = Some(width);
        self
    }
    /// Replace the typed distribution controls without changing bound inputs.
    pub fn distribution_options(mut self, options: DistributionKind) -> Self {
        self.kind = Kind::Distribution(options);
        self
    }
    /// Replace the typed univariate controls without changing bound inputs.
    pub fn univariate_options(mut self, options: UnivariateKind) -> Self {
        self.kind = Kind::Univariate(options);
        self
    }
    /// Joint-position StatSum counts, distinct from one-dimensional stat_count.
    pub fn sum_count(mut self) -> Self {
        self = self.ggplot_count();
        self.sum_count = true;
        self
    }
    /// Additional mapped source aesthetics whose distinct values partition StatSum rows.
    pub fn count_partition(mut self, mapping: impl Into<Mapping>) -> Self {
        self.count_partitions.push(mapping.into());
        self
    }
    /// Count distinct positions with reference signed-count/proportion outputs.
    pub fn ggplot_count(mut self) -> Self {
        if !matches!(self.kind, Kind::Count { .. }) {
            return self.invalid("Reference count requires count().");
        }
        self.reference_count = true;
        self
    }
    /// Reference count weights; missing weights contribute zero.
    pub fn count_weight(mut self, mapping: impl Into<Mapping>) -> Self {
        self = self.ggplot_count();
        self.count_weight = Some(mapping.into());
        self
    }
    /// Explicit reference count bar width.
    pub fn count_width(mut self, width: f64) -> Self {
        self = self.ggplot_count();
        self.count_width = Some(width);
        self
    }
    /// Summarize responses using a typed reference helper; x/y select predictor/response.
    pub fn summary_helper(mut self, helper: SummaryHelper) -> Self {
        if !matches!(self.kind, Kind::Summary { .. }) {
            return self.invalid("Summary helper requires summary().");
        }
        self.summary_helper = Some(helper);
        self
    }
    /// Partition the summary predictor using the shared reference bin arithmetic.
    pub fn summary_bins(mut self, bins: SummaryBins) -> Self {
        if !matches!(self.kind, Kind::Summary { .. }) {
            return self.invalid("Summary bins require summary().");
        }
        self.summary_helper.get_or_insert_with(Default::default);
        self.summary_bins = Some(bins);
        self
    }

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
    /// Author reference bin closure, alignment, padding and numeric source weights.
    pub fn ggplot_bin(mut self, options: GgplotBinOptions) -> Self {
        if let Kind::Bin {
            options: target, ..
        } = &mut self.kind
        {
            *target = Some(options);
        } else {
            return self.invalid("Reference bin options require a bin statistic.");
        }
        self
    }
    /// Bind a reference histogram weight through the shared source mapping resolver.
    pub fn bin_weight(mut self, weight: impl Into<Mapping>) -> Self {
        if let Kind::Bin { weight: target, .. } = &mut self.kind {
            *target = Some(weight.into());
        } else {
            return self.invalid("Bin weights require a bin statistic.");
        }
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
        if ((self.analytic_input.is_some()
            || self.analytic_weight.is_some()
            || self.analytic_width.is_some())
            && !matches!(self.kind, Kind::Distribution(_) | Kind::Univariate(_)))
            || (self.analytic_width.is_some() && matches!(self.kind, Kind::Univariate(_)))
            || (self.x.is_some()
                && !self.reference_count
                && !matches!(
                    self.kind,
                    Kind::Bin { .. }
                        | Kind::Summary { .. }
                        | Kind::Fit
                        | Kind::Distribution(_)
                        | Kind::Univariate(_)
                ))
            || (self.y.is_some()
                && !self.sum_count
                && self.summary_helper.is_none()
                && !matches!(
                    self.kind,
                    Kind::Fit | Kind::Distribution(_) | Kind::Univariate(_)
                ))
            || (self.space != StatSpace::Data
                && !matches!(
                    self.kind,
                    Kind::Bin { .. }
                        | Kind::Summary { .. }
                        | Kind::Fit
                        | Kind::Custom(..)
                        | Kind::Distribution(_)
                        | Kind::Univariate(_)
                ))
            || (self.y_space != StatSpace::Data
                && !matches!(
                    self.kind,
                    Kind::Fit | Kind::Distribution(_) | Kind::Univariate(_)
                ))
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
            Kind::Distribution(kind) => {
                let sample_y = !matches!(
                    kind,
                    DistributionKind::Density { .. }
                        | DistributionKind::Dotplot {
                            bin_axis: DotAxis::X,
                            ..
                        }
                );
                let primary = if sample_y {
                    self.y.as_ref().or(aes.y.as_ref())
                } else {
                    self.x.as_ref().or(aes.x.as_ref())
                };
                let position = if sample_y {
                    self.x.as_ref().or(aes.x.as_ref())
                } else {
                    self.y.as_ref().or(aes.y.as_ref())
                };
                let input = self
                    .analytic_input
                    .as_ref()
                    .or(primary)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "Distribution statistic requires a sample input.",
                        )
                    })?
                    .resolve(data)?;
                Statistic::distribution(DistributionSpec {
                    retained_fields: vec![],
                    retained_numeric: vec![],
                    training_range: None,
                    input,
                    position: if matches!(kind, DistributionKind::Density { .. }) {
                        None
                    } else {
                        position.map(|p| p.resolve(data)).transpose()?
                    },
                    weight: self
                        .analytic_weight
                        .as_ref()
                        .map(|w| w.resolve(data))
                        .transpose()?,
                    grouping: group,
                    space: self.space.clone(),
                    position_space: self.y_space.clone(),
                    width: self.analytic_width,
                    kind: kind.clone(),
                })
            }
            Kind::Univariate(kind) => {
                let primary = if matches!(kind, UnivariateKind::Qq { .. }) {
                    self.y
                        .as_ref()
                        .or(aes.y.as_ref())
                        .or(self.x.as_ref())
                        .or(aes.x.as_ref())
                } else {
                    self.x.as_ref().or(aes.x.as_ref())
                };
                let input = self
                    .analytic_input
                    .as_ref()
                    .or(primary)
                    .map(|p| p.resolve(data))
                    .transpose()?
                    .or_else(|| {
                        matches!(kind, UnivariateKind::Function { .. })
                            .then(|| Numeric::Expression(Expression::constant(0.)))
                    })
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "Univariate statistic requires a sample input.",
                        )
                    })?;
                let second = if matches!(
                    kind,
                    UnivariateKind::Unique | UnivariateKind::Connect { .. } | UnivariateKind::Align
                ) {
                    self.y
                        .as_ref()
                        .or(aes.y.as_ref())
                        .map(|p| p.resolve(data))
                        .transpose()?
                } else {
                    None
                };
                Statistic::univariate(UnivariateSpec {
                    default_input: matches!(kind, UnivariateKind::Function { .. })
                        && self.analytic_input.as_ref().or(primary).is_none(),
                    training_range: None,
                    output_scale: None,
                    input,
                    second,
                    weight: self
                        .analytic_weight
                        .as_ref()
                        .map(|w| w.resolve(data))
                        .transpose()?,
                    grouping: group,
                    space: self.space.clone(),
                    second_space: self.y_space.clone(),
                    retained_fields: vec![],
                    retained_numeric: vec![],
                    kind: kind.clone(),
                })
            }
            Kind::Identity => Statistic::identity(),
            Kind::Bin {
                bins,
                edges,
                outliers,
                options,
                weight,
            } => {
                let mut ggplot = options.clone();
                if let Some(weight) = weight {
                    ggplot.get_or_insert_with(Default::default).weight =
                        Some(weight.resolve(data)?);
                }
                if let Some(edges) = edges {
                    Statistic::bin(BinSpec {
                        ggplot,
                        input: x()?,
                        edges: edges.clone(),
                        outliers: *outliers,
                        grouping: group,
                        space: self.space.clone(),
                    })
                } else {
                    Statistic::auto_bin(AutoBinSpec {
                        ggplot,
                        input: x()?,
                        bins: *bins,
                        grouping: group,
                        space: self.space.clone(),
                    })
                }
            }
            Kind::Count { required } => Statistic::count(CountSpec {
                ggplot: self
                    .reference_count
                    .then(|| -> ChartResult<_> {
                        Ok(GgplotCountOptions {
                            position: self.x.as_ref().map(|m| m.resolve(data)).transpose()?,
                            joint_position: if self.sum_count {
                                Some(
                                    self.y
                                        .as_ref()
                                        .ok_or_else(|| {
                                            error(DiagnosticCode::Validation, "StatSum requires y.")
                                        })?
                                        .resolve(data)?,
                                )
                            } else {
                                None
                            },
                            joint_numeric: vec![],
                            joint_aesthetics: self
                                .count_partitions
                                .iter()
                                .map(|m| {
                                    m.resolve(data).and_then(|n| match n {
                                        Numeric::Field(id)
                                        | Numeric::Category(id)
                                        | Numeric::Timestamp { field: id, .. } => Ok(id),
                                        _ => Err(error(
                                            DiagnosticCode::Validation,
                                            "Count partitions require source fields.",
                                        )),
                                    })
                                })
                                .collect::<ChartResult<_>>()?,
                            weight: self
                                .count_weight
                                .as_ref()
                                .map(|m| m.resolve(data))
                                .transpose()?,
                            width: self.count_width,
                        })
                    })
                    .transpose()?,
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
                ggplot: self
                    .summary_helper
                    .as_ref()
                    .map(|h| -> ChartResult<_> {
                        Ok(GgplotSummaryOptions {
                            position: self.x.as_ref().map(|m| m.resolve(data)).transpose()?,
                            bins: self.summary_bins.clone(),
                            helper: h.clone(),
                        })
                    })
                    .transpose()?,
                input: if self.summary_helper.is_some() {
                    self.y
                        .as_ref()
                        .or(aes.y.as_ref())
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::SchemaConflict,
                                "Reference summary requires a response y.",
                            )
                        })?
                        .resolve(data)?
                } else {
                    x()?
                },
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
            Kind::Distribution(_) | Kind::Univariate(_) => StatAes::new(StatField::X, StatField::Y),
            Kind::Identity => return Ok(None),
            Kind::Bin { .. } => return Ok(Some(Mappings::Binned(BinAes::histogram()))),
            Kind::Fit => StatAes::new(StatField::X, StatField::Y),
            Kind::Count { .. } => StatAes::new(
                if self.reference_count && self.x.is_some() {
                    StatField::X
                } else {
                    StatField::Group
                },
                if self.sum_count {
                    StatField::Y
                } else if self.reference_count {
                    StatField::WeightedCount
                } else {
                    StatField::Count
                },
            ),
            Kind::Summary { .. } => StatAes::new(
                if self.summary_helper.is_some() && self.x.is_some() {
                    StatField::X
                } else {
                    StatField::Group
                },
                if self.summary_helper.is_some() {
                    StatField::Y
                } else {
                    StatField::Mean
                },
            ),
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
/// Reference stack; descending group order and upper anchor by default.
pub fn ggplot_stack() -> PositionBuilder {
    PositionBuilder {
        value: Position::GgplotStack(GgplotStackSpec::default()),
        failure: None,
    }
}
/// Reference fill, with separate positive and negative normalization.
pub fn ggplot_fill() -> PositionBuilder {
    PositionBuilder {
        value: Position::GgplotStack(GgplotStackSpec {
            fill: true,
            ..Default::default()
        }),
        failure: None,
    }
}
/// Reference dodge; width is in independent-axis calculation units.
pub fn ggplot_dodge() -> PositionBuilder {
    PositionBuilder {
        value: Position::GgplotDodge(GgplotDodgeSpec::default()),
        failure: None,
    }
}
/// Reference variable-width dodge with ten percent padding.
pub fn dodge2() -> PositionBuilder {
    PositionBuilder {
        value: Position::GgplotDodge2(GgplotDodgeSpec::default()),
        failure: None,
    }
}
/// Constant displacement in calculation units.
pub fn nudge(x: f64, y: f64) -> PositionBuilder {
    PositionBuilder {
        value: Position::Nudge(NudgeSpec { x, y }),
        failure: None,
    }
}
/// Reference dodge followed by stable-key FNV-1a/SplitMix64 jitter (not R RNG).
pub fn jitter_dodge(seed: u64) -> PositionBuilder {
    PositionBuilder {
        value: Position::JitterDodge(JitterDodgeSpec {
            resolved_dodge_count: None,
            resolved_resolution: None,
            auto_width: true,
            dodge: GgplotDodgeSpec {
                width: Some(0.75),
                ..Default::default()
            },
            jitter: JitterSpec {
                seed,
                x: 0.4,
                y: 0.,
                units: JitterUnits::Data,
            },
        }),
        failure: None,
    }
}
impl PositionBuilder {
    /// Reference collision width preservation.
    pub fn preserve(mut self, preserve: DodgePreserve) -> Self {
        match &mut self.value {
            Position::GgplotDodge(s) | Position::GgplotDodge2(s) => s.preserve = preserve,
            Position::JitterDodge(s) => s.dodge.preserve = preserve,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Preserve requires reference dodge.",
                ))
            }
        }
        self
    }
    /// Reverse the reference stack or dodge group ordering.
    pub fn reverse(mut self, reverse: bool) -> Self {
        match &mut self.value {
            Position::GgplotDodge(s) | Position::GgplotDodge2(s) => s.reverse = reverse,
            Position::JitterDodge(s) => s.dodge.reverse = reverse,
            Position::GgplotStack(s) => s.reverse = reverse,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Reverse requires reference stack/dodge.",
                ))
            }
        }
        self
    }
    /// Set reference point/text anchor within a stack interval.
    pub fn vjust(mut self, vjust: f64) -> Self {
        if let Position::GgplotStack(s) = &mut self.value {
            s.vjust = vjust;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Vjust requires reference stack/fill.",
            ));
        }
        self
    }
    /// Fraction removed from the width of colliding dodge2 intervals.
    pub fn padding(mut self, padding: f64) -> Self {
        if let Position::GgplotDodge2(s) = &mut self.value {
            s.padding = padding;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Position padding requires dodge2.",
            ));
        }
        self
    }
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
        } else if let Position::GgplotDodge(spec) | Position::GgplotDodge2(spec) = &mut self.value {
            spec.width = Some(width);
        } else if let Position::JitterDodge(spec) = &mut self.value {
            spec.dodge.width = Some(width);
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
        } else if let Position::JitterDodge(spec) = &mut self.value {
            spec.jitter.x = x;
            spec.jitter.y = y;
            spec.auto_width = false;
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
