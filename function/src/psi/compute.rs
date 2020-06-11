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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use super::basic::*;
use sgx_types::SGX_HASH_SIZE;
use std::ptr::copy_nonoverlapping;
use std::vec;
extern crate log;

#[derive(Clone, Default, Debug)]
pub struct SetIntersection {
    pub data: [HashDataBuffer; 2],
    pub number: u32,
}

#[derive(Clone, Default, Debug)]
pub struct HashDataBuffer {
    pub hashdata: Vec<[u8; SGX_HASH_SIZE]>,
    pub result: Vec<u8>,
}

impl SetIntersection {
    pub fn new() -> Self {
        SetIntersection::default()
    }

    pub fn psi_add_hash_data(&mut self, input: Vec<u8>, index: usize) -> bool {
        let len = input.len();
        if len % SGX_HASH_SIZE != 0 {
            log::error!("Input hash string incorrect with len = {}", len);
            return false;
        }

        let buffer = &mut self.data[index].hashdata;

        let nhash = len / SGX_HASH_SIZE;
        buffer.reserve_exact(nhash);
        unsafe {
            buffer.set_len(nhash);
            copy_nonoverlapping(
                &input[0] as *const _ as *const u8,
                &mut buffer[0] as *mut _ as *mut u8,
                len,
            );
        }
        true
    }

    pub fn compute(&mut self) -> bool {
        let cid: usize = 0;
        let other: usize = 1;
        let mut v_cid: Vec<u8> = vec![0; self.data[cid].hashdata.len()];
        let mut v_other: Vec<u8> = vec![0; self.data[other].hashdata.len()];

        oget_intersection(
            &self.data[cid].hashdata,
            &self.data[other].hashdata,
            &mut v_cid,
            &mut v_other,
        );

        self.data[cid].result = v_cid;
        self.data[other].result = v_other;
        true
    }
}
