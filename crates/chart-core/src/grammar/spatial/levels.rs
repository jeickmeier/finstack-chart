use super::super::ContourLevels;
use crate::ChartResult;
fn sequence(start: f64, end: f64, width: f64, budget: usize) -> ChartResult<Vec<f64>> {
    let count = libm::round((end - start) / width);
    if !count.is_finite() || count < 0. || count >= budget as f64 {
        return Err(super::invalid("Contour threshold budget exceeded."));
    }
    Ok((0..=count as usize)
        .map(|i| start + i as f64 * width)
        .collect())
}
fn full(range: [f64; 2], width: f64, budget: usize) -> ChartResult<Vec<f64>> {
    if !width.is_finite() || width <= 0. {
        return Err(super::invalid(
            "Contour threshold interval must be finite and positive.",
        ));
    }
    if range[0] == range[1] {
        return Ok(vec![range[0] - width, range[1] + width]);
    }
    sequence(
        libm::floor(range[0] / width) * width,
        libm::ceil(range[1] / width) * width,
        width,
        budget,
    )
}
/// Reference threshold selection shared by contours and density contours.
pub(crate) fn contour_levels(
    range: [f64; 2],
    control: &ContourLevels,
    budget: usize,
) -> ChartResult<Vec<f64>> {
    if let Some(breaks) = &control.breaks {
        if breaks.len() > budget
            || breaks.iter().any(|v| !v.is_finite())
            || breaks.windows(2).any(|v| v[0] >= v[1])
        {
            return Err(super::invalid(
                "Contour thresholds must be bounded, ordered and finite.",
            ));
        }
        return Ok(breaks.clone());
    }
    if range.iter().any(|v| !v.is_finite()) || range[0] > range[1] {
        return Err(super::invalid("Contour range must be finite and ordered."));
    }
    if let Some(n) = control.bins {
        if n == 0 || n > budget {
            return Err(super::invalid(
                "Contour bin count must be positive and bounded.",
            ));
        }
        let span = range[1] - range[0];
        if span == 0. {
            return Ok(Vec::new());
        }
        let magnitude = libm::pow(10., libm::floor(libm::log10(span)));
        let accuracy = (span / magnitude).round_ties_even() * magnitude / 10.;
        let range = [
            libm::floor(range[0] / accuracy) * accuracy,
            libm::ceil(range[1] / accuracy) * accuracy,
        ];
        if n == 1 {
            return Ok(range.to_vec());
        }
        let mut result = full(range, (range[1] - range[0]) / (n - 1) as f64, budget)?;
        if result.len() < n + 1 {
            result = full(range, (range[1] - range[0]) / n as f64, budget)?;
        }
        return Ok(result);
    }
    if let Some(width) = control.binwidth {
        return full(range, width, budget);
    }
    let span = range[1] - range[0];
    let magnitude = range[0].abs().max(range[1].abs());
    let tiny = span < magnitude * (1.4 * 10. * f64::EPSILON) * 3. || span == 0.;
    let cell = if tiny {
        let magnitude = if magnitude == 0. {
            1.
        } else if magnitude > 10. {
            9. + magnitude / 10.
        } else {
            magnitude
        };
        magnitude * 0.75 / 3.
    } else {
        span / 10.
    };
    let base = libm::pow(
        10.,
        libm::floor(libm::log10(cell.max(f64::MIN_POSITIVE / 1048576.))),
    );
    let scaled = cell / base;
    let width = base
        * if scaled <= 1.4 {
            1.
        } else if scaled <= 2.8 {
            2.
        } else if scaled <= 7. {
            5.
        } else {
            10.
        };
    let mut first = libm::floor(range[0] / width + 1e-7);
    let mut last = libm::ceil(range[1] / width - 1e-7);
    let deficit = (3. - (last - first)).max(0.) as usize;
    if deficit > 0 {
        if range[0] == 0. && first == 0. && range[1] != 0. {
            last += deficit as f64;
        } else if range[1] == 0. && last == 0. && range[0] != 0. {
            first -= deficit as f64;
        } else if first >= 0. {
            first -= deficit.div_ceil(2) as f64;
            last += (deficit / 2) as f64;
        } else {
            first -= (deficit / 2) as f64;
            last += deficit.div_ceil(2) as f64;
        }
    }
    sequence(first * width, last * width, width, budget)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_default_pretty_and_explicit_bin_thresholds() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-contour-levels.json"
        ))
        .unwrap();
        for case in f["cases"].as_array().unwrap() {
            let range = [
                case["range"][0].as_f64().unwrap(),
                case["range"][1].as_f64().unwrap(),
            ];
            let bins = case["bins"].as_u64().map(|v| v as usize);
            let actual = contour_levels(
                range,
                &ContourLevels {
                    bins,
                    ..Default::default()
                },
                100,
            )
            .unwrap();
            if case.get("error").is_some()
                || case["breaks"]
                    .as_array()
                    .is_some_and(|v| v.iter().any(|v| v.is_null()))
            {
                assert!(actual.is_empty());
                continue;
            }
            let expected = case["breaks"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{range:?}/{bins:?}");
            for (a, e) in actual.iter().zip(expected) {
                let e = e.as_f64().unwrap();
                assert!((a - e).abs() < 1e-12, "{range:?}/{bins:?}: {a} != {e}");
            }
        }
    }
}
