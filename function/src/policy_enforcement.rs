// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use anyhow::anyhow;
use teaclave_types::{FunctionArguments, FunctionRuntime};

use policy_carrying_data::{pcd, schema::Schema};
use policy_core::{col, cols, expr::count};
use policy_execution::{context::AnalysisContext, lazy::IntoLazy};

#[derive(Default)]
pub struct PolicyEnforcement;

#[derive(serde::Deserialize)]
struct PolicyEnforcementArguments {
    id: usize,
    schema: Schema,
}

impl TryFrom<FunctionArguments> for PolicyEnforcementArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialze arguments")
    }
}

impl PolicyEnforcement {
    pub const NAME: &'static str = "builtin-policy-enforcement";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        _runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        // let _args = PolicyEnforcementArguments::try_from(arguments)?;

        // Create a new context for data analysis.
        let mut ctx = AnalysisContext::new();
        let df = pcd! {
            "foo" => DataType::UInt8:  [1u8, 1, 1, 1, 2, 2, 2, 3, 4, 4, 3, 3, 4, 5, 6],
            "bar" => DataType::Float64: [0.0f64, 0.1, 0.1, 0.2, 0.1, 0.2, 0.2, 0.2, 0.3, 0.4, 0.4, 0.5, 0.5, 0.6, 0.6],
        };
        ctx.register_data(df.into())
            .map_err(|e| anyhow!("{}", e.to_string()))?;

        let df = ctx
            .lazy()
            .select(cols!("foo", "bar"))
            .filter(col!("foo").le(6u8).and(col!("bar").lt(1.0f64)))
            .groupby([col!("bar")])
            .agg([
                col!("foo").min().alias("min value"),
                col!("bar").sum(),
                count(),
            ])
            .collect()
            .map_err(|e| anyhow!("{}", e.to_string()))?;

        Ok(df.to_json())
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!()
    }
}
