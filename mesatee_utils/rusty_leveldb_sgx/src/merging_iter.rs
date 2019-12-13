#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cmp::Cmp;
use types::{current_key_val, Direction, LdbIterator};

use std::cmp::Ordering;
use std::rc::Rc;

// Warning: This module is kinda messy. The original implementation is
// not that much better though :-)
//
// Issues: 1) prev() may not work correctly at the beginning of a merging
// iterator.

#[derive(PartialEq)]
enum SL {
    Smallest,
    Largest,
}

pub struct MergingIter {
    iters: Vec<Box<dyn LdbIterator>>,
    current: Option<usize>,
    direction: Direction,
    cmp: Rc<Box<dyn Cmp>>,
}

impl MergingIter {
    /// Construct a new merging iterator.
    pub fn new(cmp: Rc<Box<dyn Cmp>>, iters: Vec<Box<dyn LdbIterator>>) -> MergingIter {
        let mi = MergingIter {
            iters: iters,
            current: None,
            direction: Direction::Forward,
            cmp: cmp,
        };
        mi
    }

    fn init(&mut self) {
        for i in 0..self.iters.len() {
            self.iters[i].reset();
            self.iters[i].advance();
            if !self.iters[i].valid() {
                self.iters[i].reset()
            }
        }
        self.find_smallest();
    }

