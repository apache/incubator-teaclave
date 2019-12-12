#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::cmp::Ordering;

use block::BlockContents;
use options::Options;

use integer_encoding::{FixedIntWriter, VarIntWriter};

/// BlockBuilder contains functionality for building a block consisting of consecutive key-value
/// entries.
pub struct BlockBuilder {
    opt: Options,
    buffer: Vec<u8>,
    restarts: Vec<u32>,

    last_key: Vec<u8>,
    restart_counter: usize,
    counter: usize,
}

impl BlockBuilder {
    pub fn new(o: Options) -> BlockBuilder {
        let mut restarts = vec![0];
        restarts.reserve(1023);

        BlockBuilder {
            buffer: Vec::with_capacity(o.block_size),
            opt: o,
            restarts: restarts,
            last_key: Vec::new(),
            restart_counter: 0,
            counter: 0,
        }
    }

    pub fn entries(&self) -> usize {
        self.counter
    }

    pub fn last_key<'a>(&'a self) -> &'a [u8] {
        &self.last_key
    }

    pub fn size_estimate(&self) -> usize {
        self.buffer.len() + 4 * self.restarts.len() + 4
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.restarts.clear();
        self.last_key.clear();
        self.restart_counter = 0;
        self.counter = 0;
    }

    pub fn add(&mut self, key: &[u8], val: &[u8]) {
        assert!(self.restart_counter <= self.opt.block_restart_interval);
        assert!(
            self.buffer.is_empty()
                || self.opt.cmp.cmp(self.last_key.as_slice(), key) == Ordering::Less
        );

        let mut shared = 0;

        if self.restart_counter < self.opt.block_restart_interval {
            let smallest = if self.last_key.len() < key.len() {
                self.last_key.len()
            } else {
                key.len()
            };

            while shared < smallest && self.last_key[shared] == key[shared] {
                shared += 1;
            }
        } else {
            self.restarts.push(self.buffer.len() as u32);
            self.last_key.resize(0, 0);
            self.restart_counter = 0;
        }

        let non_shared = key.len() - shared;

        self.buffer
            .write_varint(shared)
            .expect("write to buffer failed");
        self.buffer
            .write_varint(non_shared)
            .expect("write to buffer failed");
        self.buffer
            .write_varint(val.len())
            .expect("write to buffer failed");
        self.buffer.extend_from_slice(&key[shared..]);
        self.buffer.extend_from_slice(val);

        // Update key
        self.last_key.resize(shared, 0);
        self.last_key.extend_from_slice(&key[shared..]);

        self.restart_counter += 1;
        self.counter += 1;
    }

    pub fn finish(mut self) -> BlockContents {
        self.buffer.reserve(self.restarts.len() * 4 + 4);

        // 1. Append RESTARTS
        for r in self.restarts.iter() {
            self.buffer
                .write_fixedint(*r as u32)
                .expect("write to buffer failed");
        }

        // 2. Append N_RESTARTS
        self.buffer
            .write_fixedint(self.restarts.len() as u32)
            .expect("write to buffer failed");

        // done
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use options;

    fn get_data() -> Vec<(&'static [u8], &'static [u8])> {
        vec![
            ("key1".as_bytes(), "value1".as_bytes()),
            (
                "loooooooooooooooooooooooooooooooooongerkey1".as_bytes(),
                "shrtvl1".as_bytes(),
            ),
            ("medium length key 1".as_bytes(), "some value 2".as_bytes()),
            ("prefix_key1".as_bytes(), "value".as_bytes()),
            ("prefix_key2".as_bytes(), "value".as_bytes()),
            ("prefix_key3".as_bytes(), "value".as_bytes()),
        ]
    }

    #[test]
    fn test_block_builder_sanity() {
        let mut o = options::for_test();
        o.block_restart_interval = 3;
        let mut builder = BlockBuilder::new(o);
        let d = get_data();

        for &(k, v) in d.iter() {
            builder.add(k, v);
            assert!(builder.restart_counter <= 3);
            assert_eq!(builder.last_key(), k);
        }

        assert_eq!(149, builder.size_estimate());
        assert_eq!(d.len(), builder.entries());

        let block = builder.finish();
        assert_eq!(block.len(), 149);
    }

    #[test]
    fn test_block_builder_reset() {
        let mut o = options::for_test();
        o.block_restart_interval = 3;
        let mut builder = BlockBuilder::new(o);
        let d = get_data();

        for &(k, v) in d.iter() {
            builder.add(k, v);
            assert!(builder.restart_counter <= 3);
            assert_eq!(builder.last_key(), k);
        }

        assert_eq!(d.len(), builder.entries());
        builder.reset();
        assert_eq!(0, builder.entries());
        assert_eq!(4, builder.size_estimate());
    }

    #[test]
    #[should_panic]
    fn test_block_builder_panics() {
        let mut d = get_data();
        // Identical key as d[3].
        d[4].0 = b"prefix_key1";

        let mut builder = BlockBuilder::new(options::for_test());
        for &(k, v) in d.iter() {
            builder.add(k, v);
            assert_eq!(k, builder.last_key());
        }
    }
    // Additional test coverage is provided by tests in block.rs.
}
