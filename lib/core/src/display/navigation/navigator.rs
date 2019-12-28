use super::camera_manager::CameraManager;
use super::camera_manager::Event;
use super::camera_manager::ZoomingEvent;
use super::camera_manager::PanningEvent;
use crate::system::web::Result;
use crate::display::render::css3d::Camera;
use crate::display::render::css3d::CameraType;
use crate::display::render::css3d::DOMContainer;
use crate::traits::HasPosition;

use nalgebra::Vector3;
use nalgebra::Vector2;

// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions on the specified DOM.
pub struct Navigator {
    dom        : DOMContainer,
    events     : CameraManager,
    position   : Vector3<f32>,
    zoom_speed : f32
}

impl Navigator {
    pub fn new(dom:&DOMContainer, position:Vector3<f32>, zoom_speed:f32) -> Result<Self> {
        let events     = CameraManager::new(dom)?;
        let dom        = dom.clone();

        Ok(Self { dom, events, position, zoom_speed })
    }

    /// Navigates the camera and returns it's position.
    pub fn navigate(&mut self, camera:&mut Camera) -> Vector3<f32> {
        while let Some(event) = self.events.poll() {
            match event {
                Event::Zooming(zoom) => self.zoom(camera, zoom),
                Event::Panning(pan)   => self.pan(camera, pan)
            }
        }
        self.position
    }

    fn zoom(&mut self, camera:&mut Camera, zoom: ZoomingEvent) {
        if let CameraType::Perspective(persp) = camera.camera_type() {
            let point      = zoom.focus;
            let normalized = normalize_point2(point, self.dom.dimensions());
            let normalized = normalized_to_range2(normalized, -1.0, 1.0);

            // Scale X and Y to compensate aspect and fov.
            let x = -normalized.x * persp.aspect;
            let y =  normalized.y;
            let z = camera.get_y_scale();
            let direction = Vector3::new(x, y, z).normalize();

            self.position += direction * zoom.amount * self.zoom_speed;
            if  self.position.z < persp.near + 1.0 {
                self.position.z = persp.near + 1.0;
            }
        }
    }

    fn pan(&mut self, camera:&mut Camera, pan: PanningEvent) {
        if let CameraType::Perspective(_) = camera.camera_type() {
            let base_z = self.dom.dimensions().y / 2.0 * camera.get_y_scale();
            let scale  = camera.position().z / base_z;

            let x = pan.movement.x * scale;
            let y = pan.movement.y * scale;
            let z = 0.0;

            self.position += Vector3::new(x, y, z);
        }
    }
}

// =============
// === Utils ===
// =============

/// Normalize a `point` in (0..dimension.x, 0..dimension.y) to (0..1, 0..1).
fn normalize_point2
(point:Vector2<f32>, dimension:Vector2<f32>) -> Vector2<f32> {
    Vector2::new(point.x / dimension.x, point.y / dimension.y)
}

/// Transforms a `point` normalized in (0..1, 0..1) to (a..b,a..b).
fn normalized_to_range2(point:Vector2<f32>, a:f32, b:f32) -> Vector2<f32> {
    let width = b - a;
    Vector2::new(point.x * width + a, point.y * width + a)
}
