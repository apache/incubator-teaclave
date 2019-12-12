#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cmp::{Cmp, InternalKeyCmp};
use error::Result;
use key_types::{parse_internal_key, InternalKey, LookupKey, UserKey, ValueType};
use table_cache::TableCache;
use table_reader::TableIterator;
use types::{FileMetaData, FileNum, LdbIterator, Shared, MAX_SEQUENCE_NUMBER, NUM_LEVELS};

use std::cmp::Ordering;
use std::default::Default;
use std::rc::Rc;

/// FileMetaHandle is a reference-counted FileMetaData object with interior mutability. This is
/// necessary to provide a shared metadata container that can be modified while referenced by e.g.
/// multiple versions.
pub type FileMetaHandle = Shared<FileMetaData>;

/// Contains statistics about seeks occurred in a file.
pub struct GetStats {
    file: Option<FileMetaHandle>,
    level: usize,
}

pub struct Version {
    table_cache: Shared<TableCache>,
    user_cmp: Rc<Box<dyn Cmp>>,
    pub files: [Vec<FileMetaHandle>; NUM_LEVELS],

    pub file_to_compact: Option<FileMetaHandle>,
    pub file_to_compact_lvl: usize,
    pub compaction_score: Option<f64>,
    pub compaction_level: Option<usize>,
}

impl Version {
    pub fn new(cache: Shared<TableCache>, ucmp: Rc<Box<dyn Cmp>>) -> Version {
        Version {
            table_cache: cache,
            user_cmp: ucmp,
            files: Default::default(),
            file_to_compact: None,
            file_to_compact_lvl: 0,
            compaction_score: None,
            compaction_level: None,
        }
    }

    pub fn num_level_bytes(&self, l: usize) -> usize {
        assert!(l < NUM_LEVELS);
        total_size(self.files[l].iter())
    }

    pub fn num_level_files(&self, l: usize) -> usize {
        assert!(l < NUM_LEVELS);
        self.files[l].len()
    }

