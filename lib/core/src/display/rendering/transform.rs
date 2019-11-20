use nalgebra::Matrix4;
use nalgebra::Quaternion;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use nalgebra::RealField;

// To comply with Threejs impl, we generate a Quaternion applying rotation in
// the order: pitch -> roll -> yaw, instead of roll -> pitch -> yaw based on
// https://github.com/mrdoob/three.js/blob/master/src/math/Quaternion.js#L199
fn from_euler_angles_pry
(roll : f32, pitch : f32, yaw : f32) -> UnitQuaternion<f32> {
    let (s1, c1) : (f32, f32) = (roll  * 0.5 as f32).sin_cos();
    let (s2, c2) : (f32, f32) = (pitch * 0.5 as f32).sin_cos();
    let (s3, c3) : (f32, f32) = (yaw   * 0.5 as f32).sin_cos();

    UnitQuaternion::from_quaternion(Quaternion::new(
        c1 * c2 * c3 - s1 * s2 * s3,
        s1 * c2 * c3 + c1 * s2 * s3,
        c1 * s2 * c3 - s1 * c2 * s3,
        c1 * c2 * s3 + s1 * s2 * c3,
    ))
}

// ===================
// === Transform ===
// ===================

/// A structure representing 3D Position, Rotation and Scale.
pub struct Transform {
    pub translation : Vector3<f32>,
    pub rotation    : UnitQuaternion<f32>,
    pub scale       : Vector3<f32>,
}

impl Transform {
    /// Creates an identity transform.
    pub fn identity() -> Self {
        let translation = Vector3::new(0.0, 0.0, 0.0);
        let rotation    = UnitQuaternion::identity();
        let scale       = Vector3::new(1.0, 1.0, 1.0);
        Self { translation, rotation, scale }
    }

    /// Sets Transform's translation.
    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = Vector3::new(x, y, z);
    }

    /// Set Transform's scale.
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = Vector3::new(x, y, z);
    }

    /// Set Transform's rotation from Euler angles in radians.
    pub fn set_rotation(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.rotation = from_euler_angles_pry(roll, pitch, yaw);
    }

    /// Gets a homogeneous transform Matrix4. The rotation order is YXZ (pitch,
    /// roll, yaw).
    // Note [Transform to Matrix4 composition]
    // =======================================
    // based on
    // https://github.com/mrdoob/three.js/blob/master/src/math/Matrix4.js#L732
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
        let m01 = (xy + wz) * sx;
        let m02 = (xz - wy) * sx;
        let m03 = 0.0;

        let m10 = (xy - wz) * sy;
        let m11 = (1.0 - (xx + zz)) * sy;
        let m12 = (yz + wx) * sy;
        let m13 = 0.0;

        let m20 = (xz + wy) * sz;
        let m21 = (yz - wx) * sz;
        let m22 = (1.0 - (xx + yy)) * sz;
        let m23 = 0.0;

        let (m30, m31, m32) = (self.translation.x,
                               self.translation.y,
                               self.translation.z);
        let m33 = 1.0;
        Matrix4::new(m00, m10, m20, m30,
                     m01, m11, m21, m31,
                     m02, m12, m22, m32,
                     m03, m13, m23, m33)
    }
}

// ======================
// === Matrix Printer ===
// ======================
pub trait IntoCSSMatrix {
    fn into_css_matrix(&self) -> String;
}

impl<T : RealField> IntoCSSMatrix for Matrix4<T> {
    fn into_css_matrix(&self) -> String {
        let mut iter = self.iter();
        let item = iter.next().expect("Matrix4 should have the first item");
        let acc = format!("{}", item);
        let acc = iter.fold(acc, |acc, item| format!("{}, {}", acc, item));
        format!("matrix3d({})", acc)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn into_css_matrix() {
        use nalgebra::Matrix4;
        use super::IntoCSSMatrix;

        let matrix = Matrix4::new( 1.0, 5.0,  9.0, 13.0,
                                   2.0, 6.0, 10.0, 14.0,
                                   3.0, 7.0, 11.0, 15.0,
                                   4.0, 8.0, 12.0, 16.0);
        let column_major = matrix.into_css_matrix();
        let expected = "matrix3d(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)";
        assert_eq!(column_major, expected);
    }

    #[test]
    fn identity() {
        use super::Transform;
        use nalgebra::Vector3;
        use nalgebra::UnitQuaternion;

        let transform = Transform::identity();
        assert_eq!(transform.translation, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(transform.scale      , Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(transform.rotation   , UnitQuaternion::identity());
    }

    #[test]
    fn set_transform() {
        use super::Transform;
        use nalgebra::Vector3;
        use nalgebra::Quaternion;
        use std::f32::consts::PI;

        let mut transform = Transform::identity();
        transform.set_translation(1.0, 2.0, 3.0);
        transform.set_scale(3.0, 2.0, 1.0);
        transform.set_rotation(PI * 2.0, PI, PI / 2.0);

        assert_eq!(transform.translation, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.scale, Vector3::new(3.0, 2.0, 1.0));

        let expected = Quaternion::new(0.00000009272586,
                                      -0.7071068,
                                      -0.7071068,
                                      -0.000000030908623);
        assert_eq!(*transform.rotation.quaternion(), expected);
    }
}
