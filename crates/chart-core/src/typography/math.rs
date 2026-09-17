//! Bounded plotmath syntax. This parser never evaluates source or invokes R.
use crate::{ChartResult, Diagnostic, DiagnosticCode, Limits, services::ResourceDescriptor};
use serde::{Deserialize, Serialize};

/// Explicit face resources for mathematical typesetting. No system-font lookup occurs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MathFonts {
    /// Upright text and numeric face; absent uses the supplied destination font.
    pub regular: Option<ResourceDescriptor>,
    /// Italic identifier face.
    pub italic: Option<ResourceDescriptor>,
    /// Bold upright face.
    pub bold: Option<ResourceDescriptor>,
    /// Bold italic face.
    pub bold_italic: Option<ResourceDescriptor>,
    /// Upright Unicode mathematical symbols; absent uses the regular face.
    pub symbol: Option<ResourceDescriptor>,
}
impl MathFonts {
    /// Validate explicit face descriptors before any row or label is shaped.
    pub fn validate(&self, limits: Limits) -> ChartResult<()> {
        for font in [
            &self.regular,
            &self.italic,
            &self.bold,
            &self.bold_italic,
            &self.symbol,
        ]
        .into_iter()
        .flatten()
        {
            font.validate(limits)?;
            if font.kind != crate::services::ResourceKind::Font {
                return Err(invalid("Mathematical faces must be font resources."));
            }
        }
        Ok(())
    }
}
/// Safe mathematical syntax tree. Calls describe notation; they never execute functions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MathNode {
    /// Quoted upright text.
    Text(String),
    /// Identifier or named mathematical symbol.
    Symbol(String),
    /// Canonical finite numeric value spelling, typeset upright.
    Number(String),
    /// Prefix plus, minus, not or spacing.
    Unary {
        /// Validated plotmath operator spelling.
        operator: String,
        /// Operand.
        value: Box<MathNode>,
    },
    /// A supported plotmath infix operator.
    Binary {
        /// Validated plotmath operator spelling.
        operator: String,
        /// Left operand.
        left: Box<MathNode>,
        /// Right operand.
        right: Box<MathNode>,
    },
    /// Typesetting instruction or ordinary function notation.
    Call {
        /// Function name or constructed notation.
        head: Box<MathNode>,
        /// Ordered notation arguments.
        arguments: Vec<MathNode>,
    },
    /// Lower script; comma-separated indices are retained as arguments to list.
    Subscript {
        /// Scripted base.
        base: Box<MathNode>,
        /// Script content.
        script: Box<MathNode>,
    },
    /// Upper script.
    Superscript {
        /// Scripted base.
        base: Box<MathNode>,
        /// Script content.
        script: Box<MathNode>,
    },
    /// Parentheses are visible; braces preserve precedence without painting delimiters.
    Group {
        /// Whether parentheses paint around the contents.
        visible: bool,
        /// Grouped contents.
        value: Box<MathNode>,
    },
}
/// Parsed label with original logical source and exact caller-owned face identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MathExpression {
    /// Original logical/accessibility expression.
    pub source: String,
    /// Parsed, nonexecuting notation.
    pub ast: MathNode,
    /// Explicit mathematical face resources.
    pub fonts: MathFonts,
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Supply bounded plotmath notation; expressions are never executed.",
    )
}
impl MathExpression {
    /// Parse one plotmath expression with explicit resource and work limits.
    pub fn parse(source: impl Into<String>, fonts: MathFonts, limits: Limits) -> ChartResult<Self> {
        let source = source.into();
        let ast = parse(&source, limits)?;
        let value = Self { source, ast, fonts };
        value.validate(limits)?;
        Ok(value)
    }
    /// Measure the same mathematical boxes used by every label consumer.
    /// Font metrics come only from the supplied shaping service and resources.
    pub fn measure(
        &self,
        request: super::ShapeRequest<'_>,
        measurer: &dyn crate::services::TextMeasurer,
    ) -> ChartResult<crate::services::TextMetrics> {
        let layout = super::math_layout::layout(self, request, measurer)?;
        crate::services::TextMetrics::new(
            layout.width.abs(),
            layout.ascent.abs(),
            layout.descent.abs(),
        )
    }
    /// Validate replayed AST/source coherence and all font descriptors.
    pub fn validate(&self, limits: Limits) -> ChartResult<()> {
        if parse(&self.source, limits)? != self.ast {
            return Err(invalid("Math source and parsed syntax disagree."));
        }
        self.fonts.validate(limits)?;
        Ok(())
    }
}
// R scalar numeric labels use fifteen significant digits and prefer the shorter
// fixed/scientific representation, retaining fixed notation on equal lengths.
fn number_label(source: &str) -> String {
    let value = source
        .parse::<f64>()
        .expect("validated finite numeric token");
    if value == 0. {
        return "0".into();
    }
    let scientific = format!("{value:.14e}");
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific formatter");
    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
    let exponent = exponent.parse::<i32>().expect("formatted exponent");
    let scientific = format!("{mantissa}e{exponent:+03}");
    let rounded = scientific
        .parse::<f64>()
        .expect("finite formatted numeric value");
    if !rounded.is_finite() {
        return scientific;
    }
    let decimal = rounded.to_string();
    if scientific.len() < decimal.len() {
        scientific
    } else {
        decimal
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Name(String),
    Number(String),
    Text(String),
    Op(String),
    Open(char),
    Close(char),
    Comma,
    End,
}
fn parse(source: &str, limits: Limits) -> ChartResult<MathNode> {
    if source.len() > limits.max_text_bytes || source.len() > 65536 {
        return Err(crate::scales::error(
            DiagnosticCode::ResourceLimit,
            "Math source exceeds text budget.",
        ));
    }
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let token = match c {
            '(' | '[' | '{' => Token::Open(c),
            ')' | ']' | '}' => Token::Close(c),
            ',' => Token::Comma,
            '\'' | '"' => {
                let mut text = String::new();
                let mut closed = false;
                while let Some(v) = chars.next() {
                    if v == c {
                        closed = true;
                        break;
                    }
                    if v == '\\' {
                        let v = chars
                            .next()
                            .ok_or_else(|| invalid("Unclosed math string escape."))?;
                        match v {
                            '\\' | '\'' | '"' => text.push(v),
                            _ => return Err(invalid("Control or unknown escape in math text.")),
                        }
                    } else if v.is_control() {
                        return Err(invalid("Control character in math text."));
                    } else {
                        text.push(v)
                    }
                }
                if !closed {
                    return Err(invalid("Unclosed math string."));
                }
                Token::Text(text)
            }
            '%' => {
                let mut op = String::from("%");
                let mut closed = false;
                for v in chars.by_ref() {
                    op.push(v);
                    if v == '%' {
                        closed = true;
                        break;
                    }
                }
                if !closed || infix(&op).is_none() {
                    return Err(invalid("Unknown plotmath infix operator."));
                }
                Token::Op(op)
            }
            '+' | '-' | '*' | '/' | '^' | '!' | '=' | '<' | '>' | '~' | ':' | '|' => {
                if (c == '<' && chars.peek() == Some(&'-'))
                    || (c == '-' && chars.peek() == Some(&'>'))
                {
                    return Err(invalid("Assignments are not mathematical notation."));
                }
                let mut op = c.to_string();
                if chars.peek() == Some(&'=') || (c == '~' && chars.peek() == Some(&'~')) {
                    op.push(chars.next().unwrap());
                }
                if !matches!(
                    op.as_str(),
                    "+" | "-"
                        | "*"
                        | "/"
                        | "^"
                        | "!"
                        | "=="
                        | "!="
                        | "<"
                        | "<="
                        | ">"
                        | ">="
                        | "~"
                        | "~~"
                        | ":"
                        | "|"
                ) {
                    return Err(invalid("Unsupported math operator or assignment."));
                }
                Token::Op(op)
            }
            c if c.is_ascii_digit()
                || (c == '.' && chars.peek().is_some_and(char::is_ascii_digit)) =>
            {
                let mut n = c.to_string();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_digit() || *c == '.')
                {
                    n.push(chars.next().unwrap());
                }
                if chars.peek().is_some_and(|c| matches!(c, 'e' | 'E')) {
                    n.push(chars.next().unwrap());
                    if chars.peek().is_some_and(|c| matches!(c, '+' | '-')) {
                        n.push(chars.next().unwrap());
                    }
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        n.push(chars.next().unwrap());
                    }
                }
                if !n.parse::<f64>().is_ok_and(f64::is_finite) {
                    return Err(invalid("Math numeric literal must be finite."));
                }
                Token::Number(n)
            }
            c if c.is_alphabetic() || c == '_' || c == '.' => {
                let mut name = c.to_string();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                {
                    name.push(chars.next().unwrap());
                }
                Token::Name(name)
            }
            _ => return Err(invalid("Unsupported character in plotmath expression.")),
        };
        tokens.push(token);
        if tokens.len() > 4096 {
            return Err(crate::scales::error(
                DiagnosticCode::ResourceLimit,
                "Math syntax exceeds node budget.",
            ));
        }
    }
    tokens.push(Token::End);
    let mut p = Parser { tokens, index: 0 };
    let node = p.expression(0, 0)?;
    if p.peek() != &Token::End {
        return Err(invalid("Unexpected trailing math syntax."));
    }
    let mut stack = vec![(&node, 0usize)];
    while let Some((node, depth)) = stack.pop() {
        if depth > 64 {
            return Err(crate::scales::error(
                DiagnosticCode::ResourceLimit,
                "Math syntax tree exceeds depth budget.",
            ));
        }
        match node {
            MathNode::Unary { value, .. } | MathNode::Group { value, .. } => {
                stack.push((value, depth + 1))
            }
            MathNode::Binary { left, right, .. } => {
                stack.push((left, depth + 1));
                stack.push((right, depth + 1));
            }
            MathNode::Call { head, arguments } => {
                stack.push((head, depth + 1));
                stack.extend(arguments.iter().map(|a| (a, depth + 1)));
            }
            MathNode::Subscript { base, script } | MathNode::Superscript { base, script } => {
                stack.push((base, depth + 1));
                stack.push((script, depth + 1));
            }
            _ => {}
        }
    }
    Ok(node)
}
fn infix(op: &str) -> Option<(u8, u8)> {
    Some(match op {
        "~" | "~~" => (1, 2),
        "==" | "!=" | "<" | "<=" | ">" | ">=" | "|" => (3, 4),
        "+" | "-" => (5, 6),
        "*" | "/" => (7, 8),
        "%+-%" | "%/%" | "%*%" | "%.%" | "%~~%" | "%=~%" | "%==%" | "%prop%" | "%~%"
        | "%subset%" | "%subseteq%" | "%notsubset%" | "%supset%" | "%supseteq%" | "%in%"
        | "%notin%" | "%<->%" | "%->%" | "%<-%" | "%up%" | "%down%" | "%<=>%" | "%=>%" | "%<=%"
        | "%dblup%" | "%dbldown%" => (9, 10),
        ":" => (11, 12),
        "^" => (15, 14),
        _ => return None,
    })
}
struct Parser {
    tokens: Vec<Token>,
    index: usize,
}
impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }
    fn next(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        token
    }
    fn arguments(&mut self, close: char, depth: usize) -> ChartResult<Vec<MathNode>> {
        let mut args = Vec::new();
        if self.peek() == &Token::Close(close) {
            self.next();
            return Ok(args);
        }
        loop {
            args.push(self.expression(0, depth + 1)?);
            match self.next() {
                Token::Comma => {}
                Token::Close(c) if c == close => break,
                _ => return Err(invalid("Unclosed argument list or grouping.")),
            }
        }
        Ok(args)
    }
    fn expression(&mut self, min: u8, depth: usize) -> ChartResult<MathNode> {
        if depth > 64 {
            return Err(crate::scales::error(
                DiagnosticCode::ResourceLimit,
                "Math nesting exceeds depth budget.",
            ));
        }
        let mut left = match self.next() {
            Token::Name(s) => MathNode::Symbol(s),
            Token::Text(s) => MathNode::Text(s),
            Token::Number(s) => MathNode::Number(number_label(&s)),
            Token::Op(op) if matches!(op.as_str(), "+" | "-" | "!" | "~") => MathNode::Unary {
                operator: op,
                value: Box::new(self.expression(13, depth + 1)?),
            },
            Token::Open(c @ ('(' | '{')) => {
                let value = self.expression(0, depth + 1)?;
                if self.next() != Token::Close(if c == '(' { ')' } else { '}' }) {
                    return Err(invalid("Mismatched math grouping."));
                }
                MathNode::Group {
                    visible: c == '(',
                    value: Box::new(value),
                }
            }
            _ => return Err(invalid("Expected mathematical operand.")),
        };
        loop {
            if matches!(self.peek(), Token::Open('(' | '[')) {
                let Token::Open(open) = self.next() else {
                    unreachable!()
                };
                let args = self.arguments(if open == '(' { ')' } else { ']' }, depth + 1)?;
                left = if open == '(' {
                    MathNode::Call {
                        head: Box::new(left),
                        arguments: args,
                    }
                } else {
                    let script = if args.len() == 1 {
                        args.into_iter().next().unwrap()
                    } else {
                        MathNode::Call {
                            head: Box::new(MathNode::Symbol("list".into())),
                            arguments: args,
                        }
                    };
                    MathNode::Subscript {
                        base: Box::new(left),
                        script: Box::new(script),
                    }
                };
                continue;
            }
            let Token::Op(op) = self.peek() else { break };
            let Some((l, r)) = infix(op) else { break };
            if l < min {
                break;
            }
            let op = op.clone();
            self.next();
            let right = self.expression(r, depth + 1)?;
            left = if op == "^" {
                MathNode::Superscript {
                    base: Box::new(left),
                    script: Box::new(right),
                }
            } else {
                MathNode::Binary {
                    operator: op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            };
        }
        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_plotmath_syntax_inventory_parses_without_evaluation() {
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/plotmath-syntax-inventory.json"
        ))
        .unwrap();
        for row in v["syntax"].as_array().unwrap() {
            let source = row["syntax"].as_str().unwrap();
            if row["parseable"].as_bool() == Some(true) {
                assert!(
                    parse(source, Limits::default()).is_ok(),
                    "{source}: {:?}",
                    parse(source, Limits::default())
                );
            }
        }
    }
    #[test]
    fn malformed_and_executable_syntax_rejects() {
        for source in [
            "",
            "x <- 1",
            "x; y",
            "x$y",
            "x +",
            "sqrt(",
            "x[",
            "x %bad% y",
            "1e999",
            "\"bad\\nline\"",
        ] {
            assert!(parse(source, Limits::default()).is_err(), "{source}");
        }
        let ast = parse("x^y + z", Limits::default()).unwrap();
        assert!(
            matches!(ast,MathNode::Binary{operator,left,..} if operator=="+" && matches!(*left,MathNode::Superscript{..}))
        );
    }
    #[test]
    fn replay_coherence_and_bounded_ast_reject_invalid_payloads() {
        let mut expression =
            MathExpression::parse("frac(x,y)", MathFonts::default(), Limits::default()).unwrap();
        expression.ast = MathNode::Symbol("x".into());
        assert!(expression.validate(Limits::default()).is_err());
        let long = "x+".repeat(100) + "x";
        let limits = Limits {
            max_text_bytes: 32,
            ..Limits::default()
        };
        assert!(MathExpression::parse(long, MathFonts::default(), limits).is_err());
        let nested = "sqrt(".repeat(70) + "x" + &")".repeat(70);
        assert!(MathExpression::parse(nested, MathFonts::default(), Limits::default()).is_err());
    }
    #[test]
    fn numeric_atoms_use_reference_value_formatting() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/plotmath-metrics.json"
        ))
        .unwrap();
        for case in fixture["number_controls"].as_array().unwrap() {
            assert_eq!(
                number_label(case["source"].as_str().unwrap()),
                case["label"].as_str().unwrap()
            );
        }
    }
}
