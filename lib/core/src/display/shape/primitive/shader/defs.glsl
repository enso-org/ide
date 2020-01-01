

float mix (float a, float b, float w1, float w2) {
    return (a * w1 + b * w2) / (w1 + w2);
}

vec2 mix (vec2 a, float w1, vec2 b, float w2) {
    vec2 c;
    float ws = w1 + w2;
    c.x = (a.x * w1 + b.x * w2) / ws;
    c.y = (a.y * w1 + b.y * w2) / ws;
    return c;
}

vec3 mix (vec3 a, float w1, vec3 b, float w2) {
    vec3 c;
    float ws = w1 + w2;
    c.x = (a.x * w1 + b.x * w2) / ws;
    c.y = (a.y * w1 + b.y * w2) / ws;
    c.z = (a.z * w1 + b.z * w2) / ws;
    return c;
}

vec4 mix (vec4 a, float w1, vec4 b, float w2) {
    vec4 c;
    float ws = w1 + w2;
    c.x = (a.x * w1 + b.x * w2) / ws;
    c.y = (a.y * w1 + b.y * w2) / ws;
    c.z = (a.z * w1 + b.z * w2) / ws;
    c.w = (a.w * w1 + b.w * w2) / ws;
    return c;
}


float bismooth (float a, float exp) {
  float a2 = a * 2.0 - 1.0;
  float a3 = pow(abs(a2),exp);
  float a4 = (a3 * sign(a2) + 1.0) * 0.5;
  return a4;
}

float clamp (float a) { return clamp(a, 0.0, 1.0); }
vec2  clamp (vec2  a) { return clamp(a, 0.0, 1.0); }
vec3  clamp (vec3  a) { return clamp(a, 0.0, 1.0); }
vec4  clamp (vec4  a) { return clamp(a, 0.0, 1.0); }



float smoothstep (float a) {
    return smoothstep (0.0, 1.0, a);
}

// TODO: check if still useful
vec3 smoothMerge (float d1, float d2, vec3 c1, vec3 c2, float width) {
    return mix (c1,c2,bismooth(clamp((d1-d2+2.0*width)/(4.0*width)),2.0));
}




/////////////////////////
////// Conversions //////
/////////////////////////

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}



/////////////////////////////////////////////////////////////////




///////////////////////
////// Constants //////
///////////////////////

#define PI 3.14159265
#define TAU (2.0*PI)
#define PHI (sqrt(5.0)*0.5 + 0.5)
const float INF = 1e10;



/////////////////////
////// Helpers //////
/////////////////////

float square (float x) {return x*x;}
vec2  square (vec2  x) {return x*x;}
vec3  square (vec3  x) {return x*x;}

float lengthSqr (vec3 x) {return dot(x, x);}

float maxEl (vec2 v) {return max(v.x, v.y);}
float maxEl (vec3 v) {return max(max(v.x, v.y), v.z);}
float maxEl (vec4 v) {return max(max(v.x, v.y), max(v.z, v.w));}

float minEl (vec2 v) {return min(v.x, v.y);}
float minEl (vec3 v) {return min(min(v.x, v.y), v.z);}
float minEl (vec4 v) {return min(min(v.x, v.y), min(v.z, v.w));}

float signPlus (float x) { return (x<0.0)?-1.0:1.0; }
vec2  signPlus (vec2  v) { return vec2((v.x<0.0)?-1.0:1.0, (v.y<0.0)?-1.0:1.0);}
vec3  signPlus (vec3  v) { return vec3((v.x<0.0)?-1.0:1.0, (v.y<0.0)?-1.0:1.0, (v.z<0.0)?-1.0:1.0);}
vec4  signPlus (vec4  v) { return vec4((v.x<0.0)?-1.0:1.0, (v.y<0.0)?-1.0:1.0, (v.z<0.0)?-1:1, (v.w<0.0)?-1.0:1.0);}



///////////////////////
////// Transform //////
///////////////////////

vec2 sdf_translate (vec2 p, vec2 t) { return p - t; }

vec2 sdf_rotate (vec2 p, float angle) {
	return p*cos(angle) + vec2(p.y,-p.x)*sin(angle);
}

vec2 cartesian2polar (vec2 p) {
  return vec2(length(p), atan(p.y, p.x));
}



