#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cmp::Cmp;
use key_types::{parse_internal_key, truncate_to_userkey, LookupKey, ValueType};
use merging_iter::MergingIter;
use snapshot::Snapshot;
use types::{Direction, LdbIterator, Shared};
use version_set::VersionSet;

use std::cmp::Ordering;
use std::mem;
use std::rc::Rc;

use rand;

const READ_BYTES_PERIOD: isize = 1048576;

/// DBIterator is an iterator over the contents of a database.
pub struct DBIterator {
    // A user comparator.
    cmp: Rc<Box<dyn Cmp>>,
    vset: Shared<VersionSet>,
    iter: MergingIter,
    // By holding onto a snapshot, we make sure that the iterator iterates over the state at the
    // point of its creation.
    ss: Snapshot,
    dir: Direction,
    byte_count: isize,

    valid: bool,
    // temporarily stored user key.
    savedkey: Vec<u8>,
    // buffer for reading internal keys
    keybuf: Vec<u8>,
    savedval: Vec<u8>,
    valbuf: Vec<u8>,
}

impl DBIterator {
    pub fn new(
        cmp: Rc<Box<dyn Cmp>>,
        vset: Shared<VersionSet>,
        iter: MergingIter,
        ss: Snapshot,
    ) -> DBIterator {
        DBIterator {
            cmp: cmp,
            vset: vset,
            iter: iter,
            ss: ss,
            dir: Direction::Forward,
            byte_count: random_period(),

            valid: false,
            savedkey: vec![],
            keybuf: vec![],
            savedval: vec![],
            valbuf: vec![],
        }
    }

    /// record_read_sample records a read sample using the current contents of self.keybuf, which
    /// should be an InternalKey.
    fn record_read_sample<'a>(&mut self, len: usize) {
        self.byte_count -= len as isize;
        if self.byte_count < 0 {
            let v = self.vset.borrow().current();
            v.borrow_mut().record_read_sample(&self.keybuf);
            while self.byte_count < 0 {
                self.byte_count += random_period();
            }
        }
    }

    /// find_next_user_entry skips to the next user entry after the one saved in self.savedkey.
    fn find_next_user_entry(&mut self, mut skipping: bool) -> bool {
        assert!(self.iter.valid());
        assert!(self.dir == Direction::Forward);

        while self.iter.valid() {
            self.iter.current(&mut self.keybuf, &mut self.savedval);
            let len = self.keybuf.len() + self.savedval.len();
            self.record_read_sample(len);
            let (typ, seq, ukey) = parse_internal_key(&self.keybuf);

            // Skip keys with a sequence number after our snapshot.
            if seq <= self.ss.sequence() {
                if typ == ValueType::TypeDeletion {
                    // Mark current (deleted) key to be skipped.
                    self.savedkey.clear();
                    self.savedkey.extend_from_slice(ukey);
                    skipping = true;
                } else if typ == ValueType::TypeValue {
                    if skipping && self.cmp.cmp(ukey, &self.savedkey) <= Ordering::Equal {
                        // Entry hidden, because it's smaller than the key to be skipped.
                    } else {
                        self.valid = true;
                        self.savedkey.clear();
                        return true;
                    }
                }
            }
            self.iter.advance();
        }
        self.savedkey.clear();
        self.valid = false;
        false
    }

    /// find_prev_user_entry, on a backwards-moving iterator, stores the newest non-deleted version
    /// of the entry with the key == self.savedkey that is in the current snapshot, into
    /// savedkey/savedval.
    fn find_prev_user_entry(&mut self) -> bool {
        assert!(self.dir == Direction::Reverse);
        let mut value_type = ValueType::TypeDeletion;

        // The iterator should be already set to the previous entry if this is a direction change
        // (i.e. first prev() call after advance()). savedkey is set to the key of that entry.
        //
        // We read the current entry, ignore it for comparison (because the initial value_type is
        // Deletion), assign it to savedkey and savedval and go back another step (at the end of
        // the loop).
        //
        // We repeat this until we hit the first entry with a different user key (possibly going
        // through newer versions of the same key, because the newest entry is first in order),
        // then break. The key and value of the latest entry for the desired key have been stored
        // in the previous iteration to savedkey and savedval.
        while self.iter.valid() {
            self.iter.current(&mut self.keybuf, &mut self.valbuf);
            let len = self.keybuf.len() + self.valbuf.len();
            self.record_read_sample(len);
            let (typ, seq, ukey) = parse_internal_key(&self.keybuf);

            if seq > 0 && seq <= self.ss.sequence() {
                if value_type != ValueType::TypeDeletion
                    && self.cmp.cmp(ukey, &self.savedkey) == Ordering::Less
                {
                    // We found a non-deleted entry for a previous key (in the previous iteration)
                    break;
                }
                value_type = typ;
                if value_type == ValueType::TypeDeletion {
                    self.savedkey.clear();
                    self.savedval.clear();
                } else {
                    self.savedkey.clear();
                    self.savedkey.extend_from_slice(ukey);

                    mem::swap(&mut self.savedval, &mut self.valbuf);
                }
            }
            self.iter.prev();
        }

        if value_type == ValueType::TypeDeletion {
            self.valid = false;
            self.savedkey.clear();
            self.savedval.clear();
            self.dir = Direction::Forward;
        } else {
            self.valid = true;
        }
        true
    }
}

