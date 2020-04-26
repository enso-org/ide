//! A tree structure build on top of the `HashMap`.

use crate::prelude::*;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;


#[derive(Debug,Clone,Copy)]
pub enum AtLeastOneOfTwo<T1,T2> {
    First(T1),
    Second(T2),
    Both(T1,T2)
}

impl<T:PartialEq> AtLeastOneOfTwo<T,T> {
    pub fn same(&self) -> bool {
        match self {
            Self::Both(t1,t2) => t1==t2,
            _ => false
        }
    }
}

impl<T1,T2> AtLeastOneOfTwo<T1,T2> {
    pub fn first(&self) -> Option<&T1> {
        match self {
            Self::Both(t1,t2) => Some(t1),
            Self::First(t1)   => Some(t1),
            _                 => None
        }
    }

    pub fn second(&self) -> Option<&T2> {
        match self {
            Self::Both(t1,t2) => Some(t2),
            Self::Second(t2)  => Some(t2),
            _                 => None
        }
    }
}



// ===================
// === HashMapTree ===
// ===================

/// A tree build on top of the `HashMap`. Each node in the tree can have zero or more branches
/// accessible by the given key type.
#[derive(Derivative)]
#[derivative(Debug   (bound="K:Eq+Hash+Debug , V:Debug    , S:BuildHasher"))]
#[derivative(Default (bound="K:Eq+Hash       , V:Default  , S:BuildHasher+Default"))]
#[derivative(Clone   (bound="K:Clone         , V:Clone    , S:Clone"))]
pub struct HashMapTree<K,V,S=RandomState> {
    /// Value of the current tree node.
    pub value : V,
    /// Branches of the current tree node.
    pub branches : HashMap<K,HashMapTree<K,V,S>,S>
}

