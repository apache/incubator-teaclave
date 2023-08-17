use policy_core::{
    error::PolicyCarryingResult,
    expr::{AExpr, Node},
};

/// An arena for allocating arbitrary elements but does no drop itself unless all element are dropped.
///
/// Directly taken from polars.
#[derive(Clone)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AExprIter<'a> {
    stack: Vec<Node>,
    arena: Option<&'a Arena<AExpr>>,
}

impl<'a> Iterator for AExprIter<'a> {
    type Item = (Node, &'a AExpr);

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            // take the arena because the bchk doesn't allow a mutable borrow to the field.
            let arena = self.arena.unwrap();
            let current_expr = arena.get(node);
            current_expr.nodes(&mut self.stack);

            self.arena = Some(arena);
            (node, current_expr)
        })
    }
}

/// Simple Arena implementation
/// Allocates memory and stores item in a Vec. Only deallocates when being dropped itself.
impl<T> Arena<T> {
    pub fn add(&mut self, val: T) -> Node {
        let idx = self.items.len();
        self.items.push(val);
        Node(idx)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn new() -> Self {
        Arena { items: vec![] }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Arena {
            items: Vec::with_capacity(cap),
        }
    }

    pub fn swap(&mut self, idx_a: Node, idx_b: Node) {
        self.items.swap(idx_a.0, idx_b.0)
    }

    #[inline]
    pub fn get(&self, idx: Node) -> &T {
        self.items.get(idx.0).unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, idx: Node) -> &mut T {
        self.items.get_mut(idx.0).unwrap()
    }

    #[inline]
    pub fn replace(&mut self, idx: Node, val: T) {
        let x = self.get_mut(idx);
        *x = val;
    }
    pub fn clear(&mut self) {
        self.items.clear()
    }
}

impl<T: Default> Arena<T> {
    #[inline]
    pub fn take(&mut self, idx: Node) -> T {
        std::mem::take(self.get_mut(idx))
    }

    pub fn replace_with<F>(&mut self, idx: Node, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let val = self.take(idx);
        self.replace(idx, f(val));
    }

    pub fn try_replace_with<F>(&mut self, idx: Node, mut f: F) -> PolicyCarryingResult<()>
    where
        F: FnMut(T) -> PolicyCarryingResult<T>,
    {
        let val = self.take(idx);
        self.replace(idx, f(val)?);
        Ok(())
    }
}

impl<'a> Arena<AExpr> {
    pub fn iter(&'a self, root: Node) -> AExprIter<'a> {
        let mut stack = Vec::with_capacity(4);
        stack.push(root);
        AExprIter {
            stack,
            arena: Some(self),
        }
    }
}
