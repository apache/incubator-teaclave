//! db_impl contains the implementation of the database interface and high-level compaction and
//! maintenance logic.
#![allow(unused_attributes)]

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::path::PathEx;

use db_iter::DBIterator;

use cmp::{Cmp, InternalKeyCmp};
use env::{Env, FileLock};
use error::{err, Result, StatusCode};
use filter::{BoxedFilterPolicy, InternalFilterPolicy};
use infolog::Logger;
use key_types::{parse_internal_key, InternalKey, LookupKey, ValueType};
use log::{LogReader, LogWriter};
use memtable::MemTable;
use merging_iter::MergingIter;
use options::Options;
use snapshot::{Snapshot, SnapshotList};
use table_builder::TableBuilder;
use table_cache::{table_file_name, TableCache};
use types::{
    parse_file_name, share, FileMetaData, FileNum, FileType, LdbIterator, SequenceNumber, Shared,
    MAX_SEQUENCE_NUMBER, NUM_LEVELS,
};
use version::Version;
use version_edit::VersionEdit;
use version_set::{
    manifest_file_name, read_current_file, set_current_file, Compaction, VersionSet,
};
use write_batch::WriteBatch;

use std::cmp::Ordering;
use std::io::{self, BufWriter, Write};
use std::mem;
use std::ops::Drop;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

/// DB contains the actual database implemenation. As opposed to the original, this implementation
/// is not concurrent (yet).
pub struct DB {
    name: PathBuf,
    path: PathBuf,
    lock: Option<FileLock>,

    internal_cmp: Rc<Box<dyn Cmp>>,
    fpol: InternalFilterPolicy<BoxedFilterPolicy>,
    opt: Options,

    mem: MemTable,
    imm: Option<MemTable>,

    log: Option<LogWriter<BufWriter<Box<dyn Write>>>>,
    log_num: Option<FileNum>,
    cache: Shared<TableCache>,
    vset: Shared<VersionSet>,
    snaps: SnapshotList,

    cstats: [CompactionStats; NUM_LEVELS],
}

impl DB {
    // RECOVERY AND INITIALIZATION //

    /// new initializes a new DB object, but doesn't touch disk.
    fn new<P: AsRef<Path>>(name: P, mut opt: Options) -> DB {
        let name = name.as_ref();
        if opt.log.is_none() {
            let log = open_info_log(opt.env.as_ref().as_ref(), name);
            opt.log = Some(share(log));
        }

        // FIXME:: use std::untrusted::path::PathEx
        let path = name.canonicalize().unwrap_or(name.to_owned());

        let cache = share(TableCache::new(&name, opt.clone(), opt.max_open_files - 10));
        let vset = VersionSet::new(&name, opt.clone(), cache.clone());

        DB {
            name: name.to_owned(),
            path: path,
            lock: None,
            internal_cmp: Rc::new(Box::new(InternalKeyCmp(opt.cmp.clone()))),
            fpol: InternalFilterPolicy::new(opt.filter_policy.clone()),

            mem: MemTable::new(opt.cmp.clone()),
            imm: None,

            opt: opt,

            log: None,
            log_num: None,
            cache: cache,
            vset: share(vset),
            snaps: SnapshotList::new(),

            cstats: Default::default(),
        }
    }

    fn current(&self) -> Shared<Version> {
        self.vset.borrow().current()
    }

    /// Opens or creates a new or existing database. `name` is the name of the directory containing
    /// the database.
    ///
    /// Whether a new database is created and what happens if a database exists at the given path
    /// depends on the options set (`create_if_missing`, `error_if_exists`).
    pub fn open<P: AsRef<Path>>(name: P, opt: Options) -> Result<DB> {
        let name = name.as_ref();
        let mut db = DB::new(name, opt);
        let mut ve = VersionEdit::new();
        let save_manifest = db.recover(&mut ve)?;

        // Create log file if an old one is not being reused.
        if db.log.is_none() {
            let lognum = db.vset.borrow_mut().new_file_number();
            let logfile = db
                .opt
                .env
                .open_writable_file(Path::new(&log_file_name(&db.name, lognum)))?;
            ve.set_log_num(lognum);
            db.log = Some(LogWriter::new(BufWriter::new(logfile)));
            db.log_num = Some(lognum);
        }

        if save_manifest {
            ve.set_log_num(db.log_num.unwrap_or(0));
            db.vset.borrow_mut().log_and_apply(ve)?;
        }

        db.delete_obsolete_files()?;
        db.maybe_do_compaction()?;
        Ok(db)
    }

    /// initialize_db initializes a new database.
    fn initialize_db(&mut self) -> Result<()> {
        let mut ve = VersionEdit::new();
        ve.set_comparator_name(self.opt.cmp.id());
        ve.set_log_num(0);
        ve.set_next_file(2);
        ve.set_last_seq(0);

        {
            let manifest = manifest_file_name(&self.path, 1);
            let manifest_file = self.opt.env.open_writable_file(Path::new(&manifest))?;
            let mut lw = LogWriter::new(manifest_file);
            lw.add_record(&ve.encode())?;
            lw.flush()?;
        }
        set_current_file(&self.opt.env, &self.path, 1)
    }

    /// recover recovers from the existing state on disk. If the wrapped result is `true`, then
    /// log_and_apply() should be called after recovery has finished.
    fn recover(&mut self, ve: &mut VersionEdit) -> Result<bool> {
        if self.opt.error_if_exists && self.opt.env.exists(&self.path.as_ref()).unwrap_or(false) {
            return err(StatusCode::AlreadyExists, "database already exists");
        }

        let _ = self.opt.env.mkdir(Path::new(&self.path));
        self.acquire_lock()?;

        if let Err(e) = read_current_file(&self.opt.env, &self.path) {
            if e.code == StatusCode::NotFound && self.opt.create_if_missing {
                self.initialize_db()?;
            } else {
                return err(
                    StatusCode::InvalidArgument,
                    "database does not exist and create_if_missing is false",
                );
            }
        }

        // If save_manifest is true, we should log_and_apply() later in order to write the new
        // manifest.
        let mut save_manifest = self.vset.borrow_mut().recover()?;

        // Recover from all log files not in the descriptor.
        let mut max_seq = 0;
        let filenames = self.opt.env.children(&self.path)?;
        let mut expected = self.vset.borrow().live_files();
        let mut log_files = vec![];

        for file in &filenames {
            if let Ok((num, typ)) = parse_file_name(&file) {
                expected.remove(&num);
                if typ == FileType::Log
                    && (num >= self.vset.borrow().log_num || num == self.vset.borrow().prev_log_num)
                {
                    log_files.push(num);
                }
            }
        }
        if !expected.is_empty() {
            log!(self.opt.log, "Missing at least these files: {:?}", expected);
            return err(StatusCode::Corruption, "missing live files (see log)");
        }

        log_files.sort();
        for i in 0..log_files.len() {
            let (save_manifest_, max_seq_) =
                self.recover_log_file(log_files[i], i == log_files.len() - 1, ve)?;
            if save_manifest_ {
                save_manifest = true;
            }
            if max_seq_ > max_seq {
                max_seq = max_seq_;
            }
            self.vset.borrow_mut().mark_file_number_used(log_files[i]);
        }

        if self.vset.borrow().last_seq < max_seq {
            self.vset.borrow_mut().last_seq = max_seq;
        }

        Ok(save_manifest)
    }

