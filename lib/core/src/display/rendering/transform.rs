use nalgebra::{Matrix4, Quaternion, UnitQuaternion, Vector3};

// Note [Quaternion YXZ Euler angles]
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// To comply with Threejs impl, we generate a Quaternion applying rotation in
// the order: pitch -> roll -> yaw, instead of roll -> pitch -> yaw based on
// https://github.com/mrdoob/three.js/blob/master/src/math/Quaternion.js#L199
fn from_euler_angles_pry(roll: f32, pitch: f32, yaw: f32) -> UnitQuaternion<f32> {
    let (s1, c1): (f32, f32) = (roll * 0.5 as f32).sin_cos();
    let (s2, c2): (f32, f32) = (pitch * 0.5 as f32).sin_cos();
    let (s3, c3): (f32, f32) = (yaw * 0.5 as f32).sin_cos();

    UnitQuaternion::from_quaternion(Quaternion::new(
        c1 * c2 * c3 - s1 * s2 * s3,
        s1 * c2 * c3 + c1 * s2 * s3,
        c1 * s2 * c3 - s1 * c2 * s3,
        c1 * c2 * s3 + s1 * s2 * c3,
    ))
}

/// A structure representing 3D Position, Rotation and Scale
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation:    UnitQuaternion<f32>,
    pub scale:       Vector3<f32>,
}

impl Transform {
    /// Creates an identity transform
    pub fn identity() -> Self {
        Self {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation:    UnitQuaternion::identity(),
            scale:       Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Sets Transform's translation
    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = Vector3::new(x, y, z);
    }

    /// Set Transform's scale
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = Vector3::new(x, y, z);
    }

    /// Set Transform's rotation from Euler angles in radians
    pub fn set_rotation(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.rotation = from_euler_angles_pry(roll, pitch, yaw);
    }

    /// Gets a homogeneous transform Matrix4. The rotation order is YXZ (pitch,
    /// roll, yaw)
    // Note [Transform to Matrix4 composition]
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // based on https://github.com/mrdoob/three.js/blob/master/src/math/Matrix4.js#L732
    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        let (x, y, z, w) = (
            self.rotation.coords.x,
            self.rotation.coords.y,
            self.rotation.coords.z,
            self.rotation.coords.w,
        );
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);

        let (sx, sy, sz) = (self.scale.x, self.scale.y, self.scale.z);

        let m00 = (1.0 - (yy + zz)) * sx;
        let m10 = (xy + wz) * sx;
        let m20 = (xz - wy) * sx;
        let m30 = 0.0;

        let m01 = (xy - wz) * sy;
        let m11 = (1.0 - (xx + zz)) * sy;
        let m21 = (yz + wx) * sy;
        let m31 = 0.0;

        let m02 = (xz + wy) * sz;
        let m12 = (yz - wx) * sz;
        let m22 = (1.0 - (xx + yy)) * sz;
        let m32 = 0.0;

        let (m03, m13, m23) = (self.translation.x, self.translation.y, self.translation.z);
        let m33 = 1.0;
        Matrix4::new(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33)
    }
}