////////////////////////////////
////// Shape modification //////
////////////////////////////////

float sdf_grow   (float size, float d)  { return d - size;  }
float sdf_shrink (float size, float d)  { return d + size;  }
float sdf_border (float d)              { return abs(d);    }
float sdf_flatten(float a)              { return clamp(-a); }
float sdf_render (float d)              { return clamp((0.5 - d) / zoomLevel); }
float sdf_render (float d, float w)     { return clamp((0.5 - d) / zoomLevel / w); }

float sdf_removeOutside (float d) { return (d > 0.0) ?  INF : d; }
float sdf_removeInside  (float d) { return (d < 0.0) ? -INF : d; }


//////////////////////
////// Booleans //////
//////////////////////


////// Inverse //////

float sdf_inverse (float a) {
    return -a;
}


////// Union //////

float sdf_union (float a, float b) {
    return min(a,b);
}

float sdf_unionRound (float a, float b, float r) {
	vec2 v = max(vec2(r-a, r-b), 0.0);
	return max(r, sdf_union(a, b)) - length(v);
}

float sdf_unionChamfer (float a, float b, float r) {
	return min(sdf_union(a, b), (a - r + b) * sqrt(0.5));
}

float sdf_union (float a, float b, float r) {
    return sdf_unionRound(a,b,r);
}



////// Intersection //////

float sdf_intersection (float a, float b) {
    return max(a,b);
}

float sdf_intersectionRound (float a, float b, float r) {
	vec2 v = max(vec2(r+a, r+b), 0.0);
	return min(-r, sdf_intersection(a,b)) + length(v);
}

float sdf_intersectionChamfer (float a, float b, float r) {
	return max(sdf_intersection(a,b), (a+r+b)*sqrt(0.5));
}

float sdf_intersection (float a, float b, float r) {
    return sdf_intersectionRound(a,b,r);
}


////// Difference //////

float sdf_difference (float a, float b) {
    return sdf_intersection(a, sdf_inverse(b));
}

float sdf_difference (float a, float b, float r) {
    return sdf_intersection(a, sdf_inverse(b), r);
}

float sdf_differenceRound (float a, float b, float r) {
	return sdf_intersectionRound (a, sdf_inverse(b), r);
}

float sdf_differenceChamfer (float a, float b, float r) {
	return sdf_intersectionChamfer(a, sdf_inverse(b), r);
}



/////////////////////
////// Filters //////
/////////////////////

float sdf_blur (float d, float radius, float power) {
    return 1.0-2.0*pow(clamp((radius - d) / radius),power);
}




/////////////////////////////////
////// 2D Primitive shapes //////
/////////////////////////////////


////// Plane //////

float sdf_plane(vec2 p) {
  return -1.0;
}

float sdf_halfplaneFast(vec2 p, vec2 dir) {
  return dir.x * p.x + dir.y * p.y;
}

float sdf_halfplaneFast(vec2 p, float angle) {
  return sdf_halfplaneFast(p, sdf_rotate(vec2(0.0,1.0), angle));
}

float sdf_halfplaneFast(vec2 p) {
    return sdf_halfplaneFast(p, vec2(0.0, 1.0));
}

float sdf_halfplane(vec2 p, vec2 dir) {
  float dx = dir.x;
  float dy = dir.y;
  return (dx * p.x + dy * p.y) / sqrt(dx*dx + dy*dy);
}

float sdf_halfplane(vec2 p, float angle) {
  return sdf_halfplane(p, sdf_rotate(vec2(0.0,1.0), angle));
}

float sdf_halfplane(vec2 p) {
    return sdf_halfplane(p, vec2(0.0, 1.0));
}

float sdf_halfplaneRight  (vec2 p) { return sdf_halfplane(p, vec2( 1.0,  0.0)); }
float sdf_halfplaneLeft   (vec2 p) { return sdf_halfplane(p, vec2(-1.0,  0.0)); }
float sdf_halfplaneTop    (vec2 p) { return sdf_halfplane(p, vec2( 0.0,  1.0)); }
float sdf_halfplaneBottom (vec2 p) { return sdf_halfplane(p, vec2( 0.0, -1.0)); }

