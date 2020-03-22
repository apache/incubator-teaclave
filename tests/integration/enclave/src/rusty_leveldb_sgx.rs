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

use rand::distributions::Alphanumeric;
use rand::Rng;
use std::iter;
use std::prelude::v1::*;
use std::string::String;

use rusty_leveldb::CompressionType;
use rusty_leveldb::Options;
use rusty_leveldb::DB;

use std::error::Error;
use std::io::{self, ErrorKind};
use std::untrusted::fs;

const KEY_LEN: usize = 16;
const VAL_LEN: usize = 48;

fn gen_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}

fn fill_db(db: &mut DB, entries: usize) -> Result<(), Box<dyn Error>> {
    for i in 0..entries {
        let (k, v) = (gen_string(KEY_LEN), gen_string(VAL_LEN));
        db.put(k.as_bytes(), v.as_bytes())?;
        if i % 1000 == 0 {
            db.flush()?;

            let v2 = db
                .get(k.as_bytes())
                .ok_or_else(|| Box::new(io::Error::new(ErrorKind::NotFound, "Key not found")))?;
            assert_eq!(&v.as_bytes()[..], &v2[..]);

            db.delete(k.as_bytes())?;
            assert_eq!(true, db.get(k.as_bytes()).is_none());
        }

        if i % 100 == 0 {
            db.flush()?;
        }
    }
    Ok(())
}

fn fill_db_with_sequential_elements(db: &mut DB, entries: usize) -> Result<(), Box<dyn Error>> {
    for i in 0..entries {
        let (k, v) = (i.to_string(), i.to_string());
        db.put(k.as_bytes(), v.as_bytes())?;
        db.flush()?;
    }
    Ok(())
}

fn validate_sequential_elements(db: &mut DB, entries: usize) -> Result<(), Box<dyn Error>> {
    for i in 0..entries {
        let (k, v_expected) = (i.to_string(), i.to_string());
        let v = db.get(k.as_bytes()).ok_or_else(|| {
            error!("key: {}", k);
            Box::new(io::Error::new(ErrorKind::NotFound, "Key not found"))
        })?;
        assert_eq!(&v_expected.as_bytes()[..], &v[..]);
    }
    Ok(())
}

fn test_write_a_lot() {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let mut opt = Options::new_disk_db_with(key);
    opt.compression_type = CompressionType::CompressionSnappy;
    let mut db = DB::open("/tmp/leveldb_testdb", opt).unwrap();

    fill_db(&mut db, 32768).unwrap();

    drop(db);

    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
}

fn test_write_and_reopen() {
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
        0x08,
    ];
    let elements_count = 2000;
    let db_location = "/tmp/leveldb_testdb";

    {
        let mut opt = Options::new_disk_db_with(key);
        opt.compression_type = CompressionType::CompressionSnappy;
        let mut db = DB::open(&db_location, opt).unwrap();
        fill_db_with_sequential_elements(&mut db, elements_count).unwrap();
    }

    {
        let mut opt = Options::new_disk_db_with(key);
        opt.compression_type = CompressionType::CompressionSnappy;
        let mut db = DB::open(&db_location, opt).unwrap();
        validate_sequential_elements(&mut db, 2000).unwrap();
    }

    fs::remove_dir_all("/tmp/leveldb_testdb").expect("Cannot remove directory");
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_write_a_lot, test_write_and_reopen,)
}
