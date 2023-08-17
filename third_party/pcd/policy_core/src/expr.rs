use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::PolicyCarryingResult,
    types::{PrimitiveData, PrimitiveDataType},
};

/// Represents the index of the element it points to in the arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Node(pub usize);

impl Default for Node {
    fn default() -> Self {
        Node(usize::MAX)
    }
}

/// The aggregation type.
#[derive(Clone, Serialize, Deserialize)]
pub enum Aggregation {
    Min(Box<Expr>),
    Max(Box<Expr>),
    Sum(Box<Expr>),
    Mean(Box<Expr>),
}

impl Debug for Aggregation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Min(expr) => write!(f, "{expr:?}.min()"),
            Self::Max(expr) => write!(f, "{expr:?}.max()"),
            Self::Sum(expr) => write!(f, "{expr:?}.sum()"),
            Self::Mean(expr) => write!(f, "{expr:?}.mean()"),
        }
    }
}

impl Display for Aggregation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GroupByMethod {
    Min,
    Max,
    Median,
    Mean,
    First,
    Last,
    Sum,
    Count,
}

#[derive(Clone, Debug)]
pub enum AAggExpr {
    Min { input: Node, propagate_nans: bool },
    Max { input: Node, propagate_nans: bool },
    Sum(Node),
    Mean(Node),
}

impl From<AAggExpr> for GroupByMethod {
    fn from(value: AAggExpr) -> Self {
        match value {
            AAggExpr::Max { .. } => Self::Max,
            AAggExpr::Min { .. } => Self::Min,
            AAggExpr::Sum { .. } => Self::Sum,
            AAggExpr::Mean { .. } => Self::Mean,
        }
    }
}

impl GroupByMethod {
    pub fn need_coerce(&self) -> bool {
        matches!(self, Self::Sum)
    }
}

impl AAggExpr {
    pub fn get_input(&self) -> &Node {
        match self {
            Self::Min { input, .. }
            | Self::Max { input, .. }
            | Self::Sum(input)
            | Self::Mean(input) => input,
        }
    }
}

impl Aggregation {
    pub fn as_expr(&self) -> &Expr {
        match self {
            Self::Min(expr) | Self::Max(expr) | Self::Sum(expr) | Self::Mean(expr) => expr,
        }
    }
}

/// This enum tells the executor how to perform the distinct operation.
///
/// # Examples
///
/// ```
/// # use policy_core::expr::Keep;
/// # use policy_carrying_data::*;
///
/// let keep = Keep::Any;
/// let expr = unique(Some(vec!["foo".into(), "bar".into()]), keep);
///
/// println!("expr => {expr:?}");
///
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Keep {
    /// Keeps only the first one of all the duplicate rows.
    First,
    /// Keeps only the last one of all the duplicate rows.
    Last,
    /// Keeps only one of all the duplicate rows, but the order does not matter at all.
    Any,
    /// Removes all duplicate rows.
    Remove,
}

/// Some additional options of the distinct operation. This struct will be heavily used in execution module.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistinctOptions {
    pub keep: Keep,
    /// The columns used to perform the `unique` operation.
    /// Equivalent to `DISTINCT([col_names]? | *)`
    pub selected_columns: Option<Vec<String>>,
    /// Should the result maintains the original order of all rows.
    pub maintain_order: bool,
    /// Should the result contains other columns.
    pub include_non_selected: bool,
    /// Truncated `distinct` only returns the sliced result where the first `i64` is the
    /// starting index in the original data frame and `usize` indicates the length.
    pub sliced: Option<(usize, usize)>,
}

/// An expression type for describing a node in the query.
#[derive(Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Aggregation.
    Agg(Aggregation),
    /// Select a vector of column names.
    Column(String),
    /// Count expression.
    Count,
    /// Making alias.
    Alias {
        expr: Box<Expr>,
        name: String,
    },
    /// "*".
    Wildcard,
    /// Exclude some columns.
    Exclude(Box<Expr>, Vec<String>),
    /// Filter.
    Filter {
        input: Box<Expr>,
        filter: Box<Expr>,
    },
    /// Binary operations
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Literal(Box<dyn PrimitiveDataType>),
}

/// AExpr is an arena-ed version of [`Expr`].
#[derive(Clone, Debug, Default)]
pub enum AExpr {
    Alias(Node, String),
    Column(String),
    Count,
    Literal(Box<dyn PrimitiveDataType>),
    BinaryOp {
        left: Node,
        op: BinaryOp,
        right: Node,
    },
    Filter {
        input: Node,
        by: Node,
    },
    Agg(AAggExpr),
    #[default]
    Wildcard,
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agg(agg) => write!(f, "{agg:?}"),
            Self::Column(column) => write!(f, "col({column})"),
            Self::Count => write!(f, "COUNT"),
            Self::Wildcard => write!(f, "*"),
            Self::Alias { expr, name } => write!(f, "ALIAS {expr:?} -> {name}"),
            Self::Exclude(expr, columns) => write!(f, "{expr:?} EXCEPT {columns:?}"),
            Self::Filter {
                input: data,
                filter,
            } => write!(f, "{data:?} WHERE {filter:?}"),
            Self::BinaryOp { left, op, right } => write!(f, "({left:?} {op:?} {right:?})"),
            Self::Literal(val) => write!(f, "{val:?}"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Add,
    Sub,
    Mul,
    Div,
}

impl Debug for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Le => write!(f, "<="),
            Self::Ge => write!(f, ">="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Xor => write!(f, "^"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "<>"),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct ExprIterator<'a> {
    stack: Vec<&'a Expr>,
}

impl<'a> Iterator for ExprIterator<'a> {
    type Item = &'a Expr;