float sdf_halfplaneFastRight  (vec2 p) { return sdf_halfplaneFast(p, vec2( 1.0,  0.0)); }
float sdf_halfplaneFastLeft   (vec2 p) { return sdf_halfplaneFast(p, vec2(-1.0,  0.0)); }
float sdf_halfplaneFastTop    (vec2 p) { return sdf_halfplaneFast(p, vec2( 0.0,  1.0)); }
float sdf_halfplaneFastBottom (vec2 p) { return sdf_halfplaneFast(p, vec2( 0.0, -1.0)); }


////// Line //////

float sdf_line(vec2 p, vec2 dir, float width) {
  float len  = length(dir);
  vec2  n    = dir / len;
  vec2  proj = max(0.0, min(len, dot(p,n))) * n;
  return length(p-proj) - (width/2.0);
}


////// Rectangle //////

float sdf_rectSharp(vec2 p, vec2 size) {
    return maxEl(abs(p) - size);
}

float sdf_rect(vec2 p, vec2 size) {
  size   = size / 2.0;
  vec2 d = abs(p) - size;
  return maxEl(min(d, 0.0)) + length(max(d, 0.0));
}

float sdf_rect(vec2 p, vec2 size, float radius) {
  return sdf_grow(radius, sdf_rect(p, size-2.0*radius));
}


////// Triangle //////

float sdf_triangle (vec2 p, float width, float height) {
  vec2 n = normalize(vec2(height, width / 2.0));
  return max(abs(p).x*n.x + p.y*n.y - (height*n.y), -p.y);
}


////// Circle ///////

float sdf_pie(vec2 p, float angle) {
  return abs(p).x*cos(angle/2.0) + p.y*sin(angle/2.0);
}

float sdf_circle (vec2 p, float radius) {
  return length(p) - radius;
}

vec3 sdvf_circle (vec2 p, float radius) {
  float len = length(p);
  float d   = radius - len;
  vec2  dir = (p / len) * sign(d);
  return vec3(dir,d);
}

float sdf_circle(vec2 p, float radius, float angle) {
  return sdf_intersection(sdf_circle(p,radius), sdf_pie(p, angle));
}

float sdf_ellipse(vec2 p, float a, float b) {
  float a2  = a * a;
  float b2  = b * b;
  float px2 = p.x * p.x;
  float py2 = p.y * p.y;
  return (b2 * px2 + a2 * py2 - a2 * b2)/(a2 * b2);
}

float sdf_ring(vec2 p, float radius, float width) {
  width /= 2.0;
  radius -= width;
  return abs(sdf_circle(p, radius)) - width;
}

float sdf_ring(vec2 p, float radius, float width, float angle) {
   return sdf_difference(sdf_pie(p, angle), sdf_ring(p, radius, width));
}


////// Bezier curve //////


// Test if `p` crosses line (`a`, `b`), returns sign of result
float testPointOnLine(vec2 p, vec2 a, vec2 b) {
    return sign((b.y-a.y) * (p.x-a.x) - (b.x-a.x) * (p.y-a.y));
}

// Determine which side we're on (using barycentric parameterization)
float bezier_sign(vec2 p, vec2 A, vec2 B, vec2 C)
{
    vec2 a = C - A, b = B - A, c = p - A;
    vec2 bary = vec2(c.x*b.y-b.x*c.y,a.x*c.y-c.x*a.y) / (a.x*b.y-b.x*a.y);
    vec2 d = vec2(bary.y * 0.5, 0.0) + 1.0 - bary.x - bary.y;
    return mix(sign(d.x * d.x - d.y), mix(-1.0, 1.0,
        step(testPointOnLine(p, A, B) * testPointOnLine(p, B, C), 0.0)),
        step((d.x - d.y), 0.0)) * testPointOnLine(B, A, C);
}

// Solve cubic equation for roots
vec3 bezier_solveCubic(float a, float b, float c)
{
    float p = b - a*a / 3.0, p3 = p*p*p;
    float q = a * (2.0*a*a - 9.0*b) / 27.0 + c;
    float d = q*q + 4.0*p3 / 27.0;
    float offset = -a / 3.0;
    if(d >= 0.0) {
        float z = sqrt(d);
        vec2 x = (vec2(z, -z) - q) / 2.0;
        vec2 uv = sign(x)*pow(abs(x), vec2(1.0/3.0));
        return vec3(offset + uv.x + uv.y);
    }
    float v = acos(-sqrt(-27.0 / p3) * q / 2.0) / 3.0;
    float m = cos(v), n = sin(v)*1.732050808;
    return vec3(m + m, -n - m, n - m) * sqrt(-p / 3.0) + offset;
}

