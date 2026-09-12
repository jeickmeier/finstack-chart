use super::{Baseline, Bounds, ContinuousDomain, ScaleTransform, error};
use crate::{ChartResult, DiagnosticCode};

/// ggplot2 endpoint expansion in the scale's transformed coordinate space.
/// This policy does not transform values or change the trained population.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotExpansion {
    /// Fractions of the signed span at the first and second endpoint.
    pub mult: [f64; 2],
    /// Additive distances at the first and second endpoint; negative values contract.
    /// Datetime axes interpret these distances in seconds.
    pub add: [f64; 2],
}
impl Default for GgplotExpansion {
    fn default() -> Self {
        Self {
            mult: [0.05; 2],
            add: [0.; 2],
        }
    }
}
impl GgplotExpansion {
    /// Default discrete padding, measured in category-index units at both ends.
    pub fn discrete_default() -> Self {
        Self {
            mult: [0.; 2],
            add: [0.6; 2],
        }
    }
    /// Expand category centers (1 through count), retaining wider continuous geometry
    /// extents without expanding those extents again. Empty input falls back to 0–1.
    pub fn discrete_viewport(
        self,
        count: usize,
        continuous: Option<Bounds>,
    ) -> ChartResult<Bounds> {
        let [a, b] = self
            .discrete_position_viewport(
                count,
                continuous.map(|v| {
                    [
                        crate::interpolate::Number(v.start()),
                        crate::interpolate::Number(v.end()),
                    ]
                }),
                None,
            )?
            .map(|v| v.0);
        Bounds::new(a, b)
    }

    /// Retain reference category-index limits without relaxing finite Bounds.
    pub(crate) fn discrete_position_viewport(
        self,
        count: usize,
        continuous: Option<[crate::interpolate::Number; 2]>,
        authored: Option<&[crate::interpolate::Number]>,
    ) -> ChartResult<[crate::interpolate::Number; 2]> {
        self.discrete_mapped_viewport(
            (count > 0).then_some([
                crate::interpolate::Number(1.),
                crate::interpolate::Number(count as f64),
            ]),
            continuous,
            authored,
        )
    }

    pub(crate) fn discrete_mapped_viewport(
        self,
        discrete: Option<[crate::interpolate::Number; 2]>,
        continuous: Option<[crate::interpolate::Number; 2]>,
        authored: Option<&[crate::interpolate::Number]>,
    ) -> ChartResult<[crate::interpolate::Number; 2]> {
        use crate::interpolate::Number;
        if self
            .mult
            .into_iter()
            .chain(self.add)
            .any(|v| !v.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Expansion multipliers and offsets must be finite.",
            ));
        }
        let fallback = discrete.or(continuous).map_or([0., 1.], |v| v.map(|v| v.0));
        let limits = match authored {
            None => fallback,
            Some(values) if values.iter().any(|v| v.0.is_nan()) => {
                if values.len() != 2 {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Continuous category limits containing missing values require two endpoints.",
                    ));
                }
                [
                    if values[0].0.is_nan() {
                        fallback[0]
                    } else {
                        values[0].0
                    },
                    if values[1].0.is_nan() {
                        fallback[1]
                    } else {
                        values[1].0
                    },
                ]
            }
            Some(values) => values
                .iter()
                .fold([f64::INFINITY, f64::NEG_INFINITY], |[a, b], v| {
                    [a.min(v.0), b.max(v.0)]
                }),
        };
        // range() over a palette containing NA produces no expanded discrete range.
        // Only the already trained continuous population survives the reference union.
        if limits.iter().any(|v| v.is_nan()) {
            return continuous
                .map(|v| [Number(v[0].0.min(v[1].0)), Number(v[0].0.max(v[1].0))])
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::NumericalDomain,
                        "A missing positional palette has no finite or continuous panel range.",
                    )
                });
        }
        let [low, high] = [limits[0].min(limits[1]), limits[0].max(limits[1])];
        let mut expanded = if low.is_finite() && high.is_finite() {
            let view = self.expand(Bounds::new(low, high)?)?;
            [view.minimum(), view.maximum()]
        } else {
            [low, high]
        };
        // Discrete and observed numeric-position ranges have separate expansion.
        // A purely continuous contribution is expanded only once.
        if (discrete.is_some() || authored.is_some())
            && let Some(continuous) = continuous
        {
            expanded = [
                expanded[0].min(continuous[0].0.min(continuous[1].0)),
                expanded[1].max(continuous[0].0.max(continuous[1].0)),
            ];
        }
        Ok(expanded.map(Number))
    }

    pub(crate) fn nonlinear_transformed_viewport(
        self,
        contribution: Option<crate::grammar::Extent>,
        options: ContinuousDomain,
        transform: ScaleTransform,
    ) -> ChartResult<Bounds> {
        let forward = |v| {
            transform.forward(v)?.ok_or_else(|| {
                error(
                    DiagnosticCode::NumericalDomain,
                    "Expansion limits must be in the transform domain.",
                )
            })
        };
        let bounds = |b: Bounds| Bounds::new(forward(b.start())?, forward(b.end())?);
        let input = contribution
            .filter(|_| options.explicit.is_none())
            .map(|e| {
                let b = bounds(Bounds::new(e.minimum, e.maximum)?)?;
                Ok(crate::grammar::Extent {
                    minimum: b.minimum(),
                    maximum: b.maximum(),
                })
            })
            .transpose()?;
        let options = ContinuousDomain {
            explicit: options.explicit.map(bounds).transpose()?,
            baseline: if options.explicit.is_some() {
                Baseline::None
            } else {
                match options.baseline {
                    Baseline::None => Baseline::None,
                    Baseline::Zero => Baseline::Value(forward(0.)?),
                    Baseline::Value(v) => Baseline::Value(forward(v)?),
                }
            },
            ..options
        };
        let limits = if options.explicit.is_none() && (options.nice || options.padding != 0.) {
            options.resolve(input)?
        } else {
            options.trained_limits(input)?
        };
        self.continuous_viewport(Bounds::new(limits.minimum(), limits.maximum())?)
    }
    /// Continuous panel ranges sort their expanded endpoints, including when
    /// contraction of a constant domain crosses the endpoints.
    pub fn continuous_viewport(self, limits: Bounds) -> ChartResult<Bounds> {
        let expanded = self.expand(Bounds::new(limits.minimum(), limits.maximum())?)?;
        Bounds::new(expanded.minimum(), expanded.maximum())
    }
    /// Expand finite transformed limits, using a unit span for equal endpoints.
    /// Equal or reversed results are retained; equal viewports map to the panel midpoint.
    pub fn expand(self, limits: Bounds) -> ChartResult<Bounds> {
        if self
            .mult
            .into_iter()
            .chain(self.add)
            .any(|v| !v.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Expansion multipliers and offsets must be finite.",
            ));
        }
        let span = if super::ggplot::zero_range(limits.start(), limits.end()) {
            1.
        } else {
            limits.end() - limits.start()
        };
        if !span.is_finite() {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Expansion span exceeds finite precision.",
            ));
        }
        Bounds::new(
            limits.start() - (span * self.mult[0] + self.add[0]),
            limits.end() + (span * self.mult[1] + self.add[1]),
        )
    }
}

