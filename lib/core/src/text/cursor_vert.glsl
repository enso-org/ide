attribute vec2 position;
attribute vec2 tex_coord;

uniform highp mat3 to_window;
uniform highp vec2 clip_lower;
uniform highp vec2 clip_upper;

varying vec2 v_tex_coord;
varying vec4 v_clip_distance;

void main() {
    highp vec3 position_on_window = to_window * vec3(position, 1.0);
    v_clip_distance.x = position_on_window.x - clip_lower.x;
    v_clip_distance.y = position_on_window.y - clip_lower.y;
    v_clip_distance.z = clip_upper.x - position_on_window.x;
    v_clip_distance.w = clip_upper.y - position_on_window.y;

    v_tex_coord = tex_coord / msdf_size;
    gl_Position = vec4(position_on_window.xy, 0.0, position_on_window.z);
}
