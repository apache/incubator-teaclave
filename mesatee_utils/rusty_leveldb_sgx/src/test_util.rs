#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cmp::{Cmp, DefaultCmp};
use types::{current_key_val, LdbIterator};

use std::cmp::Ordering;

/// TestLdbIter is an LdbIterator over a vector, to be used for testing purposes.
pub struct TestLdbIter<'a> {
    v: Vec<(&'a [u8], &'a [u8])>,
    ix: usize,
    init: bool,
}

impl<'a> TestLdbIter<'a> {
    pub fn new(c: Vec<(&'a [u8], &'a [u8])>) -> TestLdbIter<'a> {
        return TestLdbIter {
            v: c,
            ix: 0,
            init: false,
        };
    }
}

impl<'a> LdbIterator for TestLdbIter<'a> {
    fn advance(&mut self) -> bool {
        if self.ix == self.v.len() - 1 {
            self.ix += 1;
            false
        } else if !self.init {
            self.init = true;
            true
        } else {
            self.ix += 1;
            true
        }
    }
    fn reset(&mut self) {
        self.ix = 0;
        self.init = false;
    }
    fn current(&self, key: &mut Vec<u8>, val: &mut Vec<u8>) -> bool {
        if self.init && self.ix < self.v.len() {
            key.clear();
            val.clear();
            key.extend_from_slice(self.v[self.ix].0);
            val.extend_from_slice(self.v[self.ix].1);
            true
        } else {
            false
        }
    }
    fn valid(&self) -> bool {
        self.init && self.ix < self.v.len()
    }
    fn seek(&mut self, k: &[u8]) {
        self.ix = 0;
        self.init = true;
        while self.ix < self.v.len() && DefaultCmp.cmp(self.v[self.ix].0, k) == Ordering::Less {
            self.ix += 1;
        }
    }
    fn prev(&mut self) -> bool {
        if !self.init || self.ix == 0 {
            self.init = false;
            false
        } else {
            self.ix -= 1;
            true
        }
    }
}

/// LdbIteratorIter implements std::iter::Iterator for an LdbIterator.
pub struct LdbIteratorIter<'a, It: 'a> {
    inner: &'a mut It,
}

impl<'a, It: LdbIterator> LdbIteratorIter<'a, It> {
    pub fn wrap(it: &'a mut It) -> LdbIteratorIter<'a, It> {
        LdbIteratorIter { inner: it }
    }
}

impl<'a, It: LdbIterator> Iterator for LdbIteratorIter<'a, It> {
    type Item = (Vec<u8>, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        LdbIterator::next(self.inner)
    }
}

/// This shared test takes an iterator with exactly four elements and tests that it fulfills the
/// generic iterator properties. Every iterator defined in this code base should pass this test.
pub fn test_iterator_properties<It: LdbIterator>(mut it: It) {
    assert!(!it.valid());
    assert!(it.advance());
    assert!(it.valid());
    let first = current_key_val(&it);
    assert!(it.advance());
    let second = current_key_val(&it);
    assert!(it.advance());
    let third = current_key_val(&it);
    // fourth (last) element
    assert!(it.advance());
    assert!(it.valid());
    let fourth = current_key_val(&it);
    // past end is invalid
    assert!(!it.advance());
    assert!(!it.valid());

    it.reset();
    it.seek(&fourth.as_ref().unwrap().0);
    assert!(it.valid());
    it.seek(&second.as_ref().unwrap().0);
    assert!(it.valid());
    it.prev();
    assert_eq!(first, current_key_val(&it));

    it.reset();
    assert!(!it.valid());
    assert!(it.advance());
    assert_eq!(first, current_key_val(&it));
    assert!(it.advance());
    assert_eq!(second, current_key_val(&it));
    assert!(it.advance());
    assert_eq!(third, current_key_val(&it));
    assert!(it.prev());
    assert_eq!(second, current_key_val(&it));
    assert!(it.prev());
    assert_eq!(first, current_key_val(&it));
    assert!(!it.prev());
    assert!(!it.valid());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_util_basic() {
        let v = vec![
            ("abc".as_bytes(), "def".as_bytes()),
            ("abd".as_bytes(), "deg".as_bytes()),
        ];
        let mut iter = TestLdbIter::new(v);
        assert_eq!(
            iter.next(),
            Some((Vec::from("abc".as_bytes()), Vec::from("def".as_bytes())))
        );
    }

    #[test]
    fn test_test_util_ldbiter_properties() {
        time_test!();
        let v;
        {
            time_test!("init");
            v = vec![
                ("abc".as_bytes(), "def".as_bytes()),
                ("abd".as_bytes(), "deg".as_bytes()),
                ("abe".as_bytes(), "deg".as_bytes()),
                ("abf".as_bytes(), "deg".as_bytes()),
            ];
        }
        test_iterator_properties(TestLdbIter::new(v));
    }
}
