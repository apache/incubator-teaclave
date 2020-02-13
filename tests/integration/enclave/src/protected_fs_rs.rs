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

use protected_fs::{remove_protected_file, ProtectedFile};
use rand::prelude::RngCore;
use std::io::{Read, Write};
use std::prelude::v1::*;

pub fn read_write_large_file() {
    const BLOCK_SIZE: usize = 2048;
    const NBLOCKS: usize = 0x001_0000;

    let key = [90u8; 16];

    let mut write_data = [0u8; BLOCK_SIZE];
    let mut read_data = [0u8; BLOCK_SIZE];
    let mut write_size;
    let mut read_size;

    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut write_data);
    let fname = "/tmp/protect_fs_test_large_file";

    {
        let opt = ProtectedFile::create_ex(fname, &key);
        assert_eq!(opt.is_ok(), true);
        let mut file = opt.unwrap();
        for _i in 0..NBLOCKS {
            let result = file.write(&write_data);
            assert_eq!(result.is_ok(), true);
            write_size = result.unwrap();
            assert_eq!(write_size, write_data.len());
        }
    }

    {
        let opt = ProtectedFile::open_ex(fname, &key);
        assert_eq!(opt.is_ok(), true);
        let mut file = opt.unwrap();
        for _i in 0..NBLOCKS {
            let result = file.read(&mut read_data);
            assert_eq!(result.is_ok(), true);
            read_size = result.unwrap();
            assert_eq!(read_size, read_data.len());
        }
    }
    assert_eq!(remove_protected_file(fname).is_ok(), true);
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(read_write_large_file)
}
