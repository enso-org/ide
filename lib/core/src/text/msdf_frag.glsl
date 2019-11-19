#extension GL_OES_standard_derivatives : enable

varying highp vec2 vTextureCoord;

uniform sampler2D msdf;
uniform int msdf_size;
uniform lowp float pxRange;
uniform lowp vec4 bgColor;
uniform lowp vec4 fgColor;

highp float median(highp float r, highp float g, highp float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    highp vec2 msdfUnit = pxRange/vec2(msdf_size, 0);
    highp vec3 smple = texture2D(msdf, vTextureCoord).rgb;
    highp float sigDist = median(smple.r, smple.g, smple.b) - 0.5;
    sigDist *= dot(msdfUnit, 0.5/fwidth(vTextureCoord));
    highp float opacity = clamp(sigDist + 0.5, 0.0, 1.0);
    gl_FragColor = mix(bgColor, fgColor, opacity);
//    gl_FragColor = vec4(sigDist, 0.0, 0.0, 1.0);
}
