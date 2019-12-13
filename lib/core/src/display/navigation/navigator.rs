use crate::prelude::*;

use super::EventHandler;
use super::Event;
use super::ZoomEvent;
use super::PanEvent;
use crate::system::web::EventListeningResult as Result;
use crate::display::rendering::Camera;
use crate::display::rendering::CameraType;
use crate::display::rendering::DOMContainer;

use nalgebra::Vector3;

// =================
// === Animation ===
// =================

/// Struct used to store animation related properties.
struct Animation {
    desired_pos  : Vector3<f32>,
    vel          : Vector3<f32>,
    desired_rate : f32,
    time_count   : f32,
    drag         : f32,
    spring_coeff : f32,
    mass         : f32,
    min_vel      : f32
}

// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions on the specified
/// DOM.
pub struct Navigator {
    dom    : DOMContainer,
    events : Rc<EventHandler>,
    anim   : Animation
}

impl Navigator {
    pub fn new(dom:&DOMContainer, camera:&Camera) -> Result<Self> {
        let events        = EventHandler::new(dom)?;
        let dom           = dom.clone();

        let desired_pos   = *camera.position();
        let vel           = Vector3::new(0.0, 0.0, 0.0);
        let time_count    = 0.0;
        let desired_rate  = 120.0; // 120fps
        let drag          = 10.0;
        let spring_coeff  = 1.5;
        let mass          = 20.0;
        let min_vel       = 0.001;

        let animation = Animation {
            desired_pos,
            vel,
            time_count,
            desired_rate,
            drag,
            spring_coeff,
            mass,
            min_vel
        };

        Ok(Self { dom, events, anim: animation })
    }

    /// Polls for mouse events and navigates the camera.
    pub fn navigate(&mut self, camera:&mut Camera, delta_seconds:f32) {
        match self.events.poll() {
            Event::Start      => self.anim.desired_pos = *camera.position(),
            Event::Zoom(zoom) => self.zoom(camera, zoom),
            Event::Pan(pan)   => self.pan(camera, pan),
            Event::None       => ()
        }
        self.animate(camera, delta_seconds);
    }

    /// For zooming into a desired focused position.
    fn zoom(&mut self, camera:&mut Camera, zoom:ZoomEvent) {
        if let CameraType::Perspective(persp) = camera.camera_type() {
            let point        = zoom.focus - self.dom.position();
            // 0..1 normalization
            let normalized_x = point.x / self.dom.dimensions().x;
            let normalized_y = point.y / self.dom.dimensions().y;
            // -1..1 normalization
            let normalized_x = (normalized_x - 0.5) * 2.0;
            let normalized_y = (normalized_y - 0.5) * 2.0;

            let z =  zoom.amount * 6.0;
            let x = -normalized_x * z / persp.aspect;
            let y =  normalized_y * z / camera.get_y_scale();

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
        self.anim.time_count += dt;
        let frame_time = 1.0 / self.anim.desired_rate;
        while self.anim.time_count > frame_time {
            self.anim.time_count  -= frame_time;
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
            let force_val     = cam_delta_len * animation.spring_coeff;
            let force         = cam_delta.normalize() * force_val;
            let acc           = force / animation.mass;
            animation.vel    += acc;
            let new_vel_val   = animation.vel.magnitude();

            if  new_vel_val   < animation.min_vel {
                animation.vel          = Vector3::new(0.0, 0.0, 0.0);
                *camera.position_mut() = animation.desired_pos;
            } else {
                animation.vel = animation.vel.normalize() * new_vel_val;
                let position  = camera.position_mut();
                *position    += animation.vel;
            }

            if new_vel_val != 0.0 {
                let vel       = animation.vel;
                let vel       = vel / (1.0 + animation.drag * new_vel_val);
                animation.vel = vel;
            }
        }
    }
}