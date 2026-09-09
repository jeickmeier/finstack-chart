//! Portable stage-typed expressions. Programs contain data, never executable host code.
use super::error;
use crate::{ChartResult, DiagnosticCode, color::Paint};

/// Scalar type checked before expression evaluation, including empty populations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionType {
    /// Finite numeric value.
    Number,
    /// Logical value.
    Boolean,
    /// UTF-8 text.
    Text,
    /// Unpremultiplied sRGB color.
    Color,
}
/// One expression value; missing values propagate unless explicitly replaced or removed.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExpressionValue {
    /// Typed missing value.
    Missing(ExpressionType),
    /// Finite numeric value.
    Number(f64),
    /// Logical value.
    Boolean(bool),
    /// UTF-8 text.
    Text(String),
    /// Exact sRGB value.
    Color(Paint),
}
impl ExpressionValue {
    /// Physical scalar type, retained for missing values.
    pub fn kind(&self) -> ExpressionType {
        match self {
            Self::Missing(kind) => *kind,
            Self::Number(_) => ExpressionType::Number,
            Self::Boolean(_) => ExpressionType::Boolean,
            Self::Text(_) => ExpressionType::Text,
            Self::Color(_) => ExpressionType::Color,
        }
    }
    /// Read a finite numeric result, preserving missingness.
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(v) if v.is_finite() => Some(*v),
            _ => None,
        }
    }
    fn numeric(value: f64) -> Self {
        if value.is_finite() {
            Self::Number(value)
        } else {
            Self::Missing(ExpressionType::Number)
        }
    }
}
/// Unary operations with explicit domains; invalid numeric results become missing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionUnary {
    /// Arithmetic negation.
    Negate,
    /// Absolute value.
    Abs,
    /// Nonnegative square root.
    Sqrt,
    /// Natural logarithm.
    Log,
    /// Base-ten logarithm.
    Log10,
    /// Exponential.
    Exp,
    /// Floor.
    Floor,
    /// Ceiling.
    Ceil,
    /// Logical negation.
    Not,
    /// Missingness test, always nonmissing.
    IsMissing,
}
/// Binary scalar operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionBinary {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division; zero denominators become missing.
    Divide,
    /// Real exponentiation.
    Power,
    /// Numeric less-than comparison.
    Less,
    /// Same-type equality.
    Equal,
    /// Logical conjunction, with three-valued missingness.
    And,
    /// Logical disjunction, with three-valued missingness.
    Or,
    /// Replace missing left values with right values of the same type.
    Coalesce,
    /// Concatenate two strings.
    Concat,
}
/// Column reduction over the explicitly supplied stage population.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionReduce {
    /// Sum; empty input returns zero.
    Sum,
    /// Arithmetic mean; empty input is missing.
    Mean,
    /// Minimum; empty input is missing.
    Min,
    /// Maximum; empty input is missing.
    Max,
    /// Number of eligible values.
    Count,
}
/// One node in a topologically ordered graph; references must precede their consumer.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExpressionNode<R> {
    /// Stage-specific typed read.
    Read(R),
    /// Explicit scalar literal, broadcast over the stage population.
    Literal(ExpressionValue),
    /// Unary operation.
    Unary {
        /// Operation.
        op: ExpressionUnary,
        /// Preceding input node index.
        input: usize,
    },
    /// Binary operation.
    Binary {
        /// Operation.
        op: ExpressionBinary,
        /// Preceding left node index.
        left: usize,
        /// Preceding right node index.
        right: usize,
    },
    /// Conditional selection; a missing condition produces a missing result.
    Select {
        /// Boolean node index.
        condition: usize,
        /// True branch node index.
        yes: usize,
        /// False branch node index.
        no: usize,
    },
    /// Reduce the input column and broadcast the result.
    Reduce {
        /// Reduction.
        op: ExpressionReduce,
        /// Preceding numeric node index.
        input: usize,
        /// Explicitly discard missing values before reducing.
        remove_missing: bool,
    },
}
/// Portable program whose read type determines its legal evaluation stage.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expression<R> {
    /// Topologically ordered, bounded nodes.
    pub nodes: Vec<ExpressionNode<R>>,
    /// Index of the result node.
    pub output: usize,
}
/// Per-program bounds, checked before allocating intermediate columns.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionLimits {
    /// Maximum nodes, including unreachable nodes that must still validate.
    pub max_nodes: usize,
    /// Maximum node-by-row cells, bounding work and retained intermediate values.
    pub max_cells: usize,
    /// Maximum total string bytes produced, including intermediate values.
    pub max_text_bytes: usize,
}
impl Default for ExpressionLimits {
    fn default() -> Self {
        Self {
            max_nodes: 256,
            max_cells: 1_000_000,
            max_text_bytes: 1_000_000,
        }
    }
}
impl<R> Expression<R> {
    /// One typed read.
    pub fn read(read: R) -> Self {
        Self {
            nodes: vec![ExpressionNode::Read(read)],
            output: 0,
        }
    }
    /// One numeric literal.
    pub fn constant(value: f64) -> Self {
        Self {
            nodes: vec![ExpressionNode::Literal(ExpressionValue::Number(value))],
            output: 0,
        }
    }
    /// Append a unary operation to the current output.
    pub fn unary(mut self, op: ExpressionUnary) -> Self {
        self.nodes.push(ExpressionNode::Unary {
            op,
            input: self.output,
        });
        self.output = self.nodes.len() - 1;
        self
    }
    /// Append a column reduction with an explicit missing-value policy.
    pub fn reduce(mut self, op: ExpressionReduce, remove_missing: bool) -> Self {
        self.nodes.push(ExpressionNode::Reduce {
            op,
            input: self.output,
            remove_missing,
        });
        self.output = self.nodes.len() - 1;
        self
    }
    /// Resolve read identities without changing the graph or its stage semantics.
    pub fn try_map_reads<T>(
        &self,
        mut map: impl FnMut(&R) -> ChartResult<T>,
    ) -> ChartResult<Expression<T>> {
        use ExpressionNode as N;
        let nodes = self
            .nodes
            .iter()
            .map(|n| {
                Ok(match n {
                    N::Read(r) => N::Read(map(r)?),
                    N::Literal(v) => N::Literal(v.clone()),
                    N::Unary { op, input } => N::Unary {
                        op: *op,
                        input: *input,
                    },
                    N::Binary { op, left, right } => N::Binary {
                        op: *op,
                        left: *left,
                        right: *right,
                    },
                    N::Select { condition, yes, no } => N::Select {
                        condition: *condition,
                        yes: *yes,
                        no: *no,
                    },
                    N::Reduce {
                        op,
                        input,
                        remove_missing,
                    } => N::Reduce {
                        op: *op,
                        input: *input,
                        remove_missing: *remove_missing,
                    },
                })
            })
            .collect::<ChartResult<_>>()?;
        Ok(Expression {
            nodes,
            output: self.output,
        })
    }
    /// Validate all nodes and infer the result type without reading any row.
    pub fn validate(
        &self,
        limits: ExpressionLimits,
        mut read_type: impl FnMut(&R) -> ChartResult<ExpressionType>,
    ) -> ChartResult<ExpressionType> {
        use ExpressionNode as N;
        use ExpressionType as T;
        if self.nodes.is_empty() || self.nodes.len() > limits.max_nodes {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Expression node budget exceeded or program is empty.",
            ));
        }
        let mut types = Vec::with_capacity(self.nodes.len());
        let mismatch = || {
            error(
                DiagnosticCode::SchemaConflict,
                "Expression operand types are incompatible.",
            )
        };
        for node in &self.nodes {
            let get = |id: usize| {
                types.get(id).copied().ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Expression contains a cycle, forward reference or absent node.",
                    )
                })
            };
            let kind = match node {
                N::Read(r) => read_type(r)?,
                N::Literal(v) => {
                    if matches!(v,ExpressionValue::Number(n) if !n.is_finite()) {
                        return Err(error(
                            DiagnosticCode::NumericalDomain,
                            "Expression literals must be finite.",
                        ));
                    }
                    if matches!(v,ExpressionValue::Text(s) if s.len()>limits.max_text_bytes) {
                        return Err(error(
                            DiagnosticCode::ResourceLimit,
                            "Expression literal text budget exceeded.",
                        ));
                    }
                    v.kind()
                }
                N::Unary { op, input } => {
                    let k = get(*input)?;
                    match op {
                        ExpressionUnary::IsMissing => T::Boolean,
                        ExpressionUnary::Not if k == T::Boolean => T::Boolean,
                        ExpressionUnary::Not => return Err(mismatch()),
                        _ if k == T::Number => T::Number,
                        _ => return Err(mismatch()),
                    }
                }
                N::Binary { op, left, right } => {
                    let a = get(*left)?;
                    let b = get(*right)?;
                    if a != b {
                        return Err(mismatch());
                    }
                    match op {
                        ExpressionBinary::Coalesce => a,
                        ExpressionBinary::Equal => T::Boolean,
                        ExpressionBinary::Concat if a == T::Text => T::Text,
                        ExpressionBinary::And | ExpressionBinary::Or if a == T::Boolean => {
                            T::Boolean
                        }
                        ExpressionBinary::Less if a == T::Number => T::Boolean,
                        ExpressionBinary::Add
                        | ExpressionBinary::Subtract
                        | ExpressionBinary::Multiply
                        | ExpressionBinary::Divide
                        | ExpressionBinary::Power
                            if a == T::Number =>
                        {
                            T::Number
                        }
                        _ => return Err(mismatch()),
                    }
                }
                N::Select { condition, yes, no } => {
                    let k = get(*yes)?;
                    if get(*condition)? != T::Boolean || get(*no)? != k {
                        return Err(mismatch());
                    }
                    k
                }
                N::Reduce { input, .. } => {
                    if get(*input)? != T::Number {
                        return Err(mismatch());
                    }
                    T::Number
                }
            };
            types.push(kind);
        }
        types.get(self.output).copied().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Expression output node is absent.",
            )
        })
    }
}
impl<R> Expression<R> {
    /// Evaluate once over a stage population, retaining no callbacks or source references.
    pub fn evaluate(
        &self,
        rows: usize,
        limits: ExpressionLimits,
        read_type: impl FnMut(&R) -> ChartResult<ExpressionType>,
        read: impl FnMut(&R, usize) -> ExpressionValue,
    ) -> ChartResult<Vec<ExpressionValue>> {
        let mut columns = self.evaluate_columns(rows, limits, read_type, read)?;
        Ok(std::mem::take(&mut columns[self.output]))
    }
    fn evaluate_columns(
        &self,
        rows: usize,
        limits: ExpressionLimits,
        mut read_type: impl FnMut(&R) -> ChartResult<ExpressionType>,
        mut read: impl FnMut(&R, usize) -> ExpressionValue,
    ) -> ChartResult<Vec<Vec<ExpressionValue>>> {
        use ExpressionNode as N;
        self.validate(limits, &mut read_type)?;
        let kinds = self
            .nodes
            .iter()
            .map(|node| match node {
                N::Read(r) => read_type(r).map(Some),
                _ => Ok(None),
            })
            .collect::<ChartResult<Vec<_>>>()?;
        if self
            .nodes
            .len()
            .checked_mul(rows.max(1))
            .is_none_or(|n| n > limits.max_cells)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Expression population work budget exceeded.",
            ));
        }
        let mut columns: Vec<Vec<ExpressionValue>> = Vec::with_capacity(self.nodes.len());
        let mut text_bytes = 0usize;
        for (node_index, node) in self.nodes.iter().enumerate() {
            let reduced = if let N::Reduce {
                op,
                input,
                remove_missing,
            } = node
            {
                Some(reduce(*op, &columns[*input], *remove_missing))
            } else {
                None
            };
            let literal = match node {
                N::Literal(ExpressionValue::Color(p)) => {
                    Some(ExpressionValue::Color(p.resolve().into()))
                }
                N::Literal(v) => Some(v.clone()),
                _ => None,
            };
            let mut column = Vec::with_capacity(rows);
            #[expect(
                clippy::needless_range_loop,
                reason = "Rows index every operand column; the outer columns vector is indexed by graph nodes, not rows."
            )]
            for row in 0..rows {
                let value = match node {
                    N::Read(r) => {
                        let value = read(r, row);
                        if Some(value.kind()) != kinds[node_index] {
                            return Err(error(
                                DiagnosticCode::SchemaConflict,
                                "Expression read value does not match its declared type.",
                            ));
                        }
                        match value {
                            ExpressionValue::Number(v) => ExpressionValue::numeric(v),
                            _ => value,
                        }
                    }
                    N::Literal(_) => literal.as_ref().expect("literal prepared").clone(),
                    N::Unary { op, input } => unary(*op, &columns[*input][row]),
                    N::Binary { op, left, right } => {
                        binary(*op, &columns[*left][row], &columns[*right][row])
                    }
                    N::Select { condition, yes, no } => match columns[*condition][row] {
                        ExpressionValue::Boolean(true) => columns[*yes][row].clone(),
                        ExpressionValue::Boolean(false) => columns[*no][row].clone(),
                        _ => ExpressionValue::Missing(columns[*yes][row].kind()),
                    },
                    N::Reduce { .. } => reduced.as_ref().expect("reduction computed").clone(),
                };
                if let ExpressionValue::Text(s) = &value {
                    text_bytes = text_bytes.checked_add(s.len()).ok_or_else(|| {
                        error(
                            DiagnosticCode::ResourceLimit,
                            "Expression text budget overflow.",
                        )
                    })?;
                    if text_bytes > limits.max_text_bytes {
                        return Err(error(
                            DiagnosticCode::ResourceLimit,
                            "Expression text budget exceeded.",
                        ));
                    }
                }
                column.push(value);
            }
            columns.push(column);
        }
        Ok(columns)
    }
    /// Combine two independently authored programs without introducing shared mutable state.
    pub fn binary(mut self, op: ExpressionBinary, mut right: Self) -> Self {
        let offset = self.nodes.len();
        for node in &mut right.nodes {
            match node {
                ExpressionNode::Unary { input, .. } | ExpressionNode::Reduce { input, .. } => {
                    *input = input.saturating_add(offset)
                }
                ExpressionNode::Binary { left, right, .. } => {
                    *left = left.saturating_add(offset);
                    *right = right.saturating_add(offset);
                }
                ExpressionNode::Select { condition, yes, no } => {
                    *condition = condition.saturating_add(offset);
                    *yes = yes.saturating_add(offset);
                    *no = no.saturating_add(offset);
                }
                _ => {}
            }
        }
        self.nodes.extend(right.nodes);
        self.nodes.push(ExpressionNode::Binary {
            op,
            left: self.output,
            right: right.output.saturating_add(offset),
        });
        self.output = self.nodes.len() - 1;
        self
    }
}
fn unary(op: ExpressionUnary, input: &ExpressionValue) -> ExpressionValue {
    use ExpressionUnary as U;
    use ExpressionValue as V;
    if op == U::IsMissing {
        return V::Boolean(matches!(input, V::Missing(_)));
    }
    if op == U::Not {
        return match input {
            V::Boolean(v) => V::Boolean(!v),
            _ => V::Missing(ExpressionType::Boolean),
        };
    }
    let Some(v) = input.number() else {
        return V::Missing(ExpressionType::Number);
    };
    V::numeric(match op {
        U::Negate => -v,
        U::Abs => v.abs(),
        U::Sqrt => v.sqrt(),
        U::Log => v.ln(),
        U::Log10 => v.log10(),
        U::Exp => v.exp(),
        U::Floor => v.floor(),
        U::Ceil => v.ceil(),
        U::Not | U::IsMissing => unreachable!(),
    })
}
fn binary(
    op: ExpressionBinary,
    left: &ExpressionValue,
    right: &ExpressionValue,
) -> ExpressionValue {
    use ExpressionBinary as B;
    use ExpressionValue as V;
    if op == B::Coalesce {
        return if matches!(left, V::Missing(_)) {
            right.clone()
        } else {
            left.clone()
        };
    }
    if matches!(op, B::And | B::Or) {
        let decisive = op == B::Or;
        if [left, right]
            .iter()
            .any(|v| matches!(v,V::Boolean(b) if *b==decisive))
        {
            return V::Boolean(decisive);
        }
        return match (left, right) {
            (V::Boolean(a), V::Boolean(b)) => {
                V::Boolean(if op == B::And { *a && *b } else { *a || *b })
            }
            _ => V::Missing(ExpressionType::Boolean),
        };
    }
    let kind = match op {
        B::Equal | B::Less => ExpressionType::Boolean,
        B::Concat => ExpressionType::Text,
        _ => ExpressionType::Number,
    };
    if matches!(left, V::Missing(_)) || matches!(right, V::Missing(_)) {
        return V::Missing(kind);
    }
    if op == B::Equal {
        return V::Boolean(left == right);
    }
    if let (B::Concat, V::Text(a), V::Text(b)) = (op, left, right) {
        return V::Text(format!("{a}{b}"));
    }
    let (Some(a), Some(b)) = (left.number(), right.number()) else {
        return V::Missing(kind);
    };
    if op == B::Less {
        return V::Boolean(a < b);
    }
    V::numeric(match op {
        B::Add => a + b,
        B::Subtract => a - b,
        B::Multiply => a * b,
        B::Divide => a / b,
        B::Power => a.powf(b),
        _ => unreachable!("validated scalar operation"),
    })
}
fn reduce(
    op: ExpressionReduce,
    values: &[ExpressionValue],
    remove_missing: bool,
) -> ExpressionValue {
    use ExpressionReduce as R;
    use ExpressionValue as V;
    let mut count = 0usize;
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for value in values {
        let Some(v) = value.number() else {
            if !remove_missing {
                return V::Missing(ExpressionType::Number);
            }
            continue;
        };
        count += 1;
        minimum = minimum.min(v);
        maximum = maximum.max(v);
    }
    V::numeric(match op {
        R::Count => count as f64,
        R::Sum => super::statistics::sum(values.iter().filter_map(ExpressionValue::number))
            .unwrap_or(f64::NAN),
        R::Mean if count > 0 => {
            super::statistics::mean(values.iter().filter_map(ExpressionValue::number), count)
                .unwrap_or(f64::NAN)
        }
        R::Min => minimum,
        R::Max => maximum,
        R::Mean => f64::NAN,
    })
}
macro_rules! arithmetic {
    ($trait:ident,$method:ident,$op:ident) => {
        impl<R> std::ops::$trait for Expression<R> {
            type Output = Self;
            fn $method(self, other: Self) -> Self {
                self.binary(ExpressionBinary::$op, other)
            }
        }
        impl<R> std::ops::$trait<f64> for Expression<R> {
            type Output = Self;
            fn $method(self, other: f64) -> Self {
                self.binary(ExpressionBinary::$op, Self::constant(other))
            }
        }
    };
}
arithmetic!(Add, add, Add);
arithmetic!(Sub, sub, Subtract);
arithmetic!(Mul, mul, Multiply);
arithmetic!(Div, div, Divide);

