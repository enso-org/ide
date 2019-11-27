function arr_to_css_matrix3d(a) {
  return "matrix3d(" + a[ 0] + ","
                     + a[ 1] + ","
                     + a[ 2] + ","
                     + a[ 3] + ","
                     + a[ 4] + ","
                     + a[ 5] + ","
                     + a[ 6] + ","
                     + a[ 7] + ","
                     + a[ 8] + ","
                     + a[ 9] + ","
                     + a[10] + ","
                     + a[11] + ","
                     + a[12] + ","
                     + a[13] + ","
                     + a[14] + ","
                     + a[15] + ")"
}

export function set_object_transform(dom, matrix_array) {
  let css = arr_to_css_matrix3d(matrix_array);
  dom.style.transform = "translate(-50%, -50%)" + css;
}

export function setup_perspective(dom, znear) {
  dom.style.perspective = znear + "px";
}

export function setup_camera_transform(
                        dom,
                        znear,
                        half_width,
                        half_height,
                        matrix_array) {
  let transform = "translateZ(" + znear + "px)"
                + arr_to_css_matrix3d(matrix_array)
                + "translate(" + half_width + "px" + ", " + half_height + "px)"
  dom.style.transform = transform;
}
