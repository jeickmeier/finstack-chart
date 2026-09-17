//! Ordered band labels use a shared precision per endpoint column.
fn column(values: &[f64], digits: usize) -> Vec<String> {
    let decimals = values
        .iter()
        .filter(|v| **v != 0.)
        .map(|v| (digits as i32 - 1 - libm::floor(libm::log10(v.abs())) as i32).max(0) as usize)
        .max()
        .unwrap_or(0)
        .min(320);
    let mut fixed = values
        .iter()
        .map(|v| format!("{v:.decimals$}"))
        .collect::<Vec<_>>();
    while fixed.iter().all(|s| s.contains('.') && s.ends_with('0')) {
        for s in &mut fixed {
            s.pop();
        }
    }
    for s in &mut fixed {
        if s.ends_with('.') {
            s.pop();
        }
    }
    let mut mantissas = Vec::new();
    let mut exponents = Vec::new();
    for v in values {
        let text = format!("{v:.p$e}", p = digits - 1);
        let (m, e) = text.split_once('e').unwrap();
        mantissas.push(m.to_string());
        exponents.push(e.parse::<i32>().unwrap());
    }
    while mantissas
        .iter()
        .all(|s| s.contains('.') && s.ends_with('0'))
    {
        for s in &mut mantissas {
            s.pop();
        }
    }
    let scientific = mantissas
        .into_iter()
        .zip(exponents)
        .map(|(mut m, e)| {
            if m.ends_with('.') {
                m.pop();
            }
            format!("{m}e{e:+03}")
        })
        .collect::<Vec<_>>();
    if scientific.iter().map(String::len).max() < fixed.iter().map(String::len).max() {
        scientific
    } else {
        fixed
    }
}
pub(crate) fn labels(intervals: &[(f64, f64)]) -> Vec<String> {
    if intervals.is_empty() {
        return vec![];
    }
    let mut endpoints = intervals
        .iter()
        .flat_map(|(a, b)| [*a, *b])
        .collect::<Vec<_>>();
    endpoints.sort_by(f64::total_cmp);
    endpoints.dedup();
    let mut digits = 3;
    while digits < 17 {
        let texts = column(&endpoints, digits);
        if texts.windows(2).all(|p| p[0] != p[1]) {
            break;
        }
        digits += 1;
    }
    let low = column(&intervals.iter().map(|p| p.0).collect::<Vec<_>>(), digits);
    let high = column(&intervals.iter().map(|p| p.1).collect::<Vec<_>>(), digits);
    low.into_iter()
        .zip(high)
        .map(|(a, b)| format!("({a}, {b}]"))
        .collect()
}
#[cfg(test)]
mod tests {
    #[test]
    fn source_common_precision() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/spatial-contour-levels.json"
        ))
        .unwrap();
        for case in fixture["labels"].as_array().unwrap() {
            let breaks = case["breaks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            let intervals = breaks.windows(2).map(|p| (p[0], p[1])).collect::<Vec<_>>();
            let expected = case["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(super::labels(&intervals), expected);
        }
    }
}
