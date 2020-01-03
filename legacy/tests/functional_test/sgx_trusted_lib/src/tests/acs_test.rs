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

use super::common_setup::setup_acs_internal_client;

use std::collections::HashSet;
use std::string::ToString;

const FUSION_TASK: &str = "data_fusion";

const FUSION_TASK_PARTY_1: &str = "usr1";
const FUSION_TASK_PARTY_2: &str = "usr2";

const FUSION_TASK_DATA_1: &str = "data1";
const FUSION_TASK_DATA_2: &str = "data2";

const FUSION_TASK_SCRIPT: &str = "fusion_script";
const FUSION_TASK_SCRIPT_WRITER: &str = "usr3";
const PUBLIC_SCRIPT: &str = "public_script";
const PUBLIC_SCRIPT_WRITER: &str = "usr4";

const IRRELEVANT_TASK: &str = "task_irrelevant";
const IRRELEVANT_PARTY: &str = "usr_irrelevant";
const IRRELEVANT_DATA: &str = "data_irrelevant";

pub fn access_control_model() {
    trace!("Test ACS: access control model.");
    let mut client = setup_acs_internal_client();

    let mut participants = HashSet::new();
    participants.insert(FUSION_TASK_PARTY_1.to_string());
    participants.insert(FUSION_TASK_PARTY_2.to_string());
    participants.insert(FUSION_TASK_SCRIPT_WRITER.to_string());

    client
        .announce_task_creation(
            FUSION_TASK.to_string(),
            FUSION_TASK_PARTY_1.to_string(),
            &participants,
        )
        .expect("fusion task creation announcement failed");

    client
        .announce_data_creation(
            FUSION_TASK_DATA_1.to_string(),
            FUSION_TASK_PARTY_1.to_string(),
        )
        .expect("fusion data n1 creation announcement failed");

    client
        .announce_data_creation(
            FUSION_TASK_DATA_2.to_string(),
            FUSION_TASK_PARTY_2.to_string(),
        )
        .expect("fusion data 2 creation announcement failed");

    client
        .announce_data_creation(IRRELEVANT_DATA.to_string(), IRRELEVANT_PARTY.to_string())
        .expect("irrelevant data creation announcement failed");

    client
        .announce_script_creation(
            FUSION_TASK_SCRIPT.to_string(),
            FUSION_TASK_SCRIPT_WRITER.to_string(),
            false,
        )
        .expect("fusion script creation announcement failed");

    client
        .announce_script_creation(
            PUBLIC_SCRIPT.to_string(),
            PUBLIC_SCRIPT_WRITER.to_string(),
            true,
        )
        .expect("public script creation announcement failed");

    let mut participants = HashSet::new();

    assert_eq!(
        client
            .enforce_task_launch(FUSION_TASK.to_string(), participants.clone(),)
            .unwrap(),
        false,
    );

    participants.insert(FUSION_TASK_PARTY_1.to_string());

    assert_eq!(
        client
            .enforce_task_launch(FUSION_TASK.to_string(), participants.clone(),)
            .unwrap(),
        false,
    );

    participants.insert(FUSION_TASK_PARTY_2.to_string());

    assert_eq!(
        client
            .enforce_task_launch(FUSION_TASK.to_string(), participants.clone(),)
            .unwrap(),
        false,
    );

    participants.insert(FUSION_TASK_SCRIPT_WRITER.to_string());

    assert_eq!(
        client
            .enforce_task_launch(FUSION_TASK.to_string(), participants,)
            .unwrap(),
        true,
    );

    // Load fusion script
    assert_eq!(
        client
            .enforce_script_access(FUSION_TASK.to_string(), FUSION_TASK_SCRIPT.to_string(),)
            .unwrap(),
        true,
    );

    // Load public script
    assert_eq!(
        client
            .enforce_script_access(FUSION_TASK.to_string(), PUBLIC_SCRIPT.to_string(),)
            .unwrap(),
        true,
    );

    // Read data1
    assert_eq!(
        client
            .enforce_data_access(FUSION_TASK.to_string(), FUSION_TASK_DATA_1.to_string(),)
            .unwrap(),
        true,
    );

    // Read data2
    assert_eq!(
        client
            .enforce_data_access(FUSION_TASK.to_string(), FUSION_TASK_DATA_2.to_string(),)
            .unwrap(),
        true,
    );

    let mut participants = HashSet::new();

    participants.insert(IRRELEVANT_PARTY.to_string());
    participants.insert(FUSION_TASK_PARTY_2.to_string());

    client
        .announce_task_creation(
            IRRELEVANT_TASK.to_string(),
            IRRELEVANT_PARTY.to_string(),
            &participants,
        )
        .expect("irrelevant task creation announcement failed");

    // Launch irrelevant task
    assert_eq!(
        client
            .enforce_task_launch(IRRELEVANT_TASK.to_string(), participants,)
            .unwrap(),
        true,
    );

    // Load fusion script; deny
    assert_eq!(
        client
            .enforce_script_access(IRRELEVANT_TASK.to_string(), FUSION_TASK_SCRIPT.to_string(),)
            .unwrap(),
        false,
    );

    // Load public script; allow
    assert_eq!(
        client
            .enforce_script_access(IRRELEVANT_TASK.to_string(), PUBLIC_SCRIPT.to_string(),)
            .unwrap(),
        true,
    );

    // Read data1; deny
    assert_eq!(
        client
            .enforce_data_access(IRRELEVANT_TASK.to_string(), FUSION_TASK_DATA_1.to_string(),)
            .unwrap(),
        false,
    );

    // Read data2; allow
    assert_eq!(
        client
            .enforce_data_access(IRRELEVANT_TASK.to_string(), FUSION_TASK_DATA_2.to_string(),)
            .unwrap(),
        true,
    );

    assert_eq!(
        client
            .enforce_data_deletion(
                FUSION_TASK_PARTY_1.to_string(),
                FUSION_TASK_DATA_1.to_string(),
            )
            .unwrap(),
        true,
    );

    assert_eq!(
        client
            .enforce_data_deletion(
                FUSION_TASK_PARTY_2.to_string(),
                FUSION_TASK_DATA_2.to_string(),
            )
            .unwrap(),
        true,
    );

    assert_eq!(
        client
            .enforce_data_deletion(
                FUSION_TASK_PARTY_1.to_string(),
                FUSION_TASK_DATA_2.to_string(),
            )
            .unwrap(),
        false,
    );

    assert_eq!(
        client
            .enforce_script_deletion(
                FUSION_TASK_PARTY_1.to_string(),
                FUSION_TASK_SCRIPT.to_string(),
            )
            .unwrap(),
        false,
    );

    assert_eq!(
        client
            .enforce_script_deletion(
                FUSION_TASK_SCRIPT_WRITER.to_string(),
                FUSION_TASK_SCRIPT.to_string(),
            )
            .unwrap(),
        true,
    );

    assert_eq!(
        client
            .enforce_script_deletion(IRRELEVANT_PARTY.to_string(), PUBLIC_SCRIPT.to_string(),)
            .unwrap(),
        false,
    );

    assert_eq!(
        client
            .enforce_script_deletion(PUBLIC_SCRIPT_WRITER.to_string(), PUBLIC_SCRIPT.to_string(),)
            .unwrap(),
        true,
    );
}
