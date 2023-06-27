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

use crate::utils::*;
use futures::FutureExt;
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_test_utils::async_test_case;

async fn test_authorize_api() {
    let mut client = get_access_control_client().await;
    let request = AuthorizeApiRequest {
        user_role: "PlatformAdmin".to_owned(),
        api: "Arbitrary_api".to_owned(),
    };
    let response_result = client.authorize_api(request).await;
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().into_inner().accept);

    let mut client = get_access_control_client().await;
    let request = AuthorizeApiRequest {
        user_role: "DataOwner".to_owned(),
        api: "get_function".to_owned(),
    };
    let response_result = client.authorize_api(request).await;
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().into_inner().accept);

    let mut client = get_access_control_client().await;
    let request = AuthorizeApiRequest {
        user_role: "FunctionOwner".to_owned(),
        api: "invoke_task".to_owned(),
    };
    let response_result = client.authorize_api(request).await;
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().into_inner().accept);
}

#[async_test_case]
async fn test_concurrency() {
    let mut thread_pool = Vec::new();
    for _i in 0..10 {
        let child = std::thread::spawn(move || async {
            for _j in 0..10 {
                test_authorize_api().await;
            }
        });
        thread_pool.push(child);
    }
    for thr in thread_pool.into_iter() {
        assert!(thr.join().is_ok());
    }
}
