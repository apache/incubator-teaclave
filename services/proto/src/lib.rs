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

extern crate sgx_types;

pub mod teaclave_access_control_service;
pub mod teaclave_authentication_service;
pub mod teaclave_common;
pub mod teaclave_frontend_service;
pub mod teaclave_management_service;
pub mod teaclave_scheduler_service;
pub mod teaclave_storage_service;

macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

pub mod teaclave_authentication_service_proto {
    include_proto!("teaclave_authentication_service_proto");
}

pub mod teaclave_common_proto {
    include_proto!("teaclave_common_proto");
}

pub mod teaclave_storage_service_proto {
    include_proto!("teaclave_storage_service_proto");
}

pub mod teaclave_frontend_service_proto {
    include_proto!("teaclave_frontend_service_proto");
}

pub mod teaclave_management_service_proto {
    include_proto!("teaclave_management_service_proto");
}

pub mod teaclave_access_control_service_proto {
    include_proto!("teaclave_access_control_service_proto");
}

pub mod teaclave_scheduler_service_proto {
    include_proto!("teaclave_scheduler_service_proto");
}
