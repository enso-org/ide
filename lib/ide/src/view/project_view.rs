//! This module contains ProjectView, the main view, responsible for managing TextEditor and
//! GraphEditor.

use super::view_layout::ViewLayout;

use basegl::display::world::WorldData;
use basegl::display::world::World;
use basegl::system::web::resize_observer::ResizeObserver;

use std::rc::Rc;
use std::cell::RefCell;
use nalgebra::Vector2;



// =======================
// === ProjectViewData ===
// =======================

#[derive(Debug)]
struct ProjectViewData {
    world           : World,
    layout          : ViewLayout,
    resize_observer : Option<ResizeObserver>
}

impl ProjectViewData {
    fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.layout.set_dimensions(dimensions);
    }
}



// ===================
// === ProjectView ===
// ===================

/// ProjectView is the main view of the project, holding instances of TextEditor and GraphEditor.
#[derive(Debug,Clone)]
pub struct ProjectView {
    data : Rc<RefCell<ProjectViewData>>
}

impl Default for ProjectView {
    fn default() -> Self {
        let world           = WorldData::new("canvas");
        let layout          = ViewLayout::default(&world);
        let resize_observer = None;
        let data            = ProjectViewData{world,layout,resize_observer};
        let data            = Rc::new(RefCell::new(data));
        Self {data}.init()
    }
}

impl ProjectView {
    /// Creates a new ProjectView.
    pub fn new() -> Self {
        Self::default()
    }

    fn init(self) -> Self {
        let data            = Rc::downgrade(&self.data);
        let scene           = self.data.borrow().world.scene();
        let resize_observer = scene.add_resize_observer(move |width,height| {
            let dimensions  = Vector2::new(width as f32,height as f32);
            data.upgrade().map(|data| data.borrow_mut().set_dimensions(dimensions));
        });
        self.data.borrow_mut().resize_observer = Some(resize_observer);
        self
    }

    /// Sets dimensions.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.data.borrow_mut().set_dimensions(dimensions);
    }

    /// Forgets ProjectView, so it won't get dropped when it goes out of scope.
    pub fn forget(self) {
        std::mem::forget(self)
    }
}