    /// Visit the expression tree.
    fn next(&mut self) -> Option<Self::Item> {
        let current_expr = self.stack.pop();

        match current_expr {
            Some(current_expr) => {
                match current_expr {
                    Expr::Wildcard
                    | Expr::Count
                    | Expr::Column(_)
                    | Expr::Literal(_)
                    | Expr::Alias { .. } => None,
                    Expr::Agg(agg) => Some(agg.as_expr()),
                    Expr::BinaryOp { left, right, .. } => {
                        // Push left and right but return the current one.
                        self.stack.push(right);
                        self.stack.push(left);
                        Some(current_expr)
                    }
                    Expr::Exclude(expr, _) => {
                        self.stack.push(expr);
                        Some(current_expr)
                    }
                    Expr::Filter {
                        input: data,
                        filter,
                    } => {
                        self.stack.push(filter);
                        self.stack.push(data);
                        Some(current_expr)
                    }
                }
            }
            None => None,
        }
    }
}

impl<'a> IntoIterator for &'a Expr {
    type Item = &'a Expr;
    type IntoIter = ExprIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = Vec::with_capacity(8);
        stack.push(self);
        Self::IntoIter { stack }
    }
}

impl Expr {
    /// Applies a function `f` on the expressiom; ignore error.
    pub fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self) -> bool,
    {
        let _ = self.try_apply(|expr| Ok(f(expr)));
    }

    /// Tries to pply a function `f` on the expression.
    pub fn try_apply<F>(&mut self, mut f: F) -> PolicyCarryingResult<()>
    where
        F: FnMut(&mut Self) -> PolicyCarryingResult<bool>,
    {
        let mut stack = Vec::with_capacity(8);
        stack.push(self);

        while let Some(node) = stack.pop() {
            if !f(node)? {
                break;
            }

            match node {
                Expr::Wildcard
                | Expr::Column(_)
                | Expr::Count
                | Expr::Literal(_)
                | Expr::Agg(_)
                | Expr::Alias { .. } => (),
                Expr::BinaryOp { left, right, .. } => {
                    // Push left and right but return the current one.
                    stack.push(right);
                    stack.push(left);
                }
                Expr::Exclude(expr, _) => {
                    stack.push(expr);
                }
                Expr::Filter {
                    input: data,
                    filter,
                } => {
                    stack.push(filter);
                    stack.push(data);
                }
            }
        }

        Ok(())
    }

    pub fn exclude(self, columns: Vec<String>) -> Self {
        Self::Exclude(Box::new(self), columns)
    }

    pub fn lt<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Lt,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn le<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Le,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn gt<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Gt,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn ge<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Ge,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn eq<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Eq,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn ne<T: PrimitiveData>(self, num: T) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Ne,
            right: Box::new(Self::Literal(Box::new(num))),
        }
    }

    pub fn and(self, other: Self) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::And,
            right: Box::new(other),
        }
    }

    pub fn or(self, other: Self) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Or,
            right: Box::new(other),
        }
    }

    pub fn xor(self, other: Self) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOp::Xor,
            right: Box::new(other),
        }
    }

    pub fn sum(self) -> Self {
        Self::Agg(Aggregation::Sum(Box::new(self)))
    }

    pub fn max(self) -> Self {
        Self::Agg(Aggregation::Max(Box::new(self)))
    }

    pub fn min(self) -> Self {
        Self::Agg(Aggregation::Min(Box::new(self)))
    }

    pub fn mean(self) -> Self {
        Self::Agg(Aggregation::Mean(Box::new(self)))
    }

    pub fn alias(self, alias: &str) -> Self {
        Self::Alias {
            expr: Box::new(self),
            name: alias.into(),
        }
    }
}

/// Constructs an vector of [`Expression::Column`] variant.
#[macro_export]
macro_rules! cols {
    ($($col:tt),*) => {{
        let mut vec = vec![];

        $(
            match $col {
                "*" => vec.push($crate::expr::Expr::Wildcard),
                _ => vec.push($crate::expr::Expr::Column(String::from($col))),
            }
        )*

        vec
    }};
}

#[macro_export]
macro_rules! col {
    ($col:tt) => {
        match $col {
            "*" => $crate::expr::Expr::Wildcard,
            _ => $crate::expr::Expr::Column(String::from($col)),
        }
    };
}

impl AExpr {
    pub fn is_leaf(&self) -> bool {
        matches!(self, AExpr::Column(_) | AExpr::Literal(_))
    }

    pub fn nodes<'a>(&'a self, container: &mut Vec<Node>) {
        let mut push = |e: &'a Node| container.push(*e);

        match self {
            Self::Wildcard | Self::Column(_) | Self::Literal(_) | Self::Count => return,
            Self::BinaryOp { left, right, .. } => {
                push(right);
                push(left);
            }
            Self::Filter { input, by } => {
                push(by);
                push(input);
            }
            Self::Alias(from, ..) => push(from),
            Self::Agg(agg) => match agg {
                AAggExpr::Max { input, .. }
                | AAggExpr::Min { input, .. }
                | AAggExpr::Mean(input)
                | AAggExpr::Sum(input) => push(input),
            },
        }
    }
}

/// Creates a count expression.
pub fn count() -> Expr {
    Expr::Count
}

#[cfg(test)]
mod test {

    #[test]
    fn test_visit() {
        let expr = (col!("some_column")
            .gt(100i8)
            .and(col!("some_column2").lt(123i8)))
        .or(col!("some_column3").lt(111i8));

        let expr = format!("{:#?}", expr);
        assert_eq!(
            r#"(((col(some_column) > 100) && (col(some_column2) < 123)) || (col(some_column3) < 111))"#,
            &expr
        );
    }
}
