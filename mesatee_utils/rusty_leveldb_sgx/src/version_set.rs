#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cmp::{Cmp, InternalKeyCmp};
use env::Env;
use error::{err, Result, Status, StatusCode};
use key_types::{parse_internal_key, InternalKey, UserKey};
use log::{LogReader, LogWriter};
use merging_iter::MergingIter;
use options::Options;
use table_cache::TableCache;
use types::{
    parse_file_name, share, FileMetaData, FileNum, FileType, LdbIterator, Shared, NUM_LEVELS,
};
use version::{new_version_iter, total_size, FileMetaHandle, Version};
use version_edit::VersionEdit;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use std::os::unix::ffi::OsStrExt;

pub struct Compaction {
    level: usize,
    max_file_size: usize,
    input_version: Option<Shared<Version>>,
    level_ixs: [usize; NUM_LEVELS],
    cmp: Rc<Box<dyn Cmp>>,
    icmp: InternalKeyCmp,

    manual: bool,

    // "parent" inputs from level and level+1.
    inputs: [Vec<FileMetaHandle>; 2],
    grandparent_ix: usize,
    // remaining inputs from level+2..NUM_LEVELS
    grandparents: Option<Vec<FileMetaHandle>>,
    overlapped_bytes: usize,
    seen_key: bool,
    edit: VersionEdit,
}

impl Compaction {
    // Note: opt.cmp should be the user-supplied or default comparator (not an InternalKeyCmp).
    pub fn new(opt: &Options, level: usize, input: Option<Shared<Version>>) -> Compaction {
        Compaction {
            level: level,
            max_file_size: opt.max_file_size,
            input_version: input,
            level_ixs: Default::default(),
            cmp: opt.cmp.clone(),
            icmp: InternalKeyCmp(opt.cmp.clone()),
            manual: false,

            inputs: Default::default(),
            grandparent_ix: 0,
            grandparents: Default::default(),
            overlapped_bytes: 0,
            seen_key: false,
            edit: VersionEdit::new(),
        }
    }

    fn add_input(&mut self, parent: usize, f: FileMetaHandle) {
        assert!(parent <= 1);
        self.inputs[parent].push(f)
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn input(&self, parent: usize, ix: usize) -> FileMetaData {
        assert!(parent < 2);
        assert!(ix < self.inputs[parent].len());
        self.inputs[parent][ix].borrow().clone()
    }

    pub fn num_inputs(&self, parent: usize) -> usize {
        assert!(parent < 2);
        self.inputs[parent].len()
    }

    pub fn edit(&mut self) -> &mut VersionEdit {
        &mut self.edit
    }

    pub fn into_edit(self) -> VersionEdit {
        self.edit
    }

    /// add_input_deletions marks the current input files as deleted in the inner VersionEdit.
    pub fn add_input_deletions(&mut self) {
        for parent in 0..2 {
            for f in &self.inputs[parent] {
                self.edit.delete_file(self.level + parent, f.borrow().num);
            }
        }
    }

    /// is_base_level_for checks whether the given key may exist in levels higher than this
    /// compaction's level plus 2. I.e., whether the levels for this compaction are the last ones
    /// to contain the key.
    pub fn is_base_level_for<'a>(&mut self, k: UserKey<'a>) -> bool {
        assert!(self.input_version.is_some());
        let inp_version = self.input_version.as_ref().unwrap();
        for level in self.level + 2..NUM_LEVELS {
            let files = &inp_version.borrow().files[level];
            while self.level_ixs[level] < files.len() {
                let f = files[self.level_ixs[level]].borrow();
                if self.cmp.cmp(k, parse_internal_key(&f.largest).2) <= Ordering::Equal {
                    if self.cmp.cmp(k, parse_internal_key(&f.smallest).2) >= Ordering::Equal {
                        // key is in this file's range, so this is not the base level.
                        return false;
                    }
                    break;
                }
                // level_ixs contains cross-call state to speed up following lookups.
                self.level_ixs[level] += 1;
            }
        }
        true
    }

    pub fn is_trivial_move(&self) -> bool {
        if self.manual {
            return false;
        }

        let inputs_size;
        if let Some(gp) = self.grandparents.as_ref() {
            inputs_size = total_size(gp.iter());
        } else {
            inputs_size = 0;
        }
        self.num_inputs(0) == 1 && self.num_inputs(1) == 0 && inputs_size < 10 * self.max_file_size
    }

    pub fn should_stop_before<'a>(&mut self, k: InternalKey<'a>) -> bool {
        if self.grandparents.is_none() {
            self.seen_key = true;
            return false;
        }
        let grandparents = self.grandparents.as_ref().unwrap();
        while self.grandparent_ix < grandparents.len()
            && self
                .icmp
                .cmp(k, &grandparents[self.grandparent_ix].borrow().largest)
                == Ordering::Greater
        {
            if self.seen_key {
                self.overlapped_bytes += grandparents[self.grandparent_ix].borrow().size;
            }
            self.grandparent_ix += 1;
        }
        self.seen_key = true;

        if self.overlapped_bytes > 10 * self.max_file_size {
            self.overlapped_bytes = 0;
            true
        } else {
            false
        }
    }
}

/// VersionSet managed the various versions that are live within a database. A single version
/// contains references to the files on disk as they were at a certain point.
pub struct VersionSet {
    dbname: PathBuf,
    opt: Options,
    cmp: InternalKeyCmp,
    cache: Shared<TableCache>,

    pub next_file_num: u64,
    pub manifest_num: u64,
    pub last_seq: u64,
    pub log_num: u64,
    pub prev_log_num: u64,

