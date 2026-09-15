//! Portable break operations shared by the independent host proofs.
use chart_core::{
    ChartResult, DiagnosticCode, Revision, grammar::*, interpolate::Number, scales::ScaleKey,
};
use std::sync::Arc;
/// Install explicit native implementations; no source strings are evaluated.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    for signature in ["limits", "n", "n.breaks", "minor_one", "minor_two"] {
        registry.register_scale_breaks(Arc::new(Breaks(signature)))?;
    }
    registry.register_scale_breaks(Arc::new(DiscreteBreaks(false)))?;
    registry.register_scale_breaks(Arc::new(DiscreteBreaks(true)))?;
    Ok(())
}
/// Pure break function used by the installed proof hosts and recording tests.
pub struct Breaks(
    /// Reference signature selected by the fixture, such as `limits` or `n`.
    pub &'static str,
);
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Domain,
    Numeric,
    Majors,
    Fixed,
    Outside,
    Mixed,
    MixedUnnamed,
    Empty,
    UntypedEmpty,
    Null,
}
fn mode(value: &serde_json::Value) -> ChartResult<Mode> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
impl CustomScaleBreaks for Breaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(format!("example.breaks_{}", self.0), Revision::new(1), true)
    }
    fn accepts_n(&self) -> bool {
        self.0 == "n"
    }
    fn accepts_n_breaks(&self) -> bool {
        self.0 == "n.breaks"
    }
    fn accepts_major_breaks(&self) -> bool {
        self.0 == "minor_two"
    }
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()> {
        mode(parameters).map(|_| ())
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        let d = input
            .domain
            .iter()
            .map(|v| match v {
                ScaleKey::Number(n) => Ok(n.0),
                _ => Err(super::error(
                    DiagnosticCode::Validation,
                    "Numeric break domain required.",
                )),
            })
            .collect::<ChartResult<Vec<_>>>()?;
        if input.domain_is_null && matches!(mode(input.parameters)?, Mode::Mixed) {
            return Err(super::error(
                DiagnosticCode::NumericalDomain,
                "Named mixed breaks cannot assign seven names to the shorter NULL-domain result.",
            ));
        }
        let (values, names) = match mode(input.parameters)? {
            Mode::Domain => ((!input.domain_is_null).then_some(d), None),
            Mode::Numeric => (Some(d), None),
            Mode::Outside => (
                Some(vec![
                    d[1],
                    d.iter().sum::<f64>() / d.len() as f64,
                    d[0],
                    d[0],
                    -100.,
                    100.,
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ]),
                None,
            ),
            Mode::Fixed => (
                Some(vec![
                    10.,
                    5.,
                    1.,
                    1.,
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ]),
                None,
            ),
            Mode::Majors => (
                input.major_breaks.map(|v| {
                    v.iter()
                        .map(|k| {
                            let ScaleKey::Number(n) = k else {
                                unreachable!()
                            };
                            n.0
                        })
                        .collect()
                }),
                None,
            ),
            Mode::MixedUnnamed => (
                Some(if input.domain_is_null {
                    vec![f64::NAN, f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
                } else {
                    vec![
                        d.get(1).copied().unwrap_or(f64::NAN),
                        d.iter().sum::<f64>() / d.len() as f64,
                        d.first().copied().unwrap_or(f64::NAN),
                        d.first().copied().unwrap_or(f64::NAN),
                        f64::NAN,
                        f64::INFINITY,
                        f64::NEG_INFINITY,
                    ]
                }),
                None,
            ),
            Mode::Mixed => (
                Some(vec![
                    d.get(1).copied().unwrap_or(f64::NAN),
                    d.iter().sum::<f64>() / d.len() as f64,
                    d.first().copied().unwrap_or(f64::NAN),
                    d.first().copied().unwrap_or(f64::NAN),
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ]),
                Some(
                    [
                        "last", "middle", "first", "again", "missing", "positive", "negative",
                    ]
                    .map(String::from)
                    .to_vec(),
                ),
            ),
            Mode::Empty | Mode::UntypedEmpty => (Some(vec![]), None),
            Mode::Null => (None, None),
        };
        Ok(ScaleBreaksOutput {
            temporal: if matches!(
                mode(input.parameters)?,
                Mode::Numeric | Mode::Null | Mode::UntypedEmpty
            ) {
                None
            } else {
                input.temporal.map(|t| t.normalization)
            },
            values: values.map(|v| v.into_iter().map(|n| ScaleKey::Number(Number(n))).collect()),
            names,
        })
    }
}

/// Portable discrete break selection; true selects the positional example domain.
pub struct DiscreteBreaks(pub bool);
impl CustomScaleBreaks for DiscreteBreaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.0 {
                "example.positional_discrete_breaks"
            } else {
                "example.discrete_breaks"
            },
            Revision::new(1),
            true,
        )
    }
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()> {
        if matches!(
            parameters.as_str(),
            Some("domain" | "mixed" | "numeric" | "empty" | "null" | "named_reverse")
        ) {
            Ok(())
        } else {
            Err(super::error(
                DiagnosticCode::Validation,
                "Unknown discrete break mode.",
            ))
        }
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        self.validate(input.parameters)?;
        let (values, names) = match input.parameters.as_str().unwrap() {
            "domain" => ((!input.domain_is_null).then(|| input.domain.to_vec()), None),
            "named_reverse" => {
                if input.domain_is_null {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Cannot assign names to NULL discrete breaks.",
                    ));
                }
                (
                    Some(input.domain.iter().rev().cloned().collect()),
                    Some(
                        (1..=input.domain.len())
                            .map(|i| format!("key{i}"))
                            .collect(),
                    ),
                )
            }
            "mixed" if self.0 => (
                Some(vec![
                    ScaleKey::Text("outside".into()),
                    ScaleKey::Text("a".into()),
                    ScaleKey::Text("a".into()),
                    ScaleKey::Null,
                    ScaleKey::Text("c".into()),
                ]),
                Some(
                    ["off", "first", "duplicate", "missing", "last"]
                        .map(String::from)
                        .to_vec(),
                ),
            ),
            "mixed" => (
                Some(vec![
                    ScaleKey::Text("z".into()),
                    ScaleKey::Text("b".into()),
                    ScaleKey::Text("b".into()),
                    ScaleKey::Null,
                    ScaleKey::Text("a".into()),
                ]),
                Some(["Z", "B", "B2", "M", "A"].map(String::from).to_vec()),
            ),
            "numeric" => (
                Some(vec![
                    ScaleKey::Number(Number(1.)),
                    ScaleKey::Null,
                    ScaleKey::Number(Number(1.)),
                ]),
                Some(["one", "missing", "again"].map(String::from).to_vec()),
            ),
            "empty" => (Some(vec![]), None),
            "null" => (None, None),
            _ => unreachable!(),
        };
        Ok(ScaleBreaksOutput {
            values,
            names,
            temporal: None,
        })
    }
}
