use std::sync::Arc;

use policy_carrying_data::DataFrame;
use policy_core::{error::PolicyCarryingError, types::FunctionArguments};

use crate::plan::physical_expr::PhysicalExpr;

/// Producer of an in memory [`DataFrame`]. This should be the deepmost executor that cannot be dependent on any
/// other executors because the data must eventually come from data frame.
#[derive(Clone)]
pub struct DataFrameExec {
    /// The id of the api set.
    pub df: Option<Arc<DataFrame>>,
    /// This is the predicate.
    pub selection: Option<Arc<dyn PhysicalExpr>>,
    /// This is the 'select' action; one should not be confused with its name.
    pub projection: Option<Arc<Vec<String>>>,
    /// [WIP]: Window function.
    pub predicate_has_windows: bool,
}

impl DataFrameExec {
    pub fn new(
        df: Option<Arc<DataFrame>>,
        selection: Option<Arc<dyn PhysicalExpr>>,
        projection: Option<Arc<Vec<String>>>,
        predicate_has_windows: bool,
    ) -> Self {
        Self {
            df,
            selection,
            projection,
            predicate_has_windows,
        }
    }
}

impl TryFrom<FunctionArguments> for DataFrameExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let selection = args.get_and_apply("selection", |ptr: Option<usize>| {
            ptr.map(|ptr| unsafe { &*(ptr as *const Arc<dyn PhysicalExpr>) })
                .cloned()
        })?;
        let projection = args.get_and_apply("projection", |vec: Option<Vec<String>>| {
            vec.map(|inner| Arc::new(inner))
        })?;
        let predicate_has_windows = args.get_and_apply("predicate_has_windows", |b: bool| b)?;

        Ok(Self::new(
            // Will be loaded later.
            None,
            selection,
            projection,
            predicate_has_windows,
        ))
    }
}