float sdf_quadraticCurve(vec2 p, vec2 A, vec2 B)
{
    vec2 a = mix(A + vec2(1e-4), A, abs(sign(A * 2.0 - B)));
    vec2 b = -A * 2.0 + B;
    vec2 c = a * 2.0;
    vec2 d = -p;
    vec3 k = vec3(3.*dot(a,b),2.*dot(a,a)+dot(d,b),dot(d,a)) / dot(b,b);
    vec3 t = clamp(bezier_solveCubic(k.x, k.y, k.z));
    vec2 pos = (c + b*t.x)*t.x;
    float dis = length(pos - p);
    pos = (c + b*t.y)*t.y;
    dis = min(dis, length(pos - p));
    return dis;
}

#define quadraticCurve_interiorCheck_helper(f) if (f>0. && f<1. && mix(a.x*f,mix(a.x,b.x,f),f)<p.x) inside=!inside;
bool quadraticCurve_interiorCheck(vec2 p, vec2 a, vec2 b) {
  const float eps = 1e-7;
  bool  inside = false;
  float root, A, B, C;

  // http://alienryderflex.com/polyspline/
  // "What happens if F is exactly 0, or exactly 1?
  // This opens up a whole can of headaches that we’d rather not deal with, for the sake of simpler code and better execution speed.
  // Probably the easiest way to avoid the problem is just to add a very small value (say, 0.000001) to the test point’s y-coordinate
  // before testing the point.  That will pretty much guarantee that F will never be exactly 0 or 1."

  // FIXME
  // It is still not working sometimes when moving slowly on scren.
  // We cannot discover here it if failed and re-run it again, because we're running for every p
  // and we dont know if it failed for other p. If we then move, we do other artifacts!
  float ydelta = 0.000007;

  A = b.y - a.y - a.y + ydelta;
  B = 2.*a.y + ydelta;
  C = -p.y + ydelta;
  if (abs(A)<eps) {
    quadraticCurve_interiorCheck_helper(-C / B);
  } else {
	root = B*B - 4.*A*C;
	if (root>0.) {
	  root = sqrt(root);
      quadraticCurve_interiorCheck_helper((-B - root) / (2.*A));
	  quadraticCurve_interiorCheck_helper((-B + root) / (2.*A));
	}
  }
  return inside;
}


#define coverSegment_line_check(f) if (f>0. && f<1. && mix(a.x,mix(a.x,b.x,f),f)<p.x) inside=!inside;
bool coverSegment_line(vec2 p, vec2 a, vec2 b) {
  const float eps = 1e-7;
  bool  inside = false;
  float root, A, B;

  A = b.y - a.y;
  B = a.y - p.y;
  root = - 4.*A*B;
  if (root>0.) {
    root = sqrt(root);
    coverSegment_line_check((-root) / (2.*A));
    coverSegment_line_check((root)  / (2.*A));
  }
  return inside;
}

bool interiorChec_union(bool c1, bool c2) {
  if(c2) {return !c1;} else {return c1;}
}

// vec2[9] bezier_convert4To3(vec2 p0, vec2 p1, vec2 p2, vec2 p3) {
//   vec2 p01    = (p0  + p1) /2.;
//   vec2 p12    = (p1  + p2) /2.;
//   vec2 p23    = (p2  + p3) /2.;
//   vec2 p0_01  = (p0  + p01)/2.;
//   vec2 p23_3  = (p23 + p3) /2.;
//   vec2 p01_12 = (p01 + p12)/2.;
//   vec2 p23_12 = (p23 + p12)/2.;
//
//   vec2 np0 = p0;
//   vec2 np8 = p3;
//   vec2 np1 = (p01 + p0_01)/2.;
//   vec2 np7 = (p23 + p23_3)/2.;
//   vec2 np4 = (p01_12 + p23_12)/2.;
//   vec2 np3 = ((p01_12 + np4)/2. + p01_12)/2.;
//   vec2 np5 = ((p23_12 + np4)/2. + p23_12)/2.;
//   vec2 np2 = (np1 + np3)/2.;
//   vec2 np6 = (np5 + np7)/2.;
//
//   return vec2[9](np0,np1,np2,np3,np4,np5,np6,np7,np8);
// }

