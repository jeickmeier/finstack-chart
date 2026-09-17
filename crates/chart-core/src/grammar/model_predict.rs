//! Shared model-to-prediction adapter; population ownership remains in the statistic stage.
use super::*;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Severity};
#[derive(Clone, Debug)]
pub(crate) struct Prediction {
    pub x: f64,
    pub mean: Option<f64>,
    pub se: Option<f64>,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub quantile: Option<f64>,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ModelPredictionOutput {
    pub rows: Vec<Prediction>,
    pub diagnostics: Vec<Diagnostic>,
}
fn warning(output: &mut ModelPredictionOutput, message: &str) {
    let mut diagnostic = error(DiagnosticCode::NumericalDomain, message);
    diagnostic.severity = Severity::Warning;
    output.diagnostics.push(diagnostic);
}
fn finite(x: f64) -> Option<f64> {
    x.is_finite().then_some(x)
}
fn matrix(
    input: &[f64],
    terms: Option<&[ModelTerm]>,
    budget: usize,
) -> ChartResult<(Vec<f64>, usize)> {
    let defaults = [ModelTerm::Power(0), ModelTerm::Power(1)];
    let terms = terms.unwrap_or(&defaults);
    let p = terms.len();
    if p == 0 || p > 64 || input.len().checked_mul(p).is_none_or(|n| n > budget) {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Model design exceeds its term or allocation budget.",
        ));
    }
    let columns = terms
        .iter()
        .map(|term| match term {
            ModelTerm::Power(power) => Ok(input
                .iter()
                .map(|v| Some(libm::pow(*v, f64::from(*power))))
                .collect::<Vec<_>>()),
            ModelTerm::Expression(expression) => Ok(expression
                .evaluate(
                    input.len(),
                    ExpressionLimits::default(),
                    |_| Ok(ExpressionType::Number),
                    |_, i| ExpressionValue::Number(input[i]),
                )?
                .iter()
                .map(ExpressionValue::number)
                .collect()),
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let mut result = Vec::with_capacity(input.len() * p);
    for i in 0..input.len() {
        for col in &columns {
            result.push(col[i].filter(|v| v.is_finite()).ok_or_else(|| {
                error(
                    DiagnosticCode::NumericalDomain,
                    "Model term evaluated to a missing or nonfinite value.",
                )
            })?);
        }
    }
    Ok((result, p))
}
fn append(
    output: &mut ModelPredictionOutput,
    x: f64,
    mean: Option<f64>,
    se: Option<f64>,
    critical: f64,
    quantile: Option<f64>,
) {
    let (lower, upper) = match (mean, se) {
        (Some(mean), Some(se)) => (finite(mean - critical * se), finite(mean + critical * se)),
        _ => (None, None),
    };
    output.rows.push(Prediction {
        x,
        mean,
        se,
        lower,
        upper,
        quantile,
    });
}
pub(crate) fn validate(options: &ModelOptions, limits: CompileLimits) -> ChartResult<()> {
    let invalid = || error(DiagnosticCode::NumericalDomain, "Invalid model controls.");
    if !options.level.is_finite() || !(0. ..1.).contains(&options.level) || options.level == 0. {
        return Err(invalid());
    }
    let rows = options.xseq.as_ref().map_or(options.n, Vec::len);
    if rows > limits.max_prepared_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Model grid exceeds row budget.",
        ));
    }
    if options
        .xseq
        .as_ref()
        .is_some_and(|g| g.iter().any(|v| !v.is_finite()))
    {
        return Err(invalid());
    }
    if let Some(terms) = &options.terms {
        if terms.is_empty() || terms.len() > 64 {
            return Err(invalid());
        }
        if matches!(
            options.method,
            ModelMethod::Auto | ModelMethod::Loess { .. } | ModelMethod::Gam { .. }
        ) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Explicit model terms require a linear, generalized-linear or quantile method.",
            ));
        }
        for term in terms {
            if let ModelTerm::Expression(expression) = term
                && expression
                    .validate(ExpressionLimits::default(), |_| Ok(ExpressionType::Number))?
                    != ExpressionType::Number
            {
                return Err(invalid());
            }
        }
    }
    let valid = match &options.method {
        ModelMethod::Auto | ModelMethod::Linear | ModelMethod::Registered { .. } => true,
        ModelMethod::Glm {
            epsilon,
            iterations,
            ..
        } => epsilon.is_finite() && *epsilon > 0. && (1..=10000).contains(iterations),
        ModelMethod::Loess {
            span,
            degree,
            cell,
            iterations,
            ..
        } => {
            span.is_finite()
                && *span > 0.
                && *degree <= 2
                && cell.is_finite()
                && *cell > 0.
                && (1..=100).contains(iterations)
        }
        ModelMethod::Gam {
            basis_dimension,
            knots,
            iterations,
            tolerance,
        } => {
            (3..=64).contains(basis_dimension)
                && (1..=1000).contains(iterations)
                && tolerance.is_finite()
                && *tolerance > 0.
                && knots.as_ref().is_none_or(|k| {
                    k.len() == *basis_dimension
                        && k.iter().all(|v| v.is_finite())
                        && k.windows(2).all(|v| v[0] < v[1])
                })
        }
        ModelMethod::Quantile {
            probabilities,
            iterations,
            solver: _,
        } => {
            if probabilities
                .len()
                .checked_mul(rows)
                .is_none_or(|v| v > limits.max_prepared_rows)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Quantile model grid exceeds row budget.",
                ));
            }
            !probabilities.is_empty()
                && (1..=1_000_000).contains(iterations)
                && probabilities
                    .iter()
                    .all(|p| p.is_finite() && (0. ..=1.).contains(p))
        }
    };
    if valid { Ok(()) } else { Err(invalid()) }
}
pub(crate) fn predict(
    x: &[f64],
    y: &[f64],
    weights: &[f64],
    options: &ModelOptions,
    grid: &[f64],
    largest_group: usize,
    limits: CompileLimits,
) -> ChartResult<ModelPredictionOutput> {
    validate(options, limits)?;
    if x.len() != y.len()
        || weights.len() != x.len()
        || x.iter()
            .chain(y)
            .chain(weights)
            .chain(grid)
            .any(|v| !v.is_finite())
        || weights.iter().any(|w| *w < 0.)
        || !options.level.is_finite()
        || options.level <= 0.
        || options.level >= 1.
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Invalid model population, prediction grid or confidence level.",
        ));
    }
    if grid.len() > limits.max_prepared_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Model prediction grid exceeds row budget.",
        ));
    }
    let mut output = ModelPredictionOutput::default();
    if x.len() < 2 || x.iter().all(|v| *v == x[0]) {
        warning(
            &mut output,
            "Model group has fewer than two distinct predictor values.",
        );
        return Ok(output);
    }
    let method = match options.method {
        ModelMethod::Auto => {
            if largest_group < 1000 {
                ModelMethod::Loess {
                    span: 0.75,
                    degree: 2,
                    cell: 0.2,
                    surface: LoessSurface::Interpolate,
                    family: LoessFamily::Gaussian,
                    iterations: 4,
                    normalize: true,
                    exact_statistics: false,
                    approximate_trace: false,
                }
            } else {
                ModelMethod::Gam {
                    basis_dimension: 10,
                    knots: None,
                    iterations: 120,
                    tolerance: 1e-9,
                }
            }
        }
        _ => options.method.clone(),
    };
    let normal = crate::scales::GgplotTransform::Normal { mean: 0., sd: 1. }
        .forward((1. + options.level) / 2.);
    if matches!(method, ModelMethod::Loess { .. } | ModelMethod::Gam { .. })
        && options.terms.is_some()
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Explicit model-matrix terms are not supported by the univariate LOESS/GAM route.",
        ));
    }
    match method {
        ModelMethod::Linear => {
            let (design, p) = matrix(x, options.terms.as_deref(), limits.max_vertices)?;
            let (prediction, _) = matrix(grid, options.terms.as_deref(), limits.max_vertices)?;
            let fit = super::model_linear::fit(&design, y, weights, p)?;
            if fit.rank < p {
                warning(
                    &mut output,
                    "Model design is rank deficient; aliased coefficients are retained.",
                );
            }
            let critical = if options.se && fit.residual_df > 0 {
                super::ggplot_summary::student_quantile(
                    (1. + options.level) / 2.,
                    fit.residual_df as f64,
                )?
            } else {
                f64::NAN
            };
            if options.se && fit.residual_df == 0 {
                warning(
                    &mut output,
                    "Model has no residual degrees of freedom for uncertainty.",
                );
            }
            for (x, row) in grid.iter().zip(prediction.chunks_exact(p)) {
                match fit.predict(row, fit.residual_variance) {
                    Ok((mean, se)) => append(
                        &mut output,
                        *x,
                        finite(mean),
                        if options.se { finite(se) } else { None },
                        critical,
                        None,
                    ),
                    Err(_) => {
                        warning(
                            &mut output,
                            "Prediction is outside the model's estimable subspace.",
                        );
                        append(&mut output, *x, None, None, critical, None);
                    }
                }
            }
        }
        ModelMethod::Glm {
            family,
            epsilon,
            iterations,
        } => {
            let (design, p) = matrix(x, options.terms.as_deref(), limits.max_vertices)?;
            let (prediction, _) = matrix(grid, options.terms.as_deref(), limits.max_vertices)?;
            let family = match family {
                ModelFamily::Gaussian => super::model_glm::Family::Gaussian,
                ModelFamily::Binomial => super::model_glm::Family::Binomial,
                ModelFamily::Poisson => super::model_glm::Family::Poisson,
            };
            let fit = super::model_glm::fit(&design, y, weights, p, family, epsilon, iterations)?;
            if !fit.converged {
                warning(
                    &mut output,
                    &format!("GLM did not converge after {} iterations.", fit.iterations),
                );
            }
            if fit.boundary {
                warning(
                    &mut output,
                    "GLM used step halving to reach finite fitted values.",
                );
            }
            if fit.linear.rank < p {
                warning(&mut output, "GLM design is rank deficient.");
            }
            for (x, row) in grid.iter().zip(prediction.chunks_exact(p)) {
                match fit.predict_link(row) {
                    Ok((eta, se)) => {
                        let se = if options.se { finite(se) } else { None };
                        output.rows.push(Prediction {
                            x: *x,
                            mean: finite(fit.family.inverse(eta)),
                            se,
                            lower: se.and_then(|se| finite(fit.family.inverse(eta - normal * se))),
                            upper: se.and_then(|se| finite(fit.family.inverse(eta + normal * se))),
                            quantile: None,
                        });
                    }
                    Err(_) => append(&mut output, *x, None, None, normal, None),
                }
            }
        }
        ModelMethod::Loess {
            span,
            degree,
            cell,
            surface,
            family,
            iterations,
            normalize,
            exact_statistics,
            approximate_trace,
        } => {
            let controls = super::model_loess::Controls {
                span,
                degree,
                cell,
                surface: match surface {
                    LoessSurface::Interpolate => super::model_loess::Surface::Interpolate,
                    LoessSurface::Direct => super::model_loess::Surface::Direct,
                },
                family: match family {
                    LoessFamily::Gaussian => super::model_loess::Family::Gaussian,
                    LoessFamily::Symmetric => super::model_loess::Family::Symmetric,
                },
                iterations,
                normalize,
                exact_statistics,
                approximate_trace,
            };
            let fit = super::model_loess::fit(x, y, weights, controls, limits.max_vertices)?;
            if options.se {
                let u = fit.uncertainty(grid, limits.max_vertices)?;
                if !u.residual_scale.is_finite() {
                    warning(
                        &mut output,
                        "LOESS residual scale is not finite; uncertainty may be unavailable.",
                    );
                }
                let critical =
                    super::ggplot_summary::student_quantile((1. + options.level) / 2., u.df)?;
                for ((x, mean), se) in grid.iter().zip(u.fitted).zip(u.standard_errors) {
                    append(
                        &mut output,
                        *x,
                        mean.and_then(finite),
                        se.and_then(finite),
                        critical,
                        None,
                    );
                }
            } else {
                for (x, mean) in grid.iter().zip(fit.predict(grid)?) {
                    append(&mut output, *x, mean.and_then(finite), None, normal, None);
                }
            }
        }
        ModelMethod::Gam {
            basis_dimension,
            knots,
            iterations,
            tolerance,
        } => {
            let fit = super::model_gam::fit(
                x,
                y,
                weights,
                super::model_gam::Controls {
                    basis_dimension,
                    knots,
                    iterations,
                    tolerance,
                },
                limits.max_vertices,
            )?;
            if !fit.smoothing_parameter.is_finite()
                || !fit.residual_variance.is_finite()
                || !fit.effective_df.is_finite()
            {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "GAM fit produced nonfinite model diagnostics.",
                ));
            }
            if !fit.converged {
                warning(
                    &mut output,
                    "GAM REML smoothing selection did not converge.",
                );
            }
            for (x, (mean, se)) in grid.iter().zip(fit.predict(grid)?) {
                append(
                    &mut output,
                    *x,
                    finite(mean),
                    if options.se { finite(se) } else { None },
                    normal,
                    None,
                );
            }
        }
        ModelMethod::Quantile {
            probabilities,
            iterations,
            solver,
        } => {
            if probabilities
                .len()
                .checked_mul(grid.len())
                .is_none_or(|n| n > limits.max_prepared_rows)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Quantile prediction rows exceed budget.",
                ));
            }
            let (design, p) = matrix(x, options.terms.as_deref(), limits.max_vertices)?;
            let (prediction, _) = matrix(grid, options.terms.as_deref(), limits.max_vertices)?;
            for tau in probabilities {
                let coefficients = match solver {
                    ModelQuantileSolver::Br => {
                        let fit = super::quantile_regression::fit(
                            &design, y, weights, p, tau, iterations,
                        )?;
                        if fit.nonunique {
                            warning(
                                &mut output,
                                "Quantile regression solution may be nonunique.",
                            );
                        }
                        fit.coefficients
                    }
                    ModelQuantileSolver::Fn => {
                        let fit =
                            super::model_quantile_fn::fit(&design, y, weights, p, tau, iterations)?;
                        if !fit.converged {
                            warning(
                                &mut output,
                                &format!(
                                    "Quantile interior-point solver did not converge after {} iterations.",
                                    fit.iterations
                                ),
                            );
                        }
                        fit.coefficients
                    }
                };
                for (x, row) in grid.iter().zip(prediction.chunks_exact(p)) {
                    let mean = row
                        .iter()
                        .zip(&coefficients)
                        .map(|(x, b)| x * b)
                        .sum::<f64>();
                    append(&mut output, *x, finite(mean), None, normal, Some(tau));
                }
            }
        }
        ModelMethod::Registered { .. } => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered model requires installed registry dispatch.",
            ));
        }
        ModelMethod::Auto => unreachable!(),
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn near(actual: Option<f64>, expected: &serde_json::Value, label: &str) {
        if expected.is_null() {
            assert_eq!(actual, None, "{label}");
        } else {
            let expected = expected.as_f64().unwrap();
            assert!(
                (actual.unwrap() - expected).abs() < 1e-7,
                "{label}: {actual:?} != {expected}"
            );
        }
    }
    #[test]
    fn invalid_controls_reject_empty_population() {
        let options = ModelOptions {
            level: 1.,
            ..ModelOptions::default()
        };
        assert!(predict(&[], &[], &[], &options, &[], 0, CompileLimits::default()).is_err());
        let options = ModelOptions {
            method: ModelMethod::Glm {
                family: ModelFamily::Gaussian,
                epsilon: 0.,
                iterations: 25,
            },
            ..ModelOptions::default()
        };
        assert!(validate(&options, CompileLimits::default()).is_err());
    }
    #[test]
    fn pinned_model_predictions_and_intervals() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/model-controls.json"
        ))
        .unwrap();
        let mut count = 0;
        for case in fixture["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let controls = &case["controls"];
            let method = if name.starts_with("lm-weighted-") {
                ModelMethod::Linear
            } else if name.starts_with("glm-") && name != "glm-nonconverged" {
                ModelMethod::Glm {
                    family: match controls["family"].as_str().unwrap() {
                        "gaussian" => ModelFamily::Gaussian,
                        "binomial" => ModelFamily::Binomial,
                        _ => ModelFamily::Poisson,
                    },
                    epsilon: 1e-8,
                    iterations: 25,
                }
            } else if name.starts_with("loess-") {
                ModelMethod::Loess {
                    span: controls["span"].as_f64().unwrap(),
                    degree: controls["method_args"]["degree"].as_u64().unwrap_or(2) as usize,
                    cell: 0.2,
                    surface: LoessSurface::Interpolate,
                    family: if name == "loess-symmetric" {
                        LoessFamily::Symmetric
                    } else {
                        LoessFamily::Gaussian
                    },
                    iterations: 4,
                    normalize: true,
                    exact_statistics: false,
                    approximate_trace: false,
                }
            } else if name == "automatic-999" || name == "automatic-1000" {
                ModelMethod::Auto
            } else if name.starts_with("quantile-br-") || name.starts_with("quantile-fn-") {
                ModelMethod::Quantile {
                    solver: if name.starts_with("quantile-fn-") {
                        ModelQuantileSolver::Fn
                    } else {
                        ModelQuantileSolver::Br
                    },
                    probabilities: vec![0.25, 0.5, 0.75],
                    iterations: 10000,
                }
            } else {
                continue;
            };
            let data = controls["data"].as_array().unwrap();
            let x: Vec<_> = data.iter().map(|r| r["x"].as_f64().unwrap()).collect();
            let y: Vec<_> = data.iter().map(|r| r["y"].as_f64().unwrap()).collect();
            let w: Vec<_> = data.iter().map(|r| r["w"].as_f64().unwrap_or(1.)).collect();
            let value = &case["result"]["value"];
            let expected = value.get("built").unwrap_or(value)["values"]
                .as_array()
                .unwrap();
            let mut grid = vec![];
            for row in expected {
                let v = row["x"].as_f64().unwrap();
                if !grid.contains(&v) {
                    grid.push(v);
                }
            }
            let options = ModelOptions {
                method,
                level: controls["level"].as_f64().unwrap_or(0.95),
                ..Default::default()
            };
            let out = predict(
                &x,
                &y,
                &w,
                &options,
                &grid,
                x.len(),
                CompileLimits::default(),
            )
            .unwrap();
            assert_eq!(out.rows.len(), expected.len(), "{name}");
            for (row, expected) in out.rows.iter().zip(expected) {
                near(Some(row.x), &expected["x"], name);
                near(row.mean, &expected["y"], name);
                near(row.se, &expected["se"], name);
                near(row.lower, &expected["ymin"], name);
                near(row.upper, &expected["ymax"], name);
                near(row.quantile, &expected["quantile"], name);
            }
            count += 1;
        }
        assert_eq!(count, 20);
    }
}
