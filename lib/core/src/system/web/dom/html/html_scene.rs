//! This module contains `HTMLScene`, a struct to hold `HTMLObject`s.

use crate::prelude::*;

use super::HTMLObject;
use crate::display::object::DisplayObjectData;
use data::opt_vec::*;



// =============
// === Scene ===
// =============

/// A collection for holding 3D `Object`s.
pub struct HTMLScene {
    display_object : DisplayObjectData,
    objects        : OptVec<HTMLObject>
}

impl HTMLScene {
    /// Searches for a HtmlElement identified by id and appends to it.
    pub fn new(logger:Logger) -> Self {
        let display_object = DisplayObjectData::new(logger);
        let objects        = default();
        Self{display_object,objects}
    }

    /// Moves a HTMLObject to the Scene and returns an index to it.
    pub fn add_child(&mut self, object: HTMLObject) -> Ix {
        self.display_object.add_child(&object.display_object);
        self.objects.insert(object)
    }

    /// Removes and retrieves a HTMLObject based on the index provided by
    pub fn remove_child(&mut self, index: Ix) -> Option<HTMLObject> {
        let object = self.objects.remove(index);
        if let Some(object) = &object {
            self.display_object.remove_child(&object.display_object);
        }
        object
    }

    /// Returns the number of `Object`s in the Scene,
    /// also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Returns true if the Scene contains no `Object`s.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Gets mutable iterator.
    pub fn iter_mut(&mut self) -> IterMut<'_, HTMLObject> { self.objects.iter_mut() }

    /// Gets iterator.
    pub fn iter(&self) -> Iter<'_, HTMLObject> { self.objects.iter() }
}

impl<'a> IntoIterator for &'a HTMLScene {
    type Item = &'a HTMLObject;
    type IntoIter = Iter<'a, HTMLObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut HTMLScene {
    type Item = &'a mut HTMLObject;
    type IntoIter = IterMut<'a, HTMLObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
