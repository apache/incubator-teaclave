#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::rc::Rc;

use integer_encoding::FixedInt;

/// Encapsulates a filter algorithm allowing to search for keys more efficiently.
/// Usually, policies are used as a BoxedFilterPolicy (see below), so they
/// can be easily cloned and nested.
pub trait FilterPolicy {
    /// Returns a string identifying this policy.
    fn name(&self) -> &'static str;
    /// Create a filter matching the given keys. Keys are given as a long byte array that is
    /// indexed by the offsets contained in key_offsets.
    fn create_filter(&self, keys: &[u8], key_offsets: &[usize]) -> Vec<u8>;
    /// Check whether the given key may match the filter.
    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool;
}

/// A boxed and refcounted filter policy (reference-counted because a Box with unsized content
/// couldn't be cloned otherwise)
pub type BoxedFilterPolicy = Rc<Box<dyn FilterPolicy>>;

impl FilterPolicy for BoxedFilterPolicy {
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn create_filter(&self, keys: &[u8], key_offsets: &[usize]) -> Vec<u8> {
        (**self).create_filter(keys, key_offsets)
    }
    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool {
        (**self).key_may_match(key, filter)
    }
}

/// Used for tables that don't have filter blocks but need a type parameter.
#[derive(Clone)]
pub struct NoFilterPolicy;

impl NoFilterPolicy {
    pub fn new() -> NoFilterPolicy {
        NoFilterPolicy
    }
}

impl FilterPolicy for NoFilterPolicy {
    fn name(&self) -> &'static str {
        "_"
    }
    fn create_filter(&self, _: &[u8], _: &[usize]) -> Vec<u8> {
        vec![]
    }
    fn key_may_match(&self, _: &[u8], _: &[u8]) -> bool {
        true
    }
}

const BLOOM_SEED: u32 = 0xbc9f1d34;

/// A filter policy using a bloom filter internally.
#[derive(Clone)]
pub struct BloomPolicy {
    bits_per_key: u32,
    k: u32,
}

/// Beware the magic numbers...
impl BloomPolicy {
    /// Returns a new boxed BloomPolicy.
    pub fn new(bits_per_key: u32) -> BloomPolicy {
        BloomPolicy::new_unwrapped(bits_per_key)
    }

    /// Returns a new BloomPolicy with the given parameter.
    fn new_unwrapped(bits_per_key: u32) -> BloomPolicy {
        let mut k = (bits_per_key as f32 * 0.69) as u32;

        if k < 1 {
            k = 1;
        } else if k > 30 {
            k = 30;
        }

        BloomPolicy {
            bits_per_key: bits_per_key,
            k: k,
        }
    }

    fn bloom_hash(&self, data: &[u8]) -> u32 {
        let m: u32 = 0xc6a4a793;
        let r: u32 = 24;

        let mut ix = 0;
        let limit = data.len();

        let mut h: u32 = BLOOM_SEED ^ (limit as u64 * m as u64) as u32;

        while ix + 4 <= limit {
            let w = u32::decode_fixed(&data[ix..ix + 4]);
            ix += 4;

            h = (h as u64 + w as u64) as u32;
            h = (h as u64 * m as u64) as u32;
            h ^= h >> 16;
        }

        // Process left-over bytes
        assert!(limit - ix < 4);

        if limit - ix > 0 {
            let mut i = 0;

            for b in data[ix..].iter() {
                h = h.overflowing_add((*b as u32) << (8 * i)).0;
                i += 1;
            }

            h = (h as u64 * m as u64) as u32;
            h ^= h >> r;
        }
        h
    }
}

impl FilterPolicy for BloomPolicy {
    fn name(&self) -> &'static str {
        "leveldb.BuiltinBloomFilter2"
    }
    fn create_filter(&self, keys: &[u8], key_offsets: &[usize]) -> Vec<u8> {
        let filter_bits = key_offsets.len() * self.bits_per_key as usize;
        let mut filter: Vec<u8>;

        if filter_bits < 64 {
            filter = Vec::with_capacity(8 + 1);
            filter.resize(8, 0);
        } else {
            filter = Vec::with_capacity(1 + ((filter_bits + 7) / 8));
            filter.resize((filter_bits + 7) / 8, 0);
        }

        let adj_filter_bits = (filter.len() * 8) as u32;

        // Encode k at the end of the filter.
        filter.push(self.k as u8);

        // Add all keys to the filter.
        offset_data_iterate(keys, key_offsets, |key| {
            let mut h = self.bloom_hash(key);
            let delta = (h >> 17) | (h << 15);
            for _ in 0..self.k {
                let bitpos = (h % adj_filter_bits) as usize;
                filter[bitpos / 8] |= 1 << (bitpos % 8);
                h = (h as u64 + delta as u64) as u32;
            }
        });

        filter
    }
    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool {
        if filter.len() == 0 {
            return true;
        }

        let bits = (filter.len() - 1) as u32 * 8;
        let k = filter[filter.len() - 1];
        let filter_adj = &filter[0..filter.len() - 1];

        if k > 30 {
            return true;
        }

        let mut h = self.bloom_hash(key);
        let delta = (h >> 17) | (h << 15);
        for _ in 0..k {
            let bitpos = (h % bits) as usize;
            if (filter_adj[bitpos / 8] & (1 << (bitpos % 8))) == 0 {
                return false;
            }
            h = (h as u64 + delta as u64) as u32;
        }
        true
    }
}

