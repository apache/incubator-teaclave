//! The lazy data frame module.

use std::fmt::{Debug, Formatter};

use policy_carrying_data::{schema::SchemaRef, DataFrame, JoinType};
use policy_core::{
    col,
    error::PolicyCarryingResult,
    expr::{DistinctOptions, Expr, Keep},
    types::ExecutorRefId,
};

use crate::{
    executor::{self, execution_epilogue},
    plan::{make_physical_plan, LogicalPlan, OptFlag, PlanBuilder},
};

#[derive(Clone)]
#[must_use = "LazyFrame must be consumed"]
pub struct LazyFrame {
    /// The executor ID for loading the library.
    /// FIXME: Multiple reference id like join.
    pub(crate) executor_ref_id: Option<ExecutorRefId>,
    /// The logical plan.
    pub(crate) plan: LogicalPlan,
    /// The optimization flag.
    pub(crate) opt_flag: OptFlag,
}

/// Utility struct for lazy groupby operation.
#[derive(Clone)]
#[must_use = "LazyGroupBy must be consumed"]
pub struct LazyGroupBy {
    /// The executor ID for loading the library.
    pub(crate) executor_ref_id: Option<ExecutorRefId>,
    /// The logical plan.
    pub(crate) plan: LogicalPlan,
    /// The optimization flag.
    pub(crate) opt_flag: OptFlag,
    /// The `group_by` keys.
    pub(crate) keys: Vec<Expr>,
    /// Whether we need to maintain order during evaluation.
    pub(crate) maintain_order: bool,
}

/// A trait for converting an object into a lazily evaluated data frame.
pub trait IntoLazy {
    /// Converts the target into [`LazyFrame`]. This does not take the ownership.
    fn lazy(&self) -> LazyFrame;
}

impl Debug for LazyFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?} with {:?}", &self.plan, &self.opt_flag)
    }
}

impl LazyFrame {
    pub fn explain(&self) -> String {
        format!("{:?}", self.plan)
    }
    /// Returns the schema of the current lazy frame.
    #[inline]
    pub fn schema(&self) -> PolicyCarryingResult<SchemaRef> {
        self.plan.schema()
    }

    /// Returns the optimization flag.
    #[inline]
    pub fn opt_flag(&self) -> OptFlag {
        self.opt_flag
    }

    /// Performs the *projection* although this API name is `select`, possibly performing some
    /// transformations on the columns it selects.
    pub fn select<T: AsRef<[Expr]>>(self, expressions: T) -> Self {
        let plan = PlanBuilder::from(self.plan)
            // Select is, in fact, a projection.
            .projection(expressions.as_ref().to_vec())
            .finish();

        Self {
            executor_ref_id: self.executor_ref_id,
            plan,
            opt_flag: self.opt_flag,
        }
    }

    /// Performs a filtering operation.
    pub fn filter(self, expression: Expr) -> Self {
        let plan = PlanBuilder::from(self.plan).filter(expression).finish();

        Self {
            executor_ref_id: self.executor_ref_id,
            plan,
            opt_flag: self.opt_flag,
        }
    }

    /// Performs a `groupby` operations that paritions the original dataframe.
    pub fn groupby<T: AsRef<[Expr]>>(self, groupby: T) -> LazyGroupBy {
        LazyGroupBy {
            executor_ref_id: self.executor_ref_id,
            plan: self.plan,
            opt_flag: self.opt_flag,
            keys: groupby.as_ref().to_vec(),
            maintain_order: false,
        }
    }

    // +======================================================================================================+
    // | To unify the behavior of aggregation (and thus the executors) that may come without the `groupby`,   |
    // | we force to add an explicit `groupby NULL` clause into the expression. In other words, the following |
    // |     SELECT SUM(bar), SUM(baz), ... FROM foo;                                                         |
    // | is equivalent to                                                                                     |
    // |     SELECT SUM(bar), SUM(baz), ... FROM foo GROUP BY NULL;                                           |
    // +======================================================================================================+

    /// Sums up the lazy frame with a wildcard.
    #[deprecated = "aggregation must use new method that applies a `groupby`"]
    pub fn _sum(self) -> Self {
        self.select(vec![col!("*").sum()])
    }

    /// Sums up the lazy frame with a wildcard.
    pub fn sum(self) -> Self {
        self.groupby([]).agg([col!("*").sum()])
    }

    /// Gets the maximum value.
    #[deprecated = "aggregation must use new method that applies a `groupby`"]
    pub fn _max(self) -> Self {
        self.select(vec![col!("*").max()])
    }

    /// Gets the maximum value.
    pub fn max(self) -> Self {
        self.groupby([]).agg([col!("*").max()])
    }

    /// Gets the minimum value.
    #[deprecated = "aggregation must use new method that applies a `groupby`"]
    pub fn _min(self) -> Self {
        self.select(vec![col!("*").min()])
    }

    /// Gets the minimum value.
    pub fn min(self) -> Self {
        self.groupby([]).agg([col!("*").min()])
    }

    /// Gets the mean value.
    #[deprecated = "aggregation must use new method that applies a `groupby`"]
    pub fn _mean(self) -> Self {
        self.select(vec![col!("*").mean()])
    }

    /// Gets the mean value.
    pub fn mean(self) -> Self {
        self.groupby([]).agg([col!("*").mean()])
    }