impl<K,T,S> HashMapTree<K,T,S>
where K : Eq+Hash,
      S : BuildHasher+Default {
    /// Constructor.
    pub fn new() -> Self where T:Default {
        default()
    }

    /// Constructor with explicit root value.
    pub fn from_value(value:T) -> Self {
        let branches = default();
        Self {value,branches}
    }

    /// Sets the value at position described by `path`. In case a required sub-branch does not
    /// exist, a default instance will be created.
    pub fn set<P,I>(&mut self, path:P, value:T)
    where P:IntoIterator<Item=I>, T:Default, I:Into<K> {
        self.get_node(path).value = value;
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, a default instance will be created. Returns mutable reference to
    /// the target tree node.
    #[inline]
    pub fn get_node<P,I>(&mut self, path:P) -> &mut HashMapTree<K,T,S>
        where P:IntoIterator<Item=I>, T:Default, I:Into<K> {
        self.get_node_with(path,default)
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, uses `cons` to construct it. Returns mutable reference to the
    /// target tree node.
    #[inline]
    pub fn get_node_with<P,I,F>(&mut self, path:P, f:F) -> &mut HashMapTree<K,T,S>
        where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T {
        self.get_node_traversing_with(path,f,|_|{})
    }

    /// Iterates over keys in `path`. For each key, traverses into the appropriate branch. In case
    /// the branch does not exist, uses `cons` to construct it. Moreover, for each traversed branch
    /// the `callback` is evaluated. Returns mutable reference to the target tree node.
    #[inline]
    pub fn get_node_traversing_with<P,I,F,M>
    (&mut self, path:P, mut cons:F, mut callback:M) -> &mut HashMapTree<K,T,S>
    where P:IntoIterator<Item=I>, I:Into<K>, F:FnMut()->T, M:FnMut(&mut HashMapTree<K,T,S>) {
        path.into_iter().fold(self,|map,t| {
            let key  = t.into();
            let node = map.branches.entry(key).or_insert_with(|| HashMapTree::from_value(cons()));
            callback(node);
            node
        })
    }

    pub fn zip_clone<T2>
    (&self, other:&HashMapTree<K,T2,S>) -> HashMapTree<K,AtLeastOneOfTwo<T,T2>,S>
    where K:Clone, T:Clone, T2:Clone {
        Self::zip_clone_branches(Some(self),Some(other))
    }

    fn zip_clone_branches<T2>
    (tree1:Option<&HashMapTree<K,T,S>>, tree2:Option<&HashMapTree<K,T2,S>>)
    -> HashMapTree<K,AtLeastOneOfTwo<T,T2>,S>
    where K:Clone, T:Clone, T2:Clone {
        match (tree1,tree2) {
            (Some(tree1),Some(tree2)) => {
                let value    = AtLeastOneOfTwo::Both(tree1.value.clone(),tree2.value.clone());
                let mut keys = tree1.branches.keys().cloned().collect::<HashSet<K>>();
                keys.extend(tree2.branches.keys().cloned());
                let branches = keys.into_iter().map(|key| {
                    let branch1 = tree1.branches.get(&key);
                    let branch2 = tree2.branches.get(&key);
                    (key,Self::zip_clone_branches(branch1,branch2))
                }).collect();
                HashMapTree {value,branches}
            }

            (Some(tree),None) => {
                let value    = AtLeastOneOfTwo::First(tree.value.clone());
                let mut keys = tree.branches.keys().cloned().collect::<HashSet<K>>();
                keys.extend(tree.branches.keys().cloned());
                let branches = tree.branches.iter().map(|(key,branch)| {
                    (key.clone(),Self::zip_clone_branches(Some(branch),None))
                }).collect();
                HashMapTree {value,branches}
            }

            (None,Some(tree)) => {
                let value    = AtLeastOneOfTwo::Second(tree.value.clone());
                let mut keys = tree.branches.keys().cloned().collect::<HashSet<K>>();
                keys.extend(tree.branches.keys().cloned());
                let branches = tree.branches.iter().map(|(key,branch)| {
                    (key.clone(),Self::zip_clone_branches(None,Some(branch)))
                }).collect();
                HashMapTree {value,branches}
            }
            _ => panic!("Impossible")
        }
    }
}

impl<K,T,S> HashMapTree<K,Option<T>,S>
where K:Eq+Hash {
    /// Gets the current value or creates new default one if missing.
    pub fn value_or_default(&mut self) -> &mut T where T:Default {
        self.value_or_set_with(default)
    }

    /// Gets the current value or creates new one if missing.
    pub fn value_or_set(&mut self, val:T) -> &mut T {
        self.value_or_set_with(move || val)
    }

    /// Gets the current value or creates new one if missing.
    pub fn value_or_set_with<F>(&mut self, cons:F) -> &mut T
    where F:FnOnce()->T {
        if self.value.is_none() {
            self.value = Some(cons());
        };
        self.value.as_mut().unwrap()
    }
}


// === Impls ===

impl<K,V,S> Semigroup for HashMapTree<K,V,S>
    where K : Eq + Hash + Clone,
          V : Semigroup,
          S : BuildHasher + Clone {
    fn concat_mut(&mut self, other:&Self) {
        self.value.concat_mut(&other.value);
        self.branches.concat_mut(&other.branches);
    }
}


// === Iterators ===

macro_rules! define_borrow_iterator {
    ($tp_name:ident $fn_name:ident $($mut:tt)?) => {
        /// Iterator.
        pub struct $tp_name<'a,K,V,S> {
            iters : Vec<std::collections::hash_map::$tp_name<'a,K,HashMapTree<K,V,S>>>,
            path  : Vec<&'a K>,
        }

        impl<'a,K,V,S> Iterator for $tp_name<'a,K,V,S> {
            type Item = (Vec<&'a K>, &'a $($mut)? V);
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match self.iters.pop() {
                        None => break None,
                        Some(mut iter) => {
                            match iter.next() {
                                None => { self.path.pop(); }
                                Some((sub_key,sub_tree)) => {
                                    self.iters.push(iter);
                                    self.iters.push(sub_tree.branches.$fn_name());
                                    self.path.push(sub_key);
                                    break Some((self.path.clone(),& $($mut)? sub_tree.value))
                                }
                            }
                        }
                    }
                }
            }
        }

        impl<'a,K,V,S> IntoIterator for &'a $($mut)? HashMapTree<K,V,S> {
            type Item     = (Vec<&'a K>,&'a $($mut)? V);
            type IntoIter = $tp_name<'a,K,V,S>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                let iters = vec![self.branches.$fn_name()];
                let path  = default();
                $tp_name {iters,path}
            }
        }
    };
}

define_borrow_iterator!(Iter iter);
define_borrow_iterator!(IterMut iter_mut mut);
