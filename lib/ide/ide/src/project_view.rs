use super::view_layout::ViewLayout;

use basegl::display::world::WorldData;
use basegl::display::world::World;

pub struct ProjectView {
    world  : World,
    layout : ViewLayout
}

impl ProjectView {
    pub fn new() -> ProjectView {
        let world  = WorldData::new("canvas");
        let layout = ViewLayout::new(&world);
        Self {world,layout}
    }

    pub fn forget(self) {
        std::mem::forget(self)
    }
}

