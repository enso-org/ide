in vec2 pos;
out vec4 color;
uniform sampler2D msdf;
uniform float pxRange;
uniform vec4 bgColor;
uniform vec4 fgColor;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec2 msdfUnit = pxRange/vec2(textureSize(msdf, 0));
    vec3 sample = texture(msdf, pos).rgb;
    float sigDist = median(sample.r, sample.g, sample.b) - 0.5;
    sigDist *= dot(msdfUnit, 0.5/fwidth(pos));
    float opacity = clamp(sigDist + 0.5, 0.0, 1.0);
    color = mix(bgColor, fgColor, opacity);
}