extern crate protected_fs;
use protected_fs::{remove_protected_file, ProtectedFile};
use rand::prelude::RngCore;
use std::io::{Read, Write};

#[test]
fn test_large_file() {
    const BLOCK_SIZE: usize = 2048;
    const NBLOCKS: usize = 0x100000;

    let key = [90u8; 16];

    let mut write_data = [0u8; BLOCK_SIZE];
    let mut read_data = [0u8; BLOCK_SIZE];
    let mut write_size;
    let mut read_size;

    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut write_data);
    let fname = "large_file";

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