impl<R: Clone> Expression<R> {
    /// Evaluate column reductions once, then retain a scalar program for each source row.
    pub(crate) fn specialize_reductions(
        &self,
        rows: usize,
        limits: ExpressionLimits,
        read_type: impl FnMut(&R) -> ChartResult<ExpressionType>,
        read: impl FnMut(&R, usize) -> ExpressionValue,
    ) -> ChartResult<Self> {
        let columns = self.evaluate_columns(rows, limits, read_type, read)?;
        let mut result = self.clone();
        for node in &mut result.nodes {
            if let ExpressionNode::Reduce {
                op,
                input,
                remove_missing,
            } = node
            {
                *node = ExpressionNode::Literal(reduce(*op, &columns[*input], *remove_missing));
            }
        }
        Ok(result)
    }
}
/// Source-stage numeric access; generated fields cannot be supplied here.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SourceRead {
    /// Numeric source column, with exact integer-to-float checks.
    Field(crate::FieldId),
    /// Timestamp relative to an exact integer origin in the declared source units.
    Timestamp {
        /// Source column.
        field: crate::FieldId,
        /// Integer origin.
        #[serde(with = "crate::portable::signed")]
        origin: i64,
    },
}
impl SourceRead {
    pub(crate) fn numeric(&self) -> super::Numeric {
        match self {
            Self::Field(f) => super::Numeric::Field(*f),
            Self::Timestamp { field, origin } => super::Numeric::Timestamp {
                field: *field,
                origin: *origin,
            },
        }
    }
}