// USAGE
// float d1       = sdf_quadraticCurve           (p1, A,B);
// bool  d1_cover = quadraticCurve_interiorCheck (p1, A,B);
// float d2       = sdf_quadraticCurve           (p2, C,D);
// bool  d2_cover = quadraticCurve_interiorCheck (p2, C,D);
// float d3       = sdf_quadraticCurve           (p3, E,F);
// bool  d3_cover = quadraticCurve_interiorCheck (p3, E,F);
// bool isInside = interiorChec_union(interiorChec_union(cover1,cover2),cover3);



vec2 sdf_repeat (vec2 p, vec2 dir) {
    return mod(p,dir);
}



///////////////////////
////// Debugging //////
///////////////////////


vec3 sdf_debug (float a, float gridScale) {
    float gridLines = smoothstep(0.0, 0.2, 2.0 * abs(mod(a/gridScale, 1.0) - 0.5));
    float zeroLines = smoothstep(0.0, 0.2, 1.0 - abs(a));
    float hue       = mod(a/1000.0 + 0.0, 1.0);
    vec3  bgCol     = hsv2rgb(vec3(hue,0.8,0.8));
    return bgCol * gridLines + vec3(zeroLines);
}

vec3 sdf_debug (float a) {
    return sdf_debug (a, 10.0);
}



float sdf_rect(vec2 p, vec2 size, vec4 corners) {
  float tl = corners[0];
  float tr = corners[1];
  float bl = corners[2];
  float br = corners[3];

  size /= 2.0;

       if (p.x <  - size.x + tl && p.y >   size.y - tl ) { return length (p - vec2(- size.x + tl,   size.y - tl)) - tl; }
  else if (p.x >    size.x - tr && p.y >   size.y - tr ) { return length (p - vec2(  size.x - tr,   size.y - tr)) - tr; }
  else if (p.x <  - size.x + bl && p.y < - size.y + bl ) { return length (p - vec2(- size.x + bl, - size.y + bl)) - bl; }
  else if (p.x >    size.x - br && p.y < - size.y + br ) { return length (p - vec2(  size.x - br, - size.y + br)) - br; }
  else {
    vec2 d = abs(p) - size;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
  }
}



vec3 Uncharted2ToneMapping(vec3 color) {
	float A = 0.15;
	float B = 0.50;
	float C = 0.10;
	float D = 0.20;
	float E = 0.02;
	float F = 0.30;
	float W = 11.2;
	float exposure = 2.;
	color *= exposure;
	color = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
	float white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
	color /= white;
	return color;
}

//
// interesting part starts here
//
// the meter uses the "fusion" gradient, which goes from dark magenta (0) to white (1)
// (often seen in heatmaps in papers etc)
//

vec3 fusion(float x) {
	float t = clamp(x,0.0,1.0);
	return clamp(vec3(sqrt(t), t*t*t, max(sin(PI*1.75*t), pow(t, 12.0))), 0.0, 1.0);
}

// HDR version
vec3 fusionHDR(float x) {
	float t = clamp(x,0.0,1.0);
	return fusion(sqrt(t))*(0.5+2.*t);
}


//
// distance meter function. needs a bit more than just the distance
// to estimate the zoom level that it paints at.
//
// if you have real opengl, you can additionally use derivatives (dFdx, dFdy)
// to detect discontinuities, i had to strip that for webgl
//
// visualizing the magnitude of the gradient is also useful
//

