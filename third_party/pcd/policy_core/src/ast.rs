//! The abstract syntax tree for the PDL.
//!
//! The structs defined in this module can be exported to `lalrpop`.

use crate::{
    expr::{Aggregation, BinaryOp, Expr},
    policy::Schema,
    types::DataType,
};

#[cfg(feature = "ast-serde")]
use serde::{Deserialize, Serialize};

/// Defines the type of the privacy scheme that should be applied.
#[cfg_attr(feature = "ast-serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PrivacyScheme {
    /// Differential privacy with epsilon and delta.
    DifferentialPrivacy(f64, f64),
    /// t-closesness with parameter `t`.
    TCloseness(usize),
}

/// Scheme for ensuring privacy.
#[cfg_attr(feature = "ast-serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Scheme {
    Redact,
    Filter(Box<Expr>),
    Op {
        ops: Vec<Aggregation>,
        privacy: PrivacyScheme,
    },

    /// s1 (and | or) s2.
    Binary {
        lhs: Box<Scheme>,
        binary_op: BinaryOp,
        rhs: Box<Scheme>,
    },
}

/// Policy clauses.
#[cfg_attr(feature = "ast-serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Clause {
    Allow {
        attribute_list: Vec<String>,
        scheme: Vec<Scheme>,
    },
    /// Deny access on a list of attributes.
    Deny(Vec<String>),
}

impl Clause {
    pub fn attribute_list_mut(&mut self) -> &mut Vec<String> {
        match self {
            Self::Allow { attribute_list, .. } | Self::Deny(attribute_list) => attribute_list,
        }
    }
}

/// The root node of the policy AST.
#[cfg_attr(feature = "ast-serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct Policy {
    name: String,
    schema: Schema,
    clause: Vec<Clause>,
}

impl Policy {
    #[inline]
    /// Constructs a policy node from the parsed AST sub-trees.
    pub fn new(name: String, schema: Schema, clause: Vec<Clause>) -> Self {
        Self {
            name,
            schema,
            clause,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn schema_mut(&mut self) -> &mut Vec<(String, DataType)> {
        &mut self.schema
    }

    pub fn schema(&self) -> &[(String, DataType)] {
        self.schema.as_ref()
    }

    pub fn clause(&self) -> &[Clause] {
        self.clause.as_ref()
    }

    /// Performs the postprocessing that removes duplications.
    pub fn postprocess(&mut self) {}

    pub fn clause_mut(&mut self) -> &mut Vec<Clause> {
        &mut self.clause
    }
}
