"""A fast GPU renderer of the Likeness Lab head, with the page's skin, brow, lip and eye paint ported to a shader. No hair.

The camera and lights follow the page's front view and studio light.
"""
import numpy as np
import moderngl

SKIN = "#e4b896"
HAIRS = ["#1b1612", "#3a2a1c", "#6b4a2b", "#a87c45", "#d9b87a", "#e6d7b0", "#8c8c8c", "#8b2d1e"]
EYE = "#5a3a22"
LIP = "#b06a62"
BROW_KEYS = ["browInner", "browOuter", "browArchPos", "browArch", "browThickIn", "browThickOut", "browLength", "browSpacing"]
BROW_DEFAULT = {"browInner": 0, "browOuter": 0, "browArchPos": 0, "browArch": 0, "browThickIn": 0.5, "browThickOut": 0.4, "browLength": 0, "browSpacing": 0, "browDensity": 0.85, "browColor": 1}


def srgb(hexs):
    return [int(hexs[i:i + 2], 16) / 255 for i in (1, 3, 5)]


VERT = """
#version 330
uniform mat4 mvp; uniform mat4 lampMvp; uniform mat3 unturn; uniform vec3 turnC;
in vec3 in_pos; in vec3 in_nrm; in vec3 in_base;
out vec3 v_nrm; out vec3 v_base; out vec3 v_pos; out vec3 v_lamp;
void main() { v_nrm = in_nrm; v_base = in_base; v_pos = unturn * (in_pos - turnC) + turnC; v_lamp = (lampMvp * vec4(in_pos, 1.0)).xyz * 0.5 + 0.5; gl_Position = mvp * vec4(in_pos, 1.0); }
"""
LIGHT = """
uniform float flash; uniform float shadow; uniform sampler2D lampDepth; in vec3 v_lamp;
vec3 toLin(vec3 c) { return pow(c, vec3(2.2)); }
vec3 toSrgb(vec3 c) { return pow(clamp(c, 0.0, 1.0), vec3(1.0 / 2.2)); }
vec3 light(vec3 albedo, vec3 n) {
  n = normalize(n);
  vec3 sky = toLin(vec3(0.875, 0.902, 0.949)), ground = toLin(vec3(0.227, 0.212, 0.188));
  vec3 amb = mix(ground, sky, 0.5 * n.y + 0.5) * 0.9 / 3.14159;
  float k = max(dot(n, normalize(vec3(1.2, 2.3, 1.8))), 0.0) * 2.2 / 3.14159;
  float f = max(dot(n, normalize(vec3(-1.6, 1.3, 1.0))), 0.0) * 0.7 / 3.14159;
  float r = max(dot(n, normalize(vec3(-0.6, 1.9, -1.8))), 0.0) * 0.8 / 3.14159;
  vec3 page = amb + k * toLin(vec3(1.0, 0.949, 0.878)) + f * toLin(vec3(0.663, 0.769, 0.910)) + r;
  // flash at 1 lights the head as a studio portrait is lit: evenly, from the camera's side
  vec3 studio = vec3(0.7 / 3.14159 + max(n.z, 0.0) * 1.3 / 3.14159);
  // a studio's lamps hang above the camera and leave the neck in the shadow of the jaw. The shadow's edge is soft, as from a wide lamp
  float lit = 0.0;
  for (int i = -3; i <= 3; i++) for (int j = -3; j <= 3; j++)
    lit += step(v_lamp.z - 0.012, texture(lampDepth, v_lamp.xy + vec2(i, j) * 0.004).r);
  studio *= mix(1.0, mix(0.65, 1.0, lit / 49.0), shadow);
  vec3 c = toLin(albedo) * mix(page, studio, flash);
  return toSrgb(c * 3.14159 * 0.55);
}
"""
FRAG_HEAD = """
#version 330
uniform vec3 skin, browC, lipC; uniform vec3 browL, eyeL, lipUL, cornerL, noseTip; uniform float chinY;
uniform vec2 bp0, bp1, bp2; uniform float bx0, thIn, thOut, browDensity; uniform int browOnly;
uniform float socket, liner, nostril; uniform int browN; uniform vec3 browCurve[22]; uniform vec3 browPaint; uniform vec2 browGroom; uniform int lidN; uniform vec3 lidCurve[34]; uniform vec4 lash; uniform vec3 lashC; uniform int lipN; uniform vec2 lipX; uniform vec3 lipCurve[65]; uniform float lipMask; uniform vec3 lipCU; uniform float lipMix; uniform vec3 canthusL, canthusR; uniform float canthus, innerFold; uniform vec2 crease;
in vec3 v_nrm; in vec3 v_base; in vec3 v_pos; out vec4 frag;
""" + LIGHT + """
float ss(float a, float b, float t) { float x = clamp((t - a) / (b - a), 0.0, 1.0); return x * x * (3.0 - 2.0 * x); }
vec2 segDist(vec2 p, vec2 a, vec2 b) { vec2 v = b - a; float t = clamp(dot(p - a, v) / max(dot(v, v), 1e-9), 0.0, 1.0); return vec2(length(p - (a + v * t)), t); }
void main() {
  vec3 p = v_base; float ax = abs(p.x);
  vec3 c = skin;
  float eyeX = eyeL.x, eyeY = eyeL.y, eyeZ = eyeL.z;
  float lipU = lipUL.x, lipLo = lipUL.y, cx = cornerL.x, cornerZ = cornerL.z;
  // painted shading, as the page's default of 0.5
  float ao = socket * ss(0.030, 0.014, length(vec2(ax - eyeX, (p.y - eyeY) * 1.2)));
  if (p.z > noseTip.z - 0.03) ao += nostril * ss(0.010, 0.003, length(vec2(ax - 0.010, p.y - (noseTip.y - 0.010))));
  if (p.z > cornerZ - 0.02) ao += 0.35 * ss(0.012, 0.004, abs(p.y - (lipLo - 0.006))) * ss(cx + 0.01, cx - 0.01, ax);
  if (p.y < chinY - 0.01) ao += 0.3 * ss(chinY - 0.015, chinY - 0.04, p.y);
  float k = 1.0 - 0.5 * min(1.0, ao) * 0.55; c *= vec3(k, k * 0.97, k * 0.92);
  // under-eye darkness at 0.15
  if (p.y < eyeY - 0.006) c = mix(c, c * vec3(0.75, 0.7, 0.75), 0.15 * ss(0.022, 0.012, length(vec2(ax - eyeX, (p.y - (eyeY - 0.014)) * 1.6))));
  // the lids' inner faces near each inner corner of the eye show where the lids leave the eyeball. They get the caruncle's pink, so the
  // pink corner painted on the eyeball goes on into them. canthusL and canthusR are the inner corners on the base head; canthus 2 shows the
  // painted area in green, for checking
  if (canthus > 0.5) {
    for (int k = 0; k < 2; k++) {
      vec3 d = p - (k == 0 ? canthusL : canthusR);
      float near = ss(0.006, 0.005, length(d.xy)) * ss(-0.0001, -0.0004, d.z);
      c = mix(c, canthus > 1.5 ? vec3(0.0, 1.0, 0.0) : skin * vec3(0.97, 0.85, 0.84), near * (1.0 - innerFold));
      // innerFold draws the fold of skin over the inner corner: a soft shadow that carries the upper lid's line on past the corner,
      // 3.5 mm toward the nose and 1.5 mm down, on the front skin only
      if (innerFold > 0.0 && d.z > -0.0003) {
        float m = k == 0 ? -1.0 : 1.0;
        vec2 a = vec2(0.0, 0.0002), b = vec2(m * 0.0035, -0.0015), v = b - a;
        float t = clamp(dot(d.xy - a, v) / dot(v, v), 0.0, 1.0);
        float dist = length(d.xy - a - v * t);
        c *= 1.0 - innerFold * 0.32 * (1.0 - t) * ss(0.0009, 0.0001, dist);
      }
    }
  }
  // lips at the page's defaults
  float mid = (lipU + lipLo) / 2.0 - 0.001;
  float lt = (v_pos.x - lipX.x) / (lipX.y - lipX.x);
  if (lipN > 0) {
    // lips drawn from an outline: lipCurve holds the top edge, the bottom edge and the line between the lips, at even steps from one corner to the other
    if (p.z > cornerZ - 0.012 && lt > -0.05 && lt < 1.05) {
      float f = clamp(lt, 0.0, 1.0) * float(lipN - 1); int i = min(int(f), lipN - 2);
      vec3 cv = mix(lipCurve[i], lipCurve[i + 1], f - float(i));
      float y = v_pos.y, inside = ss(cv.y - 0.0012, cv.y + 0.0012, y) * ss(cv.x + 0.0006, cv.x - 0.0006, y) * ss(-0.03, 0.03, lt) * ss(1.03, 0.97, lt);
      // lipMask draws only where the paint lies (red) and where its line between the lips runs (green), for measuring
      if (lipMask > 0.5) { frag = vec4(inside, ss(0.0012, 0.0002, abs(y - cv.z)) * inside, 0.0, 1.0); return; }
      // the upper lip has its own colour, and lipMix says how much of the skin the lip colour covers
      vec3 lc = y > cv.z ? lipCU : lipC;
      c = mix(c, lc, inside * lipMix);
      c = mix(c, lc * vec3(0.55, 0.45, 0.45), ss(0.0014, 0.0003, abs(y - cv.x)) * inside * 0.24);
      c = mix(c, lc * vec3(0.5, 0.4, 0.4), ss(0.0012, 0.0002, abs(y - cv.z)) * inside * 0.5);
      c = mix(c, lc * vec3(0.45, 0.38, 0.38), ss(0.12, 0.0, min(lt, 1.0 - lt)) * inside * 0.3);
    }
  }
  if (lipN > 0 && lipMask > 0.5) { frag = vec4(0.0, 0.0, 0.0, 1.0); return; }
  else if (lipN == 0 && p.z > cornerZ - 0.012 && ax < cx + 0.002) {
    float e = sqrt(max(0.0, 1.0 - (ax / cx) * (ax / cx)));
    float top = mid + 0.006 * e + 0.001 * (exp(-pow(ax - 0.005, 2.0) / (2.0 * 0.0025 * 0.0025)) - 0.9 * exp(-ax * ax / (2.0 * 0.0018 * 0.0018)));
    float bot = mid - 0.0075 * e;
    float inside = ss(bot - 0.0006, bot + 0.0006, p.y) * ss(top + 0.0006, top - 0.0006, p.y) * ss(cx + 0.002, cx - 0.003, ax);
    c = mix(c, lipC, inside * 0.44);
    float edge = min(abs(p.y - top), abs(p.y - bot));
    c = mix(c, lipC * vec3(0.55, 0.45, 0.45), ss(0.0014, 0.0003, edge) * inside * 0.24);
    c = mix(c, lipC * vec3(0.5, 0.4, 0.4), ss(0.0012, 0.0002, abs(p.y - mid)) * inside * 0.5);
    c = mix(c, lipC * vec3(0.45, 0.38, 0.38), ss(cx - 0.006, cx, ax) * inside * 0.3);
  }
  // eyebrows: the page's tapered stroke through three points
  if (browN > 0) {
    // eyebrows drawn from a measured band: browCurve holds, for each side, the x, the middle height and half the thickness at 11 even steps
    // from the inner end to the outer end, in the head's own metres as seen from the front
    for (int e = 0; e < 2; e++) {
      float x0 = browCurve[e * 11].x, x1 = browCurve[e * 11 + 10].x;
      float lt = (v_pos.x - x0) / (x1 - x0);
      if (lt < -0.05 || lt > 1.05 || v_pos.z < browL.z - 0.035) continue;
      float f = clamp(lt, 0.0, 1.0) * 10.0; int i = min(int(f), 9);
      vec3 cv = mix(browCurve[e * 11 + i], browCurve[e * 11 + i + 1], f - float(i));
      float half_t = cv.z * ss(-0.05, 0.06, lt) * ss(1.05, 0.85, lt);
      float d = abs(v_pos.y - cv.y);
      float hairs = 0.95 + 0.05 * sin((v_pos.x * sign(x1 - x0) + (v_pos.y - cv.y) * 0.7) * 5200.0);
      // the hair is sparse at the inner end, and its upper edge is softer than its lower edge
      float soft = v_pos.y > cv.y ? 0.0010 : mix(0.0013, 0.0003, browGroom.x);
      c = mix(c, browPaint, (browGroom.y + (1.0 - browGroom.y) * ss(0.0, 0.3, lt)) * hairs * ss(half_t + soft, max(half_t - soft, 0.0), d) * step(0.0002, half_t) * ss(0.3, 0.55, normalize(v_nrm).z));
    }
  } else if (p.z > browL.z - 0.03 && ax >= bx0 - 0.004) {
    vec2 q = vec2(ax, p.y);
    vec2 d0 = segDist(q, bp0, bp1), d1 = segDist(q, bp1, bp2);
    float d = min(d0.x, d1.x), t = d0.x <= d1.x ? d0.y * 0.5 : 0.5 + d1.y * 0.5;
    float th = thIn + (thOut - thIn) * t;
    c = mix(c, browC, ss(th, th * 0.5, d) * browDensity * 0.9);
    if (browOnly == 1) { frag = vec4(vec3(ss(th, th * 0.5, d)), 1.0); return; }
  }
  if (browOnly == 1) { frag = vec4(0.0, 0.0, 0.0, 1.0); return; }
  // liner at 0.5 on top and 0.2 below
  float r = length(vec3(ax - eyeX, (p.y - eyeY) * 1.35, (p.z - eyeZ) * 0.5));
  vec3 dark = vec3(0.08, 0.06, 0.05);
  if (p.y > eyeY) c = mix(c, dark, liner * 0.5 * ss(0.0135, 0.0115, r) * ss(0.0095, 0.0115, r));
  else c = mix(c, dark, liner * 0.2 * ss(0.0135, 0.0118, r) * ss(0.0098, 0.0118, r));
  if (lidN > 0) {
    // lashes drawn from the lid edges: lidCurve holds, for each eye, the x, the upper edge and the lower edge at 17 even steps from the outer corner
    // to the inner one. lash holds the upper lash line, the upper lashes, the lower lash line and the lower lashes, each from 0 to 1.
    for (int e = 0; e < 2; e++) {
      float x0 = lidCurve[e * 17].x, x1 = lidCurve[e * 17 + 16].x;
      float lt = (v_pos.x - x0) / (x1 - x0);
      if (lt < -0.1 || lt > 1.0 || v_pos.z < eyeL.z - 0.004) continue;
      float f = clamp(lt, 0.0, 1.0) * 16.0; int i = min(int(f), 15);
      vec3 cv = mix(lidCurve[e * 17 + i], lidCurve[e * 17 + i + 1], f - float(i));
      float up = v_pos.y - cv.y, low = cv.z - v_pos.y;
      float ends = ss(-0.01, 0.05, lt) * ss(0.95, 0.8, lt), outer = 0.6 + 0.4 * ss(0.8, 0.15, lt);
      float hairs = 0.8 + 0.2 * sin(v_pos.x * 7000.0 + up * 1500.0);
      // the crease of the upper lid: crease.x is its height above the lid's edge at the middle of the lid, crease.y its depth from 0 (a smooth
      // lid) to 1. It comes down to a third of its height toward the corners, and fades out at them. Under the fold the lid is a little shaded
      if (crease.y > 0.0 && up > 0.0 && up < crease.x + 0.003) {
        float arch = pow(sin(3.14159 * clamp(lt, 0.0, 1.0)), 0.7);
        float d = up - crease.x * (0.35 + 0.65 * arch);
        float fold = exp(-d * d / (2.0 * 0.0007 * 0.0007));
        float under = ss(0.0003, -0.0005, d) * ss(-0.0028, -0.0008, d);
        c *= 1.0 - crease.y * ss(-0.1, 0.05, lt) * ss(1.0, 0.82, lt) * (0.24 * fold + 0.14 * under);
      }
      if (up > -0.0004 && up < 0.006) {
        c = mix(c, lashC, 0.75 * lash.y * hairs * ends * pow(ss(0.0008 + 0.0034 * lash.y * outer, 0.0, up), 1.5));
        c = mix(c, lashC, 0.55 * lash.x * ends * ss(0.0022, 0.0002, up));
      }
      if (low > -0.0004 && low < 0.004) {
        c = mix(c, lashC, 0.45 * (0.3 + 0.7 * lash.w) * hairs * ends * pow(ss(0.0008 + 0.0014 * lash.w * outer, 0.0, low), 1.5));
        c = mix(c, lashC, 0.7 * lash.z * ends * ss(0.0008, 0.0002, low));
      }
    }
  }
  frag = vec4(light(c, v_nrm), 1.0);
}
"""
FRAG_EYE = """
#version 330
uniform vec3 irisC, axes, caruncle, wedge, skin, lidShade; uniform vec2 gaze; uniform float irisEdge, pupilEdge, limbus, eyeDebug, fold;
in vec3 v_nrm; in vec3 v_base; out vec4 frag;
""" + LIGHT + """
float ss(float a, float b, float t) { float x = clamp((t - a) / (b - a), 0.0, 1.0); return x * x * (3.0 - 2.0 * x); }
void main() {
  // the iris and pupil are drawn as circles in the eye's front view, so they stay round on a fitted, non-round eyeball
  vec3 w = normalize(v_base) * axes;
  float rho = length(w.xy - gaze) / 0.0123;  // the iris sits at the centre of the eye opening
  float cosA = w.z > 0.0 ? sqrt(max(0.0, 1.0 - rho * rho)) : -1.0;
  vec3 p = vec3(normalize(v_base).xy, cosA); float edge = irisEdge;
  vec3 c = lidShade.z > 0.5 ? vec3(0.885, 0.87, 0.85) : vec3(0.93, 0.902, 0.864);
  c = mix(c, irisC, ss(edge - 0.02, edge + 0.03, cosA));
  c = mix(c, irisC * 0.45, ss(edge + 0.03, edge - 0.01, cosA) * ss(edge - 0.02, edge + 0.01, cosA) * 0.5);
  // limbus darkens the iris toward its rim, as the dark ring around a real iris
  c = mix(c, irisC * 0.35, limbus * ss(edge + 0.06, edge + 0.005, cosA) * ss(edge - 0.02, edge + 0.01, cosA));
  c = mix(c, vec3(0.025), ss(pupilEdge - 0.012, pupilEdge + 0.012, cosA));
  if (caruncle.z > 0.5) {
    // caruncle.xy is the inner corner of the eye opening, from the eye's centre, and wedge.xy points from it into the opening, between the two lids.
    // The white of the eye ends in a round end, wedge.z short of the corner. The caruncle fills the corner around that end, so its edge is hollow.
    float round_end = 0.0065;
    float along = dot(w.xy - caruncle.xy, wedge.xy);
    float gap = along < wedge.z + round_end ? length(w.xy - caruncle.xy - wedge.xy * (wedge.z + round_end)) - round_end : -1.0;
    // an inner fold of skin covers the pink corner: at fold 1 none of it shows
    c = mix(c, skin * 0.9, 0.3 * ss(-0.0015, 0.0, gap) * (1.0 - fold));
    c = mix(c, skin * vec3(0.97, 0.85, 0.84), ss(-0.0002, 0.0002, gap) * (1.0 - fold));
  }
  // the upper lid and its lashes shade the eye below them. lidShade.xy is the upper lid's edge at the pupil and how fast it falls to each side
  if (lidShade.z > 0.5) c *= 1.0 - 0.3 * ss(0.0045, 0.0, lidShade.x - lidShade.y * w.x * w.x - w.y);
  frag = vec4(light(c, v_nrm), 1.0);
  if (eyeDebug > 0.5) frag = vec4(0.0, 1.0, 0.0, 1.0);  // eyeDebug draws the whole eyeball in green, for checking
}
"""


