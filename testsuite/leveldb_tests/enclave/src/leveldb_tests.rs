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

pub fn run_tests() {
    use sgx_tunittest::*;

    rsgx_unit_tests!(test_write_a_lot);
}