    /// Adjusts the direction of the iterator depending on whether the last
    /// call was next() or prev(). This basically sets all iterators to one
    /// entry after (Forward) or one entry before (Reverse) the current() entry.
    fn update_direction(&mut self, d: Direction) {
        if self.direction == d {
            return;
        }

        let mut keybuf = vec![];
        let mut valbuf = vec![];

        if let Some((key, _)) = current_key_val(self) {
            if let Some(current) = self.current {
                match d {
                    Direction::Forward if self.direction == Direction::Reverse => {
                        self.direction = Direction::Forward;
                        for i in 0..self.iters.len() {
                            if i != current {
                                self.iters[i].seek(&key);
                                // This doesn't work if two iterators are returning the exact same
                                // keys. However, in reality, two entries will always have differing
                                // sequence numbers.
                                if self.iters[i].current(&mut keybuf, &mut valbuf) {
                                    if self.cmp.cmp(&keybuf, &key) == Ordering::Equal {
                                        self.iters[i].advance();
                                    }
                                }
                            }
                        }
                    }
                    Direction::Reverse if self.direction == Direction::Forward => {
                        self.direction = Direction::Reverse;
                        for i in 0..self.iters.len() {
                            if i != current {
                                self.iters[i].seek(&key);
                                if self.iters[i].valid() {
                                    self.iters[i].prev();
                                } else {
                                    // seek to last.
                                    while self.iters[i].advance() {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn find_smallest(&mut self) {
        self.find(SL::Smallest)
    }
    fn find_largest(&mut self) {
        self.find(SL::Largest)
    }

    fn find(&mut self, direction: SL) {
        if self.iters.len() == 0 {
            // Iterator stays invalid.
            return;
        }

        let ord;

        if direction == SL::Smallest {
            ord = Ordering::Less;
        } else {
            ord = Ordering::Greater;
        }

        let mut next_ix = 0;
        let (mut current, mut smallest, mut valscratch) = (vec![], vec![], vec![]);

        for i in 1..self.iters.len() {
            if self.iters[i].current(&mut current, &mut valscratch) {
                if self.iters[next_ix].current(&mut smallest, &mut valscratch) {
                    if self.cmp.cmp(&current, &smallest) == ord {
                        next_ix = i;
                    }
                } else {
                    next_ix = i;
                }
            }
        }

        self.current = Some(next_ix);
    }
}

impl LdbIterator for MergingIter {
    fn advance(&mut self) -> bool {
        if let Some(current) = self.current {
            self.update_direction(Direction::Forward);
            if !self.iters[current].advance() {
                // Take this iterator out of rotation; this will return false
                // for every call to current() and thus it will be ignored
                // from here on.
                self.iters[current].reset();
            }
            self.find_smallest();
        } else {
            self.init();
        }
        self.valid()
    }
    fn valid(&self) -> bool {
        if let Some(ix) = self.current {
            self.iters[ix].valid()
        } else {
            false
        }
    }
    fn seek(&mut self, key: &[u8]) {
        for i in 0..self.iters.len() {
            self.iters[i].seek(key);
        }
        self.find_smallest();
    }
    fn reset(&mut self) {
        for i in 0..self.iters.len() {
            self.iters[i].reset();
        }
        self.current = None;
    }
    fn current(&self, key: &mut Vec<u8>, val: &mut Vec<u8>) -> bool {
        if let Some(ix) = self.current {
            self.iters[ix].current(key, val)
        } else {
            false
        }
    }
    fn prev(&mut self) -> bool {
        if let Some(current) = self.current {
            if self.iters[current].valid() {
                self.update_direction(Direction::Reverse);
                self.iters[current].prev();
                self.find_largest();
                self.valid()
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cmp::DefaultCmp;
    use skipmap::tests;
    use test_util::{test_iterator_properties, LdbIteratorIter, TestLdbIter};
    use types::{current_key_val, LdbIterator};

    #[test]
    fn test_merging_one() {
        let skm = tests::make_skipmap();
        let iter = skm.iter();
        let mut iter2 = skm.iter();

        let mut miter = MergingIter::new(Rc::new(Box::new(DefaultCmp)), vec![Box::new(iter)]);

        loop {
            if let Some((k, v)) = miter.next() {
                if let Some((k2, v2)) = iter2.next() {
                    assert_eq!(k, k2);
                    assert_eq!(v, v2);
                } else {
                    panic!("Expected element from iter2");
                }
            } else {
                break;
            }
        }
    }

    #[test]
    fn test_merging_two() {
        let skm = tests::make_skipmap();
        let iter = skm.iter();
        let iter2 = skm.iter();

        let mut miter = MergingIter::new(
            Rc::new(Box::new(DefaultCmp)),
            vec![Box::new(iter), Box::new(iter2)],
        );

        loop {
            if let Some((k, v)) = miter.next() {
                if let Some((k2, v2)) = miter.next() {
                    assert_eq!(k, k2);
                    assert_eq!(v, v2);
                } else {
                    panic!("Odd number of elements");
                }
            } else {
                break;
            }
        }
    }

    #[test]
    fn test_merging_zero() {
        let mut miter = MergingIter::new(Rc::new(Box::new(DefaultCmp)), vec![]);
        assert_eq!(0, LdbIteratorIter::wrap(&mut miter).count());
    }

    #[test]
    fn test_merging_behavior() {
        let val = "def".as_bytes();
        let iter = TestLdbIter::new(vec![(b("aba"), val), (b("abc"), val)]);
        let iter2 = TestLdbIter::new(vec![(b("abb"), val), (b("abd"), val)]);
        let miter = MergingIter::new(
            Rc::new(Box::new(DefaultCmp)),
            vec![Box::new(iter), Box::new(iter2)],
        );
        test_iterator_properties(miter);
    }

    #[test]
    fn test_merging_forward_backward() {
        let val = "def".as_bytes();
        let iter = TestLdbIter::new(vec![(b("aba"), val), (b("abc"), val), (b("abe"), val)]);
        let iter2 = TestLdbIter::new(vec![(b("abb"), val), (b("abd"), val)]);

        let mut miter = MergingIter::new(
            Rc::new(Box::new(DefaultCmp)),
            vec![Box::new(iter), Box::new(iter2)],
        );

        // miter should return the following sequence: [aba, abb, abc, abd, abe]

        // -> aba
        let first = miter.next();
        // -> abb
        let second = miter.next();
        // -> abc
        let third = miter.next();
        println!("{:?} {:?} {:?}", first, second, third);

        assert!(first != third);
        // abb <-
        assert!(miter.prev());
        assert_eq!(second, current_key_val(&miter));
        // aba <-
        assert!(miter.prev());
        assert_eq!(first, current_key_val(&miter));
        // -> abb
        assert!(miter.advance());
        assert_eq!(second, current_key_val(&miter));
        // -> abc
        assert!(miter.advance());
        assert_eq!(third, current_key_val(&miter));
        // -> abd
        assert!(miter.advance());
        assert_eq!(
            Some((b("abd").to_vec(), val.to_vec())),
            current_key_val(&miter)
        );
    }

    fn b(s: &'static str) -> &'static [u8] {
        s.as_bytes()
    }

    #[test]
    fn test_merging_real() {
        let val = "def".as_bytes();

        let it1 = TestLdbIter::new(vec![(&b("aba"), val), (&b("abc"), val), (&b("abe"), val)]);
        let it2 = TestLdbIter::new(vec![(&b("abb"), val), (&b("abd"), val)]);
        let expected = vec![b("aba"), b("abb"), b("abc"), b("abd"), b("abe")];

        let mut iter = MergingIter::new(
            Rc::new(Box::new(DefaultCmp)),
            vec![Box::new(it1), Box::new(it2)],
        );

        let mut i = 0;
        for (k, _) in LdbIteratorIter::wrap(&mut iter) {
            assert_eq!(k, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_merging_seek_reset() {
        let val = "def".as_bytes();

        let it1 = TestLdbIter::new(vec![(b("aba"), val), (b("abc"), val), (b("abe"), val)]);
        let it2 = TestLdbIter::new(vec![(b("abb"), val), (b("abd"), val)]);

        let mut iter = MergingIter::new(
            Rc::new(Box::new(DefaultCmp)),
            vec![Box::new(it1), Box::new(it2)],
        );

        assert!(!iter.valid());
        iter.advance();
        assert!(iter.valid());
        assert!(current_key_val(&iter).is_some());

        iter.seek("abc".as_bytes());
        assert_eq!(
            current_key_val(&iter),
            Some((b("abc").to_vec(), val.to_vec()))
        );
        iter.seek("ab0".as_bytes());
        assert_eq!(
            current_key_val(&iter),
            Some((b("aba").to_vec(), val.to_vec()))
        );
        iter.seek("abx".as_bytes());
        assert_eq!(current_key_val(&iter), None);

        iter.reset();
        assert!(!iter.valid());
        iter.next();
        assert_eq!(
            current_key_val(&iter),
            Some((b("aba").to_vec(), val.to_vec()))
        );
    }
}
