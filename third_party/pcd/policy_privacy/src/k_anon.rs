pub struct KAnonManager {
    k: usize,
}

impl KAnonManager {
    #[inline]
    pub fn new(k: usize) -> Self {
        Self { k }
    }

    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }
}
