// FIXME: Code still needs to be cleaned up.

use crate::prelude::*;

use nalgebra::{Vector3, Vector2, clamp};

use crate::display::rendering::{Camera, CameraType, DOMContainer};

use super::Panning;

// =================
// === Navigator ===
// =================

pub struct Navigator {
    dom         : DOMContainer,
    panning     : Rc<Panning>,
    desired_pos : Vector3<f32>,
    vel         : Vector3<f32>,
}

impl Navigator {
    pub fn new(dom:&DOMContainer, camera:&Camera) -> Self {
        let panning     = Panning::new(dom);
        let dom         = dom.clone();
        let desired_pos = *camera.position();
        let vel         = Vector3::new(0.0, 0.0, 0.0);
        Self { dom, panning, desired_pos, vel }
    }

    fn pan(&mut self, camera:&mut Camera, panning:Vector3<f32>, start:Vector2<f32>) {
        if let CameraType::Perspective(persp) = camera.camera_type() {
            let base_z = self.dom.dimensions().y / 2.0 * camera.get_y_scale();
            let scale  = camera.position().z / base_z;

            let point = start - self.dom.position();
            let normalized_x = point.x / self.dom.dimensions().x;
            let normalized_y = point.y / self.dom.dimensions().y;
            let normalized_x = (normalized_x - 0.5) * 2.0;
            let normalized_y = (normalized_y - 0.5) * 2.0;

            let z = panning.z * 6.0;
            let x = panning.x * scale - normalized_x * z / persp.aspect;
            let y = panning.y * scale + normalized_y * z / camera.get_y_scale();

            self.desired_pos  += Vector3::new(x, y, z);
            if self.desired_pos.z < persp.near + 1.0 {
                self.desired_pos.z = persp.near + 1.0;
            }
        }
    }

    pub fn navigate(&mut self, camera:&mut Camera, dt:f32) {
        if let Some(panning) = self.panning.consume() {
            if panning.begin {
                self.desired_pos = *camera.position();
            }
            self.pan(camera, panning.movement, panning.start);
        }
        self.animate(camera, dt);
    }

    fn animate(&mut self, camera:&mut Camera, dt:f32) {
        let desired_pos   = self.desired_pos;
        let cam_delta     = desired_pos - camera.position();
        let cam_delta_len = cam_delta.magnitude();
        if cam_delta_len > 0.0 {
            let drag          = 10.0;
            let spring_coeff  = 1.5;
            let mass          = 20.0;
            let min_dist      = 0.1;
            let max_dist      = 10.0;
            let max_vel       = 1.0;
            let min_vel       = 0.001;
            let force_val     = cam_delta_len * spring_coeff;
            let force         = cam_delta.normalize() * force_val;
            let acc           = force / mass;
            self.vel += acc;
            let new_vel_val = self.vel.magnitude();
            if new_vel_val < min_vel {
                self.vel = Vector3::new(0.0, 0.0, 0.0);
                *camera.position_mut() = self.desired_pos;
            } else {
                self.vel = self.vel.normalize() * new_vel_val;
                *camera.position_mut() += self.vel;
            }

            if new_vel_val != 0.0 {
                self.vel = self.vel / (1.0 + drag * new_vel_val)
            }
        }
    }
}