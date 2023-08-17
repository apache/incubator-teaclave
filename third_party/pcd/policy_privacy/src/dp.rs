use std::{
    fmt::Debug,
    ops::{Add, Range},
};

use opendp::traits::samplers::SampleDiscreteLaplaceZ2k;
use policy_carrying_data::{arithmetic::sum_impl, field::FieldDataArray};
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::Aggregation,
    policy::DpParam,
    types::PrimitiveData,
};

/// This function computes the optimal upper bound for queries like summation using the
/// Sparse Vector Technique (SVT). It returns an index!
///
/// The `above_threshold` algorithm returns (approximately) the index of the first query in
/// queries whose result exceeds the threshold. However, one must note that for small-domained
/// dataset, the result may be *inaccurate*.
///
/// # Reference
///
/// * Programming differential privacy: <https://programming-dp.com/ch10.html>
pub fn above_threshold<F, T>(
    queries: &[F],
    df: &FieldDataArray<T>,
    threshold: T,
    epsilon: f64,
) -> PolicyCarryingResult<usize>
where
    T: PrimitiveData + Add<f64, Output = f64> + Debug + Send + Sync + Clone + 'static,
    F: Fn(&FieldDataArray<T>) -> f64,
{
    let scale = 2.0 / epsilon;
    let t_hat = threshold
        + f64::sample_discrete_laplace_Z2k(0.0, scale, -1024)
            .map_err(|e| PolicyCarryingError::PrivacyError(e.to_string()))?;

    for (idx, q) in queries.into_iter().enumerate() {
        // 2^{-k}: the granularity of the floating number.
        let nu_i = f64::sample_discrete_laplace_Z2k(0.0, scale * 2.0, -1024)
            .map_err(|e| PolicyCarryingError::PrivacyError(e.to_string()))?;

        if q(df) + nu_i >= t_hat {
            return Ok(idx);
        }
    }

    // Nothing is selected: returns error.
    Err(PolicyCarryingError::PrivacyError(
        "SVT failed. Consider changing the threshold?".into(),
    ))
}

/// This function also implementes the Sparse Vector Technique (SVT) but returns multiple values at once.
/// This is because in many other applications we would like to find the indices of all queries that ex-
/// ceed the threshold.
///
/// Note that this function pays *higher* privacy cost to fulfill the task.
///
/// # Reference
///
/// * Programming differential privacy: <https://programming-dp.com/ch10.html>
/// * C Dwork, A Roth. The algorithmic foundations of differential privacy.
///   Foundations and Trends(R) in Theoretical Computer Science, 9(3–4):211–407, 2014.
pub fn above_threshold_multiple<F, T>(
    queries: &[F],
    df: &FieldDataArray<T>,
    threshold: T,
    size: usize,
    epsilon: f64,
) -> PolicyCarryingResult<Vec<usize>>
where
    T: PrimitiveData + Add<f64, Output = f64> + Debug + Send + Sync + Clone + 'static,
    F: Fn(&FieldDataArray<T>) -> f64,
{
    let mut indices = Vec::new();
    let mut pos = 0usize;
    let epsilon_i = epsilon / size as f64;
    // Stop if we reach the end of the stream of queries, or if we find c queries above the threshold.
    while pos < queries.len() && indices.len() < size {
        // Run `above_threshold` to find the next query above the threshold.
        let next_idx = match above_threshold(&queries[pos..], df, threshold.clone(), epsilon_i) {
            Ok(next_idx) => next_idx,
            Err(_) => return Ok(indices),
        };

        pos = next_idx + pos;
        indices.push(pos);
        pos += 1;
    }

    Ok(indices)
}

/// Computes the clamped upper bound for sum queries that should be \epsilon-differentially private.
///
/// Note that this operation *consumes* `epsilon` privacy budget.
pub fn sum_upper_bound(
    range: Range<usize>,
    data: &FieldDataArray<f64>,
    epsilon: f64,
) -> PolicyCarryingResult<usize> {
    // Generate a series of queries that is each 1-sensitive.
    let queries = range
        .map(|i| {
            move |df: &FieldDataArray<f64>| {
                let prev = sum_impl::<f64, f64>(df, 0.0, Some(i as f64)).unwrap();
                let cur = sum_impl::<f64, f64>(df, 0.0, Some(i as f64 + 1.0)).unwrap();

                prev - cur
            }
        })
        .collect::<Vec<_>>();

    above_threshold(&queries, data, 0.0, epsilon)
}

/// Denotes the differentially private mechanism.
pub enum DpMechanism {
    /// The Laplace mechanism.
    Laplace,
    /// The Gaussian mechanism but only works for approxiamte DP.
    Gaussian,
    /// The Exponential mechanism.
    Exponential,
}

/// Manager of differential privacy for tracking privacy budget and privacy loss. It may also
/// help maintain the internal state of privacy schemes.
#[derive(Clone, Debug)]
pub struct DpManager {
    /// The unique identifier of this manager.
    id: usize,
    /// The parameter of differential privacy.
    ///
    /// Budget: a pipeline may include multiple DP calculations, each of which has its own (\epsilon, \delta).
    /// each invocation of the dp mechanism *will* consume the budget (we can use the composition theorem).
    dp_budget: DpParam,
}

impl DpManager {
    #[inline]
    pub fn new(id: usize, dp_param: DpParam) -> Self {
        Self {
            id,
            dp_budget: dp_param,
        }
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn dp_budget(&self) -> DpParam {
        self.dp_budget
    }

    /// Converts a query `q` into a differentially private query on *scalar* types.
    ///
    /// In order to operate on vector types, we must re-define the global sensitivity
    /// by means of norms like L2 norms and then apply Gaussian mechanism.
    ///
    /// One should also note that this function takes the query function `q` as an
    /// immutable function trait [`Fn`] that takes no input. The caller may wrap the
    /// query function in another closure like below.
    ///
    /// ```
    /// use policy_function::{func::pcd_max, privacy::DpManager};
    ///
    /// let wrapper_query = || pcd_max(&arr);
    /// let dp_manager = DpManager::new(0, (1.0, 0.01));
    /// # let dp_query = dp_manager.make_dp_compliant_scalar(wrapper_query);
    /// ```
    pub fn make_dp_compliant_scalar<F, T>(
        &mut self,
        api_type: Aggregation,
        q: F,
        dp_param: DpParam,
    ) -> PolicyCarryingResult<F>
    where
        T: PrimitiveData + PartialOrd + Debug + Default + Send + Sync + Clone + 'static,
        F: Fn() -> T,
    {
        let epsilon = self.dp_budget.0;

        // Some key problems:
        //     1. How to determine the global sensitivity s?
        //     2. How to properly perform clamping on unbounded sensitivity? The tricky thing is,
        //        we do not want the query generator to specify its output range.

        // Queries with unbounded sensitivity cannot be directly answered with differential privacy
        // using the Laplace mechanism.
        todo!()
    }
}

#[cfg(test)]
mod test {
    use policy_carrying_data::field::Float64FieldData;

    use super::*;

    #[test]
    fn test_svt_correct() {
        let df = csv::Reader::from_path("../test_data/adult_with_pii.csv")
            .unwrap()
            .records()
            .into_iter()
            .map(|r| r.unwrap().get(4).unwrap().parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        let df = Float64FieldData::from(df);

        // Should be larger than 85.
        let idx = sum_upper_bound(0..150, &df, 1.0);

        assert!(idx.is_ok_and(|inner| inner >= 85));
    }
}