impl LdbIterator for DBIterator {
    fn advance(&mut self) -> bool {
        if !self.valid() {
            self.seek_to_first();
            return self.valid();
        }

        if self.dir == Direction::Reverse {
            self.dir = Direction::Forward;
            if !self.iter.valid() {
                self.iter.seek_to_first();
            } else {
                self.iter.advance();
            }
            if !self.iter.valid() {
                self.valid = false;
                self.savedkey.clear();
                return false;
            }
        } else {
            // Save current user key.
            assert!(self.iter.current(&mut self.savedkey, &mut self.savedval));
            truncate_to_userkey(&mut self.savedkey);
        }
        self.find_next_user_entry(
            // skipping=
            true,
        )
    }
    fn current(&self, key: &mut Vec<u8>, val: &mut Vec<u8>) -> bool {
        if !self.valid() {
            return false;
        }
        // If direction is forward, savedkey and savedval are not used.
        if self.dir == Direction::Forward {
            self.iter.current(key, val);
            truncate_to_userkey(key);
            true
        } else {
            key.clear();
            key.extend_from_slice(&self.savedkey);
            val.clear();
            val.extend_from_slice(&self.savedval);
            true
        }
    }
    fn prev(&mut self) -> bool {
        if !self.valid() {
            return false;
        }

        if self.dir == Direction::Forward {
            // scan backwards until we hit a different key; then use the normal scanning procedure:
            // find_prev_user_entry() wants savedkey to be the key of the entry that is supposed to
            // be left in savedkey/savedval, which is why we have to go to the previous entry before
            // calling it.
            self.iter.current(&mut self.savedkey, &mut self.savedval);
            truncate_to_userkey(&mut self.savedkey);
            loop {
                self.iter.prev();
                if !self.iter.valid() {
                    self.valid = false;
                    self.savedkey.clear();
                    self.savedval.clear();
                    return false;
                }
                // Scan until we hit the next-smaller key.
                self.iter.current(&mut self.keybuf, &mut self.savedval);
                truncate_to_userkey(&mut self.keybuf);
                if self.cmp.cmp(&self.keybuf, &self.savedkey) == Ordering::Less {
                    break;
                }
            }
            self.dir = Direction::Reverse;
        }
        self.find_prev_user_entry()
    }
    fn valid(&self) -> bool {
        self.valid
    }
    fn seek(&mut self, to: &[u8]) {
        self.dir = Direction::Forward;
        self.savedkey.clear();
        self.savedval.clear();
        self.savedkey
            .extend_from_slice(LookupKey::new(to, self.ss.sequence()).internal_key());
        self.iter.seek(&self.savedkey);
        if self.iter.valid() {
            self.find_next_user_entry(
                // skipping=
                false,
            );
        } else {
            self.valid = false;
        }
    }
    fn seek_to_first(&mut self) {
        self.dir = Direction::Forward;
        self.savedval.clear();
        self.iter.seek_to_first();
        if self.iter.valid() {
            self.find_next_user_entry(
                // skipping=
                false,
            );
        } else {
            self.valid = false;
        }
    }
    fn reset(&mut self) {
        self.iter.reset();
        self.valid = false;
        self.savedkey.clear();
        self.savedval.clear();
        self.keybuf.clear();
    }
}

