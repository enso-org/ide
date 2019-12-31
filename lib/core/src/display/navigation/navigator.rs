use super::navigator_events::NavigatorEvents;
use super::navigator_events::ZoomEvent;
use super::navigator_events::PanEvent;
use crate::system::web::Result;
use crate::display::render::css3d::Camera;
use crate::display::render::css3d::CameraType;
use crate::display::render::css3d::DOMContainer;
use crate::traits::HasPosition;
use crate::animation::physics::{PhysicsSimulator, SpringProperties, DragProperties, PhysicsProperties, KinematicsProperties};

use nalgebra::{Vector3, zero};
use nalgebra::Vector2;

// =================
// === Navigator ===
// =================

/// Navigator enables camera navigation with mouse interactions on the specified DOM.
pub struct Navigator {
    _events    : NavigatorEvents,
    _simulator : PhysicsSimulator,
}

impl Navigator {
    pub fn new(dom:&DOMContainer, camera:Camera, zoom_speed:f32) -> Result<Self> {
        let mass           = 25.0;
        let velocity       = zero();
        let kinematics     = KinematicsProperties::new(camera.position(), velocity, zero(), mass);
        let coefficient    = 10000.0;
        let fixed_point    = camera.position();
        let spring         = SpringProperties::new(coefficient, fixed_point);
        let drag           = DragProperties::new(1000.0);
        let mut properties = PhysicsProperties::new(kinematics, spring, drag);
        let _simulator     = PhysicsSimulator::new(camera.object.clone(), properties.clone());

        let dom_clone            = dom.clone();
        let camera_clone         = camera.clone();
        let mut properties_clone = properties.clone();
        let panning_callback     = move |pan: PanEvent| {
            let base_z = dom_clone.dimensions().y / 2.0 * camera_clone.get_y_scale();
            let scale  = camera_clone.position().z / base_z;

            let x = pan.movement.x * scale;
            let y = pan.movement.y * scale;
            let z = 0.0;

            properties_clone.mod_spring(|spring| {
                spring.set_fixed_point(spring.fixed_point() + Vector3::new(x, y, z));
            });
        };

        let dom_clone = dom.clone();
        let zoom_callback = move |zoom:ZoomEvent| {
            if let CameraType::Perspective(persp) = camera.camera_type() {
                let point      = zoom.focus;
                let normalized = normalize_point2(point, dom_clone.dimensions());
                let normalized = normalized_to_range2(normalized, -1.0, 1.0);

                // Scale X and Y to compensate aspect and fov.
                let x         = -normalized.x * persp.aspect;
                let y         =  normalized.y;
                let z         = camera.get_y_scale();
                let direction = Vector3::new(x, y, z).normalize();

                let mut position = properties.spring().fixed_point();
                position  += direction * zoom.amount;
                position.z = position.z.max(persp.near + 1.0);

                properties.mod_spring(|spring| spring.set_fixed_point(position));
            }
        };
        let _events  = NavigatorEvents::new(dom, panning_callback, zoom_callback, zoom_speed)?;

        Ok(Self { _events, _simulator })
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