    current: Option<Shared<Version>>,
    compaction_ptrs: [Vec<u8>; NUM_LEVELS],

    descriptor_log: Option<LogWriter<Box<dyn Write>>>,
}

impl VersionSet {
    // Note: opt.cmp should not contain an InternalKeyCmp at this point, but instead the default or
    // user-supplied one.
    pub fn new<P: AsRef<Path>>(db: P, opt: Options, cache: Shared<TableCache>) -> VersionSet {
        let v = share(Version::new(cache.clone(), opt.cmp.clone()));
        VersionSet {
            dbname: db.as_ref().to_owned(),
            cmp: InternalKeyCmp(opt.cmp.clone()),
            opt: opt,
            cache: cache,

            next_file_num: 2,
            manifest_num: 0,
            last_seq: 0,
            log_num: 0,
            prev_log_num: 0,

            current: Some(v),
            compaction_ptrs: Default::default(),
            descriptor_log: None,
        }
    }

    pub fn current_summary(&self) -> String {
        self.current.as_ref().unwrap().borrow().level_summary()
    }

    /// live_files returns the files that are currently active.
    pub fn live_files(&self) -> HashSet<FileNum> {
        let mut files = HashSet::new();
        if let Some(ref version) = self.current {
            for level in 0..NUM_LEVELS {
                for file in &version.borrow().files[level] {
                    files.insert(file.borrow().num);
                }
            }
        }
        files
    }

    /// current returns a reference to the current version. It panics if there is no current
    /// version.
    pub fn current(&self) -> Shared<Version> {
        assert!(self.current.is_some());
        self.current.as_ref().unwrap().clone()
    }

    pub fn add_version(&mut self, v: Version) {
        self.current = Some(share(v));
    }

    pub fn new_file_number(&mut self) -> FileNum {
        self.next_file_num += 1;
        self.next_file_num - 1
    }

    pub fn reuse_file_number(&mut self, n: FileNum) {
        if n == self.next_file_num - 1 {
            self.next_file_num = n;
        }
    }

    pub fn mark_file_number_used(&mut self, n: FileNum) {
        if self.next_file_num <= n {
            self.next_file_num = n + 1;
        }
    }

    /// needs_compaction returns true if a compaction makes sense at this point.
    pub fn needs_compaction(&self) -> bool {
        assert!(self.current.is_some());
        let v = self.current.as_ref().unwrap();
        let v = v.borrow();
        v.compaction_score.unwrap_or(0.0) >= 1.0 || v.file_to_compact.is_some()
    }

