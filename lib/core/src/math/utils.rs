use nalgebra::Matrix4;
use nalgebra::RealField;

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
        let ret = iter.fold(acc, |acc, item| format!("{}, {}", acc, item));
        format!("matrix3d({})", ret)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn into_css_matrix() {
        use nalgebra::Matrix4;
        use super::IntoCSSMatrix;

        let matrix = Matrix4::new
            ( 1.0, 5.0,  9.0, 13.0
            , 2.0, 6.0, 10.0, 14.0
            , 3.0, 7.0, 11.0, 15.0
            , 4.0, 8.0, 12.0, 16.0 );
        let column_major = matrix.into_css_matrix();
        let expected = "matrix3d(1, 2, 3, 4, \
                                 5, 6, 7, 8, \
                                 9, 10, 11, 12, \
                                13, 14, 15, 16)";
        assert_eq!(column_major, expected);
    }
}