vec3 distanceMeter(float dist, float rayLength, vec3 rayDir, float camHeight) {
    float idealGridDistance = 20.0/rayLength*pow(abs(rayDir.y),0.8);
    float nearestBase = floor(log(idealGridDistance)/log(10.));
    float relativeDist = abs(dist/camHeight);

    float largerDistance = pow(10.0,nearestBase+1.);
    float smallerDistance = pow(10.0,nearestBase);


    vec3 col = fusionHDR(log(1.+relativeDist));
    col = max(vec3(0.),col);
    if (sign(dist) < 0.) {
        col = col.grb*3.;
    }

    float l0 = (pow(0.5+0.5*cos(dist*PI*2.*smallerDistance),10.0));
    float l1 = (pow(0.5+0.5*cos(dist*PI*2.*largerDistance),10.0));

    float x = fract(log(idealGridDistance)/log(10.));
    l0 = mix(l0,0.,smoothstep(0.5,1.0,x));
    l1 = mix(0.,l1,smoothstep(0.0,0.5,x));

    col.rgb *= 0.1+0.9*(1.-l0)*(1.-l1);
    return col;
}







// ---------------------------


//
// float sdf_render_bak(float p) {
//   float d = p ;
//   float aa = 1.0;
//   float anti = fwidth(d) * aa;
//   return (1.0 - smoothstep(-anti, anti, d));
//   // other approach (https://github.com/Chlumsky/msdfgen/issues/22)
//   //float v = d / fwidth(d);
//   //return 1.0 - clamp( v + 0.5, 0.0, 1.0);
// }



// float sdf_render(float d, float width) {
//   float anti = fwidth(d) + width;
//   return (1.0 - smoothstep(-anti, anti, d));
// }
//


// ----------------------------------------


float packColor(vec3 color) {
    return color.r + color.g * 256.0 + color.b * 256.0 * 256.0;
}

vec3 unpackColor(float f) {
    vec3 color;
    color.r = floor(f / 256.0 / 256.0);
    color.g = floor((f - color.r * 256.0 * 256.0) / 256.0);
    color.b = floor(f - color.r * 256.0 * 256.0 - color.g * 256.0);
    return color / 255.0;
}



int newIDLayer (float a, int i) {
    return (a <= 0.0) ? i : 0;
}

float newIDLayer (float a, float i) {
    return (a <= 0.0) ? i : 0.0;
}

int id_union        (float a, float b, int ida, int idb) { return (b <= 0.0) ? idb : ida; }
int id_difference   (float a, float b, int ida)          { return (sdf_difference(a,b) <= 0.0) ? ida : 0 ; }
int id_intersection (float a, float b, int ida)          { return ((a <= 0.0) && (b <= 0.0)) ? ida : 0 ; }


vec4 bbox_new (float w, float h) {
    return vec4(-w, -h, w, h);
}

vec4 bbox_union (vec4 a, vec4 b) {
    float xmin = min(a[0],b[0]);
    float ymin = min(a[1],b[1]);
    float xmax = max(a[2],b[2]);
    float ymax = max(a[3],b[3]);
    return vec4(xmin, ymin, xmax, ymax);
}

vec4 bbox_grow (float d, vec4 bbox) {
    return bbox + vec4(-d,-d,d,d);
}

struct sdf_shape {
  float density;
  int   id;
  vec4  bb;
  vec4  cd;
};





//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////

// FIXME: glslify renames functions, so we cannot concat outputs after running glslify!!!
// #pragma glslify: toLinear = require('glsl-gamma/in')
// #pragma glslify: toGamma  = require('glsl-gamma/out')

float gm = 2.2;
vec3 toGamma(vec3 v) {
  return pow(v, vec3(1.0 / gm));
}

vec4 toGamma(vec4 v) {
  return vec4(toGamma(v.rgb), v.a);
}

vec3 toLinear(vec3 v) {
  return pow(v, vec3(gm));
}

vec4 toLinear(vec4 v) {
  return vec4(toLinear(v.rgb), v.a);
}


const vec3 wref =  vec3(1.0, 1.0, 1.0);

float sRGB(float t){ return mix(1.055*pow(t, 1./2.4) - 0.055, 12.92*t, step(t, 0.0031308)); }
vec3 sRGB(in vec3 c) { return vec3 (sRGB(c.x), sRGB(c.y), sRGB(c.z)); }

//-----------------Lch-----------------