    fn approximate_offset<'a>(&self, v: &Shared<Version>, key: InternalKey<'a>) -> usize {
        let mut offset = 0;
        for level in 0..NUM_LEVELS {
            for f in &v.borrow().files[level] {
                if self.opt.cmp.cmp(&f.borrow().largest, key) <= Ordering::Equal {
                    offset += f.borrow().size;
                } else if self.opt.cmp.cmp(&f.borrow().smallest, key) == Ordering::Greater {
                    // In higher levels, files are sorted; we don't need to search further.
                    if level > 0 {
                        break;
                    }
                } else {
                    if let Ok(tbl) = self.cache.borrow_mut().get_table(f.borrow().num) {
                        offset += tbl.approx_offset_of(key);
                    }
                }
            }
        }
        offset
    }

    pub fn pick_compaction(&mut self) -> Option<Compaction> {
        assert!(self.current.is_some());
        let current = self.current();
        let current = current.borrow();

        let mut c = Compaction::new(&self.opt, 0, self.current.clone());
        let level;

        // Size compaction?
        if current.compaction_score.unwrap_or(0.0) >= 1.0 {
            level = current.compaction_level.unwrap();
            assert!(level < NUM_LEVELS - 1);

            for f in &current.files[level] {
                if self.compaction_ptrs[level].is_empty()
                    || self
                        .cmp
                        .cmp(&f.borrow().largest, &self.compaction_ptrs[level])
                        == Ordering::Greater
                {
                    c.add_input(0, f.clone());
                    break;
                }
            }

            if c.num_inputs(0) == 0 {
                // Add first file in level. This will also reset the compaction pointers.
                c.add_input(0, current.files[level][0].clone());
            }
        } else if let Some(ref ftc) = current.file_to_compact {
            // Seek compaction?
            level = current.file_to_compact_lvl;
            c.add_input(0, ftc.clone());
        } else {
            return None;
        }

        c.level = level;
        c.input_version = self.current.clone();

        if level == 0 {
            let (smallest, largest) = get_range(&self.cmp, c.inputs[0].iter());
            // This call intentionally overwrites the file previously put into c.inputs[0].
            c.inputs[0] = current.overlapping_inputs(0, &smallest, &largest);
            assert!(!c.inputs[0].is_empty());
        }

        self.setup_other_inputs(&mut c);
        Some(c)
    }

    pub fn compact_range<'a, 'b>(
        &mut self,
        level: usize,
        from: InternalKey<'a>,
        to: InternalKey<'b>,
    ) -> Option<Compaction> {
        assert!(self.current.is_some());
        let mut inputs = self
            .current
            .as_ref()
            .unwrap()
            .borrow()
            .overlapping_inputs(level, from, to);
        if inputs.is_empty() {
            return None;
        }

        if level > 0 {
            let mut total = 0;
            for i in 0..inputs.len() {
                total += inputs[i].borrow().size;
                if total > self.opt.max_file_size {
                    inputs.truncate(i + 1);
                    break;
                }
            }
        }

        let mut c = Compaction::new(&self.opt, level, self.current.clone());
        c.inputs[0] = inputs;
        c.manual = true;
        self.setup_other_inputs(&mut c);
        Some(c)
    }

    fn setup_other_inputs(&mut self, compaction: &mut Compaction) {
        assert!(self.current.is_some());
        let current = self.current.as_ref().unwrap();
        let current = current.borrow();

        let level = compaction.level;
        let (mut smallest, mut largest) = get_range(&self.cmp, compaction.inputs[0].iter());

        // Set up level+1 inputs.
        compaction.inputs[1] = current.overlapping_inputs(level + 1, &smallest, &largest);

        let (mut allstart, mut alllimit) = get_range(
            &self.cmp,
            compaction.inputs[0]
                .iter()
                .chain(compaction.inputs[1].iter()),
        );

        // Check if we can add more inputs in the current level without having to compact more
        // inputs from level+1.
        if !compaction.inputs[1].is_empty() {
            let expanded0 = current.overlapping_inputs(level, &allstart, &alllimit);
            let inputs1_size = total_size(compaction.inputs[1].iter());
            let expanded0_size = total_size(expanded0.iter());
            // ...if we picked up more files in the current level, and the total size is acceptable
            if expanded0.len() > compaction.num_inputs(0)
                && (inputs1_size + expanded0_size) < 25 * self.opt.max_file_size
            {
                let (new_start, new_limit) = get_range(&self.cmp, expanded0.iter());
                let expanded1 = current.overlapping_inputs(level + 1, &new_start, &new_limit);
                if expanded1.len() == compaction.num_inputs(1) {
                    log!(
                        self.opt.log,
                        "Expanding inputs@{} {}+{} ({}+{} bytes) to {}+{} ({}+{} bytes)",
                        level,
                        compaction.inputs[0].len(),
                        compaction.inputs[1].len(),
                        total_size(compaction.inputs[0].iter()),
                        total_size(compaction.inputs[1].iter()),
                        expanded0.len(),
                        expanded1.len(),
                        total_size(expanded0.iter()),
                        total_size(expanded1.iter())
                    );

                    smallest = new_start;
                    largest = new_limit;
                    compaction.inputs[0] = expanded0;
                    compaction.inputs[1] = expanded1;
                    let (newallstart, newalllimit) = get_range(
                        &self.cmp,
                        compaction.inputs[0]
                            .iter()
                            .chain(compaction.inputs[1].iter()),
                    );
                    allstart = newallstart;
                    alllimit = newalllimit;
                }
            }
        }

        // Set the list of grandparent (l+2) inputs to the files overlapped by the current overall
        // range.
        if level + 2 < NUM_LEVELS {
            let grandparents = self.current.as_ref().unwrap().borrow().overlapping_inputs(
                level + 2,
                &allstart,
                &alllimit,
            );
            compaction.grandparents = Some(grandparents);
        }

        log!(
            self.opt.log,
            "Compacting @{} {:?} .. {:?}",
            level,
            smallest,
            largest
        );

        compaction.edit().set_compact_pointer(level, &largest);
        self.compaction_ptrs[level] = largest;
    }

    /// write_snapshot writes the current version, with all files, to the manifest.
    fn write_snapshot(&mut self) -> Result<usize> {
        assert!(self.descriptor_log.is_some());

        let mut edit = VersionEdit::new();
        edit.set_comparator_name(self.opt.cmp.id());

        // Save compaction pointers.
        for level in 0..NUM_LEVELS {
            if !self.compaction_ptrs[level].is_empty() {
                edit.set_compact_pointer(level, &self.compaction_ptrs[level]);
            }
        }

        let current = self.current.as_ref().unwrap().borrow();
        // Save files.
        for level in 0..NUM_LEVELS {
            let fs = &current.files[level];
            for f in fs {
                edit.add_file(level, f.borrow().clone());
            }
        }
        self.descriptor_log
            .as_mut()
            .unwrap()
            .add_record(&edit.encode())
    }

    /// log_and_apply merges the given edit with the current state and generates a new version. It
    /// writes the VersionEdit to the manifest.
    pub fn log_and_apply(&mut self, mut edit: VersionEdit) -> Result<()> {
        assert!(self.current.is_some());

        if edit.log_number.is_none() {
            edit.set_log_num(self.log_num);
        } else {
            assert!(edit.log_number.unwrap() >= self.log_num);
            assert!(edit.log_number.unwrap() < self.next_file_num);
        }
        if edit.prev_log_number.is_none() {
            edit.set_prev_log_num(self.prev_log_num);
        }
        edit.set_next_file(self.next_file_num);
        edit.set_last_seq(self.last_seq);

        let mut v = Version::new(self.cache.clone(), self.opt.cmp.clone());
        {
            let mut builder = Builder::new();
            builder.apply(&edit, &mut self.compaction_ptrs);
            builder.save_to(&self.cmp, self.current.as_ref().unwrap(), &mut v);
        }
        self.finalize(&mut v);

        if self.descriptor_log.is_none() {
            let descname = manifest_file_name(&self.dbname, self.manifest_num);
            edit.set_next_file(self.next_file_num);
            self.descriptor_log = Some(LogWriter::new(
                self.opt.env.open_writable_file(Path::new(&descname))?,
            ));
            self.write_snapshot()?;
        }

        let encoded = edit.encode();
        if let Some(ref mut lw) = self.descriptor_log {
            lw.add_record(&encoded)?;
            lw.flush()?;
        }
        set_current_file(&self.opt.env, &self.dbname, self.manifest_num)?;

        self.add_version(v);
        // log_number was set above.
        self.log_num = edit.log_number.unwrap();

        // TODO: Roll back written files if something went wrong.
        Ok(())
    }

    fn finalize(&self, v: &mut Version) {
        let mut best_lvl = None;
        let mut best_score = None;

        for l in 0..NUM_LEVELS - 1 {
            let score: f64;
            if l == 0 {
                score = v.files[l].len() as f64 / 4.0;
            } else {
                let mut max_bytes = 10.0 * f64::from(1 << 20);
                for _ in 0..l - 1 {
                    max_bytes *= 10.0;
                }
                score = total_size(v.files[l].iter()) as f64 / max_bytes;
            }
            if let Some(ref mut b) = best_score {
                if *b < score {
                    *b = score;
                    best_lvl = Some(l);
                }
            } else {
                best_score = Some(score);
                best_lvl = Some(l);
            }
        }
        v.compaction_score = best_score;
        v.compaction_level = best_lvl;
    }

    /// recover recovers the state of a LevelDB instance from the files on disk. If recover()
    /// returns true, the a manifest needs to be written eventually (using log_and_apply()).
    pub fn recover(&mut self) -> Result<bool> {
        assert!(self.current.is_some());

        let mut current = read_current_file(&self.opt.env, &self.dbname)?;
        let len = current.len();
        current.truncate(len - 1);
        let current = Path::new(&current);

        let descfilename = self.dbname.join(current);
        let mut builder = Builder::new();
        {
            let mut descfile = self
                .opt
                .env
                .open_sequential_file(Path::new(&descfilename))?;
            let mut logreader = LogReader::new(
                &mut descfile,
                // checksum=
                true,
            );

            let mut log_number = None;
            let mut prev_log_number = None;
            let mut next_file_number = None;
            let mut last_seq = None;

            let mut buf = Vec::new();
            while let Ok(size) = logreader.read(&mut buf) {
                if size == 0 {
                    break;
                }
                let edit = VersionEdit::decode_from(&buf)?;
                builder.apply(&edit, &mut self.compaction_ptrs);
                if let Some(ln) = edit.log_number {
                    log_number = Some(ln);
                }
                if let Some(nfn) = edit.next_file_number {
                    next_file_number = Some(nfn);
                }
                if let Some(ls) = edit.last_seq {
                    last_seq = Some(ls);
                }
                if let Some(pln) = edit.prev_log_number {
                    prev_log_number = Some(pln);
                }
            }

            if let Some(ln) = log_number {
                self.log_num = ln;
                self.mark_file_number_used(ln);
            } else {
                return err(
                    StatusCode::Corruption,
                    "no meta-lognumber entry in descriptor",
                );
            }
            if let Some(nfn) = next_file_number {
                self.next_file_num = nfn + 1;
            } else {
                return err(
                    StatusCode::Corruption,
                    "no meta-next-file entry in descriptor",
                );
            }
            if let Some(ls) = last_seq {
                self.last_seq = ls;
            } else {
                return err(
                    StatusCode::Corruption,
                    "no last-sequence entry in descriptor",
                );
            }
            if let Some(pln) = prev_log_number {
                self.prev_log_num = pln;
                self.mark_file_number_used(prev_log_number.unwrap());
            } else {
                self.prev_log_num = 0;
            }
        }

        let mut v = Version::new(self.cache.clone(), self.opt.cmp.clone());
        builder.save_to(&self.cmp, self.current.as_ref().unwrap(), &mut v);
        self.finalize(&mut v);
        self.add_version(v);
        self.manifest_num = self.next_file_num - 1;
        log!(
            self.opt.log,
            "Recovered manifest with next_file={} manifest_num={} log_num={} prev_log_num={} \
             last_seq={}",
            self.next_file_num,
            self.manifest_num,
            self.log_num,
            self.prev_log_num,
            self.last_seq
        );

        // A new manifest needs to be written only if we don't reuse the existing one.
        Ok(!self.reuse_manifest(&descfilename, &current))
    }

    /// reuse_manifest checks whether the current manifest can be reused.
    fn reuse_manifest(
        &mut self,
        current_manifest_path: &Path,
        current_manifest_base: &Path,
    ) -> bool {
        // Note: The original has only one option, reuse_logs; reuse_logs has to be set in order to
        // reuse manifests.
        // However, there's not much that stops us from reusing manifests without reusing logs or
        // vice versa. One issue exists though: If no write operations are done, empty log files
        // will accumulate every time a DB is opened, until at least one write happens (otherwise,
        // the logs won't be compacted and deleted).
        if !self.opt.reuse_manifest {
            return false;
        }
        // The original doesn't reuse manifests; we do.
        if let Ok((num, typ)) = parse_file_name(current_manifest_base) {
            if typ != FileType::Descriptor {
                return false;
            }
            if let Ok(size) = self.opt.env.size_of(Path::new(current_manifest_path)) {
                if size > self.opt.max_file_size {
                    return false;
                }
            } else {
                return false;
            }

            assert!(self.descriptor_log.is_none());
            let s = self
                .opt
                .env
                .open_appendable_file(Path::new(current_manifest_path));
            if let Ok(f) = s {
                log!(self.opt.log, "reusing manifest {:?}", current_manifest_path);
                self.descriptor_log = Some(LogWriter::new(f));
                self.manifest_num = num;
                return true;
            } else {
                log!(self.opt.log, "reuse_manifest: {}", s.err().unwrap());
            }
        }
        false
    }

    /// make_input_iterator returns an iterator over the inputs of a compaction.
    pub fn make_input_iterator(&self, c: &Compaction) -> Box<dyn LdbIterator> {
        let cap = if c.level == 0 { c.num_inputs(0) + 1 } else { 2 };
        let mut iters: Vec<Box<dyn LdbIterator>> = Vec::with_capacity(cap);
        for i in 0..2 {
            if c.num_inputs(i) == 0 {
                continue;
            }
            if c.level + i == 0 {
                // Add individual iterators for L0 tables.
                for fi in 0..c.num_inputs(i) {
                    let f = &c.inputs[i][fi];
                    let s = self.cache.borrow_mut().get_table(f.borrow().num);
                    if let Ok(tbl) = s {
                        iters.push(Box::new(tbl.iter()));
                    } else {
                        log!(
                            self.opt.log,
                            "error opening table {}: {}",
                            f.borrow().num,
                            s.err().unwrap()
                        );
                    }
                }
            } else {
                // Create concatenating iterator higher levels.
                iters.push(Box::new(new_version_iter(
                    c.inputs[i].clone(),
                    self.cache.clone(),
                    self.opt.cmp.clone(),
                )));
            }
        }
        assert!(iters.len() <= cap);
        let cmp: Rc<Box<dyn Cmp>> = Rc::new(Box::new(self.cmp.clone()));
        Box::new(MergingIter::new(cmp, iters))
    }
}