/// A filter policy wrapping another policy; extracting the user key from internal keys for all
/// operations.
/// A User Key is u8*.
/// An Internal Key is u8* u8{8} (where the second part encodes a tag and a sequence number).
#[derive(Clone)]
pub struct InternalFilterPolicy<FP: FilterPolicy> {
    internal: FP,
}

impl<FP: FilterPolicy> InternalFilterPolicy<FP> {
    pub fn new(inner: FP) -> InternalFilterPolicy<FP> {
        InternalFilterPolicy { internal: inner }
    }
}

impl<FP: FilterPolicy> FilterPolicy for InternalFilterPolicy<FP> {
    fn name(&self) -> &'static str {
        self.internal.name()
    }

    fn create_filter(&self, keys: &[u8], key_offsets: &[usize]) -> Vec<u8> {
        let mut mod_keys = Vec::with_capacity(keys.len() - key_offsets.len() * 8);
        let mut mod_key_offsets = Vec::with_capacity(key_offsets.len());

        offset_data_iterate(keys, key_offsets, |key| {
            mod_key_offsets.push(mod_keys.len());
            mod_keys.extend_from_slice(&key[0..key.len() - 8]);
        });
        self.internal.create_filter(&mod_keys, &mod_key_offsets)
    }

    fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool {
        self.internal.key_may_match(&key[0..key.len() - 8], filter)
    }
}

/// offset_data_iterate iterates over the entries in data that are indexed by the offsets given in
/// offsets. This is e.g. the internal format of a FilterBlock.
fn offset_data_iterate<F: FnMut(&[u8])>(data: &[u8], offsets: &[usize], mut f: F) {
    for offix in 0..offsets.len() {
        let upper = if offix == offsets.len() - 1 {
            data.len()
        } else {
            offsets[offix + 1]
        };
        let piece = &data[offsets[offix]..upper];
        f(piece);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use key_types::LookupKey;

    const _BITS_PER_KEY: u32 = 12;

    fn input_data() -> (Vec<u8>, Vec<usize>) {
        let mut concat = vec![];
        let mut offs = vec![];

        for d in [
            "abc123def456".as_bytes(),
            "xxx111xxx222".as_bytes(),
            "ab00cd00ab".as_bytes(),
            "908070605040302010".as_bytes(),
        ]
        .iter()
        {
            offs.push(concat.len());
            concat.extend_from_slice(d);
        }
        (concat, offs)
    }

    /// Creates a filter using the keys from input_data().
    fn create_filter() -> Vec<u8> {
        let fpol = BloomPolicy::new(_BITS_PER_KEY);
        let (data, offs) = input_data();
        let filter = fpol.create_filter(&data, &offs);

        assert_eq!(filter, vec![194, 148, 129, 140, 192, 196, 132, 164, 8]);
        filter
    }

    /// Creates a filter using the keys from input_data() but converted to InternalKey format.
    fn create_internalkey_filter() -> Vec<u8> {
        let fpol = Rc::new(Box::new(InternalFilterPolicy::new(BloomPolicy::new(
            _BITS_PER_KEY,
        ))));
        let (data, offs) = input_data();
        let (mut intdata, mut intoffs) = (vec![], vec![]);

        offset_data_iterate(&data, &offs, |key| {
            let ikey = LookupKey::new(key, 123);
            intoffs.push(intdata.len());
            intdata.extend_from_slice(ikey.internal_key());
        });
        let filter = fpol.create_filter(&intdata, &intoffs);

        filter
    }

    #[test]
    fn test_filter_bloom() {
        let f = create_filter();
        let fp = BloomPolicy::new(_BITS_PER_KEY);
        let (data, offs) = input_data();

        offset_data_iterate(&data, &offs, |key| {
            assert!(fp.key_may_match(key, &f));
        });
    }

    /// This test verifies that InternalFilterPolicy works correctly.
    #[test]
    fn test_filter_internal_keys_identical() {
        assert_eq!(create_filter(), create_internalkey_filter());
    }

    #[test]
    fn test_filter_bloom_hash() {
        let d1 = vec![0x62];
        let d2 = vec![0xc3, 0x97];
        let d3 = vec![0xe2, 0x99, 0xa5];
        let d4 = vec![0xe1, 0x80, 0xb9, 0x32];

        let fp = BloomPolicy::new_unwrapped(_BITS_PER_KEY);

        assert_eq!(fp.bloom_hash(&d1), 0xef1345c4);
        assert_eq!(fp.bloom_hash(&d2), 0x5b663814);
        assert_eq!(fp.bloom_hash(&d3), 0x323c078f);
        assert_eq!(fp.bloom_hash(&d4), 0xed21633a);
    }
}
