use crate::ChartResult;
/// Hyndman-Fan definitions on a finite, nonempty, sorted sample.
pub(crate) fn quantile(sorted: &[f64], prob: f64, kind: u8) -> ChartResult<f64> {
    if sorted.is_empty() || !(0. ..=1.).contains(&prob) || !(1..=9).contains(&kind) {
        return Err(super::invalid(
            "Quantile requires a nonempty sample, probability in [0,1], and type 1..=9.",
        ));
    }
    let n = sorted.len() as f64;
    let m = match kind {
        1 | 2 | 4 => 0.,
        3 => -0.5,
        5 => 0.5,
        6 => prob,
        7 => 1. - prob,
        8 => (prob + 1.) / 3.,
        9 => prob / 4. + 0.375,
        _ => unreachable!(),
    };
    let h = if kind == 7 {
        1. + (n - 1.) * prob
    } else {
        n * prob + m
    };
    let fuzz = if kind == 7 { 0. } else { 4. * f64::EPSILON };
    let j = (h + fuzz).floor();
    let g = if (h - j).abs() < fuzz { 0. } else { h - j };
    let gamma = match kind {
        1 => {
            if g == 0. {
                0.
            } else {
                1.
            }
        }
        2 => {
            if g == 0. {
                0.5
            } else {
                1.
            }
        }
        3 => {
            if g == 0. && j as i64 % 2 == 0 {
                0.
            } else {
                1.
            }
        }
        _ => g,
    };
    let get = |k: f64| sorted[(k as isize - 1).clamp(0, sorted.len() as isize - 1) as usize];
    let a = get(j);
    let b = get(j + 1.);
    Ok(if gamma == 0. {
        a
    } else if gamma == 1. {
        b
    } else {
        (1. - gamma) * a + gamma * b
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_nine_quantile_rules() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap();
            if !name.starts_with("box-") || name.contains("weighted") || name.contains("missing") {
                continue;
            }
            let mut y = c["controls"]["y"].as_array().map_or_else(
                || vec![c["controls"]["y"].as_f64().unwrap()],
                |v| v.iter().map(|v| v.as_f64().unwrap()).collect(),
            );
            y.sort_by(f64::total_cmp);
            let kind = c["controls"]["quantile_type"].as_u64().unwrap() as u8;
            let row = &c["result"]["value"][0];
            for (p, k) in [(0.25, "lower"), (0.5, "middle"), (0.75, "upper")] {
                let q = quantile(&y, p, kind).unwrap();
                assert!(
                    (q - row[k].as_f64().unwrap()).abs() < 1e-12,
                    "{name} {k} {q}"
                );
            }
        }
    }
}