    /// Joins two lazy frame.
    ///
    /// # Examples
    ///
    /// ```
    /// # use policy_carrying_data::*;
    /// # use policy_execution::{lazy::*, context::*};
    /// # use policy_core::{types::*, *};
    ///
    /// let df = pcd! {
    ///     "foo" => DataType::Int8: [1i8, 2i8, 3i8],
    ///     "bar" => DataType::Int8: [1i8, 2i8, 3i8],
    /// };
    ///
    /// let mut ctx = AnalysisContext::new();
    /// ctx.register_data(df.into()).unwrap();
    /// let df = ctx.lazy().join(ctx.lazy().clone(), [col!("bar")], [col!("bar")], JoinType::Left).collect();
    /// println!("df => {df:?}");
    /// ```
    pub fn join<T: AsRef<[Expr]>>(
        self,
        other: Self,
        left_on: T,
        right_on: T,
        how: JoinType,
    ) -> Self {
        let left_on = left_on.as_ref().to_vec();
        let right_on = right_on.as_ref().to_vec();
        let lp = PlanBuilder::from(self.plan)
            .join(other.plan, left_on, right_on, how)
            .finish();

        Self {
            plan: lp,
            opt_flag: self.opt_flag | other.opt_flag,
            executor_ref_id: self.executor_ref_id,
        }
    }

    /// Performs the `DISTINCT` operation that retains unique rows and removes duplicate ones
    /// according to the given columns.
    ///
    /// # Examples
    ///
    /// Exclude other columns:
    /// 
    /// ```
    /// # use policy_carrying_data::pcd;
    /// # use policy_execution::{lazy::*, context::*};
    /// # use policy_core::{types::*, expr::Keep, *};
    ///
    /// let df = pcd! {
    ///     "foo" => DataType::Int8: [1i8, 2i8, 2i8],
    ///     "bar" => DataType::Int8: [1i8, 2i8, 3i8],
    /// };
    ///
    /// let mut ctx = AnalysisContext::new();
    /// ctx.register_data(df.into()).unwrap();
    /// let df = ctx.lazy().distinct_with(vec!["foo".into()], Keep::Any, false).collect();
    /// // remove any of the`2i8`.
    /// println!("df => {df:?}");
    /// ```
    /// 
    /// This is equivalent to the following SQL query:
    /// 
    /// ```sql
    /// SELECT DISTINCT(`foo`) FROM `df`
    /// ```
    /// 
    /// To include other columns, please call `distinct_with(vec!["foo"].into(), Keep::Any, true)`.
    /// 
    /// ```sql
    /// SELECT DISTINCT(`foo`), * FROM `df`
    /// ```
    pub fn distinct_with(
        self,
        selected_columns: Vec<String>,
        keep: Keep,
        include_non_selected: bool,
    ) -> Self {
        let options = DistinctOptions {
            keep,
            selected_columns: Some(selected_columns),
            sliced: None,
            include_non_selected,
            maintain_order: false,
        };

        self.distinct_with_options(options)
    }

    /// Performs the `DISTINCT` operation that retains unique rows and removes duplicate ones
    /// using *all* columns.
    ///
    /// # Examples
    ///
    /// ```
    /// # use policy_carrying_data::*;
    /// # use policy_execution::{lazy::*, context::*};
    /// # use policy_core::{types::*, *};
    ///
    /// let df = pcd! {
    ///     "foo" => DataType::Int8: [1i8, 2i8, 2i8],
    ///     "bar" => DataType::Int8: [1i8, 2i8, 3i8],
    /// };
    ///
    /// let mut ctx = AnalysisContext::new();
    /// ctx.register_data(df.into()).unwrap();
    /// let df = ctx.lazy().distinct_with(vec!["foo".into()], Keep::Any).collect();
    /// // nothing is removed.
    /// println!("df => {df:?}");
    /// ```
    pub fn distinct(self, keep: Keep) -> Self {
        let options = DistinctOptions {
            keep,
            selected_columns: None,
            sliced: None,
            include_non_selected: false,
            maintain_order: false,
        };

        self.distinct_with_options(options)
    }

    fn distinct_with_options(self, options: DistinctOptions) -> Self {
        let plan = LogicalPlan::Distinct {
            input: Box::new(self.plan),
            options,
        };

        Self {
            executor_ref_id: self.executor_ref_id,
            opt_flag: self.opt_flag,
            plan,
        }
    }

    /// Executes the query plan, checking the policy if the query can be executed or should be
    /// sanitized to meet the privacy constraints and finally returning the results.
    ///
    /// The execution works as follows.
    ///
    /// 1. Prepare the execution by optimizing and checking the query plan [`LogicalPlan`].
    /// 2. Prepare the physical query plan and gets the data.
    /// 3. Return the data which may be sanitized.
    #[must_use = "unused dafaframe must be used"]
    pub fn collect(self) -> PolicyCarryingResult<DataFrame> {
        // Generate a phyiscal plan.
        let (state, executor) = make_physical_plan(self.plan, self.opt_flag, self.executor_ref_id)?;
        let df = executor::execute(self.executor_ref_id, executor)?;

        execution_epilogue(df, &state)
    }
}

impl LazyGroupBy {
    /// Group by and aggregate.
    pub fn agg<T: AsRef<[Expr]>>(self, expr: T) -> LazyFrame {
        let plan = PlanBuilder::from(self.plan)
            .groupby(self.keys, expr, self.maintain_order)
            .finish();

        LazyFrame {
            executor_ref_id: self.executor_ref_id,
            plan,
            opt_flag: self.opt_flag,
        }
    }
}
