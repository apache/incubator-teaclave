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
extern crate protected_fs;
use protected_fs::{remove_protected_file, ProtectedFile};
use rand_core::RngCore;
use std::fs;
use std::io::{Read, Write};

#[test]
fn test_rename() {
    const BLOCK_SIZE: usize = 2048;

    let key = [90u8; 16];
    let mut auth_tag = [0u8; 16];

    let mut write_data = [0u8; BLOCK_SIZE];
    let mut read_data = [0u8; BLOCK_SIZE];

    let mut rng = rdrand::RdRand::new().unwrap();
    rng.fill_bytes(&mut write_data);

    let old_name = "old_file";
    let new_name = "new_file";

    {
        // create and write old_file
        let mut file = ProtectedFile::create_ex(&old_name, &key).unwrap();
        let write_size = file.write(&write_data).unwrap();
        assert_eq!(write_size, write_data.len());
    }

    {
        // rename_meta requires append mode
        let mut file = ProtectedFile::append_ex(&old_name, &key).unwrap();
        // rename_meta in protected file
        file.rename_meta(&old_name, &new_name).unwrap();

        // flush before we get the final auth_tag
        file.flush().unwrap();

        // get the latest gmac
        file.get_current_meta_gmac(&mut auth_tag).unwrap();
    }

    // rename file after close
    fs::rename(old_name, new_name).unwrap();

    {
        let mut auth_tag_in_file = [0xffu8; 16];
        let mut file = ProtectedFile::open_ex(&new_name, &key).unwrap();

        file.get_current_meta_gmac(&mut auth_tag_in_file).unwrap();
        assert_eq!(auth_tag_in_file, auth_tag);

        let read_size = file.read(&mut read_data).unwrap();
        assert_eq!(read_size, read_data.len());

        assert_eq!(&read_data[..], &write_data[..]);
    }
    assert_eq!(remove_protected_file(&new_name).is_ok(), true);
}
