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

use anyhow::{bail, Result};
use casbin::prelude::*;
use csv::{ReaderBuilder, StringRecord};

pub async fn init_memory_enforcer() -> Result<Enforcer> {
    const MODEL_TEXT: &str = include_str!("../../model.conf");
    const POLICY_TEXT: &str = include_str!("../../policy.csv");

    let model = DefaultModel::from_str(MODEL_TEXT).await?;
    let adapter = MemoryAdapter::default();
    let mut enforcer = Enforcer::new(model, adapter).await?;

    let (general, grouping) = parse_policy_str(POLICY_TEXT)?;
    enforcer.add_policies(general).await?;
    enforcer.add_grouping_policies(grouping).await?;

    Ok(enforcer)
}

type Policy = Vec<String>;

/// Parse casbin polices in bytes to general and grouping policies
fn parse_policy_str(polices: &str) -> Result<(Vec<Policy>, Vec<Policy>)> {
    let mut general = Vec::new();
    let mut grouping = Vec::new();

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(polices.as_bytes());
    for result in rdr.records() {
        let policy = result?;
        let policy_type = policy.get(0);

        match policy_type {
            Some("p") => {
                let rule = strip_first_element(policy);
                general.push(rule);
            }
            Some("g") => {
                let rule = strip_first_element(policy);
                grouping.push(rule);
            }
            _ => bail!("invalid policy type: {:?}", policy_type),
        }
    }

    Ok((general, grouping))
}

fn strip_first_element(record: StringRecord) -> Vec<String> {
    record
        .into_iter()
        .skip(1)
        .map(|s| s.trim().to_owned())
        .collect()
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub async fn test_access_api() {
        let e = init_memory_enforcer().await.unwrap();

        assert!(e.enforce(("PlatformAdmin", "arbitrary_api")).unwrap());
        assert!(e.enforce(("PlatformAdmin", "query_audit_logs")).unwrap());

        assert!(!e.enforce(("Invalid", "register_function")).unwrap());
        assert!(!e.enforce(("Invalid", "register_input_file")).unwrap());
        assert!(!e.enforce(("Invalid", "get_function")).unwrap());
        assert!(!e.enforce(("Invalid", "query_audit_logs")).unwrap());

        assert!(e.enforce(("FunctionOwner", "register_function")).unwrap());
        assert!(e.enforce(("FunctionOwner", "update_function")).unwrap());
        assert!(e.enforce(("FunctionOwner", "delete_function")).unwrap());
        assert!(e.enforce(("FunctionOwner", "disable_function")).unwrap());
        assert!(e.enforce(("FunctionOwner", "get_function")).unwrap());
        assert!(e.enforce(("FunctionOwner", "list_functions")).unwrap());
        assert!(e
            .enforce(("FunctionOwner", "get_function_usage_stats"))
            .unwrap());
        assert!(!e.enforce(("FunctionOwner", "get_task")).unwrap());
        assert!(!e.enforce(("FunctionOwner", "query_audit_logs")).unwrap());

        assert!(e.enforce(("DataOwner", "register_input_file")).unwrap());
        assert!(e.enforce(("DataOwner", "register_output_file")).unwrap());
        assert!(e.enforce(("DataOwner", "update_input_file")).unwrap());
        assert!(e.enforce(("DataOwner", "update_output_file")).unwrap());
        assert!(e.enforce(("DataOwner", "register_fusion_output")).unwrap());
        assert!(e
            .enforce(("DataOwner", "register_input_from_output"))
            .unwrap());
        assert!(e.enforce(("DataOwner", "get_input_file")).unwrap());
        assert!(e.enforce(("DataOwner", "get_output_file")).unwrap());
        assert!(e.enforce(("DataOwner", "create_task")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "get_task")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "assign_data")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "approve_task")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "invoke_task")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "cancel_task")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "get_function")).unwrap());
        assert!(e.enforce(("DataOwnerManager", "list_functions")).unwrap());
        assert!(e
            .enforce(("DataOwnerManager", "get_function_usage_stats"))
            .unwrap());
        assert!(!e.enforce(("DataOwner", "register_function")).unwrap());
        assert!(!e.enforce(("DataOwnerManager", "query_audit_logs")).unwrap());
    }
}
