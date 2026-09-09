use super::transform::{TransformSyntax, rotation, translation};
use super::{ChartResult, DiagnosticCode, error};
use crate::path::Affine;
const MAX_BYTES: usize = 16_384;
const MAX_OPERATIONS: usize = 1024;
#[derive(Debug)]
struct Quantity {
    value: f64,
    unit: String,
}
fn syntax() -> crate::Diagnostic {
    error(
        DiagnosticCode::Validation,
        "Malformed or unsupported 2D transform syntax.",
    )
}
fn arguments(body: &str, css: bool) -> ChartResult<Vec<Quantity>> {
    let mut rest = body.trim();
    let mut args = vec![];
    if rest.is_empty() {
        return Ok(args);
    }
    loop {
        let (end, value) = super::text::number_prefix(rest).ok_or_else(syntax)?;
        if !value.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Transform arguments must be finite.",
            ));
        }
        let suffix = &rest[end..];
        let unit_end = suffix
            .bytes()
            .take_while(|v| v.is_ascii_alphabetic() || *v == b'%')
            .count();
        args.push(Quantity {
            value,
            unit: suffix[..unit_end].to_ascii_lowercase(),
        });
        if args.len() > 6 {
            return Err(syntax());
        }
        rest = &suffix[unit_end..];
        let separated = rest.starts_with(char::is_whitespace);
        rest = rest.trim_start();
        if rest.is_empty() {
            return Ok(args);
        }
        if let Some(tail) = rest.strip_prefix(',') {
            rest = tail.trim_start();
            if rest.is_empty() {
                return Err(syntax());
            }
        } else if css || !separated && !rest.starts_with(['+', '-']) {
            return Err(syntax());
        }
    }
}
fn raw(v: &Quantity) -> ChartResult<f64> {
    if v.unit.is_empty() {
        Ok(v.value)
    } else {
        Err(syntax())
    }
}
fn length(v: &Quantity, css: bool) -> ChartResult<f64> {
    if !css {
        return raw(v);
    }
    let unit = match v.unit.as_str() {
        "px" => 1.,
        "in" => 96.,
        "cm" => 96. / 2.54,
        "mm" => 96. / 25.4,
        "q" => 96. / 101.6,
        "pt" => 96. / 72.,
        "pc" => 16.,
        "" if v.value == 0. => 1.,
        "%" | "em" | "rem" | "ex" | "ch" | "vw" | "vh" | "vmin" | "vmax" | "lh" | "rlh" => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Context-dependent transform length requires an explicitly resolved matrix.",
            ));
        }
        _ => return Err(syntax()),
    };
    Ok(v.value * unit)
}
fn angle(v: &Quantity, css: bool) -> ChartResult<f64> {
    if !css {
        return raw(v);
    }
    let unit = match v.unit.as_str() {
        "deg" => 1.,
        "rad" => 180. / std::f64::consts::PI,
        "grad" => 0.9,
        "turn" => 360.,
        "" if v.value == 0. => 1.,
        _ => return Err(syntax()),
    };
    Ok(v.value * unit)
}
fn scale(v: &Quantity, css: bool) -> ChartResult<f64> {
    if css && v.unit == "%" {
        Ok(v.value / 100.)
    } else {
        raw(v)
    }
}
fn operation(name: &str, args: &[Quantity], css: bool) -> ChartResult<Affine> {
    let n = args.len();
    let name = if css {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    };
    match name.as_str() {
        "matrix" if n == 6 => Affine::new([
            raw(&args[0])?,
            raw(&args[1])?,
            raw(&args[2])?,
            raw(&args[3])?,
            raw(&args[4])?,
            raw(&args[5])?,
        ]),
        "translate" if n == 1 || n == 2 => translation(
            length(&args[0], css)?,
            if n == 2 { length(&args[1], css)? } else { 0. },
        ),
        "translatex" if css && n == 1 => translation(length(&args[0], true)?, 0.),
        "translatey" if css && n == 1 => translation(0., length(&args[0], true)?),
        "scale" if n == 1 || n == 2 => {
            let x = scale(&args[0], css)?;
            let y = if n == 2 { scale(&args[1], css)? } else { x };
            Affine::new([x, 0., 0., y, 0., 0.])
        }
        "scalex" if css && n == 1 => Affine::new([scale(&args[0], true)?, 0., 0., 1., 0., 0.]),
        "scaley" if css && n == 1 => Affine::new([1., 0., 0., scale(&args[0], true)?, 0., 0.]),
        "rotate" if n == 1 || !css && n == 3 => {
            let r = rotation(angle(&args[0], css)?)?;
            if n == 3 {
                let x = raw(&args[1])?;
                let y = raw(&args[2])?;
                translation(x, y)?
                    .concatenate(r)?
                    .concatenate(translation(-x, -y)?)
            } else {
                Ok(r)
            }
        }
        "skewx" if css && n == 1 => Affine::new([
            1.,
            0.,
            (angle(&args[0], true)? * std::f64::consts::PI / 180.).tan(),
            1.,
            0.,
            0.,
        ]),
        "skewX" if !css && n == 1 => Affine::new([
            1.,
            0.,
            (raw(&args[0])? * std::f64::consts::PI / 180.).tan(),
            1.,
            0.,
            0.,
        ]),
        "skewy" if css && n == 1 => Affine::new([
            1.,
            (angle(&args[0], true)? * std::f64::consts::PI / 180.).tan(),
            0.,
            1.,
            0.,
            0.,
        ]),
        "skewY" if !css && n == 1 => Affine::new([
            1.,
            (raw(&args[0])? * std::f64::consts::PI / 180.).tan(),
            0.,
            1.,
            0.,
            0.,
        ]),
        "skew" if css && (n == 1 || n == 2) => Affine::new([
            1.,
            if n == 2 {
                (angle(&args[1], true)? * std::f64::consts::PI / 180.).tan()
            } else {
                0.
            },
            (angle(&args[0], true)? * std::f64::consts::PI / 180.).tan(),
            1.,
            0.,
            0.,
        ]),
        _ => Err(syntax()),
    }
}
/// Parse bounded absolute 2D CSS/SVG transform lists without a DOM or font context.
/// Percent translations and context-relative lengths require a resolved matrix.
pub fn parse_transform(text: &str, syntax_kind: TransformSyntax) -> ChartResult<Affine> {
    if text.len() > MAX_BYTES {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Transform text exceeds its byte budget.",
        ));
    }
    let css = syntax_kind == TransformSyntax::Css;
    let mut rest = text.trim();
    if rest.is_empty() || rest.eq_ignore_ascii_case("none") {
        return Ok(Affine::default());
    }
    let mut matrix = Affine::default();
    let mut count = 0;
    while !rest.is_empty() {
        if !rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) {
            return Err(syntax());
        }
        let end = rest.bytes().take_while(u8::is_ascii_alphanumeric).count();
        if end == 0 {
            return Err(syntax());
        }
        let name = &rest[..end];
        if [
            "matrix3d",
            "translate3d",
            "translatez",
            "rotatex",
            "rotatey",
            "rotatez",
            "rotate3d",
            "scale3d",
            "scalez",
            "perspective",
        ]
        .contains(&name.to_ascii_lowercase().as_str())
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "3D/perspective transforms require an explicitly resolved 2D matrix.",
            ));
        }
        rest = &rest[end..];
        if !css {
            rest = rest.trim_start();
        }
        rest = rest.strip_prefix('(').ok_or_else(syntax)?;
        let end = rest.find(')').ok_or_else(syntax)?;
        let args = arguments(&rest[..end], css)?;
        matrix = matrix.concatenate(operation(name, &args, css)?)?;
        rest = rest[end + 1..].trim_start();
        if !css && let Some(tail) = rest.strip_prefix(',') {
            rest = tail.trim_start();
            if rest.is_empty() {
                return Err(syntax());
            }
        }
        count += 1;
        if count > MAX_OPERATIONS {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Transform operation count exceeds its budget.",
            ));
        }
    }
    Ok(matrix)
}
