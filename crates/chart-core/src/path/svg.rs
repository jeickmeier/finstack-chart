use super::{ChartResult, Command, Precision, domain, finite, require};

pub(super) fn serialize(
    commands: &[Command],
    precision: Precision,
    max_bytes: usize,
) -> ChartResult<String> {
    if matches!(precision, Precision::Digits(16..)) {
        return Err(domain("Path precision exceeds 15 digits"));
    }
    let mut result = String::new();
    for command in commands {
        let (letter, values): (&str, Vec<f64>) = match *command {
            Command::MoveTo(p) => ("M", p.to_vec()),
            Command::LineTo(p) => ("L", p.to_vec()),
            Command::QuadraticTo(p) => ("Q", p.to_vec()),
            Command::CubicTo(p) => ("C", p.to_vec()),
            Command::Horizontal(v) => ("h", vec![v]),
            Command::Vertical(v) => ("v", vec![v]),
            Command::Close => ("Z", vec![]),
            Command::Arc {
                radius,
                large,
                clockwise,
                to,
            } => (
                "A",
                vec![
                    radius,
                    radius,
                    0.,
                    u8::from(large) as f64,
                    u8::from(clockwise) as f64,
                    to[0],
                    to[1],
                ],
            ),
        };
        append(&mut result, letter, max_bytes)?;
        for (i, value) in values.into_iter().enumerate() {
            if i > 0 {
                append(&mut result, ",", max_bytes)?;
            }
            append(&mut result, &number(value, precision)?, max_bytes)?;
        }
    }
    Ok(result)
}
fn append(output: &mut String, value: &str, max_bytes: usize) -> ChartResult<()> {
    require(
        value.len() <= max_bytes.saturating_sub(output.len()),
        "Path SVG byte limit",
    )?;
    output.push_str(value);
    Ok(())
}
fn number(mut value: f64, precision: Precision) -> ChartResult<String> {
    if let Precision::Digits(digits) = precision {
        let scale = 10_f64.powi(i32::from(digits));
        let scaled = value * scale;
        finite(&[scaled])?;
        let floor = scaled.floor();
        value = (if scaled - floor >= 0.5 {
            floor + 1.0
        } else {
            floor
        }) / scale;
    }
    finite(&[value])?;
    Ok(crate::number::ecmascript(value))
}
