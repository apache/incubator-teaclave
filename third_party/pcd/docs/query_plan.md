# Query Plan and Optimizations

> **Warning**
>
> Optimizations are a WIP goal and may be implemented in the future but not now.

Internally, executing a query on a policy-carrying data will be split into several steps. First, the query plan is constructed lazily. Second, the policy enforcement is checked. Finally, once the program calls `execute` to run the query, all the optimizations and data sanization occur to allow the best performance, during which query logical plans would be re-ordered, re-written, and pushed down; those do not comply to the policy would be sanitized or propagated with an error that can be passed to other query plans.

With relational algebra equivalence, we are able to perform the following optimizations:

* Selection/predicate push down: we push down the select operator below join to reduce the number of data returned. Similarly, we can evaluate policies beforehand.

Thoughts: The optimizations may be implemented using the egg.

# Physical Plans

The logical plan will first be optimized, and then we can generate the physical plans (i.e., the executors, so to speak) that perform the actual data retrieval. In the implementation of policy-carrying data, executors will eventually call the API sets offered by it. Because executors can be chained, we aim to eliminate the type of each executor. In Rust, this can be done using trait object `dyn TraitObject` wrapped in an atomic reference counter `std::sync::Arc<T>` or a box `Box<T>`. In this proof-of-concept framwork, we provide with the following the executors:

* Filter `FilterExec` which filters out the desired rows from the table.
* Join `JoinExec` which joins two tables given a set of keys.
* Groupby and aggregation which perform the common aggregate operations like `sum`, `max`, etc.
* Projection `ProjectionExec` which 'select's the given columns.
* Custom transformation functions (e.g., the zip code must be redacted according to HIPAA). These functions can be abstracted as `fn(array) -> array` using function traits in Rust. This is still under developement.