def fit_eyeball(q, inset=0.97, lo=0.0110, hi=0.0160):
    """Least squares for the axis-aligned ellipsoid x²/a² + y²/b² + z²/c² = 1 through the lid edge points q, shrunk a little so the lids overlap it."""
    A = q ** 2
    k, *_ = np.linalg.lstsq(A, np.ones(len(q)), rcond=None)
    axes = 1 / np.sqrt(np.clip(k, 1 / hi ** 2, 1 / lo ** 2))
    return (axes * inset).astype(np.float32)


def perspective(fov_deg, aspect, near, far):
    f = 1 / np.tan(np.radians(fov_deg) / 2)
    return np.array([[f / aspect, 0, 0, 0], [0, f, 0, 0], [0, 0, (far + near) / (near - far), 2 * far * near / (near - far)], [0, 0, -1, 0]])


def look_at(eye, target):
    f = target - eye
    f = f / np.linalg.norm(f)
    s = np.cross(f, [0, 1, 0])
    s /= np.linalg.norm(s)
    u = np.cross(s, f)
    m = np.eye(4)
    m[0, :3], m[1, :3], m[2, :3] = s, u, -f
    m[:3, 3] = -m[:3, :3] @ eye
    return m


class GLHead:
    def __init__(self, head, size=256, fov=26.0, dist=0.62):
        self.h = head
        self.size, self.fov, self.dist = size, fov, dist
        self.ctx = moderngl.create_standalone_context()
        self.ctx.enable(moderngl.DEPTH_TEST)
        self.prog = self.ctx.program(vertex_shader=VERT, fragment_shader=FRAG_HEAD)
        self.eprog = self.ctx.program(vertex_shader=VERT, fragment_shader=FRAG_EYE)
        n = head.n
        self.vbo_pos = self.ctx.buffer(reserve=n * 12)
        self.vbo_nrm = self.ctx.buffer(reserve=n * 12)
        self.vbo_base = self.ctx.buffer(head.base.astype("f4").tobytes())
        self.ibo = self.ctx.buffer(head.tris.astype("i4").tobytes())
        self.vao = self.ctx.vertex_array(self.prog, [(self.vbo_pos, "3f", "in_pos"), (self.vbo_nrm, "3f", "in_nrm"), (self.vbo_base, "3f", "in_base")], self.ibo)
        # a unit sphere for the eyes; in_base carries the direction from the centre
        th, ph = np.meshgrid(np.linspace(0, np.pi, 33), np.linspace(0, 2 * np.pi, 49), indexing="ij")
        sph = np.stack([np.sin(th) * np.cos(ph), np.sin(th) * np.sin(ph), np.cos(th)], -1).reshape(-1, 3).astype("f4")
        tri = []
        for i in range(32):
            for j in range(48):
                a = i * 49 + j
                tri += [a, a + 49, a + 1, a + 1, a + 49, a + 50]
        self.sph = sph
        self.e_pos = self.ctx.buffer(reserve=sph.nbytes)
        self.e_dir = self.ctx.buffer(sph.tobytes())
        self.evao = self.ctx.vertex_array(self.eprog, [(self.e_pos, "3f", "in_pos"), (self.e_dir, "3f", "in_nrm"), (self.e_dir, "3f", "in_base")], self.ctx.buffer(np.array(tri, dtype="i4").tobytes()))
        self.fbo_ms = self.ctx.framebuffer(self.ctx.renderbuffer((size, size), samples=4), self.ctx.depth_renderbuffer((size, size), samples=4))
        self.fbo = self.ctx.framebuffer(self.ctx.renderbuffer((size, size)))
        # the head as the studio's lamp sees it, for the shadow that the jaw throws on the neck
        self.lamp_prog = self.ctx.program(vertex_shader="#version 330\nuniform mat4 lampMvp; in vec3 in_pos; void main() { gl_Position = lampMvp * vec4(in_pos, 1.0); }", fragment_shader="#version 330\nvoid main() {}")
        self.lamp_vao = self.ctx.vertex_array(self.lamp_prog, [(self.vbo_pos, "3f", "in_pos")], self.ibo)
        self.lamp_tex = self.ctx.depth_texture((1024, 1024))
        self.lamp_tex.compare_func = ""
        self.lamp_tex.repeat_x = self.lamp_tex.repeat_y = False
        self.lamp_fbo = self.ctx.framebuffer(depth_attachment=self.lamp_tex)
        # dense targets so a deform is one matrix product
        T = np.zeros((len(head.targets), n * 3), dtype=np.float32)
        for ti, (idx, d) in enumerate(head.targets):
            np.add.at(T[ti].reshape(n, 3), idx, d.astype(np.float32))
        self.T = T
        key = np.round(head.base, 4)
        _, self.group = np.unique(key, axis=0, return_inverse=True)
        self.group = self.group.ravel()
        L = head.landmarks
        for k, v in {"eyeL": L["eyeL"], "lipUL": [L["lipUpper"][1], L["lipLower"][1], 0], "cornerL": L["cornerL"], "noseTip": L["noseTip"], "browL": L["browL"]}.items():
            self.prog[k].value = tuple(float(x) for x in v)
        self.prog["chinY"].value = float(L["chin"][1])
        self.prog["browOnly"].value = 0
        self.prog["lipN"].value = 0
        self.prog["socket"].value = 0.5  # the painted shade in the eye sockets, at the page's default
        self.prog["nostril"].value = 0.5  # the painted shade under the nose, at the page's default
        self.prog["lidN"].value = 0
        self.prog["browN"].value = 0
        self.prog["liner"].value = 1.0  # the dark band painted along the lids, at the page's default
        self.prog["skin"].value = tuple(srgb(SKIN))
        self.set_lip_colour(srgb(LIP))
        self.eprog["irisC"].value = tuple(srgb(EYE))
        self.eprog["limbus"].value = 0.0  # the page has no dark iris rim
        self.eprog["eyeDebug"].value = 0.0
        self.prog["canthus"].value = 0.0  # the page does not paint the lids' inner faces
        self.set_inner_fold(0.0)
        self.set_crease(0.0, 0.0)
        self.set_eye_paint(0.5, 0.5)
        self.set_flash(0)

    def set_lips(self, curve=None):
        """Draws the lips from an outline in place of the page's fixed lip shape. curve is up to 65 rows of x, top edge y, bottom edge y and the y of the line between the lips, in the head's own metres. None brings the fixed shape back."""
        if curve is None:
            self.prog["lipN"].value = 0
            return
        self.prog["lipN"].value = len(curve)
        self.prog["lipX"].value = (float(curve[0, 0]), float(curve[-1, 0]))
        rows = np.zeros((65, 3), "f4")
        rows[:len(curve)] = curve[:, 1:]
        self.prog["lipCurve"].write(rows.tobytes())

    def set_crease(self, height_mm, depth):
        """The crease of the upper lid: its height above the lid's edge at the middle of the lid in mm, and its depth from 0 (a smooth lid,
        no crease) to 1 (a deep fold). The page draws no crease."""
        self.prog["crease"].value = (float(height_mm) / 1000, float(depth))

    def set_inner_fold(self, amount):
        """The fold of skin over the inner corner of the eye, from 0 (none: the pink corner shows) to 1 (it hides the pink corner)."""
        self.prog["innerFold"].value = float(amount)
        self.eprog["fold"].value = float(amount)

    def set_lip_colour(self, lower, upper=None, cover=0.44):
        """The colour of the painted lips, in sRGB from 0 to 1: the lower lip's, and the upper lip's if it differs. cover is how much of the
        skin the lip colour covers, 0.44 as on the page."""
        self.prog["lipC"].value = tuple(float(v) for v in lower)
        self.prog["lipCU"].value = tuple(float(v) for v in (lower if upper is None else upper))
        self.prog["lipMix"].value = float(cover)

    def set_brow_paint(self, rows=None, colour=(0.2, 0.14, 0.1), groom=(0.5, 0.6)):
        """Draws the eyebrows from a band in place of the page's stroke. rows holds, for the left side and then the right, 11 rows of x, middle
        height and half the thickness from the inner end to the outer end, in the head's own metres. groom holds two settings from 0 to 1: how cleanly the lower edge is trimmed,
        and how full the inner end is. None brings the page's stroke back."""
        self.prog["browN"].value = 0 if rows is None else 11
        if rows is not None:
            self.prog["browCurve"].write(np.ascontiguousarray(np.vstack([rows["L"], rows["R"]]), dtype="f4").tobytes())
            self.prog["browPaint"].value = tuple(float(x) for x in colour)
            self.prog["browGroom"].value = tuple(float(x) for x in groom)

    def set_lashes(self, lids=None, lash=(0.5, 0.5, 0.2, 0.2), colour=(0.10, 0.07, 0.06)):
        """Draws lashes from the lid edges in place of the page's fixed liner band, and shades the white of the eye under the upper lid.
        lids holds, for the left eye and then the right, 17 rows of x, upper edge y and lower edge y from the outer corner to the inner one,
        in the head's own metres. lash is the upper lash line, the upper lashes, the lower lash line and the lower lashes, each from 0 to 1.
        None brings the page's paint back."""
        self.lids = lids
        self.prog["lidN"].value = 0 if lids is None else 17
        self.prog["liner"].value = 1.0 if lids is None else 0.0
        if lids is not None:
            self.prog["lidCurve"].write(np.ascontiguousarray(np.vstack([lids["L"], lids["R"]]), dtype="f4").tobytes())
            self.prog["lash"].value = tuple(float(x) for x in lash)
            self.prog["lashC"].value = tuple(float(x) for x in colour)

    def set_flash(self, amount):
        """0 is the page's light, from above. 1 is an even light from the camera's side, as in a studio portrait."""
        self.prog["flash"].value = float(amount)
        self.eprog["flash"].value = float(amount)
        self.prog["shadow"].value = float(amount)
        self.eprog["shadow"].value = 0.0  # the eyeballs keep their own painted shade under the upper lid

    def set_eye_paint(self, iris, pupil):
        """The page's formulas: iris 0 to 1 widens the iris, pupil 0 to 1 widens the pupil."""
        self.eprog["irisEdge"].value = float(0.88 - iris * 0.1)
        self.eprog["pupilEdge"].value = float(0.965 + (0.5 - pupil) * 0.03)

    def weights_vector(self, values, gender, age, regions):
        w = np.zeros(len(self.h.targets), dtype=np.float32)
        for ti, x in self.h.target_weights(values, gender, age, regions).items():
            w[ti] = x
        return w

    def positions(self, w):
        return self.h.base.astype(np.float32) + (w @ self.T).reshape(-1, 3)

    def normals(self, pos):
        t = self.h.tris
        fn = np.cross(pos[t[:, 1]] - pos[t[:, 0]], pos[t[:, 2]] - pos[t[:, 0]])
        acc = np.zeros((self.group.max() + 1, 3), dtype=np.float32)
        for k in range(3):
            np.add.at(acc, self.group[t[:, k]], fn)
        n = acc[self.group]
        return n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-12)

    def eyeball_axes(self, pos, side, centre):
        """The eyeball's three semi-axes, fitted so its surface passes just inside the current edge of the lids."""
        if not hasattr(self, "_margin"):
            from eye_targets import lid_margin
            self._margin = {sd: lid_margin(self.h.base, self.h.eye_centres(self.h.base)[sd]) for sd in "LR"}
        return fit_eyeball(pos[self._margin[side]] - centre)

    def opening_offset(self, pos, side, centre):
        """How far the middle of the eye opening sits from the eyeball's centre, sideways, in metres.

        Aiming the iris there keeps a straight-ahead gaze centred in the opening whatever the lid sliders do.
        """
        if not hasattr(self, "_margin"):
            self.eyeball_axes(pos, side, centre)
        x = pos[self._margin[side], 0] - centre[0]
        return float(np.clip((x.max() + x.min()) / 2, -0.004, 0.004))

    def set_brows(self, brow):
        b = dict(BROW_DEFAULT, **(brow or {}))
        bl = self.h.landmarks["browL"]
        bx0, blen = 0.011 + b["browSpacing"] * 0.006, 0.040 + b["browLength"] * 0.012
        pts = [(bx0, bl[1] - 0.004 + b["browInner"] * 0.008), (bx0 + blen * (0.55 + b["browArchPos"] * 0.25), bl[1] + 0.003 + b["browArch"] * 0.008), (bx0 + blen, bl[1] - 0.004 + b["browOuter"] * 0.010)]
        if "stroke" in b:
            # the brow as three points and two thicknesses in the base head's own metres, in place of the sliders
            pts, th_in, th_out = b["stroke"]
            bx0 = pts[0][0]
        for k, v in zip(("bp0", "bp1", "bp2"), pts):
            self.prog[k].value = tuple(float(x) for x in v)
        self.prog["bx0"].value = float(bx0)
        self.prog["thIn"].value = float(th_in if "stroke" in b else 0.0025 + b["browThickIn"] * 0.005)
        self.prog["thOut"].value = float(th_out if "stroke" in b else 0.0012 + b["browThickOut"] * 0.004)
        self.prog["browDensity"].value = float(b["browDensity"])
        self.prog["browC"].value = tuple(x * 0.75 for x in srgb(HAIRS[int(b["browColor"])]))

    def brow_mask(self, values, gender, age, regions, brow, yaw=0.0, pitch=0.0):
        """The painted brow stroke alone, as a 0 to 1 image in the same frame as render()."""
        self.prog["browOnly"].value = 1
        try:
            img, _ = self.render(values, gender, age, regions, brow=brow, yaw=yaw, pitch=pitch, eyes=False)
        finally:
            self.prog["browOnly"].value = 0
        return img[..., 0].astype(np.float32) / 255

    def render(self, values, gender=0.0, age=0.0, regions=(1 / 3,) * 3, brow=None, pos=None, yaw=0.0, pitch=0.0, eyes=True):
        """Returns the RGB image and the five ArcFace points (pupils, nose tip, mouth corners), from the page's front view.

        yaw turns the head to the character's left and pitch tilts the chin up, in degrees, around the point between the eyes.
        """
        if pos is None:
            pos = self.positions(self.weights_vector(values, gender, age, regions))
        unturn, centre = np.eye(3), np.zeros(3)
        if yaw or pitch:
            e = self.h.eye_centres(pos)
            c = (e["L"] + e["R"]) / 2
            centre = c
            cy, sy = np.cos(np.radians(yaw)), np.sin(np.radians(yaw))
            cp, sp = np.cos(np.radians(-pitch)), np.sin(np.radians(-pitch))
            Ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
            Rx = np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]])
            pos = ((pos - c) @ (Ry @ Rx).T + c).astype(np.float32)
            unturn = (Ry @ Rx).T
        # the painted lips are laid out on the head as it faces the front, so the shader gets the way back from a turned head
        self.prog["unturn"].write(np.ascontiguousarray(unturn.T, dtype="f4").tobytes())
        self.prog["turnC"].value = tuple(float(x) for x in centre)
        self.set_brows(brow)
        jl, jr = self.h.eye_joints["L"], self.h.eye_joints["R"]
        el, er = pos[jl].mean(0), pos[jr].mean(0)
        target = (el + er) / 2 - np.array([0, 0.03, 0])
        eye = target + np.array([0, 0, self.dist])
        fov = np.degrees(2 * np.arctan(np.tan(np.radians(self.fov) / 2) * 0.62 / self.dist))
        mvp = (perspective(fov, 1, 0.05, 10) @ look_at(eye, target)).astype("f4")
        self.prog["mvp"].write(mvp.T.tobytes())
        self.eprog["mvp"].write(mvp.T.tobytes())
        self.vbo_pos.write(pos.astype("f4").tobytes())
        self.vbo_nrm.write(self.normals(pos).astype("f4").tobytes())
        # the lamp stands above the camera and shines down on the face at 35 degrees
        lamp_eye = target + np.array([0, np.sin(np.radians(35)), np.cos(np.radians(35))])
        box = np.diag([1 / 0.22, 1 / 0.22, -2 / 1.0, 1.0])
        box[2, 3] = -2.0
        lamp = (box @ look_at(lamp_eye, target)).astype("f4")
        for prog in (self.prog, self.eprog, self.lamp_prog):
            prog["lampMvp"].write(lamp.T.tobytes())
        self.lamp_fbo.use()
        self.lamp_fbo.viewport = (0, 0, 1024, 1024)
        self.ctx.clear(depth=1.0)
        self.lamp_vao.render()
        self.lamp_tex.use(0)
        self.prog["lampDepth"].value = 0
        self.eprog["lampDepth"].value = 0
        self.fbo_ms.use()
        self.ctx.clear(0.063, 0.071, 0.086, 1.0)
        self.vao.render()
        for side, c in ((("L", el), ("R", er)) if eyes else ()):
            ax = self.eyeball_axes(pos, side, c)
            self.eprog["axes"].value = tuple(float(x) for x in ax)
            # iris_shift, when set, places each iris sideways by hand, in metres from the eyeball's centre
            shift = self.iris_shift[side] if getattr(self, "iris_shift", None) else self.opening_offset(pos, side, c)
            self.eprog["gaze"].value = (float(shift), 0.0)
            spot = self.caruncle[side] if getattr(self, "caruncle", None) else None
            self.eprog["caruncle"].value = (float(spot[0]), float(spot[1]), 1.0) if spot is not None else (0.0, 0.0, 0.0)
            lids = getattr(self, "lids", None)
            if lids is None:
                self.eprog["lidShade"].value = (0.0, 0.0, 0.0)
            else:
                x, top = lids[side][:, 0] - c[0], lids[side][:, 1] - c[1]
                k = np.polyfit(x, top, 2)
                self.eprog["lidShade"].value = (float(np.polyval(k, 0)), float(-k[0]), 1.0)
            if spot is not None:
                self.eprog["wedge"].value = tuple(float(x) for x in spot[2:5])
            if spot is not None:
                self.eprog["skin"].value = self.prog["skin"].value
            self.e_pos.write((c + self.sph * ax).astype("f4").tobytes())
            self.evao.render()
        self.ctx.copy_framebuffer(self.fbo, self.fbo_ms)
        img = np.frombuffer(self.fbo.read(components=3), dtype=np.uint8).reshape(self.size, self.size, 3)[::-1]
        return img.copy(), self.five(pos, mvp)

    def five(self, pos, mvp):
        import json, os
        if not hasattr(self, "_corr"):
            self._corr = {o["name"]: o["vertex"] for o in json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "correspondence.json")))["landmarks"]}
        c = self._corr
        e = self.h.eye_centres(pos)
        P = np.array([e["R"], e["L"], pos[c["noseTip"]], pos[c["mouthCornerR"]], pos[c["mouthCornerL"]]], dtype=np.float64)
        h = np.c_[P, np.ones(5)] @ mvp.T.astype(np.float64)
        ndc = h[:, :2] / h[:, 3:]
        return np.stack([(ndc[:, 0] + 1) / 2 * self.size, (1 - ndc[:, 1]) / 2 * self.size], 1).astype(np.float32)