struct Builder {
    // (added, deleted) files per level.
    deleted: [Vec<FileNum>; NUM_LEVELS],
    added: [Vec<FileMetaHandle>; NUM_LEVELS],
}

impl Builder {
    fn new() -> Builder {
        Builder {
            deleted: Default::default(),
            added: Default::default(),
        }
    }

    /// apply applies the edits recorded in edit to the builder state. compaction pointers are
    /// copied to the supplied compaction_ptrs array.
    fn apply(&mut self, edit: &VersionEdit, compaction_ptrs: &mut [Vec<u8>; NUM_LEVELS]) {
        for c in edit.compaction_ptrs.iter() {
            compaction_ptrs[c.level] = c.key.clone();
        }
        for &(level, num) in edit.deleted.iter() {
            self.deleted[level].push(num);
        }
        for &(level, ref f) in edit.new_files.iter() {
            let mut f = f.clone();
            f.allowed_seeks = f.size / 16384;
            if f.allowed_seeks < 100 {
                f.allowed_seeks = 100;
            }
            // Remove this file from the list of deleted files.
            self.deleted[level] = self.deleted[level]
                .iter()
                .filter_map(|d| if *d != f.num { Some(*d) } else { None })
                .collect();
            self.added[level].push(share(f));
        }
    }

