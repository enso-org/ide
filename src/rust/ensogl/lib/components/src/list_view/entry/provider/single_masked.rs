use crate::prelude::*;

use crate::list_view::entry;
use crate::list_view::Entry;



// ====================
// === SingleMasked ===
// ====================

/// An Entry Model Provider that wraps a `entry::provider::Any` and allows the masking of a single item.
#[derive(Clone,Debug)]
pub struct SingleMasked<E> {
    content : entry::provider::Any<E>,
    mask    : Cell<Option<entry::Id>>,
}

impl<E:Debug> entry::Provider<E> for SingleMasked<E> {
    fn len(&self) -> usize {
        match self.mask.get() {
            None    => self.content.len(),
            Some(_) => self.content.len().saturating_sub(1),
        }
    }

    fn get(&self, ix:usize) -> Option<E::Model>
    where E : Entry {
        let internal_ix = self.unmasked_index(ix);
        self.content.get(internal_ix)
    }
}

impl<E> SingleMasked<E> {

    /// Return the index to the unmasked underlying data. Will only be valid to use after
    /// calling `clear_mask`.
    ///
    /// Transform index of an element visible in the menu, to the index of the all the objects,
    /// accounting for the removal of the selected item.
    ///
    /// Example:
    /// ```text
    /// Mask              `Some(1)`
    /// Masked indices    [0,     1, 2]
    /// Unmasked Index    [0, 1,  2, 3]
    /// -------------------------------
    /// Mask              `None`
    /// Masked indices    [0, 1, 2, 3]
    /// Unmasked Index    [0, 1, 2, 3]
    /// ```
    pub fn unmasked_index(&self, ix:entry::Id) -> entry::Id {
        match self.mask.get() {
            None                 => ix,
            Some(id) if ix < id  => ix,
            Some(_)              => ix+1,
        }
    }

    /// Mask out the given index. All methods will now skip this item and the `SingleMasked`
    /// will behave as if it was not there.
    ///
    /// *Important:* The index is interpreted according to the _masked_ position of elements.
    pub fn set_mask(&self, ix:entry::Id) {
        let internal_ix = self.unmasked_index(ix);
        self.mask.set(Some(internal_ix));
    }

    /// Mask out the given index. All methods will now skip this item and the `SingleMasked`
    /// will behave as if it was not there.
    ///
    /// *Important:* The index is interpreted according to the _unmasked_ position of elements.
    pub fn set_mask_raw(&self, ix:entry::Id) {
        self.mask.set(Some(ix));
    }

    /// Clear the masked item.
    pub fn clear_mask(&self) {
        self.mask.set(None)
    }
}

impl<E> From<entry::provider::Any<E>> for SingleMasked<E> {
    fn from(content:entry::provider::Any<E>) -> Self {
        let mask = default();
        SingleMasked{content,mask}
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masked_provider() {
        let test_data   = vec!["A", "B", "C", "D"];
        let test_models = test_data.into_iter().map(|label| label.to_owned()).collect_vec();
        let provider    = entry::provider::Any::<Label>::new(test_models);
        let provider:SingleMasked<Label> = provider.into();

        assert_eq!(provider.len(), 4);
        assert_eq!(provider.get(0).unwrap(), "A");
        assert_eq!(provider.get(1).unwrap(), "B");
        assert_eq!(provider.get(2).unwrap(), "C");
        assert_eq!(provider.get(3).unwrap(), "D");

        provider.set_mask_raw(0);
        assert_eq!(provider.len(), 3);
        assert_eq!(provider.get(0).unwrap(), "B");
        assert_eq!(provider.get(1).unwrap(), "C");
        assert_eq!(provider.get(2).unwrap(), "D");

        provider.set_mask_raw(1);
        assert_eq!(provider.len(), 3);
        assert_eq!(provider.get(0).unwrap(), "A");
        assert_eq!(provider.get(1).unwrap(), "C");
        assert_eq!(provider.get(2).unwrap(), "D");

        provider.set_mask_raw(2);
        assert_eq!(provider.len(), 3);
        assert_eq!(provider.get(0).unwrap(), "A");
        assert_eq!(provider.get(1).unwrap(), "B");
        assert_eq!(provider.get(2).unwrap(), "D");

        provider.set_mask_raw(3);
        assert_eq!(provider.len(), 3);
        assert_eq!(provider.get(0).unwrap(), "A");
        assert_eq!(provider.get(1).unwrap(), "B");
        assert_eq!(provider.get(2).unwrap(), "C");
    }
}