    /// recover_log_file reads a single log file into a memtable, writing new L0 tables if
    /// necessary. If is_last is true, it checks whether the log file can be reused, and sets up
    /// the database's logging handles appropriately if that's the case.
    fn recover_log_file(
        &mut self,
        log_num: FileNum,
        is_last: bool,
        ve: &mut VersionEdit,
    ) -> Result<(bool, SequenceNumber)> {
        let filename = log_file_name(&self.path, log_num);
        let mut compactions = 0;
        let mut max_seq = 0;
        let mut save_manifest = false;
        let cmp: Rc<Box<dyn Cmp>> = self.opt.cmp.clone();
        let mut mem = MemTable::new(cmp.clone());
        {
            let logfile = self.opt.env.open_sequential_file(Path::new(&filename))?;
            // Use the user-supplied comparator; it will be wrapped inside a MemtableKeyCmp.

            let mut logreader = LogReader::new(
                logfile, // checksum=
                true,
            );
            log!(self.opt.log, "Recovering log file {:?}", filename);
            let mut scratch = vec![];
            let mut batch = WriteBatch::new();

            while let Ok(len) = logreader.read(&mut scratch) {
                if len == 0 {
                    break;
                }
                if len < 12 {
                    log!(
                        self.opt.log,
                        "corruption in log file {:06}: record shorter than 12B",
                        log_num
                    );
                    continue;
                }

                batch.set_contents(&scratch);
                batch.insert_into_memtable(batch.sequence(), &mut mem);

                let last_seq = batch.sequence() + batch.count() as u64 - 1;
                if last_seq > max_seq {
                    max_seq = last_seq
                }
                if mem.approx_mem_usage() > self.opt.write_buffer_size {
                    compactions += 1;
                    self.write_l0_table(&mem, ve, None)?;
                    save_manifest = true;
                    mem = MemTable::new(cmp.clone());
                }
                batch.clear();
            }
        }

        // Check if we can reuse the last log file.
        if self.opt.reuse_logs && is_last && compactions == 0 {
            assert!(self.log.is_none());
            log!(self.opt.log, "reusing log file {:?}", filename);
            let oldsize = self.opt.env.size_of(Path::new(&filename))?;
            let oldfile = self.opt.env.open_appendable_file(Path::new(&filename))?;
            let lw = LogWriter::new_with_off(BufWriter::new(oldfile), oldsize);
            self.log = Some(lw);
            self.log_num = Some(log_num);
            self.mem = mem;
        } else if mem.len() > 0 {
            // Log is not reused, so write out the accumulated memtable.
            save_manifest = true;
            self.write_l0_table(&mem, ve, None)?;
        }

        Ok((save_manifest, max_seq))
    }

    /// delete_obsolete_files removes files that are no longer needed from the file system.
    fn delete_obsolete_files(&mut self) -> Result<()> {
        let files = self.vset.borrow().live_files();
        let filenames = self.opt.env.children(Path::new(&self.path))?;
        for name in filenames {
            if let Ok((num, typ)) = parse_file_name(&name) {
                match typ {
                    FileType::Log => {
                        if num >= self.vset.borrow().log_num {
                            continue;
                        }
                    }
                    FileType::Descriptor => {
                        if num >= self.vset.borrow().manifest_num {
                            continue;
                        }
                    }
                    FileType::Table => {
                        if files.contains(&num) {
                            continue;
                        }
                    }
                    // NOTE: In this non-concurrent implementation, we likely never find temp
                    // files.
                    FileType::Temp => {
                        if files.contains(&num) {
                            continue;
                        }
                    }
                    FileType::Current | FileType::DBLock | FileType::InfoLog => continue,
                }

                // If we're here, delete this file.
                if typ == FileType::Table {
                    let _ = self.cache.borrow_mut().evict(num);
                }
                log!(self.opt.log, "Deleting file type={:?} num={}", typ, num);
                if let Err(e) = self.opt.env.delete(&self.path.join(&name)) {
                    log!(self.opt.log, "Deleting file num={} failed: {}", num, e);
                }
            }
        }
        Ok(())
    }

    /// acquire_lock acquires the lock file.
    fn acquire_lock(&mut self) -> Result<()> {
        let lock_r = self.opt.env.lock(Path::new(&lock_file_name(&self.path)));
        match lock_r {
            Ok(lockfile) => {
                self.lock = Some(lockfile);
                Ok(())
            }
            Err(ref e) if e.code == StatusCode::LockError => err(
                StatusCode::LockError,
                "database lock is held by another instance",
            ),
            Err(e) => Err(e),
        }
    }

    /// release_lock releases the lock file, if it's currently held.
    fn release_lock(&mut self) -> Result<()> {
        if let Some(l) = self.lock.take() {
            self.opt.env.unlock(l)
        } else {
            Ok(())
        }
    }
}

impl DB {
    // WRITE //

    /// Adds a single entry. It's a short, non-synchronous, form of `write()`; in order to make
    /// sure that the written entry is on disk, call `flush()` afterwards.
    pub fn put(&mut self, k: &[u8], v: &[u8]) -> Result<()> {
        let mut wb = WriteBatch::new();
        wb.put(k, v);
        self.write(wb, false)
    }

    /// Deletes a single entry. Like with `put()`, you can call `flush()` to guarantee that
    /// the operation made it to disk.
    pub fn delete(&mut self, k: &[u8]) -> Result<()> {
        let mut wb = WriteBatch::new();
        wb.delete(k);
        self.write(wb, false)
    }

    /// Writes an entire WriteBatch. `sync` determines whether the write should be flushed to
    /// disk.
    pub fn write(&mut self, batch: WriteBatch, sync: bool) -> Result<()> {
        assert!(self.log.is_some());

        self.make_room_for_write(false)?;

        let entries = batch.count() as u64;
        let log = self.log.as_mut().unwrap();
        let next = self.vset.borrow().last_seq + 1;

        batch.insert_into_memtable(next, &mut self.mem);
        log.add_record(&batch.encode(next))?;
        if sync {
            log.flush()?;
        }
        self.vset.borrow_mut().last_seq += entries;
        Ok(())
    }

    /// flush makes sure that all pending changes (e.g. from put()) are stored on disk.
    pub fn flush(&mut self) -> Result<()> {
        assert!(self.log.is_some());
        self.log.as_mut().unwrap().flush()
    }
}

impl DB {
    // READ //