    /// maybe_add_file adds a file f at level to version v, if it's not already marked as deleted
    /// in this edit. It also asserts that the ordering of files is preserved.
    fn maybe_add_file(
        &mut self,
        cmp: &InternalKeyCmp,
        v: &mut Version,
        level: usize,
        f: FileMetaHandle,
    ) {
        // Only add file if it's not already deleted.
        if self.deleted[level].iter().any(|d| *d == f.borrow().num) {
            return;
        }
        {
            let files = &v.files[level];
            if level > 0 && !files.is_empty() {
                // File must be after last file in level.
                assert_eq!(
                    cmp.cmp(
                        &files[files.len() - 1].borrow().largest,
                        &f.borrow().smallest
                    ),
                    Ordering::Less
                );
            }
        }
        v.files[level].push(f);
    }

    /// save_to saves the edits applied to the builder to v, adding all non-deleted files from
    /// Version base to v.
    fn save_to(&mut self, cmp: &InternalKeyCmp, base: &Shared<Version>, v: &mut Version) {
        for level in 0..NUM_LEVELS {
            sort_files_by_smallest(cmp, &mut self.added[level]);
            // The base version should already have sorted files.
            sort_files_by_smallest(cmp, &mut base.borrow_mut().files[level]);

            let added = self.added[level].clone();
            let basefiles = base.borrow().files[level].clone();
            v.files[level].reserve(basefiles.len() + self.added[level].len());

            let iadded = added.into_iter();
            let ibasefiles = basefiles.into_iter();
            let merged = merge_iters(iadded, ibasefiles, |a, b| {
                cmp.cmp(&a.borrow().smallest, &b.borrow().smallest)
            });
            for m in merged {
                self.maybe_add_file(cmp, v, level, m);
            }

            // Make sure that there is no overlap in higher levels.
            if level == 0 {
                continue;
            }
            for i in 1..v.files[level].len() {
                let (prev_end, this_begin) = (
                    &v.files[level][i - 1].borrow().largest,
                    &v.files[level][i].borrow().smallest,
                );
                assert!(cmp.cmp(prev_end, this_begin) < Ordering::Equal);
            }
        }
    }
}

fn manifest_name(file_num: FileNum) -> PathBuf {
    Path::new(&format!("MANIFEST-{:06}", file_num)).to_owned()
}

pub fn manifest_file_name<P: AsRef<Path>>(dbname: P, file_num: FileNum) -> PathBuf {
    dbname.as_ref().join(manifest_name(file_num)).to_owned()
}

fn temp_file_name<P: AsRef<Path>>(dbname: P, file_num: FileNum) -> PathBuf {
    dbname
        .as_ref()
        .join(format!("{:06}.dbtmp", file_num))
        .to_owned()
}

fn current_file_name<P: AsRef<Path>>(dbname: P) -> PathBuf {
    dbname.as_ref().join("CURRENT").to_owned()
}

