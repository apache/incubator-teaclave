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

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum UserRole {
    PlatformAdmin,
    FunctionOwner,
    DataOwnerManager(String),
    DataOwner(String),
    Invalid,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Invalid
    }
}

impl UserRole {
    pub fn new(role: &str, attribute: &str) -> Self {
        match role {
            "PlatformAdmin" => UserRole::PlatformAdmin,
            "FunctionOwner" => UserRole::FunctionOwner,
            "DataOwnerManager" => UserRole::DataOwnerManager(attribute.to_owned()),
            "DataOwner" => UserRole::DataOwner(attribute.to_owned()),
            _ => UserRole::Invalid,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(role: &str) -> UserRole {
        match role {
            "PlatformAdmin" => UserRole::PlatformAdmin,
            "FunctionOwner" => UserRole::FunctionOwner,
            _ => {
                if let Some(a) = role.strip_prefix("DataOwner-") {
                    UserRole::DataOwner(a.to_owned())
                } else if let Some(a) = role.strip_prefix("DataOwnerManager-") {
                    UserRole::DataOwnerManager(a.to_owned())
                } else {
                    UserRole::Invalid
                }
            }
        }
    }

    pub fn is_platform_admin(&self) -> bool {
        matches!(self, UserRole::PlatformAdmin)
    }

    pub fn is_function_owner(&self) -> bool {
        matches!(self, UserRole::FunctionOwner)
    }

    pub fn is_data_owner(&self) -> bool {
        matches!(self, UserRole::DataOwnerManager(_)) || matches!(self, UserRole::DataOwner(_))
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::PlatformAdmin => write!(f, "PlatformAdmin"),
            UserRole::FunctionOwner => write!(f, "FunctionOwner"),
            UserRole::DataOwnerManager(s) => write!(f, "DataOwnerManager-{}", s),
            UserRole::DataOwner(s) => write!(f, "DataOwner-{}", s),
            UserRole::Invalid => write!(f, "Invalid"),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserAuthClaims {
    // user id
    pub sub: String,
    // role
    pub role: String,
    // issuer
    pub iss: String,
    // expiration time
    pub exp: u64,
}

impl UserAuthClaims {
    pub fn get_role(&self) -> UserRole {
        UserRole::from_str(&self.role)
    }
}