#[cfg(test)]
mod continuous_category_limit_tests {
    use super::*;
    use crate::interpolate::Number;
    #[test]
    fn category_index_ranges_match_reference_authored_limits() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/discrete-continuous-limits.json"
        ))
        .unwrap();
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 756);
        for case in cases {
            if case["limits_name"] == "invalid_type" {
                assert!(serde_json::from_value::<Vec<Number>>(case["limits"].clone()).is_err());
                continue;
            }
            let input = case["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>();
            let labels = case["levels"]
                .as_array()
                .map(|v| v.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>())
                .unwrap_or_else(|| {
                    input
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect()
                });
            let indices = input
                .iter()
                .filter_map(|label| labels.iter().position(|v| v == label));
            let continuous = super::super::spacing::observed_extent(indices, !input.is_empty());
            let expansion = match case["expansion"].as_str().unwrap() {
                "default" => GgplotExpansion::discrete_default(),
                "none" => GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                },
                "asymmetric" => GgplotExpansion {
                    mult: [0.1, 0.2],
                    add: [0.3, 0.7],
                },
                "contract" => GgplotExpansion {
                    mult: [-0.5, -0.25],
                    add: [-0.2, -0.1],
                },
                _ => unreachable!(),
            };
            let number = |v: &serde_json::Value| {
                Number(if let Some(v) = v.as_f64() {
                    v
                } else if let Some(v) = v.as_bool() {
                    if v { 1. } else { 0. }
                } else {
                    match v.as_str() {
                        Some("Infinity") => f64::INFINITY,
                        Some("-Infinity") => f64::NEG_INFINITY,
                        _ => f64::NAN,
                    }
                })
            };
            let limits = case["limits"]
                .as_array()
                .map(|v| v.iter().map(number).collect::<Vec<_>>());
            let actual =
                expansion.discrete_position_viewport(labels.len(), continuous, limits.as_deref());
            if case["result"].get("error").is_some() {
                assert_eq!(actual.unwrap_err().code, DiagnosticCode::SchemaConflict);
                continue;
            }
            let actual = actual.unwrap_or_else(|e| panic!("{e:?}: {case}"));
            for (actual, expected) in actual
                .into_iter()
                .zip(case["result"]["range"].as_array().unwrap())
            {
                let expected = number(expected).0;
                if expected.is_infinite() {
                    assert_eq!(actual.0, expected, "{case}");
                } else {
                    assert!(
                        (actual.0 - expected).abs() < 1e-12,
                        "{actual:?} != {expected}: {case}"
                    );
                }
            }
        }
    }
}