pub fn read_current_file(env: &Box<dyn Env>, dbname: &Path) -> Result<String> {
    let mut current = String::new();
    let mut f = env.open_sequential_file(Path::new(&current_file_name(dbname)))?;
    f.read_to_string(&mut current)?;
    if current.is_empty() || !current.ends_with('\n') {
        return err(
            StatusCode::Corruption,
            "current file is empty or has no newline",
        );
    }
    Ok(current)
}

pub fn set_current_file<P: AsRef<Path>>(
    env: &Box<dyn Env>,
    dbname: P,
    manifest_file_num: FileNum,
) -> Result<()> {
    let dbname = dbname.as_ref();
    let manifest_base = manifest_name(manifest_file_num);
    let tempfile = temp_file_name(dbname, manifest_file_num);
    {
        let mut f = env.open_writable_file(Path::new(&tempfile))?;
        f.write(manifest_base.as_os_str().as_bytes())?;
        f.write("\n".as_bytes())?;
    }
    let currentfile = current_file_name(dbname);
    if let Err(e) = env.rename(Path::new(&tempfile), Path::new(&currentfile)) {
        // ignore error.
        let _ = env.delete(Path::new(&tempfile));
        return Err(Status::from(e));
    }
    Ok(())
}

/// sort_files_by_smallest sorts the list of files by the smallest keys of the files.
fn sort_files_by_smallest<C: Cmp>(cmp: &C, files: &mut Vec<FileMetaHandle>) {
    files.sort_by(|a, b| cmp.cmp(&a.borrow().smallest, &b.borrow().smallest))
}

/// merge_iters merges and collects the items from two sorted iterators.
fn merge_iters<
    Item,
    C: Fn(&Item, &Item) -> Ordering,
    I: Iterator<Item = Item>,
    J: Iterator<Item = Item>,
>(
    mut iter_a: I,
    mut iter_b: J,
    cmp: C,
) -> Vec<Item> {
    let mut a = iter_a.next();
    let mut b = iter_b.next();
    let mut out = vec![];
    while a.is_some() && b.is_some() {
        let ord = cmp(a.as_ref().unwrap(), b.as_ref().unwrap());
        if ord == Ordering::Less {
            out.push(a.unwrap());
            a = iter_a.next();
        } else {
            out.push(b.unwrap());
            b = iter_b.next();
        }
    }

    // Push cached elements.
    if let Some(a_) = a {
        out.push(a_);
    }
    if let Some(b_) = b {
        out.push(b_);
    }

    // Push remaining elements from either iterator.
    for a in iter_a {
        out.push(a);
    }
    for b in iter_b {
        out.push(b);
    }
    out
}

