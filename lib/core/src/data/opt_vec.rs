use crate::prelude::*;

// ==============
// === OptVec ===
// ==============

/// A contiguous growable sparse array type. Similar to `Vec<T>`, but allowing
/// missing values. After a value is removed, it remembers the index for reuse
/// in the future.
#[derive(Clone, Debug, Shrinkwrap)]
pub struct OptVec<T> {
    #[shrinkwrap(main_field)]
    pub items: Vec<Option<T>>,
    pub free_ixs: Vec<usize>,
}

impl<T> OptVec<T> {
    /// Constructs a new, empty `Vec<T>`. It will not allocate until elements
    /// are pushed onto it.
    pub const fn new() -> Self {
        let items = Vec::new();
        let free_ixs = Vec::new();
        Self { items, free_ixs }
    }

    /// Finds a free index and inserts the element. The index is re-used in case
    /// the array is sparse or is added in case of no free places.
    pub fn insert<F: FnOnce(usize) -> T>(&mut self, f: F) -> usize {
        match self.free_ixs.pop() {
            None => {
                let ix = self.items.len();
                self.items.push(Some(f(ix)));
                ix
            }
            Some(ix) => {
                self.items[ix] = Some(f(ix));
                ix
            }
        }
    }

    /// Removes the element at provided index and marks the index to be reused.
    /// Does nothing if the index was already empty. Panics if the index was out
    /// of bounds.
    pub fn remove(&mut self, ix: usize) -> Option<T> {
        let item = self.items[ix].take();
        item.iter().for_each(|_| self.free_ixs.push(ix));
        item
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.items.len() - self.free_ixs.len()
    }

    /// Returns true if vector contains no element.
    pub fn is_empty(&self) -> bool {
        self.items.len() == self.free_ixs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut v = OptVec::new();
        assert!(v.is_empty());

        let ix1 = v.insert(|_| 1);
        assert_eq!(v.len(), 1);
        assert!(!v.is_empty());

        let ix2 = v.insert(|_| 2);
        assert_eq!(v.len(), 2);

        v.remove(ix1);
        assert_eq!(v.len(), 1);

        v.remove(ix2);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());

        let ix3 = v.insert(|_| 3);
        assert_eq!(v.len(), 1);

        let ix4 = v.insert(|_| 4);
        assert_eq!(v.len(), 2);

        assert_eq!(ix1, 0);
        assert_eq!(ix2, 1);
        assert_eq!(ix3, 1);
        assert_eq!(ix4, 0);
    }
}
