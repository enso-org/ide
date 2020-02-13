//! This module contains ProjectView, the main view, responsible for managing TextEditor and
//! GraphEditor.

use crate::prelude::*;

use super::view_layout::ViewLayout;

use basegl::display::world::WorldData;
use basegl::display::world::World;
use basegl::system::web;
use basegl::control::callback::CallbackHandle;

use nalgebra::Vector2;



// =======================
// === ProjectViewData ===
// =======================

#[derive(Debug)]
struct ProjectViewData {
    world           : World,
    layout          : ViewLayout,
    resize_callback: Option<CallbackHandle>
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
        let world           = WorldData::new(&web::body());
        let layout          = ViewLayout::default(&world);
        let resize_callback = None;
        let data            = ProjectViewData{world,layout,resize_callback};
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
        let resize_callback = scene.camera().add_screen_update_callback(
            move |dimensions:&Vector2<f32>| {
                data.upgrade().map(|data| data.borrow_mut().set_dimensions(*dimensions));
            }
        );
        self.data.borrow_mut().resize_callback = Some(resize_callback);
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

