use nalgebra::{Vector3, UnitQuaternion, Quaternion, Matrix4};

// To comply with Threejs impl, we generate a Quaternion applying rotation in the order: pitch -> roll -> yaw, instead of roll -> pitch -> yaw
fn from_euler_angles_pry(roll : f32, pitch : f32, yaw : f32) -> UnitQuaternion<f32> {
    let (s1, c1) : (f32, f32) = (roll * 0.5 as f32).sin_cos();
    let (s2, c2) : (f32, f32) = (pitch * 0.5 as f32).sin_cos();
    let (s3, c3) : (f32, f32) = (yaw * 0.5 as f32).sin_cos();

    UnitQuaternion::from_quaternion(Quaternion::new(
        c1 * c2 * c3 - s1 * s2 * s3,
        s1 * c2 * c3 + c1 * s2 * s3,
        c1 * s2 * c3 - s1 * c2 * s3,
        c1 * c2 * s3 + s1 * s2 * c3,
    ))
}

pub struct Transform {
    pub position : Vector3<f32>,
    pub quaternion : UnitQuaternion<f32>,
    pub scale : Vector3<f32>
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            position : Vector3::new(0.0, 0.0, 0.0),
            quaternion : UnitQuaternion::identity(),
            scale : Vector3::new(1.0, 1.0, 1.0)
        }
    }

    pub fn set_position(&mut self, x : f32, y : f32, z : f32) {
        self.position = Vector3::new(x, y, z);
    }

    pub fn set_scale(&mut self, x : f32, y : f32, z : f32) {
        self.scale = Vector3::new(x, y, z);
    }

    pub fn set_rotation(&mut self, roll : f32, pitch : f32, yaw : f32) {
        self.quaternion = from_euler_angles_pry(roll, pitch, yaw);
    }

    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        let (x, y, z, w) = (self.quaternion.coords.x, self.quaternion.coords.y, self.quaternion.coords.z, self.quaternion.coords.w);
		let (x2, y2, z2) = (x + x, y + y, z + z);
		let (xx, xy, xz) = (x * x2, x * y2, x * z2);
		let (yy, yz, zz) = (y * y2, y * z2, z * z2);
		let (wx, wy, wz) = (w * x2, w * y2, w * z2);

		let (sx, sy, sz) = (self.scale.x, self.scale.y, self.scale.z);


        let m00 = ( 1.0 - ( yy + zz ) ) * sx;
		let m10 = ( xy + wz ) * sx;
		let m20 = ( xz - wy ) * sx;
		let m30 = 0.0;

		let m01 = ( xy - wz ) * sy;
		let m11 = ( 1.0 - ( xx + zz ) ) * sy;
		let m21 = ( yz + wx ) * sy;
		let m31 = 0.0;

		let m02 = ( xz + wy ) * sz;
		let m12 = ( yz - wx ) * sz;
		let m22 = ( 1.0 - ( xx + yy ) ) * sz;
		let m32 = 0.0;

		let (m03, m13, m23) = (self.position.x, self.position.y, self.position.z);
		let m33 = 1.0;
        Matrix4::new(m00, m01, m02, m03,
                     m10, m11, m12, m13,
                     m20, m21, m22, m23,
                     m30, m31, m32, m33)
    }
}