fn random_period() -> isize {
    rand::random::<isize>() % 2 * READ_BYTES_PERIOD
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_impl::testutil::*;
    use db_impl::DB;
    use test_util::LdbIteratorIter;
    use types::{current_key_val, Direction};

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn db_iter_basic_test() {
        let mut db = build_db().0;
        let mut iter = db.new_iter().unwrap();

        // keys and values come from make_version(); they are each the latest entry.
        let keys: &[&[u8]] = &[
            b"aaa", b"aab", b"aax", b"aba", b"bab", b"bba", b"cab", b"cba",
        ];
        let vals: &[&[u8]] = &[
            b"val1", b"val2", b"val2", b"val3", b"val4", b"val5", b"val2", b"val3",
        ];

        for (k, v) in keys.iter().zip(vals.iter()) {
            assert!(iter.advance());
            assert_eq!((k.to_vec(), v.to_vec()), current_key_val(&iter).unwrap());
        }
    }

    #[test]
    fn db_iter_reset() {
        let mut db = build_db().0;
        let mut iter = db.new_iter().unwrap();

        assert!(iter.advance());
        assert!(iter.valid());
        iter.reset();
        assert!(!iter.valid());
        assert!(iter.advance());
        assert!(iter.valid());
    }

    #[test]
    fn db_iter_test_fwd_backwd() {
        let mut db = build_db().0;
        let mut iter = db.new_iter().unwrap();

        // keys and values come from make_version(); they are each the latest entry.
        let keys: &[&[u8]] = &[
            b"aaa", b"aab", b"aax", b"aba", b"bab", b"bba", b"cab", b"cba",
        ];
        let vals: &[&[u8]] = &[
            b"val1", b"val2", b"val2", b"val3", b"val4", b"val5", b"val2", b"val3",
        ];

        // This specifies the direction that the iterator should move to. Based on this, an index
        // into keys/vals is incremented/decremented so that we get a nice test checking iterator
        // move correctness.
        let dirs: &[Direction] = &[
            Direction::Forward,
            Direction::Forward,
            Direction::Forward,
            Direction::Reverse,
            Direction::Reverse,
            Direction::Reverse,
            Direction::Forward,
            Direction::Forward,
            Direction::Reverse,
            Direction::Forward,
            Direction::Forward,
            Direction::Forward,
            Direction::Forward,
        ];
        let mut i = 0;
        iter.advance();
        for d in dirs {
            assert_eq!(
                (keys[i].to_vec(), vals[i].to_vec()),
                current_key_val(&iter).unwrap()
            );
            match *d {
                Direction::Forward => {
                    assert!(iter.advance());
                    i += 1;
                }
                Direction::Reverse => {
                    assert!(iter.prev());
                    i -= 1;
                }
            }
        }
    }

    #[test]
    fn db_iter_test_seek() {
        let mut db = build_db().0;
        let mut iter = db.new_iter().unwrap();

        // gca is the deleted entry.
        let keys: &[&[u8]] = &[b"aab", b"aaa", b"cab", b"eaa", b"aaa", b"iba", b"fba"];
        let vals: &[&[u8]] = &[
            b"val2", b"val1", b"val2", b"val1", b"val1", b"val2", b"val3",
        ];

        for (k, v) in keys.iter().zip(vals.iter()) {
            println!("{:?}", String::from_utf8(k.to_vec()).unwrap());
            iter.seek(k);
            assert_eq!((k.to_vec(), v.to_vec()), current_key_val(&iter).unwrap());
        }

        // seek past last.
        iter.seek(b"xxx");
        assert!(!iter.valid());
        iter.seek(b"aab");
        assert!(iter.valid());

        // Seek skips over deleted entry.
        iter.seek(b"gca");
        assert!(iter.valid());
        assert_eq!(
            (b"gda".to_vec(), b"val5".to_vec()),
            current_key_val(&iter).unwrap()
        );
    }

    #[test]
    fn db_iter_deleted_entry_not_returned() {
        let mut db = build_db().0;
        let mut iter = db.new_iter().unwrap();
        let must_not_appear = b"gca";

        for (k, _) in LdbIteratorIter::wrap(&mut iter) {
            assert!(k.as_slice() != must_not_appear);
        }
    }

    #[test]
    fn db_iter_deleted_entry_not_returned_memtable() {
        let mut db = build_db().0;

        db.put(b"xyz", b"123").unwrap();
        db.delete(b"xyz").unwrap();

        let mut iter = db.new_iter().unwrap();
        let must_not_appear = b"xyz";

        for (k, _) in LdbIteratorIter::wrap(&mut iter) {
            assert!(k.as_slice() != must_not_appear);
        }
    }

    #[test]
    fn db_iter_repeated_open_close() {
        let opt;
        {
            let (mut db, opt_) = build_db();
            opt = opt_;

            db.put(b"xx1", b"111").unwrap();
            db.put(b"xx2", b"112").unwrap();
            db.put(b"xx3", b"113").unwrap();
            db.put(b"xx4", b"114").unwrap();
            db.delete(b"xx2").unwrap();
        }

        {
            let mut db = DB::open("db", opt.clone()).unwrap();
            db.put(b"xx4", b"222").unwrap();
        }

        {
            let mut db = DB::open("db", opt).unwrap();

            let ss = db.get_snapshot();
            // xx5 should not be visible.
            db.put(b"xx5", b"223").unwrap();

            let expected: HashMap<Vec<u8>, Vec<u8>> = HashMap::from_iter(
                vec![
                    (b"xx1".to_vec(), b"111".to_vec()),
                    (b"xx4".to_vec(), b"222".to_vec()),
                    (b"aaa".to_vec(), b"val1".to_vec()),
                    (b"cab".to_vec(), b"val2".to_vec()),
                ]
                .into_iter(),
            );
            let non_existing: HashSet<Vec<u8>> = HashSet::from_iter(
                vec![b"gca".to_vec(), b"xx2".to_vec(), b"xx5".to_vec()].into_iter(),
            );

            let mut iter = db.new_iter_at(ss.clone()).unwrap();
            for (k, v) in LdbIteratorIter::wrap(&mut iter) {
                if let Some(ev) = expected.get(&k) {
                    assert_eq!(ev, &v);
                }
                assert!(!non_existing.contains(&k));
            }
        }
    }
}