    fn get_internal(&mut self, seq: SequenceNumber, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Using this lookup key will skip all entries with higher sequence numbers, because they
        // will compare "Lesser" using the InternalKeyCmp
        let lkey = LookupKey::new(key, seq);

        match self.mem.get(&lkey) {
            (Some(v), _) => return Ok(Some(v)),
            // deleted entry
            (None, true) => return Ok(None),
            // not found entry
            (None, false) => {}
        }

        if let Some(imm) = self.imm.as_ref() {
            match imm.get(&lkey) {
                (Some(v), _) => return Ok(Some(v)),
                // deleted entry
                (None, true) => return Ok(None),
                // not found entry
                (None, false) => {}
            }
        }

        let mut do_compaction = false;
        let mut result = None;

        // Limiting the borrow scope of self.current.
        {
            let current = self.current();
            let mut current = current.borrow_mut();
            if let Ok(Some((v, st))) = current.get(lkey.internal_key()) {
                if current.update_stats(st) {
                    do_compaction = true;
                }
                result = Some(v)
            }
        }

        if do_compaction {
            if let Err(e) = self.maybe_do_compaction() {
                log!(self.opt.log, "error while doing compaction in get: {}", e);
            }
        }
        Ok(result)
    }

    /// get_at reads the value for a given key at or before snapshot. It returns Ok(None) if the
    /// entry wasn't found, and Err(_) if an error occurred.
    pub fn get_at(&mut self, snapshot: &Snapshot, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.get_internal(snapshot.sequence(), key)
    }

    /// get is a simplified version of get_at(), translating errors to None.
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let seq = self.vset.borrow().last_seq;
        if let Ok(v) = self.get_internal(seq, key) {
            v
        } else {
            None
        }
    }
}

impl DB {
    // ITERATOR //

    /// new_iter returns a DBIterator over the current state of the database. The iterator will not
    /// return elements added to the database after its creation.
    pub fn new_iter(&mut self) -> Result<DBIterator> {
        let snapshot = self.get_snapshot();
        self.new_iter_at(snapshot)
    }

    /// new_iter_at returns a DBIterator at the supplied snapshot.
    pub fn new_iter_at(&mut self, ss: Snapshot) -> Result<DBIterator> {
        Ok(DBIterator::new(
            self.opt.cmp.clone(),
            self.vset.clone(),
            self.merge_iterators()?,
            ss,
        ))
    }

    /// merge_iterators produces a MergingIter merging the entries in the memtable, the immutable
    /// memtable, and table files from all levels.
    fn merge_iterators(&mut self) -> Result<MergingIter> {
        let mut iters: Vec<Box<dyn LdbIterator>> = vec![];
        if self.mem.len() > 0 {
            iters.push(Box::new(self.mem.iter()));
        }
        if let Some(ref imm) = self.imm {
            if imm.len() > 0 {
                iters.push(Box::new(imm.iter()));
            }
        }

        // Add iterators for table files.
        let current = self.current();
        let current = current.borrow();
        iters.extend(current.new_iters()?);

        Ok(MergingIter::new(self.internal_cmp.clone(), iters))
    }
}

impl DB {
    // SNAPSHOTS //

    /// Returns a snapshot at the current state. It can be used to retrieve entries from the
    /// database as they were at an earlier point in time.
    pub fn get_snapshot(&mut self) -> Snapshot {
        self.snaps.new_snapshot(self.vset.borrow().last_seq)
    }
}

impl DB {
    // STATISTICS //
    fn add_stats(&mut self, level: usize, cs: CompactionStats) {
        assert!(level < NUM_LEVELS);
        self.cstats[level].add(cs);
    }

    /// Trigger a compaction based on where this key is located in the different levels.
    fn record_read_sample<'a>(&mut self, k: InternalKey<'a>) {
        let current = self.current();
        if current.borrow_mut().record_read_sample(k) {
            if let Err(e) = self.maybe_do_compaction() {
                log!(self.opt.log, "record_read_sample: compaction failed: {}", e);
            }
        }
    }
}

impl DB {
    // COMPACTIONS //

    /// make_room_for_write checks if the memtable has become too large, and triggers a compaction
    /// if it's the case.
    fn make_room_for_write(&mut self, force: bool) -> Result<()> {
        if !force && self.mem.approx_mem_usage() < self.opt.write_buffer_size {
            Ok(())
        } else if self.mem.len() == 0 {
            Ok(())
        } else {
            // Create new memtable.
            let logn = self.vset.borrow_mut().new_file_number();
            let logf = self
                .opt
                .env
                .open_writable_file(Path::new(&log_file_name(&self.path, logn)));
            if logf.is_err() {
                self.vset.borrow_mut().reuse_file_number(logn);
                Err(logf.err().unwrap())
            } else {
                self.log = Some(LogWriter::new(BufWriter::new(logf.unwrap())));
                self.log_num = Some(logn);

                let mut imm = MemTable::new(self.opt.cmp.clone());
                mem::swap(&mut imm, &mut self.mem);
                self.imm = Some(imm);
                self.maybe_do_compaction()
            }
        }
    }

