use crate::ChartResult;
#[derive(Clone, Debug)]
pub(crate) struct BoxSummary {
    pub ymin: f64,
    pub lower: f64,
    pub middle: f64,
    pub upper: f64,
    pub ymax: f64,
    pub notchlower: f64,
    pub notchupper: f64,
    pub relvarwidth: f64,
    pub outliers: Vec<usize>,
    pub nonunique: bool,
}
pub(crate) fn boxplot(
    values: &[f64],
    weights: Option<&[f64]>,
    coef: f64,
    quantile_type: u8,
) -> ChartResult<BoxSummary> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) || !coef.is_finite() || coef < 0.
    {
        return Err(super::invalid(
            "Boxplot requires finite observations and a nonnegative outlier multiplier.",
        ));
    }
    let mut stats = [0.; 5];
    let mut nonunique = false;
    if let Some(w) = weights {
        for (i, tau) in [0., 0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
            let fit = super::super::quantile_regression::fit(
                &vec![1.; values.len()],
                values,
                w,
                1,
                tau,
                values.len().saturating_mul(20).max(100),
            )?;
            stats[i] = fit.coefficients[0];
            nonunique |= fit.nonunique;
        }
    } else {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        for (i, tau) in [0., 0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
            stats[i] = super::quantile(&sorted, tau, quantile_type)?;
        }
    }
    let iqr = stats[3] - stats[1];
    let outliers = values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v < stats[1] - coef * iqr || **v > stats[3] + coef * iqr)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if !outliers.is_empty() {
        let mut low = stats[1];
        let mut high = stats[3];
        for (i, v) in values.iter().enumerate() {
            if outliers.binary_search(&i).is_err() {
                low = low.min(*v);
                high = high.max(*v);
            }
        }
        stats[0] = low;
        stats[4] = high;
    }
    let n = weights.map_or(values.len() as f64, |w| w.iter().sum());
    let width = libm::sqrt(n);
    let half = 1.58 * iqr / width;
    Ok(BoxSummary {
        ymin: stats[0],
        lower: stats[1],
        middle: stats[2],
        upper: stats[3],
        ymax: stats[4],
        notchlower: stats[2] - half,
        notchupper: stats[2] + half,
        relvarwidth: width,
        outliers,
        nonunique,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_boxplot_summaries() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        let vals = |v: &serde_json::Value| {
            v.as_array().map_or_else(
                || vec![v.as_f64().unwrap()],
                |a| a.iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>(),
            )
        };
        for c in f["cases"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap();
            if !name.starts_with("box-") || name.contains("missing") {
                continue;
            }
            let y = vals(&c["controls"]["y"]);
            let w = c["controls"].get("weight").map(vals);
            let result = boxplot(
                &y,
                w.as_deref(),
                1.5,
                c["controls"]["quantile_type"].as_u64().unwrap_or(7) as u8,
            );
            let rows = c["result"]["value"].as_array().unwrap();
            if rows.is_empty() {
                assert!(result.is_err());
                continue;
            }
            let s = result.unwrap();
            let r = &rows[0];
            for (k, a) in [
                ("ymin", s.ymin),
                ("lower", s.lower),
                ("middle", s.middle),
                ("upper", s.upper),
                ("ymax", s.ymax),
                ("notchlower", s.notchlower),
                ("notchupper", s.notchupper),
            ] {
                assert!(
                    (a - r[k].as_f64().unwrap()).abs() < 1e-10,
                    "{name} {k}: {a}"
                );
            }
        }
    }
}
