// FIXME: This enum design is super dirty, I will clean it up.

use crate::display::rendering::CameraPerspective;
use crate::display::rendering::CameraOrthographic;
use crate::display::rendering::Transform;

use nalgebra::base::Matrix4;
use nalgebra::{Vector3, UnitQuaternion};

// ==============
// === Camera ===
// ==============

pub enum Camera {
    Perspective(CameraPerspective),
    Orthographic(CameraOrthographic)
}

impl Camera {
    pub fn get_y_scale(&self) -> f32 {
        match self {
            Camera::Perspective (camera) => camera.projection.m11,
            Camera::Orthographic(camera) => camera.projection.m11
        }
    }

    pub fn projection(&self) -> &Matrix4<f32> {
        match self {
            Camera::Perspective (camera) => &camera.projection,
            Camera::Orthographic(camera) => &camera.projection
        }
    }

    pub fn transform(&self) -> &Transform {
        match self {
            Camera::Perspective (camera) => &camera.transform,
            Camera::Orthographic(camera) => &camera.transform
        }
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            Camera::Perspective (camera) => &mut camera.transform,
            Camera::Orthographic(camera) => &mut camera.transform
        }
    }

    pub fn perspective(fov:f32, aspect:f32, z_near:f32, z_far:f32) -> Self {
        Camera::Perspective(CameraPerspective::new(fov, aspect, z_near, z_far))
    }

    pub fn orthographic(left   : f32,
                        right  : f32,
                        bottom : f32,
                        top    : f32,
                        near   : f32,
                        far    : f32) -> Self {
        Camera::Orthographic(
            CameraOrthographic::new(left, right, bottom, top, near, far)
        )
    }

    pub fn position_mut(&mut self) -> &mut Vector3<f32> {
        match self {
            Camera::Perspective (camera) => camera.transform.translation_mut(),
            Camera::Orthographic(camera) => camera.transform.translation_mut()
        }
    }

    pub fn rotation_mut(&mut self) -> &mut UnitQuaternion<f32> {
        match self {
            Camera::Perspective (camera) => camera.transform.rotation_mut(),
            Camera::Orthographic(camera) => camera.transform.rotation_mut()
        }
    }

    pub fn set_rotation(&mut self, roll:f32, pitch:f32, yaw:f32) {
        match self {
            Camera::Perspective (camera) => camera.set_rotation(roll, pitch, yaw),
            Camera::Orthographic(camera) => camera.set_rotation(roll, pitch, yaw)
        }
    }
}