    /// get returns the value for the specified key using the persistent tables contained in this
    /// Version.
    #[allow(unused_assignments)]
    pub fn get<'a>(&self, key: InternalKey<'a>) -> Result<Option<(Vec<u8>, GetStats)>> {
        let levels = self.get_overlapping(key);
        let ikey = key;
        let ukey = parse_internal_key(ikey).2;

        let mut stats = GetStats {
            file: None,
            level: 0,
        };

        for level in 0..levels.len() {
            let files = &levels[level];
            let mut last_read = None;
            let mut last_read_level: usize = 0;
            for f in files {
                if last_read.is_some() && stats.file.is_none() {
                    stats.file = last_read.clone();
                    stats.level = last_read_level;
                }
                last_read_level = level;
                last_read = Some(f.clone());

                // We receive both key and value from the table. Because we're using InternalKey
                // keys, we now need to check whether the found entry's user key is equal to the
                // one we're looking for (get() just returns the next-bigger key).
                if let Ok(Some((k, v))) = self.table_cache.borrow_mut().get(f.borrow().num, ikey) {
                    // We don't need to check the sequence number; get() will not return an entry
                    // with a higher sequence number than the one in the supplied key.
                    let (typ, _, foundkey) = parse_internal_key(&k);
                    if typ == ValueType::TypeValue
                        && self.user_cmp.cmp(foundkey, ukey) == Ordering::Equal
                    {
                        return Ok(Some((v, stats)));
                    } else if typ == ValueType::TypeDeletion {
                        // Skip looking once we have found a deletion.
                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
    }

    /// get_overlapping returns the files overlapping key in each level.
    fn get_overlapping<'a>(&self, key: InternalKey<'a>) -> [Vec<FileMetaHandle>; NUM_LEVELS] {
        let mut levels: [Vec<FileMetaHandle>; NUM_LEVELS] = Default::default();
        let ikey = key;
        let ukey = parse_internal_key(key).2;

        let files = &self.files[0];
        levels[0].reserve(files.len());
        for f_ in files {
            let f = f_.borrow();
            let (fsmallest, flargest) = (
                parse_internal_key(&f.smallest).2,
                parse_internal_key(&f.largest).2,
            );
            if self.user_cmp.cmp(ukey, fsmallest) >= Ordering::Equal
                && self.user_cmp.cmp(ukey, flargest) <= Ordering::Equal
            {
                levels[0].push(f_.clone());
            }
        }
        // Sort by newest first.
        levels[0].sort_by(|a, b| b.borrow().num.cmp(&a.borrow().num));

        let icmp = InternalKeyCmp(self.user_cmp.clone());
        for level in 1..NUM_LEVELS {
            let files = &self.files[level];
            if let Some(ix) = find_file(&icmp, files, ikey) {
                let f = files[ix].borrow();
                let fsmallest = parse_internal_key(&f.smallest).2;
                if self.user_cmp.cmp(ukey, fsmallest) >= Ordering::Equal {
                    levels[level].push(files[ix].clone());
                }
            }
        }

        levels
    }

    /// level_summary returns a summary of the distribution of tables and bytes in this version.
    pub fn level_summary(&self) -> String {
        let mut acc = String::with_capacity(256);
        for level in 0..NUM_LEVELS {
            let fs = &self.files[level];
            if fs.is_empty() {
                continue;
            }
            let filedesc: Vec<(FileNum, usize)> = fs
                .iter()
                .map(|f| (f.borrow().num, f.borrow().size))
                .collect();
            let desc = format!(
                "level {}: {} files, {} bytes ({:?}); ",
                level,
                fs.len(),
                total_size(fs.iter()),
                filedesc
            );
            acc.push_str(&desc);
        }
        acc
    }

    pub fn pick_memtable_output_level<'a, 'b>(&self, min: UserKey<'a>, max: UserKey<'b>) -> usize {
        let mut level = 0;
        if !self.overlap_in_level(0, min, max) {
            // Go to next level as long as there is no overlap in that level and a limited overlap
            // in the next-higher level.
            let start = LookupKey::new(min, MAX_SEQUENCE_NUMBER);
            let limit = LookupKey::new_full(max, 0, ValueType::TypeDeletion);

            const MAX_MEM_COMPACT_LEVEL: usize = 2;
            while level < MAX_MEM_COMPACT_LEVEL {
                if self.overlap_in_level(level + 1, min, max) {
                    break;
                }
                if level + 2 < NUM_LEVELS {
                    let overlaps = self.overlapping_inputs(
                        level + 2,
                        start.internal_key(),
                        &limit.internal_key(),
                    );
                    let size = total_size(overlaps.iter());
                    if size > 10 * (2 << 20) {
                        break;
                    }
                }
                level += 1;
            }
        }
        return level;
    }

    /// record_read_sample returns true if there is a new file to be compacted. It counts the
    /// number of files overlapping a key, and which level contains the first overlap.
    #[allow(unused_assignments)]
    pub fn record_read_sample<'a>(&mut self, key: InternalKey<'a>) -> bool {
        let levels = self.get_overlapping(key);
        let mut contained_in = 0;
        let mut i = 0;
        let mut first_file = None;
        let mut first_file_level = None;
        for level in &levels {
            if !level.is_empty() {
                if first_file.is_none() && first_file_level.is_none() {
                    first_file = Some(level[0].clone());
                    first_file_level = Some(i);
                }
            }
            contained_in += level.len();
            i += 1;
        }

        if contained_in > 1 {
            self.update_stats(GetStats {
                file: first_file,
                level: first_file_level.unwrap_or(0),
            })
        } else {
            false
        }
    }

    /// update_stats updates the number of seeks, and remembers files with too many seeks as
    /// compaction candidates. It returns true if a compaction makes sense.
    pub fn update_stats(&mut self, stats: GetStats) -> bool {
        if let Some(file) = stats.file {
            if file.borrow().allowed_seeks <= 1 && self.file_to_compact.is_none() {
                self.file_to_compact = Some(file.clone());
                self.file_to_compact_lvl = stats.level;
                return true;
            } else if file.borrow().allowed_seeks > 0 {
                file.borrow_mut().allowed_seeks -= 1;
            }
        }
        false
    }

    /// max_next_level_overlapping_bytes returns how many bytes of tables are overlapped in l+1 by
    /// tables in l, for the maximum case.
    fn max_next_level_overlapping_bytes(&self) -> usize {
        let mut max = 0;
        for lvl in 1..NUM_LEVELS - 1 {
            for f in &self.files[lvl] {
                let f = f.borrow();
                let ols = self.overlapping_inputs(lvl + 1, &f.smallest, &f.largest);
                let sum = total_size(ols.iter());
                if sum > max {
                    max = sum;
                }
            }
        }
        max
    }

    /// overlap_in_level returns true if the specified level's files overlap the range [smallest;
    /// largest].
    pub fn overlap_in_level<'a, 'b>(
        &self,
        level: usize,
        smallest: UserKey<'a>,
        largest: UserKey<'a>,
    ) -> bool {
        assert!(level < NUM_LEVELS);
        if level == 0 {
            some_file_overlaps_range_disjoint(
                &InternalKeyCmp(self.user_cmp.clone()),
                &self.files[level],
                smallest,
                largest,
            )
        } else {
            some_file_overlaps_range(
                &InternalKeyCmp(self.user_cmp.clone()),
                &self.files[level],
                smallest,
                largest,
            )
        }
    }

    /// overlapping_inputs returns all files that may contain keys between begin and end.
    pub fn overlapping_inputs<'a, 'b>(
        &self,
        level: usize,
        begin: InternalKey<'a>,
        end: InternalKey<'b>,
    ) -> Vec<FileMetaHandle> {
        assert!(level < NUM_LEVELS);
        let (mut ubegin, mut uend) = (
            parse_internal_key(begin).2.to_vec(),
            parse_internal_key(end).2.to_vec(),
        );

        loop {
            match do_search(self, level, ubegin, uend) {
                (Some((newubegin, newuend)), _) => {
                    ubegin = newubegin;
                    uend = newuend;
                }
                (None, result) => return result,
            }
        }

        // the actual search happens in this inner function. This is done to enhance the control
        // flow. It takes the smallest and largest user keys and returns a new pair of user keys if
        // the search range should be expanded, or a list of overlapping files.
        fn do_search(
            myself: &Version,
            level: usize,
            ubegin: Vec<u8>,
            uend: Vec<u8>,
        ) -> (Option<(Vec<u8>, Vec<u8>)>, Vec<FileMetaHandle>) {
            let mut inputs = vec![];
            for f_ in myself.files[level].iter() {
                let f = f_.borrow();
                let (fsmallest, flargest) = (
                    parse_internal_key(&f.smallest).2,
                    parse_internal_key(&f.largest).2,
                );
                // Skip files that are not overlapping.
                if !ubegin.is_empty() && myself.user_cmp.cmp(flargest, &ubegin) == Ordering::Less {
                    continue;
                } else if !uend.is_empty()
                    && myself.user_cmp.cmp(fsmallest, &uend) == Ordering::Greater
                {
                    continue;
                } else {
                    inputs.push(f_.clone());
                    // In level 0, files may overlap each other. Check if the new file begins
                    // before ubegin or ends after uend, and expand the range, if so. Then, restart
                    // the search.
                    if level == 0 {
                        if !ubegin.is_empty()
                            && myself.user_cmp.cmp(fsmallest, &ubegin) == Ordering::Less
                        {
                            return (Some((fsmallest.to_vec(), uend)), inputs);
                        } else if !uend.is_empty()
                            && myself.user_cmp.cmp(flargest, &uend) == Ordering::Greater
                        {
                            return (Some((ubegin, flargest.to_vec())), inputs);
                        }
                    }
                }
            }
            (None, inputs)
        }
    }

    /// new_concat_iter returns an iterator that iterates over the files in a level. Note that this
    /// only really makes sense for levels > 0.
    fn new_concat_iter(&self, level: usize) -> VersionIter {
        new_version_iter(
            self.files[level].clone(),
            self.table_cache.clone(),
            self.user_cmp.clone(),
        )
    }

    /// new_iters returns a set of iterators that can be merged to yield all entries in this
    /// version.
    pub fn new_iters(&self) -> Result<Vec<Box<dyn LdbIterator>>> {
        let mut iters: Vec<Box<dyn LdbIterator>> = vec![];
        for f in &self.files[0] {
            iters.push(Box::new(
                self.table_cache
                    .borrow_mut()
                    .get_table(f.borrow().num)?
                    .iter(),
            ));
        }

        for l in 1..NUM_LEVELS {
            if !self.files[l].is_empty() {
                iters.push(Box::new(self.new_concat_iter(l)));
            }
        }

        Ok(iters)
    }
}

/// new_version_iter returns an iterator over the entries in the specified ordered list of table
/// files.
pub fn new_version_iter(
    files: Vec<FileMetaHandle>,
    cache: Shared<TableCache>,
    ucmp: Rc<Box<dyn Cmp>>,
) -> VersionIter {
    VersionIter {
        files: files,
        cache: cache,
        cmp: InternalKeyCmp(ucmp),
        current: None,
        current_ix: 0,
    }
}

/// VersionIter iterates over the entries in an ordered list of table files (specifically, for
/// example, the tables in a level).
///
/// Note that VersionIter returns entries of type Deletion.
pub struct VersionIter {
    // NOTE: Maybe we need to change this to Rc to support modification of the file set after
    // creation of the iterator. Versions should be immutable, though.
    files: Vec<FileMetaHandle>,
    cache: Shared<TableCache>,
    cmp: InternalKeyCmp,

    current: Option<TableIterator>,
    current_ix: usize,
}

impl LdbIterator for VersionIter {
    fn advance(&mut self) -> bool {
        assert!(!self.files.is_empty());

        if let Some(ref mut t) = self.current {
            if t.advance() {
                return true;
            } else if self.current_ix >= self.files.len() - 1 {
                // Already on last table; can't advance further.
                return false;
            }

            // Load next table if current table is exhausted and we have more tables to go through.
            self.current_ix += 1;
        }

        // Initialize iterator or load next table.
        if let Ok(tbl) = self
            .cache
            .borrow_mut()
            .get_table(self.files[self.current_ix].borrow().num)
        {
            self.current = Some(tbl.iter());
        } else {
            return false;
        }
        self.advance()
    }
    fn current(&self, key: &mut Vec<u8>, val: &mut Vec<u8>) -> bool {
        if let Some(ref t) = self.current {
            t.current(key, val)
        } else {
            false
        }
    }
    fn seek(&mut self, key: &[u8]) {
        if let Some(ix) = find_file(&self.cmp, &self.files, key) {
            if let Ok(tbl) = self
                .cache
                .borrow_mut()
                .get_table(self.files[ix].borrow().num)
            {
                let mut iter = tbl.iter();
                iter.seek(key);
                if iter.valid() {
                    self.current_ix = ix;
                    self.current = Some(iter);
                    return;
                }
            }
        }
        self.reset();
    }
    fn reset(&mut self) {
        self.current = None;
        self.current_ix = 0;
    }
    fn valid(&self) -> bool {
        self.current.as_ref().map(|t| t.valid()).unwrap_or(false)
    }
    fn prev(&mut self) -> bool {
        if let Some(ref mut t) = self.current {
            if t.prev() {
                return true;
            } else if self.current_ix > 0 {
                let f = &self.files[self.current_ix - 1];
                // Find previous table, seek to last entry.
                if let Ok(tbl) = self.cache.borrow_mut().get_table(f.borrow().num) {
                    let mut iter = tbl.iter();
                    iter.seek(&f.borrow().largest);
                    // The saved largest key must be in the table.
                    assert!(iter.valid());
                    self.current_ix -= 1;
                    *t = iter;
                    return true;
                }
            }
        }
        self.reset();
        false
    }
}

/// total_size returns the sum of sizes of the given files.
pub fn total_size<'a, I: Iterator<Item = &'a FileMetaHandle>>(files: I) -> usize {
    files.fold(0, |a, f| a + f.borrow().size)
}

/// key_is_after_file returns true if the given user key is larger than the largest key in f.
fn key_is_after_file<'a>(cmp: &InternalKeyCmp, key: UserKey<'a>, f: &FileMetaHandle) -> bool {
    let f = f.borrow();
    let ulargest = parse_internal_key(&f.largest).2;
    !key.is_empty() && cmp.cmp_inner(key, ulargest) == Ordering::Greater
}

