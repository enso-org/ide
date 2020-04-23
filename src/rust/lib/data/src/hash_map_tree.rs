/// A tree structure build on top of the `HashMap`.

use crate::prelude::*;

// ===================
// === HashMapTree ===
// ===================

#[derive(Derivative)]
#[derivative(Debug   (bound="K:Eq+Hash+Debug , T:Debug"))]
#[derivative(Default (bound="K:Eq+Hash       , T:Default"))]
pub struct HashMapTree<K,T> {
    pub value    : T,
    pub branches : HashMap<K,HashMapTree<K,T>>
}

impl<K,T> HashMapTree<K,T>
where K:Eq+Hash {
    /// Constructor.
    pub fn new() -> Self where T:Default {
        default()
    }

    /// Constructor with explicit root value.
    pub fn from_value(value:T) -> Self {
        let branches = default();
        Self {value,branches}
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, a default instance will be created.
    #[inline]
    pub fn focus<P,I>(&mut self, path:P) -> &mut HashMapTree<K,T>
        where P:IntoIterator<Item=I>, T:Default, I:Into<K> {
        self.focus_with(path,default)
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, uses `cons` to construct it.
    #[inline]
    pub fn focus_with<P,I,F>(&mut self, path:P, mut f:F) -> &mut HashMapTree<K,T>
        where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T {
        self.focus_map_with(path,f,|_|{})
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, uses `cons` to construct it. Moreover, for each traversed branch
    /// the `callback` is evaluated.
    #[inline]
    pub fn focus_map_with<P,I,F,M>
    (&mut self, path:P, mut cons:F, mut callback:M) -> &mut HashMapTree<K,T>
    where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T, M:FnMut(&mut HashMapTree<K,T>) {
        path.into_iter().fold(self,|map,t| {
            let key  = t.into();
            let node = map.branches.entry(key).or_insert_with(|| HashMapTree::from_value(cons()));
            callback(node);
            node
        })
    }
}


// === Impls ===

impl<'a,K,T> IntoIterator for &'a HashMapTree<K,T> {
    type Item     = (&'a K, &'a HashMapTree<K,T>);
    type IntoIter = std::collections::hash_map::Iter<'a,K,HashMapTree<K,T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.branches).into_iter()
    }
}

impl<'a,K,T> IntoIterator for &'a mut HashMapTree<K,T> {
    type Item     = (&'a K, &'a mut HashMapTree<K,T>);
    type IntoIter = std::collections::hash_map::IterMut<'a,K,HashMapTree<K,T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.branches).into_iter()
    }
}