/// get_range returns the indices of the files within files that have the smallest lower bound
/// respectively the largest upper bound.
fn get_range<'a, C: Cmp, I: Iterator<Item = &'a FileMetaHandle>>(
    c: &C,
    files: I,
) -> (Vec<u8>, Vec<u8>) {
    let mut smallest = None;
    let mut largest = None;
    for f in files {
        if smallest.is_none() {
            smallest = Some(f.borrow().smallest.clone());
        }
        if largest.is_none() {
            largest = Some(f.borrow().largest.clone());
        }
        let f = f.borrow();
        if c.cmp(&f.smallest, smallest.as_ref().unwrap()) == Ordering::Less {
            smallest = Some(f.smallest.clone());
        }
        if c.cmp(&f.largest, largest.as_ref().unwrap()) == Ordering::Greater {
            largest = Some(f.largest.clone());
        }
    }
    (smallest.unwrap(), largest.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmp::DefaultCmp;
    use key_types::LookupKey;
    use test_util::LdbIteratorIter;
    use types::FileMetaData;
    use version::testutil::make_version;

    fn example_files() -> Vec<FileMetaHandle> {
        let mut f1 = FileMetaData::default();
        f1.num = 1;
        f1.size = 10;
        f1.smallest = "f".as_bytes().to_vec();
        f1.largest = "g".as_bytes().to_vec();
        let mut f2 = FileMetaData::default();
        f2.num = 2;
        f2.size = 20;
        f2.smallest = "e".as_bytes().to_vec();
        f2.largest = "f".as_bytes().to_vec();
        let mut f3 = FileMetaData::default();
        f3.num = 3;
        f3.size = 30;
        f3.smallest = "a".as_bytes().to_vec();
        f3.largest = "b".as_bytes().to_vec();
        let mut f4 = FileMetaData::default();
        f4.num = 4;
        f4.size = 40;
        f4.smallest = "q".as_bytes().to_vec();
        f4.largest = "z".as_bytes().to_vec();
        vec![f1, f2, f3, f4].into_iter().map(share).collect()
    }

    #[test]
    fn test_version_set_merge_iters() {
        let v1 = vec![2, 4, 6, 8, 10];
        let v2 = vec![1, 3, 5, 7];
        assert_eq!(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 10],
            merge_iters(v1.into_iter(), v2.into_iter(), |a, b| a.cmp(&b))
        );
    }

    #[test]
    fn test_version_set_total_size() {
        assert_eq!(100, total_size(example_files().iter()));
    }

    #[test]
    fn test_version_set_get_range() {
        let cmp = DefaultCmp;
        let fs = example_files();
        assert_eq!(
            ("a".as_bytes().to_vec(), "z".as_bytes().to_vec()),
            get_range(&cmp, fs.iter())
        );
    }

    #[test]
    fn test_version_set_builder() {
        let (v, opt) = make_version();
        let v = share(v);

        let mut fmd = FileMetaData::default();
        fmd.num = 21;
        fmd.size = 123;
        fmd.smallest = LookupKey::new("klm".as_bytes(), 777)
            .internal_key()
            .to_vec();
        fmd.largest = LookupKey::new("kop".as_bytes(), 700)
            .internal_key()
            .to_vec();

        let mut ve = VersionEdit::new();
        ve.add_file(1, fmd);
        // This deletion should be undone by apply().
        ve.delete_file(1, 21);
        ve.delete_file(0, 2);
        ve.set_compact_pointer(2, LookupKey::new("xxx".as_bytes(), 123).internal_key());

        let mut b = Builder::new();
        let mut ptrs: [Vec<u8>; NUM_LEVELS] = Default::default();
        b.apply(&ve, &mut ptrs);

        assert_eq!(
            &[120 as u8, 120, 120, 1, 123, 0, 0, 0, 0, 0, 0],
            ptrs[2].as_slice()
        );
        assert_eq!(2, b.deleted[0][0]);
        assert_eq!(1, b.added[1].len());

        let mut v2 = Version::new(
            share(TableCache::new("db", opt.clone(), 100)),
            opt.cmp.clone(),
        );
        b.save_to(&InternalKeyCmp(opt.cmp.clone()), &v, &mut v2);
        // Second file in L0 was removed.
        assert_eq!(1, v2.files[0].len());
        // File was added to L1.
        assert_eq!(4, v2.files[1].len());
        assert_eq!(21, v2.files[1][3].borrow().num);
    }

    #[test]
    fn test_version_set_log_and_apply() {
        let (_, opt) = make_version();
        let mut vs = VersionSet::new(
            "db",
            opt.clone(),
            share(TableCache::new("db", opt.clone(), 100)),
        );

        assert_eq!(2, vs.new_file_number());
        // Simulate NewDB
        {
            let mut ve = VersionEdit::new();
            ve.set_comparator_name("leveldb.BytewiseComparator");
            ve.set_log_num(10);
            ve.set_next_file(20);
            ve.set_last_seq(30);

            // Write first manifest to be recovered from.
            let manifest = manifest_file_name("db", 19);
            let mffile = opt.env.open_writable_file(Path::new(&manifest)).unwrap();
            let mut lw = LogWriter::new(mffile);
            lw.add_record(&ve.encode()).unwrap();
            lw.flush().unwrap();
            set_current_file(&opt.env.as_ref(), "db", 19).unwrap();
        }

        // Recover from new state.
        {
            vs.recover().unwrap();
            assert_eq!(10, vs.log_num);
            assert_eq!(21, vs.next_file_num);
            assert_eq!(30, vs.last_seq);
            assert_eq!(0, vs.current.as_ref().unwrap().borrow().files[0].len());
            assert_eq!(0, vs.current.as_ref().unwrap().borrow().files[1].len());
            assert_eq!(35, vs.write_snapshot().unwrap());
        }

        // Simulate compaction by adding a file.
        {
            let mut ve = VersionEdit::new();
            ve.set_log_num(11);
            let mut fmd = FileMetaData::default();
            fmd.num = 21;
            fmd.size = 123;
            fmd.smallest = LookupKey::new("abc".as_bytes(), 777)
                .internal_key()
                .to_vec();
            fmd.largest = LookupKey::new("def".as_bytes(), 700)
                .internal_key()
                .to_vec();
            ve.add_file(1, fmd);
            vs.log_and_apply(ve).unwrap();

            assert!(opt.env.exists(Path::new("db/CURRENT")).unwrap());
            assert!(opt.env.exists(Path::new("db/MANIFEST-000019")).unwrap());
            // next_file_num and last_seq are untouched by log_and_apply
            assert_eq!(21, vs.new_file_number());
            assert_eq!(22, vs.next_file_num);
            assert_eq!(30, vs.last_seq);
            // the following fields are touched by log_and_apply.
            assert_eq!(11, vs.log_num);

            // The previous "compaction" should have added one file to the first level in the
            // current version.
            assert_eq!(0, vs.current.as_ref().unwrap().borrow().files[0].len());
            assert_eq!(1, vs.current.as_ref().unwrap().borrow().files[1].len());
            assert_eq!(63, vs.write_snapshot().unwrap());
        }
    }

    #[test]
    fn test_version_set_utils() {
        let (v, opt) = make_version();
        let mut vs = VersionSet::new("db", opt.clone(), share(TableCache::new("db", opt, 100)));
        vs.add_version(v);
        // live_files()
        assert_eq!(9, vs.live_files().len());
        assert!(vs.live_files().contains(&3));

        let v = vs.current();
        let v = v.borrow();
        // num_level_bytes()
        assert_eq!(483, v.num_level_bytes(0));
        assert_eq!(651, v.num_level_bytes(1));
        assert_eq!(468, v.num_level_bytes(2));
        // num_level_files()
        assert_eq!(2, v.num_level_files(0));
        assert_eq!(3, v.num_level_files(1));
        assert_eq!(2, v.num_level_files(2));
        // new_file_number()
        assert_eq!(2, vs.new_file_number());
        assert_eq!(3, vs.new_file_number());
    }

    #[test]
    fn test_version_set_pick_compaction() {
        let (mut v, opt) = make_version();
        let mut vs = VersionSet::new("db", opt.clone(), share(TableCache::new("db", opt, 100)));

        v.compaction_score = Some(2.0);
        v.compaction_level = Some(0);
        vs.add_version(v);

        // Size compaction
        {
            let c = vs.pick_compaction().unwrap();
            assert_eq!(2, c.inputs[0].len());
            assert_eq!(1, c.inputs[1].len());
            assert_eq!(0, c.level);
            assert!(c.input_version.is_some());
        }
        // Seek compaction
        {
            let current = vs.current();
            current.borrow_mut().compaction_score = None;
            current.borrow_mut().compaction_level = None;
            current.borrow_mut().file_to_compact_lvl = 1;

            let fmd = current.borrow().files[1][0].clone();
            current.borrow_mut().file_to_compact = Some(fmd);

            let c = vs.pick_compaction().unwrap();
            assert_eq!(3, c.inputs[0].len()); // inputs on l+0 are expanded.
            assert_eq!(1, c.inputs[1].len());
            assert_eq!(1, c.level);
            assert!(c.input_version.is_some());
        }
    }

    /// iterator_properties tests that it contains len elements and that they are ordered in
    /// ascending order by cmp.
    fn iterator_properties<It: LdbIterator>(mut it: It, len: usize, cmp: Rc<Box<dyn Cmp>>) {
        let mut wr = LdbIteratorIter::wrap(&mut it);
        let first = wr.next().unwrap();
        let mut count = 1;
        wr.fold(first, |(a, _), (b, c)| {
            assert_eq!(Ordering::Less, cmp.cmp(&a, &b));
            count += 1;
            (b, c)
        });
        assert_eq!(len, count);
    }

    #[test]
    fn test_version_set_compaction() {
        let (v, opt) = make_version();
        let mut vs = VersionSet::new("db", opt.clone(), share(TableCache::new("db", opt, 100)));
        time_test!();
        vs.add_version(v);

        {
            // approximate_offset()
            let v = vs.current();
            assert_eq!(
                0,
                vs.approximate_offset(&v, LookupKey::new("aaa".as_bytes(), 9000).internal_key())
            );
            assert_eq!(
                232,
                vs.approximate_offset(&v, LookupKey::new("bab".as_bytes(), 9000).internal_key())
            );
            assert_eq!(
                1134,
                vs.approximate_offset(&v, LookupKey::new("fab".as_bytes(), 9000).internal_key())
            );
        }
        // The following tests reuse the same version set and verify that various compactions work
        // like they should.
        {
            time_test!("compaction tests");
            // compact level 0 with a partial range.
            let from = LookupKey::new("000".as_bytes(), 1000);
            let to = LookupKey::new("ab".as_bytes(), 1010);
            let c = vs
                .compact_range(0, from.internal_key(), to.internal_key())
                .unwrap();
            assert_eq!(2, c.inputs[0].len());
            assert_eq!(1, c.inputs[1].len());
            assert_eq!(1, c.grandparents.unwrap().len());

            // compact level 0, but entire range of keys in version.
            let from = LookupKey::new("000".as_bytes(), 1000);
            let to = LookupKey::new("zzz".as_bytes(), 1010);
            let c = vs
                .compact_range(0, from.internal_key(), to.internal_key())
                .unwrap();
            assert_eq!(2, c.inputs[0].len());
            assert_eq!(1, c.inputs[1].len());
            assert_eq!(1, c.grandparents.as_ref().unwrap().len());
            iterator_properties(
                vs.make_input_iterator(&c),
                12,
                Rc::new(Box::new(vs.cmp.clone())),
            );

            // Expand input range on higher level.
            let from = LookupKey::new("dab".as_bytes(), 1000);
            let to = LookupKey::new("eab".as_bytes(), 1010);
            let c = vs
                .compact_range(1, from.internal_key(), to.internal_key())
                .unwrap();
            assert_eq!(3, c.inputs[0].len());
            assert_eq!(1, c.inputs[1].len());
            assert_eq!(0, c.grandparents.as_ref().unwrap().len());
            iterator_properties(
                vs.make_input_iterator(&c),
                12,
                Rc::new(Box::new(vs.cmp.clone())),
            );

            // is_trivial_move
            let from = LookupKey::new("fab".as_bytes(), 1000);
            let to = LookupKey::new("fba".as_bytes(), 1010);
            let mut c = vs
                .compact_range(2, from.internal_key(), to.internal_key())
                .unwrap();
            // pretend it's not manual
            c.manual = false;
            assert!(c.is_trivial_move());

            // should_stop_before
            let from = LookupKey::new("000".as_bytes(), 1000);
            let to = LookupKey::new("zzz".as_bytes(), 1010);
            let mid = LookupKey::new("abc".as_bytes(), 1010);
            let mut c = vs
                .compact_range(0, from.internal_key(), to.internal_key())
                .unwrap();
            assert!(!c.should_stop_before(from.internal_key()));
            assert!(!c.should_stop_before(mid.internal_key()));
            assert!(!c.should_stop_before(to.internal_key()));

            // is_base_level_for
            let from = LookupKey::new("000".as_bytes(), 1000);
            let to = LookupKey::new("zzz".as_bytes(), 1010);
            let mut c = vs
                .compact_range(0, from.internal_key(), to.internal_key())
                .unwrap();
            assert!(c.is_base_level_for("aaa".as_bytes()));
            assert!(!c.is_base_level_for("hac".as_bytes()));

            // input/add_input_deletions
            let from = LookupKey::new("000".as_bytes(), 1000);
            let to = LookupKey::new("zzz".as_bytes(), 1010);
            let mut c = vs
                .compact_range(0, from.internal_key(), to.internal_key())
                .unwrap();
            for inp in &[(0, 0, 1), (0, 1, 2), (1, 0, 3)] {
                let f = &c.inputs[inp.0][inp.1];
                assert_eq!(inp.2, f.borrow().num);
            }
            c.add_input_deletions();
            assert_eq!(23, c.edit().encode().len())
        }
    }
}
