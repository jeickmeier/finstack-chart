//! Host expressions retain their stage type and delegate every operation to the core graph.
use super::*;
use crate::grammar::{
    AfterScaleRead, BinField, Expression, ExpressionBinary as B, ExpressionReduce as R,
    ExpressionUnary as U, StatField,
};
#[derive(Clone)]
pub(super) enum Expr {
    Source(Expression<Mapping>),
    Stat(Expression<StatField>),
    Bin(Expression<BinField>),
    Scale(Expression<AfterScaleRead>),
}
fn binary(name: &str) -> Option<B> {
    Some(match name {
        "add" => B::Add,
        "sub" => B::Subtract,
        "mul" => B::Multiply,
        "div" => B::Divide,
        "pow" => B::Power,
        "less" => B::Less,
        "equal" => B::Equal,
        "and" => B::And,
        "or" => B::Or,
        "coalesce" => B::Coalesce,
        "concat" => B::Concat,
        _ => return None,
    })
}
fn set<T: Clone>(expr: &Expression<T>, name: &str, args: &Args) -> ChartResult<Expression<T>> {
    if let Some(op) = binary(name) {
        return Ok(expr.clone().binary(op, Expression::constant(args.one()?)));
    }
    let unary = match name {
        "negate" => Some(U::Negate),
        "abs" => Some(U::Abs),
        "sqrt" => Some(U::Sqrt),
        "log" => Some(U::Log),
        "log10" => Some(U::Log10),
        "exp" => Some(U::Exp),
        "floor" => Some(U::Floor),
        "ceil" => Some(U::Ceil),
        "not" => Some(U::Not),
        "is_missing" => Some(U::IsMissing),
        _ => None,
    };
    if let Some(op) = unary {
        args.count(0)?;
        return Ok(expr.clone().unary(op));
    }
    let op = match name {
        "sum" => R::Sum,
        "mean" => R::Mean,
        "min" => R::Min,
        "max" => R::Max,
        "count" => R::Count,
        _ => return Err(unsupported(name)),
    };
    Ok(expr.clone().reduce(
        op,
        if args.0.is_empty() {
            false
        } else {
            args.one()?
        },
    ))
}
impl Expr {
    fn bounded(self) -> ChartResult<Self> {
        let count = match &self {
            Self::Source(e) => e.nodes.len(),
            Self::Stat(e) => e.nodes.len(),
            Self::Bin(e) => e.nodes.len(),
            Self::Scale(e) => e.nodes.len(),
        };
        if count > crate::grammar::ExpressionLimits::default().max_nodes {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Expression builder exceeds its node budget.",
            ));
        }
        Ok(self)
    }
    pub(super) fn set(&self, method: &str, args: &Args) -> ChartResult<Self> {
        match self {
            Self::Source(e) => Self::Source(set(e, method, args)?),
            Self::Stat(e) => Self::Stat(set(e, method, args)?),
            Self::Bin(e) => Self::Bin(set(e, method, args)?),
            Self::Scale(e) => Self::Scale(set(e, method, args)?),
        }
        .bounded()
    }
    pub(super) fn with(&self, method: &str, other: &Self) -> ChartResult<Self> {
        let op = binary(method).ok_or_else(|| unsupported(method))?;
        match (self, other) {
            (Self::Source(a), Self::Source(b)) => Self::Source(a.clone().binary(op, b.clone())),
            (Self::Stat(a), Self::Stat(b)) => Self::Stat(a.clone().binary(op, b.clone())),
            (Self::Bin(a), Self::Bin(b)) => Self::Bin(a.clone().binary(op, b.clone())),
            (Self::Scale(a), Self::Scale(b)) => Self::Scale(a.clone().binary(op, b.clone())),
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Expression operands belong to different evaluation stages.",
                ));
            }
        }
        .bounded()
    }
}
