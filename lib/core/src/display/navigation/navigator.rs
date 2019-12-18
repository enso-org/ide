use super::EventHandler;
use super::Event;
use super::ZoomEvent;
use super::PanEvent;
use crate::system::web::Result;
use crate::display::rendering::Camera;
use crate::display::rendering::CameraType;
use crate::display::rendering::DOMContainer;

use nalgebra::{Vector3, zero};
use nalgebra::Vector2;

// =================
// === Animation ===
// =================

/// Struct used to store animation related properties.
struct Animation {
    desired_pos      : Vector3<f32>, // position
    velocity         : Vector3<f32>, // velocity
    min_vel          : f32, // velocity
    desired_fps      : f32, // animation manager
    accumulated_time : f32, // animation manager
    drag             : f32, // physics property
    spring_coeff     : f32, // physics property
    mass             : f32, // physics property
}

impl Animation {
    pub fn new(desired_pos:Vector3<f32>) -> Self {
        let velocity         = zero();
        let accumulated_time = zero();
        let desired_fps      = 120.0;
        let drag             = 10.0;
        let spring_coeff     = 1.5;
        let mass             = 20.0;
        let min_vel          = 0.001;

        Self {
            desired_pos,
            velocity,
            desired_fps,
            accumulated_time,
            drag,
            spring_coeff,
            mass,
            min_vel
        }
    }
}

// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions on the specified
/// DOM.
pub struct Navigator {
    dom    : DOMContainer,
    events : EventHandler,
    anim   : Animation
}

impl Navigator {
    pub fn new(dom:&DOMContainer, camera:&Camera) -> Result<Self> {
        let events    = EventHandler::new(dom).unwrap();
        let dom       = dom.clone();
        let animation = Animation::new(*camera.position());

        Ok(Self { dom, events, anim: animation })
    }

    /// Polls for mouse events and navigates the camera.
    pub fn navigate(&mut self, camera:&mut Camera, delta_seconds:f32) {
        while let Some(event) = self.events.poll() {
            match event {
                Event::Zoom(zoom) => self.zoom(camera, zoom),
                Event::Pan(pan)   => self.pan(camera, pan)
            }
        }
        self.animate(camera, delta_seconds);
    }

    /// For zooming into a desired focused position.
    fn zoom(&mut self, camera:&mut Camera, zoom:ZoomEvent) {
        if let CameraType::Perspective(persp) = camera.camera_type() {
            let point      = zoom.focus;
            let normalized = normalize_point2(point, self.dom.dimensions());
            let normalized = normalized_to_range2(normalized, -1.0, 1.0);

            let z =  zoom.amount * 6.0;
            let x = -normalized.x * z / persp.aspect;
            let y =  normalized.y * z / camera.get_y_scale();

            let animation = &mut self.anim;
            animation.desired_pos += Vector3::new(x, y, z);
            if  animation.desired_pos.z < persp.near + 1.0 {
                animation.desired_pos.z = persp.near + 1.0;
            }
        }
    }

    /// To pan to an offset.
    fn pan(&mut self, camera:&mut Camera, pan:PanEvent) {
        if let CameraType::Perspective(_) = camera.camera_type() {
            let base_z = self.dom.dimensions().y / 2.0 * camera.get_y_scale();
            let scale  = camera.position().z / base_z;

            let x = pan.movement.x * scale;
            let y = pan.movement.y * scale;
            let z = 0.0;

            self.anim.desired_pos += Vector3::new(x, y, z);
        }
    }

    /// Animates the camera in a fixed frame rate.
    fn animate(&mut self, camera:&mut Camera, dt:f32) {
        self.anim.accumulated_time += dt;
        let frame_time = 1.0 / self.anim.desired_fps;
        while self.anim.accumulated_time > frame_time {
            self.anim.accumulated_time -= frame_time;
            self.tick(camera);
        }
    }

    /// Runs camera animation.
    fn tick(&mut self, camera:&mut Camera) {
        let animation     = &mut self.anim;
        let desired_pos   = animation.desired_pos;
        let cam_delta     = desired_pos - camera.position();
        let cam_delta_len = cam_delta.magnitude();

        if cam_delta_len > 0.0 {
            let force_val       = cam_delta_len * animation.spring_coeff;
            let force           = cam_delta.normalize() * force_val;
            let acc             = force / animation.mass;
            animation.velocity += acc;
            let new_vel_val     = animation.velocity.magnitude();

            if new_vel_val < animation.min_vel {
                animation.velocity = zero();
                *camera.position_mut() = animation.desired_pos;
            } else {
                animation.velocity = animation.velocity.normalize() * new_vel_val;
                let position  = camera.position_mut();
                *position    += animation.velocity;
            }

            animation.velocity /= 1.0 + animation.drag + new_vel_val;
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