float xyzF(float t){ return mix(pow(t,1./3.), 7.787037*t + 0.139731, step(t,0.00885645)); }
float xyzR(float t){ return mix(t*t*t , 0.1284185*(t - 0.139731), step(t,0.20689655)); }
vec3 rgb2lch(in vec3 c)
{
	c  *= mat3( 0.4124, 0.3576, 0.1805,
          		0.2126, 0.7152, 0.0722,
                0.0193, 0.1192, 0.9505);
    c.x = xyzF(c.x/wref.x);
	c.y = xyzF(c.y/wref.y);
	c.z = xyzF(c.z/wref.z);
	vec3 lab = vec3(max(0.,116.0*c.y - 16.0), 500.0*(c.x - c.y), 200.0*(c.y - c.z));
    return vec3(lab.x, length(vec2(lab.y,lab.z)), atan(lab.z, lab.y));
}

vec4 rgb2lch(vec4 c) {
    return vec4(rgb2lch(c.rgb), c.a);
}
vec3 hue2rgb(float hue) {
    float R = abs(hue * 6.0 - 3.0) - 1.0;
    float G = 2.0 - abs(hue * 6.0 - 2.0);
    float B = 2.0 - abs(hue * 6.0 - 4.0);
    return clamp(vec3(R,G,B), 0.0, 1.0);
}
vec3 hsl2rgb(vec3 hsl) {
    vec3 rgb = hue2rgb(hsl.x);
    float C = (1.0 - abs(2.0 * hsl.z - 1.0)) * hsl.y;
    return (rgb - 0.5) * C + hsl.z;
}
vec3 hsl2lch(vec3 c) {
    return rgb2lch(hsl2rgb(c));
}
vec4 hsl2lch(vec4 c) {
    return vec4(hsl2lch(c.xyz), c.a);
}

vec3 lch2rgb(in vec3 c)
{
    c = vec3(c.x, cos(c.z) * c.y, sin(c.z) * c.y);

    float lg = 1./116.*(c.x + 16.);
    vec3 xyz = vec3(wref.x*xyzR(lg + 0.002*c.y),
    				wref.y*xyzR(lg),
    				wref.z*xyzR(lg - 0.005*c.z));

    vec3 rgb = xyz*mat3( 3.2406, -1.5372,-0.4986,
          		        -0.9689,  1.8758, 0.0415,
                	     0.0557,  -0.2040, 1.0570);

    return rgb;
}

//cheaply lerp around a circle
float lerpAng(in float a, in float b, in float x)
{
    float ang = mod(mod((a-b), TAU) + PI*3., TAU)-PI;
    return ang*x+b;
}

//Linear interpolation between two colors in Lch space
vec3 lerpLch(in vec3 a, in vec3 b, in float x)
{
    float hue = lerpAng(a.z, b.z, x);
    return vec3(mix(b.xy, a.xy, x), hue);
}









// FIXME: fix transparent aa - fwidth is obsolete now, see sdf_render for reference
vec4 color_mergeLCH (float d2, float d1, vec4 c2, vec4 c1, float width) {
  float w1  = width + fwidth(d1);
  float w2  = width + fwidth(d2);
  float p1  = sdf_render(d1);
  float p2  = sdf_render(d2);
  float pb1 = c1.a * c1.a * smoothstep(1.0-clamp((d1/w1) + 0.5));
  float pb2 = c2.a * c2.a * smoothstep(1.0-clamp((d2/w2)));
  vec3  c3  = mix (c1.rgb, pb1, c2.rgb, (1.0-pb1)*pb2);
  float aa  = p1 * c1.a + p2 * c2.a;
  aa /= max(p1, p2); // unpremultiply
  return vec4(c3, aa);
}

// vec3 color_mergeLCH (float d1, float d2, vec3 c1, vec3 c2, float width) {
//   float w1  = width + fwidth(d1);
//   float w2  = width + fwidth(d2);
//   float pb1 = smoothstep(1.0-clamp((d1/w1) + 0.5));
//   float pb2 = smoothstep(1.0-clamp((d2/w2)));
//   vec3  c3  = mix (c1, pb1, c2, (1.0-pb1)*pb2);
//   return c3;
// }

vec4 color_mergeLCH (float d1, float d2, vec4 c1, vec4 c2) {
    return color_mergeLCH(d1, d2, c1, c2, 0.0);
}

// vec3 color_mergeLCH (float d1, float d2, vec3 c1, vec3 c2) {
//     return color_mergeLCH(d1, d2, c1, c2, 0.0);
// }