attribute vec4 position;
varying highp vec2 vTextureCoord;

void main() {
    gl_Position = position;
    vTextureCoord = position.xy/2.0 + vec2(0.5, 0.5);
}