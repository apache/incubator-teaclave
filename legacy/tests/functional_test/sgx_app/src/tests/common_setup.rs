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

use fns_client::FNSClient;
use mesatee_core::config::{get_trusted_enclave_attr, OutboundDesc, TargetDesc};
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tdfs_external_client::TDFSClient;
use tms_external_client::TMSClient;

pub(crate) struct User {
    pub user_id: &'static str,
    pub user_token: &'static str,
}

pub(crate) const USER_ONE: User = User {
    user_id: "user1",
    user_token: "token1",
};

pub(crate) const USER_TWO: User = User {
    user_id: "user2",
    user_token: "token2",
};

pub(crate) const USER_ERR: User = User {
    user_id: "error_user",
    user_token: "error_token",
};

pub(crate) const USER_THREE: User = User {
    user_id: "fake_file_owner",
    user_token: "token3",
};

pub(crate) const USER_FAKE: User = User {
    user_id: "fake",
    user_token: "fake",
};

pub(crate) const USER_FOUR: User = User {
    user_id: "user_four",
    user_token: "token4",
};

#[inline]
fn target_tms() -> TargetDesc {
    TargetDesc::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5554),
        OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tms"])),
    )
}

#[inline]
fn target_tdfs() -> TargetDesc {
    TargetDesc::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5065),
        OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tdfs"])),
    )
}

#[inline]
fn outbound_default() -> OutboundDesc {
    OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns"]))
}

pub(crate) fn setup_tms_external_client(user: &User) -> TMSClient {
    let target = target_tms();
    TMSClient::new(&target, user.user_id, user.user_token).unwrap()
}

pub(crate) fn setup_fns_client(ip: IpAddr, port: u16) -> FNSClient {
    let desc = outbound_default();
    let target = TargetDesc::new(SocketAddr::new(ip, port), desc);
    FNSClient::new(&target).unwrap()
}

pub(crate) fn setup_tdfs_external_client(user: &User) -> TDFSClient {
    let target = target_tdfs();
    TDFSClient::new(&target, user.user_id, user.user_token).unwrap()
}

pub(crate) fn save_file_for_user(user: &User, content: &[u8], save_path: &str) -> String {
    let mut tdfs_client = setup_tdfs_external_client(user);
    fs::write(save_path, content).unwrap();
    let file_name = "unittest";
    tdfs_client.save_file(save_path, file_name).unwrap()
}

pub(crate) fn read_file_for_user(user: &User, file_id: &str) -> Vec<u8> {
    let mut tdfs_client = setup_tdfs_external_client(user);
    tdfs_client.read_file(&file_id).unwrap()
}
