use std::collections::HashMap;
use types::{share, SequenceNumber, Shared, MAX_SEQUENCE_NUMBER};

use std::rc::Rc;

/// Opaque snapshot handle; Represents index to SnapshotList.map
type SnapshotHandle = u64;

/// An InnerSnapshot is shared by several Snapshots. This enables cloning snapshots, and a snapshot
/// is released once the last instance is dropped.
#[derive(Clone)]
struct InnerSnapshot {
    id: SnapshotHandle,
    seq: SequenceNumber,
    sl: Shared<InnerSnapshotList>,
}

impl Drop for InnerSnapshot {
    fn drop(&mut self) {
        self.sl.borrow_mut().delete(self.id);
    }
}

#[derive(Clone)]
pub struct Snapshot {
    inner: Rc<InnerSnapshot>,
}

impl Snapshot {
    pub fn sequence(&self) -> SequenceNumber {
        (*self.inner).seq
    }
}

/// A list of all snapshots is kept in the DB.
struct InnerSnapshotList {
    map: HashMap<SnapshotHandle, SequenceNumber>,
    newest: SnapshotHandle,
    oldest: SnapshotHandle,
}

pub struct SnapshotList {
    inner: Shared<InnerSnapshotList>,
}

impl SnapshotList {
    pub fn new() -> SnapshotList {
        SnapshotList {
            inner: share(InnerSnapshotList {
                map: HashMap::new(),
                newest: 0,
                oldest: 0,
            }),
        }
    }

    pub fn new_snapshot(&mut self, seq: SequenceNumber) -> Snapshot {
        let inner = self.inner.clone();
        let mut sl = self.inner.borrow_mut();

        sl.newest += 1;
        let newest = sl.newest;
        sl.map.insert(newest, seq);

        if sl.oldest == 0 {
            sl.oldest = sl.newest;
        }

        Snapshot {
            inner: Rc::new(InnerSnapshot {
                id: sl.newest,
                seq: seq,
                sl: inner,
            }),
        }
    }

    /// oldest returns the lowest sequence number of all snapshots. It returns 0 if no snapshots
    /// are present.
    pub fn oldest(&self) -> SequenceNumber {
        let oldest = self
            .inner
            .borrow()
            .map
            .iter()
            .fold(
                MAX_SEQUENCE_NUMBER,
                |s, (seq, _)| if *seq < s { *seq } else { s },
            );
        if oldest == MAX_SEQUENCE_NUMBER {
            0
        } else {
            oldest
        }
    }

    /// newest returns the newest sequence number of all snapshots. If no snapshots are present, it
    /// returns 0.
    pub fn newest(&self) -> SequenceNumber {
        self.inner
            .borrow()
            .map
            .iter()
            .fold(0, |s, (seq, _)| if *seq > s { *seq } else { s })
    }

    pub fn empty(&self) -> bool {
        self.inner.borrow().oldest == 0
    }
}

impl InnerSnapshotList {
    fn delete(&mut self, id: SnapshotHandle) {
        self.map.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused_variables)]
    #[test]
    fn test_snapshot_list() {
        let mut l = SnapshotList::new();

        {
            assert!(l.empty());
            let a = l.new_snapshot(1);

            {
                let b = l.new_snapshot(2);

                {
                    let c = l.new_snapshot(3);

                    assert!(!l.empty());
                    assert_eq!(l.oldest(), 1);
                    assert_eq!(l.newest(), 3);
                }

                assert_eq!(l.newest(), 2);
                assert_eq!(l.oldest(), 1);
            }

            assert_eq!(l.oldest(), 1);
        }
        assert_eq!(l.oldest(), 0);
    }
}
