// ===========
// === Add ===
// ===========

/// An abstraction for container which can be provided with new elements. The
/// element type is polymorphic, allowing the container to reuse the function
/// for different item types.
pub trait Add<T> {
    type Result = ();
    fn add(&mut self, component:T) -> Self::Result;
}

pub type AddResult<T,S> = <T as Add<S>>::Result;

// =======================
// === CachingIterator ===
// =======================

/// Iterator wrapper caching the last retrieved value
///
/// The item type is `(Option<T>, T)` where the second tuple element is
/// a current value and first element is a previous one `None` on the first
/// iteration.
pub struct CachingIterator<T:Clone, It:Iterator<Item=T>> {
    last : Option<T>,
    iter : It
}

impl<T:Clone, It:Iterator<Item=T>> Iterator for CachingIterator<T, It> {
    type Item = (Option<T>, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| {
            let new_last = Some(value.clone());
            let old_last = std::mem::replace(&mut self.last, new_last);
            (old_last, value)
        })
    }
}

/// A trait for wrapping in caching iterator
///
/// It is implemented for each iterator over cloneable items.
pub trait IntoCachingIterator {
    type Item : Clone;
    type Iter : Iterator<Item = Self::Item>;

    fn cache_last_value(self) -> CachingIterator<Self::Item, Self::Iter>;
}

impl<T : Clone, It : Iterator<Item=T>> IntoCachingIterator for It {
    type Item = T;
    type Iter = Self;

    fn cache_last_value(self) -> CachingIterator<Self::Item, Self::Iter> {
        CachingIterator { last : None, iter : self }
    }
}

#[cfg(test)]
mod tests {
    
}