    /// maybe_do_compaction starts a blocking compaction if it makes sense.
    fn maybe_do_compaction(&mut self) -> Result<()> {
        if self.imm.is_some() {
            self.compact_memtable()
        } else if self.vset.borrow().needs_compaction() {
            let c = self.vset.borrow_mut().pick_compaction();
            if let Some(c) = c {
                self.start_compaction(c)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// compact_range triggers an immediate compaction on the specified key range. Repeatedly
    /// calling this without actually adding new keys is not useful.
    ///
    /// Compactions in general will cause the database to find entries more quickly, and take up
    /// less space on disk.
    pub fn compact_range(&mut self, from: &[u8], to: &[u8]) -> Result<()> {
        let mut max_level = 1;
        {
            let v = self.vset.borrow().current();
            let v = v.borrow();
            for l in 1..NUM_LEVELS - 1 {
                if v.overlap_in_level(l, from, to) {
                    max_level = l;
                }
            }
        }

        // Compact memtable.
        self.make_room_for_write(true)?;

        let mut ifrom = LookupKey::new(from, MAX_SEQUENCE_NUMBER)
            .internal_key()
            .to_vec();
        let iend = LookupKey::new_full(to, 0, ValueType::TypeDeletion);

        for l in 0..max_level + 1 {
            loop {
                let c_ = self
                    .vset
                    .borrow_mut()
                    .compact_range(l, &ifrom, iend.internal_key());
                if let Some(c) = c_ {
                    // Update ifrom to the largest key of the last file in this compaction.
                    let ix = c.num_inputs(0) - 1;
                    ifrom = c.input(0, ix).largest.clone();
                    self.start_compaction(c)?;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    /// start_compaction dispatches the different kinds of compactions depending on the current
    /// state of the database.
    fn start_compaction(&mut self, mut compaction: Compaction) -> Result<()> {
        if compaction.is_trivial_move() {
            assert_eq!(1, compaction.num_inputs(0));
            let f = compaction.input(0, 0);
            let num = f.num;
            let size = f.size;
            let level = compaction.level();

            compaction.edit().delete_file(level, num);
            compaction.edit().add_file(level + 1, f);

            let r = self.vset.borrow_mut().log_and_apply(compaction.into_edit());
            if let Err(e) = r {
                log!(self.opt.log, "trivial move failed: {}", e);
                Err(e)
            } else {
                log!(
                    self.opt.log,
                    "Moved num={} bytes={} from L{} to L{}",
                    num,
                    size,
                    level,
                    level + 1
                );
                log!(
                    self.opt.log,
                    "Summary: {}",
                    self.vset.borrow().current_summary()
                );
                Ok(())
            }
        } else {
            let smallest = if self.snaps.empty() {
                self.vset.borrow().last_seq
            } else {
                self.snaps.oldest()
            };
            let mut state = CompactionState::new(compaction, smallest);
            if let Err(e) = self.do_compaction_work(&mut state) {
                state.cleanup(&self.opt.env, &self.path);
                log!(self.opt.log, "Compaction work failed: {}", e);
            }
            self.install_compaction_results(state)?;
            log!(
                self.opt.log,
                "Compaction finished: {}",
                self.vset.borrow().current_summary()
            );

            self.delete_obsolete_files()
        }
    }

    fn compact_memtable(&mut self) -> Result<()> {
        assert!(self.imm.is_some());

        let mut ve = VersionEdit::new();
        let base = self.current();

        let imm = self.imm.take().unwrap();
        if let Err(e) = self.write_l0_table(&imm, &mut ve, Some(&base.borrow())) {
            self.imm = Some(imm);
            return Err(e);
        }
        ve.set_log_num(self.log_num.unwrap_or(0));
        self.vset.borrow_mut().log_and_apply(ve)?;
        if let Err(e) = self.delete_obsolete_files() {
            log!(self.opt.log, "Error deleting obsolete files: {}", e);
        }
        Ok(())
    }

    /// write_l0_table writes the given memtable to a table file.
    fn write_l0_table(
        &mut self,
        memt: &MemTable,
        ve: &mut VersionEdit,
        base: Option<&Version>,
    ) -> Result<()> {
        let start_ts = self.opt.env.micros();
        let num = self.vset.borrow_mut().new_file_number();
        log!(self.opt.log, "Start write of L0 table {:06}", num);
        let fmd = build_table(&self.path, &self.opt, memt.iter(), num)?;
        log!(self.opt.log, "L0 table {:06} has {} bytes", num, fmd.size);

        // Wrote empty table.
        if fmd.size == 0 {
            self.vset.borrow_mut().reuse_file_number(num);
            return Ok(());
        }

        let cache_result = self.cache.borrow_mut().get_table(num);
        if let Err(e) = cache_result {
            log!(
                self.opt.log,
                "L0 table {:06} not returned by cache: {}",
                num,
                e
            );
            let _ = self
                .opt
                .env
                .delete(Path::new(&table_file_name(&self.path, num)));
            return Err(e);
        }

        let mut stats = CompactionStats::default();
        stats.micros = self.opt.env.micros() - start_ts;
        stats.written = fmd.size;

        let mut level = 0;
        if let Some(b) = base {
            level = b.pick_memtable_output_level(
                parse_internal_key(&fmd.smallest).2,
                parse_internal_key(&fmd.largest).2,
            );
        }

        self.add_stats(level, stats);
        ve.add_file(level, fmd);

        Ok(())
    }

    fn do_compaction_work(&mut self, cs: &mut CompactionState) -> Result<()> {
        {
            let current = self.vset.borrow().current();
            assert!(current.borrow().num_level_files(cs.compaction.level()) > 0);
            assert!(cs.builder.is_none());
        }
        let start_ts = self.opt.env.micros();
        log!(
            self.opt.log,
            "Compacting {} files at L{} and {} files at L{}",
            cs.compaction.num_inputs(0),
            cs.compaction.level(),
            cs.compaction.num_inputs(1),
            cs.compaction.level() + 1
        );

        let mut input = self.vset.borrow().make_input_iterator(&cs.compaction);
        input.seek_to_first();

        let (mut key, mut val) = (vec![], vec![]);
        let mut last_seq_for_key = MAX_SEQUENCE_NUMBER;

        let mut have_ukey = false;
        let mut current_ukey = vec![];

        while input.valid() {
            // TODO: Do we need to do a memtable compaction here? Probably not, in the sequential
            // case.
            assert!(input.current(&mut key, &mut val));
            if cs.compaction.should_stop_before(&key) && cs.builder.is_none() {
                self.finish_compaction_output(cs, key.clone())?;
            }
            let (ktyp, seq, ukey) = parse_internal_key(&key);
            if seq == 0 {
                // Parsing failed.
                log!(self.opt.log, "Encountered seq=0 in key: {:?}", &key);
                last_seq_for_key = MAX_SEQUENCE_NUMBER;
                have_ukey = false;
                current_ukey.clear();
                input.advance();
                continue;
            }

            if !have_ukey || self.opt.cmp.cmp(ukey, &current_ukey) != Ordering::Equal {
                // First occurrence of this key.
                current_ukey.clear();
                current_ukey.extend_from_slice(ukey);
                have_ukey = true;
                last_seq_for_key = MAX_SEQUENCE_NUMBER;
            }

            // We can omit the key under the following conditions:
            if last_seq_for_key <= cs.smallest_seq {
                last_seq_for_key = seq;
                input.advance();
                continue;
            }
            // Entry is deletion; no older version is observable by any snapshot; and all entries
            // in compacted levels with smaller sequence numbers will
            if ktyp == ValueType::TypeDeletion
                && seq <= cs.smallest_seq
                && cs.compaction.is_base_level_for(ukey)
            {
                last_seq_for_key = seq;
                input.advance();
                continue;
            }

            last_seq_for_key = seq;

            if cs.builder.is_none() {
                let fnum = self.vset.borrow_mut().new_file_number();
                let mut fmd = FileMetaData::default();
                fmd.num = fnum;

                let fname = table_file_name(&self.path, fnum);
                let f = self.opt.env.open_writable_file(Path::new(&fname))?;
                let f = Box::new(BufWriter::new(f));
                cs.builder = Some(TableBuilder::new(self.opt.clone(), f));
                cs.outputs.push(fmd);
            }
            if cs.builder.as_ref().unwrap().entries() == 0 {
                cs.current_output().smallest = key.clone();
            }
            cs.builder.as_mut().unwrap().add(&key, &val)?;
            // NOTE: Adjust max file size based on level.
            if cs.builder.as_ref().unwrap().size_estimate() > self.opt.max_file_size {
                self.finish_compaction_output(cs, key.clone())?;
            }

            input.advance();
        }

        if cs.builder.is_some() {
            self.finish_compaction_output(cs, key)?;
        }

        let mut stats = CompactionStats::default();
        stats.micros = self.opt.env.micros() - start_ts;
        for parent in 0..2 {
            for inp in 0..cs.compaction.num_inputs(parent) {
                stats.read += cs.compaction.input(parent, inp).size;
            }
        }
        for output in &cs.outputs {
            stats.written += output.size;
        }
        self.cstats[cs.compaction.level()].add(stats);
        Ok(())
    }

    fn finish_compaction_output(
        &mut self,
        cs: &mut CompactionState,
        largest: Vec<u8>,
    ) -> Result<()> {
        assert!(cs.builder.is_some());
        let output_num = cs.current_output().num;
        assert!(output_num > 0);

        // The original checks if the input iterator has an OK status. For this, we'd need to
        // extend the LdbIterator interface though -- let's see if we can without for now.
        // (it's not good for corruptions, in any case)
        let b = cs.builder.take().unwrap();
        let entries = b.entries();
        let bytes = b.finish()?;
        cs.total_bytes += bytes;

        cs.current_output().largest = largest;
        cs.current_output().size = bytes;

        if entries > 0 {
            // Verify that table can be used. (Separating get_table() because borrowing in an if
            // let expression is dangerous).
            let r = self.cache.borrow_mut().get_table(output_num);
            if let Err(e) = r {
                log!(self.opt.log, "New table can't be read: {}", e);
                return Err(e);
            }
            log!(
                self.opt.log,
                "New table num={}: keys={} size={}",
                output_num,
                entries,
                bytes
            );
        }
        Ok(())
    }

    fn install_compaction_results(&mut self, mut cs: CompactionState) -> Result<()> {
        log!(
            self.opt.log,
            "Compacted {} L{} files + {} L{} files => {}B",
            cs.compaction.num_inputs(0),
            cs.compaction.level(),
            cs.compaction.num_inputs(1),
            cs.compaction.level() + 1,
            cs.total_bytes
        );
        cs.compaction.add_input_deletions();
        let level = cs.compaction.level();
        for output in &cs.outputs {
            cs.compaction.edit().add_file(level + 1, output.clone());
        }
        self.vset
            .borrow_mut()
            .log_and_apply(cs.compaction.into_edit())
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        let _ = self.release_lock();
    }
}

struct CompactionState {
    compaction: Compaction,
    smallest_seq: SequenceNumber,
    outputs: Vec<FileMetaData>,
    builder: Option<TableBuilder<Box<dyn Write>>>,
    total_bytes: usize,
}

impl CompactionState {
    fn new(c: Compaction, smallest: SequenceNumber) -> CompactionState {
        CompactionState {
            compaction: c,
            smallest_seq: smallest,
            outputs: vec![],
            builder: None,
            total_bytes: 0,
        }
    }

    fn current_output(&mut self) -> &mut FileMetaData {
        let len = self.outputs.len();
        &mut self.outputs[len - 1]
    }

    /// cleanup cleans up after an aborted compaction.
    fn cleanup<P: AsRef<Path>>(&mut self, env: &Box<dyn Env>, name: P) {
        for o in self.outputs.drain(..) {
            let name = table_file_name(name.as_ref(), o.num);
            let _ = env.delete(&name);
        }
    }
}

#[derive(Debug, Default)]
struct CompactionStats {
    micros: u64,
    read: usize,
    written: usize,
}

impl CompactionStats {
    fn add(&mut self, cs: CompactionStats) {
        self.micros += cs.micros;
        self.read += cs.read;
        self.written += cs.written;
    }
}

pub fn build_table<I: LdbIterator, P: AsRef<Path>>(
    dbname: P,
    opt: &Options,
    mut from: I,
    num: FileNum,
) -> Result<FileMetaData> {
    from.reset();
    let filename = table_file_name(dbname.as_ref(), num);

    let (mut kbuf, mut vbuf) = (vec![], vec![]);
    let mut firstkey = None;
    // lastkey is what remains in kbuf.

    // Clean up file if write fails at any point.
    //
    // TODO: Replace with catch {} when available.
    let r = (|| -> Result<()> {
        let f = opt.env.open_writable_file(Path::new(&filename))?;
        let f = BufWriter::new(f);
        let mut builder = TableBuilder::new(opt.clone(), f);
        while from.advance() {
            assert!(from.current(&mut kbuf, &mut vbuf));
            if firstkey.is_none() {
                firstkey = Some(kbuf.clone());
            }
            builder.add(&kbuf, &vbuf)?;
        }
        builder.finish()?;
        Ok(())
    })();

    if let Err(e) = r {
        let _ = opt.env.delete(Path::new(&filename));
        return Err(e);
    }

    let mut md = FileMetaData::default();
    if firstkey.is_none() {
        let _ = opt.env.delete(Path::new(&filename));
    } else {
        md.num = num;
        md.size = opt.env.size_of(Path::new(&filename))?;
        md.smallest = firstkey.unwrap();
        md.largest = kbuf;
    }
    Ok(md)
}

fn log_file_name(db: &Path, num: FileNum) -> PathBuf {
    db.join(format!("{:06}.log", num))
}

fn lock_file_name(db: &Path) -> PathBuf {
    db.join("LOCK")
}

/// open_info_log opens an info log file in the given database. It transparently returns a
/// /dev/null logger in case the open fails.
fn open_info_log<E: Env + ?Sized, P: AsRef<Path>>(env: &E, db: P) -> Logger {
    let db = db.as_ref();
    let logfilename = db.join("LOG");
    let oldlogfilename = db.join("LOG.old");
    let _ = env.mkdir(Path::new(db));
    if let Ok(e) = env.exists(Path::new(&logfilename)) {
        if e {
            let _ = env.rename(Path::new(&logfilename), Path::new(&oldlogfilename));
        }
    }
    if let Ok(w) = env.open_writable_file(Path::new(&logfilename)) {
        Logger(w)
    } else {
        Logger(Box::new(io::sink()))
    }
}

#[cfg(test)]
pub mod testutil {
    use super::*;

    use version::testutil::make_version;

    /// build_db creates a database filled with the tables created by make_version().
    pub fn build_db() -> (DB, Options) {
        let name = "db";
        let (v, mut opt) = make_version();
        opt.reuse_logs = false;
        opt.reuse_manifest = false;
        let mut ve = VersionEdit::new();
        ve.set_comparator_name(opt.cmp.id());
        ve.set_log_num(0);
        // 9 files + 1 manifest we write below.
        ve.set_next_file(11);
        // 30 entries in these tables.
        ve.set_last_seq(30);

        for l in 0..NUM_LEVELS {
            for f in &v.files[l] {
                ve.add_file(l, f.borrow().clone());
            }
        }

        let manifest = manifest_file_name(name, 10);
        let manifest_file = opt.env.open_writable_file(Path::new(&manifest)).unwrap();
        let mut lw = LogWriter::new(manifest_file);
        lw.add_record(&ve.encode()).unwrap();
        lw.flush().unwrap();
        set_current_file(&opt.env, name, 10).unwrap();

        (DB::open(name, opt.clone()).unwrap(), opt)
    }

    /// set_file_to_compact ensures that the specified table file will be compacted next.
    pub fn set_file_to_compact(db: &mut DB, num: FileNum) {
        let v = db.current();
        let mut v = v.borrow_mut();

        let mut ftc = None;
        for l in 0..NUM_LEVELS {
            for f in &v.files[l] {
                if f.borrow().num == num {
                    ftc = Some((f.clone(), l));
                }
            }
        }
        if let Some((f, l)) = ftc {
            v.file_to_compact = Some(f);
            v.file_to_compact_lvl = l;
        } else {
            panic!("file number not found");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::{build_db, set_file_to_compact};
    use super::*;

    use error::Status;
    use key_types::LookupKey;
    use mem_env::MemEnv;
    use options;
    use test_util::LdbIteratorIter;
    use version::testutil::make_version;

    #[test]
    fn test_db_impl_open_info_log() {
        let e = MemEnv::new();
        {
            let l = Some(share(open_info_log(&e, "abc")));
            assert!(e.exists(Path::new("abc/LOG")).unwrap());
            log!(l, "hello {}", "world");
            assert_eq!(12, e.size_of(Path::new("abc/LOG")).unwrap());
        }
        {
            let l = Some(share(open_info_log(&e, "abc")));
            assert!(e.exists(Path::new("abc/LOG.old")).unwrap());
            assert!(e.exists(Path::new("abc/LOG")).unwrap());
            assert_eq!(12, e.size_of(Path::new("abc/LOG.old")).unwrap());
            assert_eq!(0, e.size_of(Path::new("abc/LOG")).unwrap());
            log!(l, "something else");
            log!(l, "and another {}", 1);

            let mut s = String::new();
            let mut r = e.open_sequential_file(Path::new("abc/LOG")).unwrap();
            r.read_to_string(&mut s).unwrap();
            assert_eq!("something else\nand another 1\n", &s);
        }
    }

    fn build_memtable() -> MemTable {
        let mut mt = MemTable::new(options::for_test().cmp);
        let mut i = 1;
        for k in ["abc", "def", "ghi", "jkl", "mno", "aabc", "test123"].iter() {
            mt.add(
                i,
                ValueType::TypeValue,
                k.as_bytes(),
                "looooongval".as_bytes(),
            );
            i += 1;
        }
        mt
    }

    #[test]
    fn test_db_impl_init() {
        // A sanity check for recovery and basic persistence.
        let opt = options::for_test();
        let env = opt.env.clone();

        // Several test cases with different options follow. The printlns can eventually be
        // removed.

        {
            let mut opt = opt.clone();
            opt.reuse_manifest = false;
            let _ = DB::open("otherdb", opt.clone()).unwrap();

            println!(
                "children after: {:?}",
                env.children(Path::new("otherdb/")).unwrap()
            );
            assert!(env.exists(Path::new("otherdb/CURRENT")).unwrap());
            // Database is initialized and initial manifest reused.
            assert!(!env.exists(Path::new("otherdb/MANIFEST-000001")).unwrap());
            assert!(env.exists(Path::new("otherdb/MANIFEST-000002")).unwrap());
            assert!(env.exists(Path::new("otherdb/000003.log")).unwrap());
        }

        {
            let mut opt = opt.clone();
            opt.reuse_manifest = true;
            let mut db = DB::open("db", opt.clone()).unwrap();

            println!(
                "children after: {:?}",
                env.children(Path::new("db/")).unwrap()
            );
            assert!(env.exists(Path::new("db/CURRENT")).unwrap());
            // Database is initialized and initial manifest reused.
            assert!(env.exists(Path::new("db/MANIFEST-000001")).unwrap());
            assert!(env.exists(Path::new("db/LOCK")).unwrap());
            assert!(env.exists(Path::new("db/000003.log")).unwrap());

            db.put("abc".as_bytes(), "def".as_bytes()).unwrap();
            db.put("abd".as_bytes(), "def".as_bytes()).unwrap();
        }

        {
            println!(
                "children before: {:?}",
                env.children(Path::new("db/")).unwrap()
            );
            let mut opt = opt.clone();
            opt.reuse_manifest = false;
            opt.reuse_logs = false;
            let mut db = DB::open("db", opt.clone()).unwrap();

            println!(
                "children after: {:?}",
                env.children(Path::new("db/")).unwrap()
            );
            // Obsolete manifest is deleted.
            assert!(!env.exists(Path::new("db/MANIFEST-000001")).unwrap());
            // New manifest is created.
            assert!(env.exists(Path::new("db/MANIFEST-000002")).unwrap());
            // Obsolete log file is deleted.
            assert!(!env.exists(Path::new("db/000003.log")).unwrap());
            // New L0 table has been added.
            assert!(env.exists(Path::new("db/000003.ldb")).unwrap());
            assert!(env.exists(Path::new("db/000004.log")).unwrap());
            // Check that entry exists and is correct. Phew, long call chain!
            let current = db.current();
            log!(opt.log, "files: {:?}", current.borrow().files);
            assert_eq!(
                "def".as_bytes(),
                current
                    .borrow_mut()
                    .get(LookupKey::new("abc".as_bytes(), 1).internal_key())
                    .unwrap()
                    .unwrap()
                    .0
                    .as_slice()
            );
            db.put("abe".as_bytes(), "def".as_bytes()).unwrap();
        }

        {
            println!(
                "children before: {:?}",
                env.children(Path::new("db/")).unwrap()
            );
            // reuse_manifest above causes the old manifest to be deleted as obsolete, but no new
            // manifest is written. CURRENT becomes stale.
            let mut opt = opt.clone();
            opt.reuse_logs = true;
            let db = DB::open("db", opt).unwrap();

            println!(
                "children after: {:?}",
                env.children(Path::new("db/")).unwrap()
            );
            assert!(!env.exists(Path::new("db/MANIFEST-000001")).unwrap());
            assert!(env.exists(Path::new("db/MANIFEST-000002")).unwrap());
            assert!(!env.exists(Path::new("db/MANIFEST-000005")).unwrap());
            assert!(env.exists(Path::new("db/000004.log")).unwrap());
            // 000004 should be reused, no new log file should be created.
            assert!(!env.exists(Path::new("db/000006.log")).unwrap());
            // Log is reused, so memtable should contain last written entry from above.
            assert_eq!(1, db.mem.len());
            assert_eq!(
                "def".as_bytes(),
                db.mem
                    .get(&LookupKey::new("abe".as_bytes(), 3))
                    .0
                    .unwrap()
                    .as_slice()
            );
        }
    }

    #[test]
    fn test_db_impl_compact_range() {
        let (mut db, opt) = build_db();
        let env = &opt.env;

        println!(
            "children before: {:?}",
            env.children(Path::new("db/")).unwrap()
        );
        db.compact_range(b"aaa", b"dba").unwrap();
        println!(
            "children after: {:?}",
            env.children(Path::new("db/")).unwrap()
        );

        assert_eq!(250, opt.env.size_of(Path::new("db/000007.ldb")).unwrap());
        assert_eq!(200, opt.env.size_of(Path::new("db/000008.ldb")).unwrap());
        assert_eq!(200, opt.env.size_of(Path::new("db/000009.ldb")).unwrap());
        assert_eq!(435, opt.env.size_of(Path::new("db/000015.ldb")).unwrap());

        assert!(!opt.env.exists(Path::new("db/000001.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000002.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000004.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000005.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000006.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000013.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000014.ldb")).unwrap());

        assert_eq!(b"val1".to_vec(), db.get(b"aaa").unwrap());
        assert_eq!(b"val2".to_vec(), db.get(b"cab").unwrap());
        assert_eq!(b"val3".to_vec(), db.get(b"aba").unwrap());
        assert_eq!(b"val3".to_vec(), db.get(b"fab").unwrap());
    }

    #[test]
    fn test_db_impl_compact_range_memtable() {
        let (mut db, opt) = build_db();
        let env = &opt.env;

        db.put(b"xxx", b"123").unwrap();

        println!(
            "children before: {:?}",
            env.children(Path::new("db/")).unwrap()
        );
        db.compact_range(b"aaa", b"dba").unwrap();
        println!(
            "children after: {:?}",
            env.children(Path::new("db/")).unwrap()
        );

        assert_eq!(250, opt.env.size_of(Path::new("db/000007.ldb")).unwrap());
        assert_eq!(200, opt.env.size_of(Path::new("db/000008.ldb")).unwrap());
        assert_eq!(200, opt.env.size_of(Path::new("db/000009.ldb")).unwrap());
        assert_eq!(182, opt.env.size_of(Path::new("db/000014.ldb")).unwrap());
        assert_eq!(435, opt.env.size_of(Path::new("db/000017.ldb")).unwrap());

        assert!(!opt.env.exists(Path::new("db/000001.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000002.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000003.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000004.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000005.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000006.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000015.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000016.ldb")).unwrap());

        assert_eq!(b"val1".to_vec(), db.get(b"aaa").unwrap());
        assert_eq!(b"val2".to_vec(), db.get(b"cab").unwrap());
        assert_eq!(b"val3".to_vec(), db.get(b"aba").unwrap());
        assert_eq!(b"val3".to_vec(), db.get(b"fab").unwrap());
        assert_eq!(b"123".to_vec(), db.get(b"xxx").unwrap());
    }

    #[allow(unused_variables)]
    #[test]
    fn test_db_impl_locking() {
        let opt = options::for_test();
        let db = DB::open("db", opt.clone()).unwrap();
        let want_err = Status::new(
            StatusCode::LockError,
            "database lock is held by another instance",
        );
        assert_eq!(want_err, DB::open("db", opt.clone()).err().unwrap());
    }

    #[test]
    fn test_db_impl_build_table() {
        let mut opt = options::for_test();
        opt.block_size = 128;
        let mt = build_memtable();

        let f = build_table("db", &opt, mt.iter(), 123).unwrap();
        let path = Path::new("db/000123.ldb");

        assert_eq!(
            LookupKey::new("aabc".as_bytes(), 6).internal_key(),
            f.smallest.as_slice()
        );
        assert_eq!(
            LookupKey::new("test123".as_bytes(), 7).internal_key(),
            f.largest.as_slice()
        );
        assert_eq!(379, f.size);
        assert_eq!(123, f.num);
        assert!(opt.env.exists(path).unwrap());

        {
            // Read table back in.
            let mut tc = TableCache::new("db", opt.clone(), 100);
            let tbl = tc.get_table(123).unwrap();
            assert_eq!(mt.len(), LdbIteratorIter::wrap(&mut tbl.iter()).count());
        }

        {
            // Corrupt table; make sure it doesn't load fully.
            let mut buf = vec![];
            opt.env
                .open_sequential_file(path)
                .unwrap()
                .read_to_end(&mut buf)
                .unwrap();
            buf[150] += 1;
            opt.env
                .open_writable_file(path)
                .unwrap()
                .write_all(&buf)
                .unwrap();

            let mut tc = TableCache::new("db", opt.clone(), 100);
            let tbl = tc.get_table(123).unwrap();
            // The last two entries are skipped due to the corruption above.
            assert_eq!(
                5,
                LdbIteratorIter::wrap(&mut tbl.iter())
                    .map(|v| println!("{:?}", v))
                    .count()
            );
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn test_db_impl_build_db_sanity() {
        let db = build_db().0;
        let env = &db.opt.env;
        let name = &db.name;

        assert!(env.exists(Path::new(&log_file_name(name, 12))).unwrap());
    }

    #[test]
    fn test_db_impl_get_from_table_with_snapshot() {
        let mut db = build_db().0;

        assert_eq!(30, db.vset.borrow().last_seq);

        // seq = 31
        db.put("xyy".as_bytes(), "123".as_bytes()).unwrap();
        let old_ss = db.get_snapshot();
        // seq = 32
        db.put("xyz".as_bytes(), "123".as_bytes()).unwrap();
        db.flush().unwrap();
        assert!(db.get_at(&old_ss, "xyy".as_bytes()).unwrap().is_some());
        assert!(db.get_at(&old_ss, "xyz".as_bytes()).unwrap().is_none());

        // memtable get
        assert_eq!(
            "123".as_bytes(),
            db.get("xyz".as_bytes()).unwrap().as_slice()
        );
        assert!(db.get_internal(31, "xyy".as_bytes()).unwrap().is_some());
        assert!(db.get_internal(32, "xyy".as_bytes()).unwrap().is_some());

        assert!(db.get_internal(31, "xyz".as_bytes()).unwrap().is_none());
        assert!(db.get_internal(32, "xyz".as_bytes()).unwrap().is_some());

        // table get
        assert_eq!(
            "val2".as_bytes(),
            db.get("eab".as_bytes()).unwrap().as_slice()
        );
        assert!(db.get_internal(3, "eab".as_bytes()).unwrap().is_none());
        assert!(db.get_internal(32, "eab".as_bytes()).unwrap().is_some());

        {
            let ss = db.get_snapshot();
            assert_eq!(
                "val2".as_bytes(),
                db.get_at(&ss, "eab".as_bytes())
                    .unwrap()
                    .unwrap()
                    .as_slice()
            );
        }

        // from table.
        assert_eq!(
            "val2".as_bytes(),
            db.get("cab".as_bytes()).unwrap().as_slice()
        );
    }

    #[test]
    fn test_db_impl_delete() {
        let mut db = build_db().0;

        db.put(b"xyy", b"123").unwrap();
        db.put(b"xyz", b"123").unwrap();

        assert!(db.get(b"xyy").is_some());
        assert!(db.get(b"gaa").is_some());

        // Delete one memtable entry and one table entry.
        db.delete(b"xyy").unwrap();
        db.delete(b"gaa").unwrap();

        assert!(db.get(b"xyy").is_none());
        assert!(db.get(b"gaa").is_none());
        assert!(db.get(b"xyz").is_some());
    }

    #[test]
    fn test_db_impl_compact_single_file() {
        let mut db = build_db().0;
        set_file_to_compact(&mut db, 4);
        db.maybe_do_compaction().unwrap();

        let env = &db.opt.env;
        let name = &db.name;
        assert!(!env.exists(Path::new(&table_file_name(name, 3))).unwrap());
        assert!(!env.exists(Path::new(&table_file_name(name, 4))).unwrap());
        assert!(!env.exists(Path::new(&table_file_name(name, 5))).unwrap());
        assert!(env.exists(Path::new(&table_file_name(name, 13))).unwrap());
    }

    #[test]
    fn test_db_impl_compaction_trivial_move() {
        let mut db = DB::open("db", options::for_test()).unwrap();

        db.put("abc".as_bytes(), "xyz".as_bytes()).unwrap();
        db.put("ab3".as_bytes(), "xyz".as_bytes()).unwrap();
        db.put("ab0".as_bytes(), "xyz".as_bytes()).unwrap();
        db.put("abz".as_bytes(), "xyz".as_bytes()).unwrap();
        assert_eq!(4, db.mem.len());
        let mut imm = MemTable::new(db.opt.cmp.clone());
        mem::swap(&mut imm, &mut db.mem);
        db.imm = Some(imm);
        db.compact_memtable().unwrap();

        println!(
            "children after: {:?}",
            db.opt.env.children(Path::new("db/")).unwrap()
        );
        assert!(db.opt.env.exists(Path::new("db/000004.ldb")).unwrap());

        {
            let v = db.current();
            let mut v = v.borrow_mut();
            v.file_to_compact = Some(v.files[2][0].clone());
            v.file_to_compact_lvl = 2;
        }

        db.maybe_do_compaction().unwrap();

        {
            let v = db.current();
            let v = v.borrow_mut();
            assert_eq!(1, v.files[3].len());
        }
    }

    #[test]
    fn test_db_impl_memtable_compaction() {
        let mut opt = options::for_test();
        opt.write_buffer_size = 25;
        let mut db = DB::new("db", opt);

        // Fill up memtable.
        db.mem = build_memtable();

        // Trigger memtable compaction.
        db.make_room_for_write(true).unwrap();
        assert_eq!(0, db.mem.len());
        assert!(db.opt.env.exists(Path::new("db/000002.log")).unwrap());
        assert!(db.opt.env.exists(Path::new("db/000003.ldb")).unwrap());
        assert_eq!(351, db.opt.env.size_of(Path::new("db/000003.ldb")).unwrap());
        assert_eq!(
            7,
            LdbIteratorIter::wrap(&mut db.cache.borrow_mut().get_table(3).unwrap().iter()).count()
        );
    }

    #[test]
    fn test_db_impl_compaction() {
        let mut db = build_db().0;
        let v = db.current();
        v.borrow_mut().compaction_score = Some(2.0);
        v.borrow_mut().compaction_level = Some(1);

        db.maybe_do_compaction().unwrap();

        assert!(!db.opt.env.exists(Path::new("db/000003.ldb")).unwrap());
        assert!(db.opt.env.exists(Path::new("db/000013.ldb")).unwrap());
        assert_eq!(345, db.opt.env.size_of(Path::new("db/000013.ldb")).unwrap());

        // New current version.
        let v = db.current();
        assert_eq!(0, v.borrow().files[1].len());
        assert_eq!(2, v.borrow().files[2].len());
    }

    #[test]
    fn test_db_impl_compaction_trivial() {
        let (mut v, opt) = make_version();

        let to_compact = v.files[2][0].clone();
        v.file_to_compact = Some(to_compact);
        v.file_to_compact_lvl = 2;

        let mut db = DB::new("db", opt.clone());
        db.vset.borrow_mut().add_version(v);
        db.vset.borrow_mut().next_file_num = 10;

        db.maybe_do_compaction().unwrap();

        assert!(opt.env.exists(Path::new("db/000006.ldb")).unwrap());
        assert!(!opt.env.exists(Path::new("db/000010.ldb")).unwrap());
        assert_eq!(218, opt.env.size_of(Path::new("db/000006.ldb")).unwrap());

        let v = db.current();
        assert_eq!(1, v.borrow().files[2].len());
        assert_eq!(3, v.borrow().files[3].len());
    }

    #[test]
    fn test_db_impl_compaction_state_cleanup() {
        let env: Box<dyn Env> = Box::new(MemEnv::new());
        let name = "db";

        let stuff = "abcdefghijkl".as_bytes();
        env.open_writable_file(Path::new("db/000001.ldb"))
            .unwrap()
            .write_all(stuff)
            .unwrap();
        let mut fmd = FileMetaData::default();
        fmd.num = 1;

        let mut cs = CompactionState::new(Compaction::new(&options::for_test(), 2, None), 12);
        cs.outputs = vec![fmd];
        cs.cleanup(&env, name);

        assert!(!env.exists(Path::new("db/000001.ldb")).unwrap());
    }

    #[test]
    fn test_db_impl_open_close_reopen() {
        let opt;
        {
            let mut db = build_db().0;
            opt = db.opt.clone();
            db.put(b"xx1", b"111").unwrap();
            db.put(b"xx2", b"112").unwrap();
            db.put(b"xx3", b"113").unwrap();
            db.put(b"xx4", b"114").unwrap();
            db.put(b"xx5", b"115").unwrap();
            db.delete(b"xx2").unwrap();
        }

        {
            let mut db = DB::open("db", opt.clone()).unwrap();
            db.delete(b"xx5").unwrap();
        }

        {
            let mut db = DB::open("db", opt.clone()).unwrap();

            assert_eq!(None, db.get(b"xx5"));

            let ss = db.get_snapshot();
            db.put(b"xx4", b"222").unwrap();
            let ss2 = db.get_snapshot();

            assert_eq!(Some(b"113".to_vec()), db.get_at(&ss, b"xx3").unwrap());
            assert_eq!(None, db.get_at(&ss, b"xx2").unwrap());
            assert_eq!(None, db.get_at(&ss, b"xx5").unwrap());

            assert_eq!(Some(b"114".to_vec()), db.get_at(&ss, b"xx4").unwrap());
            assert_eq!(Some(b"222".to_vec()), db.get_at(&ss2, b"xx4").unwrap());
        }

        {
            let mut db = DB::open("db", opt).unwrap();

            let ss = db.get_snapshot();
            assert_eq!(Some(b"113".to_vec()), db.get_at(&ss, b"xx3").unwrap());
            assert_eq!(Some(b"222".to_vec()), db.get_at(&ss, b"xx4").unwrap());
            assert_eq!(None, db.get_at(&ss, b"xx2").unwrap());
        }
    }
}
