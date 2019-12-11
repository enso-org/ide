use crate::prelude::*;

use nalgebra::{Vector3, Vector2, clamp};

use crate::display::rendering::{Camera, DOMContainer};

use super::Zooming;
use super::Zoom;
use super::Panning;

// =================
// === Navigator ===
// =================

pub struct Navigator {
    panning : Rc<Panning>,
    zooming : Rc<Zooming>
}

impl Navigator {
    pub fn new(dom:&DOMContainer) -> Self {
        let panning = Panning::new(dom);
        let zooming = Zooming::new(dom);
        Self { panning, zooming }
    }

    fn pan(&self, camera:&mut Camera, panning:Vector2<f32>) {
        let scale = camera.transform().scale();
        let x = panning.x * scale.x;
        let y = panning.y * scale.y;
        *camera.transform_mut().translation_mut() += Vector3::new(x, y, 0.0)
    }

    fn zoom(&self, camera:&mut Camera, zooming:Zoom) {
        self.pan(camera, zooming.panning);

        let scale = camera.transform_mut().scale_mut();
        *scale *= zooming.amount;
    }

    pub fn navigate(&self, camera:&mut Camera) {
        if let Some(panning) = self.panning.consume() {
            self.pan(camera, panning);
        }

        if let Some(zooming) = self.zooming.consume(0.01) {
            self.zoom(camera, zooming);
        }
    }
}