/// key_is_before_file returns true if the given user key is larger than the largest key in f.
fn key_is_before_file<'a>(cmp: &InternalKeyCmp, key: UserKey<'a>, f: &FileMetaHandle) -> bool {
    let f = f.borrow();
    let usmallest = parse_internal_key(&f.smallest).2;
    !key.is_empty() && cmp.cmp_inner(key, usmallest) == Ordering::Less
}

/// find_file returns the index of the file in files that potentially contains the internal key
/// key. files must not overlap and be ordered ascendingly. If no file can contain the key, None is
/// returned.
fn find_file<'a>(
    cmp: &InternalKeyCmp,
    files: &[FileMetaHandle],
    key: InternalKey<'a>,
) -> Option<usize> {
    let (mut left, mut right) = (0, files.len());
    while left < right {
        let mid = (left + right) / 2;
        if cmp.cmp(&files[mid].borrow().largest, key) == Ordering::Less {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    if right < files.len() {
        Some(right)
    } else {
        None
    }
}

/// some_file_overlaps_range_disjoint returns true if any of the given disjoint files (i.e. level >
/// 1) contain keys in the range defined by the user keys [smallest; largest].
fn some_file_overlaps_range_disjoint<'a, 'b>(
    cmp: &InternalKeyCmp,
    files: &[FileMetaHandle],
    smallest: UserKey<'a>,
    largest: UserKey<'b>,
) -> bool {
    let ikey = LookupKey::new(smallest, MAX_SEQUENCE_NUMBER);
    if let Some(ix) = find_file(cmp, files, ikey.internal_key()) {
        !key_is_before_file(cmp, largest, &files[ix])
    } else {
        false
    }
}

/// some_file_overlaps_range returns true if any of the given possibly overlapping files contains
/// keys in the range [smallest; largest].
fn some_file_overlaps_range<'a, 'b>(
    cmp: &InternalKeyCmp,
    files: &[FileMetaHandle],
    smallest: UserKey<'a>,
    largest: UserKey<'b>,
) -> bool {
    for f in files {
        if !(key_is_after_file(cmp, smallest, f) || key_is_before_file(cmp, largest, f)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub mod testutil {
    use super::*;
    use cmp::DefaultCmp;
    use env::Env;
    use key_types::ValueType;
    use options::{self, Options};
    use table_builder::TableBuilder;
    use table_cache::table_file_name;
    use types::{share, FileMetaData, FileNum};

    use std::path::Path;

    pub fn new_file(
        num: u64,
        smallest: &[u8],
        smallestix: u64,
        largest: &[u8],
        largestix: u64,
    ) -> FileMetaHandle {
        share(FileMetaData {
            allowed_seeks: 10,
            size: 163840,
            num: num,
            smallest: LookupKey::new(smallest, smallestix).internal_key().to_vec(),
            largest: LookupKey::new(largest, largestix).internal_key().to_vec(),
        })
    }

    /// write_table creates a table with the given number and contents (must be sorted!) in the
    /// memenv. The sequence numbers given to keys start with startseq.
    pub fn write_table(
        me: &Box<dyn Env>,
        contents: &[(&[u8], &[u8], ValueType)],
        startseq: u64,
        num: FileNum,
    ) -> FileMetaHandle {
        let dst = me
            .open_writable_file(Path::new(&table_file_name("db", num)))
            .unwrap();
        let mut seq = startseq;
        let keys: Vec<Vec<u8>> = contents
            .iter()
            .map(|&(k, _, typ)| {
                seq += 1;
                LookupKey::new_full(k, seq - 1, typ).internal_key().to_vec()
            })
            .collect();

        let mut tbl = TableBuilder::new(options::for_test(), dst);
        for i in 0..contents.len() {
            tbl.add(&keys[i], contents[i].1).unwrap();
            seq += 1;
        }

        let f = new_file(
            num,
            contents[0].0,
            startseq,
            contents[contents.len() - 1].0,
            startseq + (contents.len() - 1) as u64,
        );
        f.borrow_mut().size = tbl.finish().unwrap();
        f
    }

    pub fn make_version() -> (Version, Options) {
        let opts = options::for_test();
        let env = opts.env.clone();

        // The different levels overlap in a sophisticated manner to be able to test compactions
        // and so on.
        // The sequence numbers are in "natural order", i.e. highest levels have lowest sequence
        // numbers.

        // Level 0 (overlapping)
        let f2: &[(&[u8], &[u8], ValueType)] = &[
            ("aac".as_bytes(), "val1".as_bytes(), ValueType::TypeDeletion),
            ("aax".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("aba".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
            ("bab".as_bytes(), "val4".as_bytes(), ValueType::TypeValue),
            ("bba".as_bytes(), "val5".as_bytes(), ValueType::TypeValue),
        ];
        let t2 = write_table(&env, f2, 26, 2);
        let f1: &[(&[u8], &[u8], ValueType)] = &[
            ("aaa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("aab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("aac".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
            ("aba".as_bytes(), "val4".as_bytes(), ValueType::TypeValue),
        ];
        let t1 = write_table(&env, f1, 22, 1);
        // Level 1
        let f3: &[(&[u8], &[u8], ValueType)] = &[
            ("aaa".as_bytes(), "val0".as_bytes(), ValueType::TypeValue),
            ("cab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("cba".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
        ];
        let t3 = write_table(&env, f3, 19, 3);
        let f4: &[(&[u8], &[u8], ValueType)] = &[
            ("daa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("dab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("dba".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
        ];
        let t4 = write_table(&env, f4, 16, 4);
        let f5: &[(&[u8], &[u8], ValueType)] = &[
            ("eaa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("eab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("fab".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
        ];
        let t5 = write_table(&env, f5, 13, 5);
        // Level 2
        let f6: &[(&[u8], &[u8], ValueType)] = &[
            ("cab".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("fab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("fba".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
        ];
        let t6 = write_table(&env, f6, 10, 6);
        let f7: &[(&[u8], &[u8], ValueType)] = &[
            ("gaa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("gab".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
            ("gba".as_bytes(), "val3".as_bytes(), ValueType::TypeValue),
            ("gca".as_bytes(), "val4".as_bytes(), ValueType::TypeDeletion),
            ("gda".as_bytes(), "val5".as_bytes(), ValueType::TypeValue),
        ];
        let t7 = write_table(&env, f7, 5, 7);
        // Level 3 (2 * 2 entries, for iterator behavior).
        let f8: &[(&[u8], &[u8], ValueType)] = &[
            ("haa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("hba".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
        ];
        let t8 = write_table(&env, f8, 3, 8);
        let f9: &[(&[u8], &[u8], ValueType)] = &[
            ("iaa".as_bytes(), "val1".as_bytes(), ValueType::TypeValue),
            ("iba".as_bytes(), "val2".as_bytes(), ValueType::TypeValue),
        ];
        let t9 = write_table(&env, f9, 1, 9);

        let cache = TableCache::new("db", opts.clone(), 100);
        let mut v = Version::new(share(cache), Rc::new(Box::new(DefaultCmp)));
        v.files[0] = vec![t1, t2];
        v.files[1] = vec![t3, t4, t5];
        v.files[2] = vec![t6, t7];
        v.files[3] = vec![t8, t9];
        (v, opts)
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::*;
    use super::*;

    use cmp::DefaultCmp;
    use error::Result;
    use merging_iter::MergingIter;
    use options;
    use test_util::{test_iterator_properties, LdbIteratorIter};

    #[test]
    fn test_version_concat_iter() {
        let v = make_version().0;

        let expected_entries = vec![0, 9, 8, 4];
        for l in 1..4 {
            let mut iter = v.new_concat_iter(l);
            let iter = LdbIteratorIter::wrap(&mut iter);
            assert_eq!(iter.count(), expected_entries[l]);
        }
    }

    #[test]
    fn test_version_concat_iter_properties() {
        let v = make_version().0;
        let iter = v.new_concat_iter(3);
        test_iterator_properties(iter);
    }

    #[test]
    fn test_version_max_next_level_overlapping() {
        let v = make_version().0;
        assert_eq!(218, v.max_next_level_overlapping_bytes());
    }

    #[test]
    fn test_version_all_iters() {
        let v = make_version().0;
        let iters = v.new_iters().unwrap();
        let mut opt = options::for_test();
        opt.cmp = Rc::new(Box::new(InternalKeyCmp(Rc::new(Box::new(DefaultCmp)))));

        let mut miter = MergingIter::new(opt.cmp.clone(), iters);
        assert_eq!(LdbIteratorIter::wrap(&mut miter).count(), 30);

        // Check that all elements are in order.
        let init = LookupKey::new("000".as_bytes(), MAX_SEQUENCE_NUMBER);
        let cmp = InternalKeyCmp(Rc::new(Box::new(DefaultCmp)));
        LdbIteratorIter::wrap(&mut miter).fold(init.internal_key().to_vec(), |b, (k, _)| {
            assert!(cmp.cmp(&b, &k) == Ordering::Less);
            k
        });
    }

    #[test]
    fn test_version_summary() {
        let v = make_version().0;
        let expected = "level 0: 2 files, 483 bytes ([(1, 232), (2, 251)]); level 1: 3 files, 651 \
                        bytes ([(3, 218), (4, 216), (5, 217)]); level 2: 2 files, 468 bytes ([(6, \
                        218), (7, 250)]); level 3: 2 files, 400 bytes ([(8, 200), (9, 200)]); ";
        assert_eq!(expected, &v.level_summary());
    }

    #[test]
    fn test_version_get_simple() {
        let v = make_version().0;
        let cases: &[(&[u8], u64, Result<Option<Vec<u8>>>)] = &[
            ("aaa".as_bytes(), 1, Ok(None)),
            ("aaa".as_bytes(), 100, Ok(Some("val1".as_bytes().to_vec()))),
            ("aaa".as_bytes(), 21, Ok(Some("val0".as_bytes().to_vec()))),
            ("aab".as_bytes(), 0, Ok(None)),
            ("aab".as_bytes(), 100, Ok(Some("val2".as_bytes().to_vec()))),
            ("aac".as_bytes(), 100, Ok(None)),
            ("aac".as_bytes(), 25, Ok(Some("val3".as_bytes().to_vec()))),
            ("aba".as_bytes(), 100, Ok(Some("val3".as_bytes().to_vec()))),
            ("aba".as_bytes(), 25, Ok(Some("val4".as_bytes().to_vec()))),
            ("daa".as_bytes(), 100, Ok(Some("val1".as_bytes().to_vec()))),
            ("dab".as_bytes(), 1, Ok(None)),
            ("dac".as_bytes(), 100, Ok(None)),
            ("gba".as_bytes(), 100, Ok(Some("val3".as_bytes().to_vec()))),
            // deleted key
            ("gca".as_bytes(), 100, Ok(None)),
            ("gbb".as_bytes(), 100, Ok(None)),
        ];

        for ref c in cases {
            match v.get(LookupKey::new(c.0, c.1).internal_key()) {
                Ok(Some((val, _))) => assert_eq!(c.2.as_ref().unwrap().as_ref().unwrap(), &val),
                Ok(None) => assert!(c.2.as_ref().unwrap().as_ref().is_none()),
                Err(_) => assert!(c.2.is_err()),
            }
        }
    }

    #[test]
    fn test_version_get_overlapping_basic() {
        let v = make_version().0;

        // Overlapped by tables 1 and 2.
        let ol = v.get_overlapping(LookupKey::new(b"aay", 50).internal_key());
        // Check that sorting order is newest-first in L0.
        assert_eq!(2, ol[0][0].borrow().num);
        // Check that table from L1 matches.
        assert_eq!(3, ol[1][0].borrow().num);

        let ol = v.get_overlapping(LookupKey::new(b"cb", 50).internal_key());
        assert_eq!(3, ol[1][0].borrow().num);
        assert_eq!(6, ol[2][0].borrow().num);

        let ol = v.get_overlapping(LookupKey::new(b"x", 50).internal_key());
        for i in 0..NUM_LEVELS {
            assert!(ol[i].is_empty());
        }
    }

    #[test]
    fn test_version_overlap_in_level() {
        let v = make_version().0;

        for &(level, (k1, k2), want) in &[
            (0, ("000".as_bytes(), "003".as_bytes()), false),
            (0, ("aa0".as_bytes(), "abx".as_bytes()), true),
            (1, ("012".as_bytes(), "013".as_bytes()), false),
            (1, ("abc".as_bytes(), "def".as_bytes()), true),
            (2, ("xxx".as_bytes(), "xyz".as_bytes()), false),
            (2, ("gac".as_bytes(), "gaz".as_bytes()), true),
        ] {
            if want {
                assert!(v.overlap_in_level(level, k1, k2));
            } else {
                assert!(!v.overlap_in_level(level, k1, k2));
            }
        }
    }

    #[test]
    fn test_version_pick_memtable_output_level() {
        let v = make_version().0;

        for c in [
            ("000".as_bytes(), "abc".as_bytes(), 0),
            ("gab".as_bytes(), "hhh".as_bytes(), 1),
            ("000".as_bytes(), "111".as_bytes(), 2),
        ]
        .iter()
        {
            assert_eq!(c.2, v.pick_memtable_output_level(c.0, c.1));
        }
    }

    #[test]
    fn test_version_overlapping_inputs() {
        let v = make_version().0;

        time_test!("overlapping-inputs");
        {
            // Range is expanded in overlapping level-0 files.
            let from = LookupKey::new("aab".as_bytes(), MAX_SEQUENCE_NUMBER);
            let to = LookupKey::new("aae".as_bytes(), 0);
            let r = v.overlapping_inputs(0, from.internal_key(), to.internal_key());
            assert_eq!(r.len(), 2);
            assert_eq!(r[0].borrow().num, 1);
            assert_eq!(r[1].borrow().num, 2);
        }
        {
            let from = LookupKey::new("cab".as_bytes(), MAX_SEQUENCE_NUMBER);
            let to = LookupKey::new("cbx".as_bytes(), 0);
            // expect one file.
            let r = v.overlapping_inputs(1, from.internal_key(), to.internal_key());
            assert_eq!(r.len(), 1);
            assert_eq!(r[0].borrow().num, 3);
        }
        {
            let from = LookupKey::new("cab".as_bytes(), MAX_SEQUENCE_NUMBER);
            let to = LookupKey::new("ebx".as_bytes(), 0);
            let r = v.overlapping_inputs(1, from.internal_key(), to.internal_key());
            // Assert that correct number of files and correct files were returned.
            assert_eq!(r.len(), 3);
            assert_eq!(r[0].borrow().num, 3);
            assert_eq!(r[1].borrow().num, 4);
            assert_eq!(r[2].borrow().num, 5);
        }
        {
            let from = LookupKey::new("hhh".as_bytes(), MAX_SEQUENCE_NUMBER);
            let to = LookupKey::new("ijk".as_bytes(), 0);
            let r = v.overlapping_inputs(2, from.internal_key(), to.internal_key());
            assert_eq!(r.len(), 0);
            let r = v.overlapping_inputs(1, from.internal_key(), to.internal_key());
            assert_eq!(r.len(), 0);
        }
    }

    #[test]
    fn test_version_record_read_sample() {
        let mut v = make_version().0;
        let k = LookupKey::new("aab".as_bytes(), MAX_SEQUENCE_NUMBER);
        let only_in_one = LookupKey::new("cax".as_bytes(), MAX_SEQUENCE_NUMBER);

        assert!(!v.record_read_sample(k.internal_key()));
        assert!(!v.record_read_sample(only_in_one.internal_key()));

        for fs in v.files.iter() {
            for f in fs {
                f.borrow_mut().allowed_seeks = 0;
            }
        }
        assert!(v.record_read_sample(k.internal_key()));
    }

    #[test]
    fn test_version_key_ordering() {
        time_test!();
        let fmh = new_file(1, &[1, 0, 0], 0, &[2, 0, 0], 1);
        let cmp = InternalKeyCmp(Rc::new(Box::new(DefaultCmp)));

        // Keys before file.
        for k in &[&[0][..], &[1], &[1, 0], &[0, 9, 9, 9]] {
            assert!(key_is_before_file(&cmp, k, &fmh));
            assert!(!key_is_after_file(&cmp, k, &fmh));
        }
        // Keys in file.
        for k in &[
            &[1, 0, 0][..],
            &[1, 0, 1],
            &[1, 2, 3, 4],
            &[1, 9, 9],
            &[2, 0, 0],
        ] {
            assert!(!key_is_before_file(&cmp, k, &fmh));
            assert!(!key_is_after_file(&cmp, k, &fmh));
        }
        // Keys after file.
        for k in &[&[2, 0, 1][..], &[9, 9, 9], &[9, 9, 9, 9]] {
            assert!(!key_is_before_file(&cmp, k, &fmh));
            assert!(key_is_after_file(&cmp, k, &fmh));
        }
    }

    #[test]
    fn test_version_file_overlaps() {
        time_test!();

        let files_disjoint = [
            new_file(1, &[2, 0, 0], 0, &[3, 0, 0], 1),
            new_file(2, &[3, 0, 1], 0, &[4, 0, 0], 1),
            new_file(3, &[4, 0, 1], 0, &[5, 0, 0], 1),
        ];
        let files_joint = [
            new_file(1, &[2, 0, 0], 0, &[3, 0, 0], 1),
            new_file(2, &[2, 5, 0], 0, &[4, 0, 0], 1),
            new_file(3, &[3, 5, 1], 0, &[5, 0, 0], 1),
        ];
        let cmp = InternalKeyCmp(Rc::new(Box::new(DefaultCmp)));

        assert!(some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[2, 5, 0],
            &[3, 1, 0]
        ));
        assert!(some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[2, 5, 0],
            &[7, 0, 0]
        ));
        assert!(some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[0, 0],
            &[2, 0, 0]
        ));
        assert!(some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[0, 0],
            &[7, 0, 0]
        ));
        assert!(!some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[0, 0],
            &[0, 5]
        ));
        assert!(!some_file_overlaps_range(
            &cmp,
            &files_joint,
            &[6, 0],
            &[7, 5]
        ));

        assert!(some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[2, 0, 1],
            &[2, 5, 0]
        ));
        assert!(some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[3, 0, 1],
            &[4, 9, 0]
        ));
        assert!(some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[2, 0, 1],
            &[6, 5, 0]
        ));
        assert!(some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[0, 0, 1],
            &[2, 5, 0]
        ));
        assert!(some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[0, 0, 1],
            &[6, 5, 0]
        ));
        assert!(!some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[0, 0, 1],
            &[0, 1]
        ));
        assert!(!some_file_overlaps_range_disjoint(
            &cmp,
            &files_disjoint,
            &[6, 0, 1],
            &[7, 0, 1]
        ));
    }
}
