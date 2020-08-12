//! Select List Component
use crate::prelude::*;

use ensogl::display;

pub struct Item {
    pub icon  : display::object::Instance,
    pub label : String,
}

pub trait ItemProvider {
    fn entries_count(&self) -> usize;

    fn get(&self, index:usize) -> Item;
}


struct Model {
    item_provider : Box<dyn ItemProvider>,
    displayed_items : Vec<Item>,
}