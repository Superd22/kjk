// The nose page. The heads are built by the Rust crate head_sliders compiled to WebAssembly (/wasm/head_sliders), the same code as
// the bench that made the numbers. Everything is drawn in the frame of each head's eyes, in millimetres on a face whose eyes are 63 mm
// apart: x across, y up, z out of the face.
import { loadCrate, fromBase64, timed } from "/wasm/load.js?v=22";

const D = "data/";
// the eyes' "Proposed default eyes" button sets these sliders to these values (the lead may change them)
const PROPOSED_EYES = { "Eyeball size": 1.0 };
// the eyeball's radius in mm when its slider is at 0; the slider shows the radius itself
const BALL_MM = 12.3;
// the version of the head_eyes browser module, and the lash strip's size in mm, as on the lid piece page
const EYES_MODULE_VER = "22";
const EYE_LASH_LENGTH = 1.2, EYE_LASH_THICKNESS = 0.25;
// bumped whenever the data or the module is rebuilt, so the browser does not keep old copies
const VER = "?b=48";
const SAVE = "http://127.0.0.1:8766";
const $ = (id) => document.getElementById(id);
// the bench rewrites the data while the page is open, so every file is checked against the server before a cached copy is used
const fetchOk = async (u) => { const r = await fetch(u, { cache: "no-cache" }); if (!r.ok) throw new Error(`${u}: ${r.status}`); return r; };
const getJSON = async (u) => (await fetchOk(u)).json();
const getText = async (u) => (await fetchOk(u)).text();
const fetchTyped = async (u, Type) => new Type(await (await fetchOk(u)).arrayBuffer());
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

const page = await getJSON(D + "page.json" + VER);
const heads = page.heads;
const headIndex = Object.fromEntries(heads.map((h, i) => [h.name, i]));
const P = page.measure.proportions;
const O = page.measure.outlines;
const marks = page.measure.marks;
const base = await fetchTyped(D + "base.bin" + VER, Float32Array);
const tris = await fetchTyped(D + "triangles.bin" + VER, Uint32Array);
const keep = await fetchTyped(D + "keep.bin" + VER, Uint8Array);
const layoutRaw = await getJSON(D + "layout.json" + VER);
const layout = { n: layoutRaw.n, uv: fromBase64(layoutRaw.uv, Float32Array), tris: fromBase64(layoutRaw.tris, Uint32Array), vmap: fromBase64(layoutRaw.vmap, Uint32Array) };
const wasm = await loadCrate("head_sliders", "/wasm", VER.slice(3));
const VC = base.length / 3;
// the 57 stand-in faces of a kit, one file each: the nose's, the jaw's, and the ones with both features taken out
const startsCache = {};
async function startOf(i, file = "nose/starts.bin") {
  startsCache[file] ??= fetchTyped(D + file + VER, Float32Array);
  const all = await startsCache[file];
  return all.subarray(i * VC * 3, (i + 1) * VC * 3);
}
const baseEyes = new Float64Array(page.base_eyes.flat());
const label = (h) => (h.title ? `${h.title} (${h.name})` : h.name);
const fmt = (v, d = 1) => (v == null || Number.isNaN(v) ? "–" : v.toFixed(d));
const unit = (p) => (p.unit === "degrees" ? "°" : p.unit === "share" ? " %" : " mm");
// a measure that is a share of the eye's width is shown as a percentage
const scaleOf = (p) => (p && p.unit === "share" ? 100 : 1);
const fmtP = (v, p, d = 1) => fmt(v == null ? v : v * scaleOf(p), d);
const unitWord = (p) => (p.unit === "degrees" ? "degrees" : p.unit === "share" ? "percent" : "mm");

// short names for the measure chips, and the view that shows each measure best
const SHORT = { width_wings: "wings", width_bridge: "bridge", width_tip: "tip width", length: "length", projection: "tip out", root: "root", tip_turn: "tip turn", dorsum: "bridge line", flare: "flare", bridge_height: "bridge height", round_profile: "tip round", round_below: "tip blunt", root_dip: "root dip", bend: "bridge bend", profile: "profile", front: "front", base: "base", side: "side", jaw_width: "jaw width", jaw_low: "jaw near chin", jaw_slope: "jaw slope", chin_bottom: "chin down", chin_width: "chin width", chin_projection: "chin out", under_angle: "under chin", brow_projection: "brow depth", brow_height: "brow edge height", brow_slant: "brow slant", brow_middle: "brow middle", forehead_slope: "forehead slope", forehead_curve: "forehead round", temple_width: "temples", brow_sweep: "brow sweep", brow_bulge: "brow bulge", brow_line: "brow line", forehead_profile: "forehead profile", cheek_width: "cheekbone width", cheekbone_projection: "cheekbones out", cheek_fullness: "cheek fullness", cheek_side: "lower cheek width", cheek_profile: "cheek profile", face_front: "face outline", width: "width", height: "height", tilt: "tilt", upper_rise: "upper arch", lower_drop: "lower curve", upper_peak: "upper peak", lower_low: "lower low point", inner_angle: "inner corner", outer_angle: "outer corner", upper_depth: "upper lid depth", lower_depth: "lower lid depth", upper: "upper lid", lower: "lower lid", ball: "eyeball size", upper_inner: "upper lid inner", upper_outer: "upper lid outer", lower_inner: "lower lid inner", lower_outer: "lower lid outer", inner_height: "inner corner height", outer_lift: "outer corner lift" };
const VIEW_FOR = { width_wings: "front", width_bridge: "front", flare: "front", width_tip: "below", round_below: "below" };
const viewFor = (key) => VIEW_FOR[key] || "side";
const measureKeys = (pg) => [...pg.measure.proportions.map((p) => p.key), ...pg.measure.outlines.map((o) => o.key)];
const measureLabels = (pg) => [...pg.measure.proportions.map((p) => p.label), ...pg.measure.outlines.map((o) => o.label)];

// ---------- the kits of sliders: the nose's two rounds, and the jaw and chin, each loaded when first needed ----------
const FEATURES = {
  nose: { key: "nose", title: "Nose", plural: "noses", one: "nose", crops: "kotor_nose" },
  jaw: { key: "jaw", title: "Jaw and chin", plural: "jaws and chins", one: "jaw and chin", crops: "kotor_jaw" },
  brow: { key: "brow", title: "Brow and forehead", plural: "brows and foreheads", one: "brow and forehead", crops: "kotor_brow" },
  cheeks: { key: "cheeks", title: "Cheekbones and cheeks", plural: "cheekbones and cheeks", one: "cheekbones and cheeks", crops: "kotor_cheeks" },
  eyes: { key: "eyes", title: "Eyes", plural: "eyes", one: "eyes", crops: "kotor_eyes" },
};
const FEATURE_ITEMS = [["nose", "Nose"], ["jaw", "Jaw and chin"], ["brow", "Brow and forehead"], ["cheeks", "Cheekbones and cheeks"], ["eyes", "Eyes"]];
const ROUNDS = {
  r1: { feature: "nose", label: "Round 1 sliders", page: "page.json", sliders: "nose/sliders_r1.json", made: "nose/made_r1", startsFile: "nose/starts.bin", solvedMode: "learned", modes: [["held_out", "Learned without this head"], ["learned", "Learned with this head"]] },
  r2: { feature: "nose", label: "Round 2 sliders", page: "nose/page_r2.json", sliders: "nose/sliders_r2.json", made: "nose/made_r2", startsFile: "nose/starts.bin", solvedMode: "full", modes: [["held_out", "Ranges set without this head"], ["full", "Ranges set from all heads"]] },
  jaw: { feature: "jaw", label: "Jaw and chin sliders", page: "jaw/page.json", sliders: "jaw/sliders.json", made: "jaw/made", startsFile: "jaw/starts.bin", solvedMode: "full", modes: [["held_out", "Ranges set without this head"], ["full", "Ranges set from all heads"]] },
  brow: { feature: "brow", label: "Brow and forehead sliders", page: "brow/page.json", sliders: "brow/sliders.json", made: "brow/made", startsFile: "brow/starts.bin", solvedMode: "full", modes: [["held_out", "Ranges set without this head"], ["full", "Ranges set from all heads"]] },
  cheeks: { feature: "cheeks", label: "Cheekbones and cheeks sliders", page: "cheeks/page.json", sliders: "cheeks/sliders.json", made: "cheeks/made", startsFile: "cheeks/starts.bin", solvedMode: "full", modes: [["held_out", "Ranges set without this head"], ["full", "Ranges set from all heads"]] },
  eyes: { feature: "eyes", label: "Eye sliders", page: "eyes/page.json", sliders: "eyes/sliders.json", made: "eyes/made", startsFile: "eyes/starts.bin", solvedMode: "full", modes: [["held_out", "Ranges set without this head"], ["full", "Ranges set from all heads"]] },
};
const sliderMeta = (s) => ({ name: s.name, hint: s.hint, group: s.group, measure: s.measure, unit: s.unit, kind: s.kind, low: s.low, high: s.high });
const roundCache = {};
function round(k) {
  return (roundCache[k] ??= (async () => {
    const R = ROUNDS[k];
    const text = await getText(D + R.sliders + VER);
    const meta = JSON.parse(text).sliders.map(sliderMeta);
    const pg = k === "r1" ? page : await getJSON(D + R.page + VER).catch(() => null);
    const [builder, prepMs] = timed(() => new wasm.Builder(base, tris, keep, text));
    const byName = pg ? Object.fromEntries(pg.heads.map((h) => [h.name, h])) : {};
    const counts = pg?.counts ? pg.counts.map(String) : ["4", "6", "8", "10", "12"];
    // the results list their sliders in their own order; values are matched to the set's sliders by name, so a set that gained a
    // slider since the results were written still loads them
    const place = pg?.sliders ? pg.sliders.map((s) => meta.findIndex((m) => m.name === s.name)) : null;
    if (pg?.sliders) pg.sliders.forEach((s, i) => place[i] >= 0 && (meta[place[i]].spread = s.spread));
    const toSet = (values) => { const out = new Float64Array(meta.length); (values || []).forEach((v, i) => { const j = place ? place[i] : i; if (j >= 0 && j < out.length) out[j] = v; }); return out; };
    const measureOf = (world, eyes) => JSON.parse(R.feature !== "nose" ? builder.measure_feature(R.feature, world, eyes) : builder.measure(world, eyes));
    return { key: k, ...R, text, meta, page: pg, builder, prepMs, byName, counts, toSet, measureOf, madeIndex: null, madeBin: null };
  })());
}
const R1 = await round("r1");
const jawKit = () => round("jaw");
// a slider's value as shown: the Eyeball size slider shows the radius itself, the others show the plain number
const fmtVal = (v, s, d = 2) => (s.measure === "ball" ? `${fmt(BALL_MM + v, d)} mm` : fmt(v, d));
const unitShort = (s) => { const m = /^1 (mm|degrees)\b/.exec(s.unit || ""); return m ? m[1] : /^1 spread/.test(s.unit || "") ? "spreads" : s.kind === "proportion" ? "mm" : ""; };

// ---------- drawing ----------
const texLoader = new THREE.TextureLoader();
const texCache = {};
const tex = (file) => (texCache[file] ??= new Promise((res) => texLoader.load(D + "tex/" + file, (t) => { t.anisotropy = 4; res(t); }, undefined, () => res(null))));

function toEye(frame, p, out) {
  const n = p.length / 3;
  const o = frame.origin, a = frame.axes, s = frame.scale;
  for (let i = 0; i < n; i++) {
    const x = p[3 * i] - o[0], y = p[3 * i + 1] - o[1], z = p[3 * i + 2] - o[2];
    for (let c = 0; c < 3; c++) out[3 * i + c] = (x * a[c][0] + y * a[c][1] + z * a[c][2]) * s;
  }
  return out;
}

// normals averaged over every copy of a vertex at the same place, so seams do not show
function weldedNormals(pos, index) {
  const n = pos.length / 3, key = new Int32Array(n), ids = new Map();
  for (let i = 0; i < n; i++) {
    const k = `${Math.round(pos[3 * i] * 100)},${Math.round(pos[3 * i + 1] * 100)},${Math.round(pos[3 * i + 2] * 100)}`;
    if (!ids.has(k)) ids.set(k, ids.size);
    key[i] = ids.get(k);
  }
  const acc = new Float32Array(3 * ids.size);
  const count = index ? index.length / 3 : n / 3;
  for (let t = 0; t < count; t++) {
    const a = index ? index[3 * t] : 3 * t, b = index ? index[3 * t + 1] : 3 * t + 1, c = index ? index[3 * t + 2] : 3 * t + 2;
    const ux = pos[3 * b] - pos[3 * a], uy = pos[3 * b + 1] - pos[3 * a + 1], uz = pos[3 * b + 2] - pos[3 * a + 2];
    const vx = pos[3 * c] - pos[3 * a], vy = pos[3 * c + 1] - pos[3 * a + 1], vz = pos[3 * c + 2] - pos[3 * a + 2];
    const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
    for (const v of [a, b, c]) { acc[3 * key[v]] += nx; acc[3 * key[v] + 1] += ny; acc[3 * key[v] + 2] += nz; }
  }
  const out = new Float32Array(3 * n);
  for (let i = 0; i < n; i++) {
    const k = key[i], x = acc[3 * k], y = acc[3 * k + 1], z = acc[3 * k + 2], l = Math.hypot(x, y, z) || 1;
    out[3 * i] = x / l; out[3 * i + 1] = y / l; out[3 * i + 2] = z / l;
  }
  return out;
}

const clayMat = () => new THREE.MeshStandardMaterial({ color: 0xb8aca0, roughness: 0.75, metalness: 0 });
const paintMat = (t) => new THREE.MeshStandardMaterial({ map: t, roughness: 0.85, metalness: 0 });
const ourPaint = async (name) => paintMat(await tex("v2/" + name + "_ours.jpg"));

// our head (positions in metres, 3 per vertex of our mesh) as a mesh in the frame of the eyes, with our texture layout
function ourGeometry(world, frame, geo) {
  const eye = toEye(frame, world, new Float32Array(world.length));
  const n = layout.n, p = new Float32Array(3 * n);
  for (let i = 0; i < n; i++) { const v = layout.vmap[i]; p[3 * i] = eye[3 * v]; p[3 * i + 1] = eye[3 * v + 1]; p[3 * i + 2] = eye[3 * v + 2]; }
  if (!geo) {
    geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(p, 3));
    geo.setAttribute("uv", new THREE.BufferAttribute(layout.uv, 2));
    geo.setAttribute("normal", new THREE.BufferAttribute(weldedNormals(p, layout.tris), 3));
    geo.setIndex(new THREE.BufferAttribute(layout.tris, 1));
  } else {
    geo.attributes.position.array.set(p);
    geo.attributes.normal.array.set(weldedNormals(p, layout.tris));
    geo.attributes.position.needsUpdate = true;
    geo.attributes.normal.needsUpdate = true;
  }
  return geo;
}

const kotorCache = {};
function kotorGroup(name, painted = true, folder = "kotor_nose") {
  const key = folder + ":" + name + (painted ? "" : ":clay");
  return (kotorCache[key] ??= (async () => {
    const mats = await getJSON(D + folder + "/" + name + ".json" + VER);
    const g = new THREE.Group();
    for (const m of mats) {
      const geo = new THREE.BufferGeometry();
      const p = fromBase64(m.pos, Float32Array);
      geo.setAttribute("position", new THREE.BufferAttribute(p, 3));
      geo.setAttribute("uv", new THREE.BufferAttribute(fromBase64(m.uv, Float32Array), 2));
      geo.setAttribute("normal", new THREE.BufferAttribute(weldedNormals(p, null), 3));
      const mat = painted && m.texture ? paintMat(await tex(m.texture)) : clayMat();
      mat.side = THREE.DoubleSide;
      g.add(new THREE.Mesh(geo, mat));
    }
    return g;
  })());
}

// ---------- the eyeballs and the generated upper lid piece, drawn wherever the eyes are shown ----------
// The pieces are made by the browser module head_eyes, the same code as the lid piece page. Everything here is in the world frame
// of the head (metres) and is put into the frame of the eyes (mm) by one matrix.
let lidKit = null, lidKitReady = null;
const lidMaker = () => (lidKit ??= (async () => {
  const [m, edges] = await Promise.all([loadCrate("head_eyes", "/wasm", EYES_MODULE_VER), getJSON("/lids/data/lid_edges.json")]);
  const maker = new m.LidMaker(tris, base.subarray(0, 3 * edges.vertices));
  for (const key of ["L", "R"]) {
    const e = edges.eyes[key];
    maker.add_eye(new Uint32Array(e.edge), new Uint8Array(e.lid.map((l) => (l === "upper" ? 1 : 0))), e.inner_corner, e.outer_corner, new Uint32Array(e.eyeball_centre_vertices), e.temple_is_plus_x);
  }
  const uv = new Float32Array(2 * edges.vertices);
  for (let i = layout.n - 1; i >= 0; i--) { const v = layout.vmap[i]; uv[2 * v] = layout.uv[2 * i]; uv[2 * v + 1] = layout.uv[2 * i + 1]; }
  return (lidKitReady = { maker, edges, uv });
})());
// the skin's triangles without the ones the piece replaces, as an index into the drawn mesh
let layoutKeys = null;
function cutIndex(hidden) {
  const hide = new Set();
  for (const f of hidden) { const t = [tris[3 * f], tris[3 * f + 1], tris[3 * f + 2]].sort((a, b) => a - b); hide.add((t[0] * VC + t[1]) * VC + t[2]); }
  if (!layoutKeys) {
    layoutKeys = new Float64Array(layout.tris.length / 3);
    for (let t = 0; t < layoutKeys.length; t++) {
      const k = [layout.vmap[layout.tris[3 * t]], layout.vmap[layout.tris[3 * t + 1]], layout.vmap[layout.tris[3 * t + 2]]].sort((a, b) => a - b);
      layoutKeys[t] = (k[0] * VC + k[1]) * VC + k[2];
    }
  }
  const keep = [];
  for (let t = 0; t < layoutKeys.length; t++) if (!hide.has(layoutKeys[t])) keep.push(layout.tris[3 * t], layout.tris[3 * t + 1], layout.tris[3 * t + 2]);
  return new Uint32Array(keep);
}
const showAllSkin = (geo) => { if (geo.userData.cut) { geo.setIndex(new THREE.BufferAttribute(layout.tris, 1)); geo.userData.cut = false; } };
function hideSkin(geo, hidden) { geo.setIndex(new THREE.BufferAttribute(cutIndex(hidden), 1)); geo.userData.cut = true; }
// the eye's picture, seen from the front: a white eye, a brown iris with a darker rim, and a black pupil, one eye in each half
let eyeballTexture = null;
function eyeballMap() {
  if (eyeballTexture) return eyeballTexture;
  const c = document.createElement("canvas");
  c.width = 1024; c.height = 512;
  const g = c.getContext("2d");
  g.fillStyle = "#f1eee8";
  g.fillRect(0, 0, 1024, 512);
  for (const cx of [256, 768]) {
    const grad = g.createRadialGradient(cx, 256, 20, cx, 256, 123);
    grad.addColorStop(0, "#8a6035"); grad.addColorStop(0.75, "#6b4727"); grad.addColorStop(1, "#2e1f12");
    g.fillStyle = grad;
    g.beginPath(); g.arc(cx, 256, 123, 0, 7); g.fill();
    g.fillStyle = "#050403";
    g.beginPath(); g.arc(cx, 256, 50, 0, 7); g.fill();
  }
  eyeballTexture = new THREE.CanvasTexture(c);
  return eyeballTexture;
}
function eyeCentreOf(edges, world, key) {
  const ids = edges.eyes[key].eyeball_centre_vertices, c = [0, 0, 0];
  for (const v of ids) for (let k = 0; k < 3; k++) c[k] += world[3 * v + k] / ids.length;
  return c;
}
// an eye's radius in metres, read from its 8 eyeball-centre vertices: the Eyeball size slider scales their ring (6.907 mm from its mean at
// the radius of 12.3 mm) with the radius. Viewer 2 reads it the same way.
function eyeRadiusOf(edges, world, key) {
  const ids = edges.eyes[key].eyeball_centre_vertices, c = eyeCentreOf(edges, world, key);
  let s = 0;
  for (const v of ids) for (let k = 0; k < 3; k++) s += (world[3 * v + k] - c[k]) ** 2;
  return 0.0123 * Math.sqrt(s / ids.length) / 0.006907;
}
// the caruncles, the pink wedges in the inner corners, made by head_eyes from the built head. Each piece fills the whole hollow between
// the lids and the eyeball, and its far part shows the eyeball's own picture, read as the eyeball reads it, so that only its pink part
// stands out.
function caruncleMesh(kit, world, width) {
  const t = kit.maker.caruncle_triangles(), P = [], C = [], N = [], UV = [], idx = [];
  ["L", "R"].forEach((key, e) => {
    const p = kit.maker.caruncle_positions(world, e, width), off = P.length / 3;
    const c = eyeCentreOf(kit.edges, world, key);
    for (let i = 0; i < p.length / 3; i++) {
      const d = [0, 1, 2].map((k) => p[3 * i + k] - c[k]), n = Math.hypot(...d);
      UV.push(d[0] / n / 4 + 0.25 + 0.5 * e, d[1] / n / 2 + 0.5);
    }
    for (const x of p) P.push(x);
    for (const x of kit.maker.caruncle_colours(world, e, width)) C.push(x);
    for (const x of kit.maker.caruncle_normals(world, e, width)) N.push(x);
    for (const i of t) idx.push(i + off);
  });
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(P), 3));
  g.setAttribute("color", new THREE.BufferAttribute(new Float32Array(C), 3));
  g.setAttribute("normal", new THREE.BufferAttribute(new Float32Array(N), 3));
  g.setAttribute("uv", new THREE.BufferAttribute(new Float32Array(UV), 2));
  g.setIndex(idx);
  const mat = new THREE.MeshPhysicalMaterial({ color: 0xffffff, map: eyeballMap(), vertexColors: true, roughness: 0.35, metalness: 0, clearcoat: 0.35, clearcoatRoughness: 0.3, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(g, mat);
  mesh.frustumCulled = false;
  mesh.name = "caruncles";
  return mesh;
}
// the two eyeballs, spheres at each eye's own radius, as one mesh
function eyeballMesh(edges, world) {
  const R = 20, per = (R + 1) * 2 * R;
  const P = new Float32Array(3 * 2 * per), UV = new Float32Array(2 * 2 * per), idx = [];
  ["L", "R"].forEach((key, e) => {
    const c = eyeCentreOf(edges, world, key), r = eyeRadiusOf(edges, world, key);
    for (let j = 0; j < per; j++) {
      const a = Math.floor(j / (2 * R)) * Math.PI / R, b = (j % (2 * R)) * Math.PI / R;
      const u = [Math.sin(a) * Math.cos(b), Math.sin(a) * Math.sin(b), Math.cos(a)];
      const i = e * per + j;
      for (let k = 0; k < 3; k++) P[3 * i + k] = c[k] + r * u[k];
      UV[2 * i] = u[0] / 4 + 0.25 + 0.5 * e; UV[2 * i + 1] = u[1] / 2 + 0.5;
    }
    for (let ra = 0; ra < R; ra++) for (let b = 0; b < 2 * R; b++) {
      const i0 = e * per + ra * 2 * R + b, i1 = e * per + ra * 2 * R + ((b + 1) % (2 * R));
      idx.push(i0, i1 + 2 * R, i1, i0, i0 + 2 * R, i1 + 2 * R);
    }
  });
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(P, 3));
  g.setAttribute("uv", new THREE.BufferAttribute(UV, 2));
  g.setIndex(idx);
  g.computeVertexNormals();
  const mesh = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ color: 0xffffff, map: eyeballMap(), roughness: 0.35, metalness: 0 }));
  mesh.frustumCulled = false;
  return mesh;
}
const pixelCache = new WeakMap();
function pixelsOf(t) {
  if (!t || !t.image) return null;
  if (!pixelCache.has(t)) {
    const im = t.image, c = document.createElement("canvas");
    c.width = im.width; c.height = im.height;
    const g = c.getContext("2d");
    g.drawImage(im, 0, 0);
    pixelCache.set(t, { w: c.width, h: c.height, data: g.getImageData(0, 0, c.width, c.height).data });
  }
  return pixelCache.get(t);
}
// the mean colour of a square of (2r + 1) pixels round a point of the texture, in the same numbers the skin is drawn with on this page
function sampleColour(px, u, v, r) {
  const cx = Math.round(u * (px.w - 1)), cy = Math.round((1 - v) * (px.h - 1)), acc = [0, 0, 0];
  let n = 0;
  for (let dy = -r; dy <= r; dy++) for (let dx = -r; dx <= r; dx++) {
    const x = Math.min(px.w - 1, Math.max(0, cx + dx)), y = Math.min(px.h - 1, Math.max(0, cy + dy)), o = 4 * (y * px.w + x);
    for (let k = 0; k < 3; k++) acc[k] += px.data[o + k] / 255;
    n++;
  }
  return acc.map((a) => a / n);
}
// one eye's piece and lash strip, in a group that turns about the pivot. The piece takes its colour from the skin above it, as on the lid piece page.
function pieceGroup(kit, e, map, closed) {
  const { maker, uv: uv4750 } = kit;
  const info = JSON.parse(maker.info(e));
  const pos = maker.positions(e), t = maker.triangles(e), from = maker.colour_from(e), w = maker.colour_weights(e), shade = maker.shade(e), column = maker.column(e);
  const pv = info.pivot, n = pos.length / 3;
  const local = new Float32Array(pos.length), uv = new Float32Array(2 * n), col = new Float32Array(3 * n), raw = new Float32Array(3 * n);
  for (let i = 0; i < n; i++) {
    for (let k = 0; k < 3; k++) local[3 * i + k] = pos[3 * i + k] - pv[k];
    for (let k = 0; k < 4; k++) { const v = from[4 * i + k]; uv[2 * i] += w[4 * i + k] * uv4750[2 * v]; uv[2 * i + 1] += w[4 * i + k] * uv4750[2 * v + 1]; }
  }
  const px = pixelsOf(map);
  for (let i = 0; i < n; i++) raw.set(px ? sampleColour(px, uv[2 * i], uv[2 * i + 1], 6) : [0xb8 / 255, 0xac / 255, 0xa0 / 255], 3 * i);
  const stride = column.filter((c) => c === 0).length, cols = Math.max(...column) + 1;
  for (let i = 0; i < n; i++) {
    const j = column[i], acc = [0, 0, 0];
    let ws = 0;
    for (let d = -4; d <= 4; d++) {
      const jj = j + d;
      if (jj < 0 || jj >= cols || i + d * stride >= stride * cols || i + d * stride < 0) continue;
      const wt = Math.exp(-(d * d) / 8), o = 3 * (i + d * stride);
      for (let k = 0; k < 3; k++) acc[k] += wt * raw[o + k];
      ws += wt;
    }
    for (let k = 0; k < 3; k++) col[3 * i + k] = (acc[k] / ws) * shade[i];
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(local, 3));
  g.setAttribute("color", new THREE.BufferAttribute(col, 3));
  g.setIndex(new THREE.BufferAttribute(t, 1));
  g.computeVertexNormals();
  const group = new THREE.Group();
  group.position.set(pv[0], pv[1], pv[2]);
  group.rotation.x = closed ? (info.closed_angle * Math.PI) / 180 : 0;
  const mesh = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ color: 0xffffff, vertexColors: true, roughness: 0.82, metalness: 0, side: THREE.DoubleSide }));
  mesh.frustumCulled = false;
  group.add(mesh);
  const lp = maker.lash_positions(e), lt = maker.lash_triangles(e);
  for (let i = 0; i < lp.length / 3; i++) for (let k = 0; k < 3; k++) lp[3 * i + k] -= pv[k];
  const lg = new THREE.BufferGeometry();
  lg.setAttribute("position", new THREE.BufferAttribute(lp, 3));
  lg.setIndex(new THREE.BufferAttribute(lt, 1));
  lg.computeVertexNormals();
  const lash = new THREE.Mesh(lg, new THREE.MeshStandardMaterial({ color: 0x1c1612, roughness: 0.9, metalness: 0, side: THREE.DoubleSide }));
  lash.frustumCulled = false;
  group.add(lash);
  return group;
}
// the eyeballs and both pieces of a head, in the frame of the eyes, with the skin faces they replace. `skin` is the head's mesh, which gets those faces hidden.
function eyeObjects(kit, world, frame, skin, closed, map, caruncle = 0, cover = 0) {
  const { maker, edges } = kit;
  if (maker.set_lower) maker.set_lower(cover);
  maker.build(world, 0, EYE_LASH_LENGTH, EYE_LASH_THICKNESS, 35, false, 0);
  const hidden = [...maker.hidden_faces(0), ...maker.hidden_faces(1)];
  const inner = new THREE.Group();
  inner.add(eyeballMesh(edges, world));
  inner.add(caruncleMesh(kit, world, caruncle));
  for (const e of [0, 1]) inner.add(pieceGroup(kit, e, map, closed));
  const o = frame.origin, a = frame.axes, s = frame.scale, m = new THREE.Matrix4();
  const row = (c) => [a[c][0] * s, a[c][1] * s, a[c][2] * s, -s * (a[c][0] * o[0] + a[c][1] * o[1] + a[c][2] * o[2])];
  m.set(...row(0), ...row(1), ...row(2), 0, 0, 0, 1);
  const root = new THREE.Group();
  root.matrixAutoUpdate = false;
  root.matrix.copy(m);
  root.matrixWorldNeedsUpdate = true;
  root.add(inner);
  hideSkin(skin.geometry, hidden);
  return root;
}
function disposeEyes(root) {
  root.traverse((x) => { if (x.geometry) x.geometry.dispose(); if (x.material && x.material.map !== eyeballTexture) x.material.dispose(); });
}

const VIEWS = {
  front: { label: "Front", pos: [0, -28, 400], at: [0, -28, 0], up: [0, 1, 0], half: 42 },
  three: { label: "Three-quarter", pos: [283, -30, 303], at: [0, -30, 28], up: [0, 1, 0], half: 42 },
  side: { label: "Side", pos: [400, -32, 22], at: [0, -32, 22], up: [0, 1, 0], half: 46 },
  below: { label: "From below", pos: [0, -260, 200], at: [0, -42, 26], up: [0, 0, 1], half: 27 },
  // the same four views framed for the jaw and chin, which reach from the mouth down to the throat
  jfront: { label: "Front", pos: [0, -100, 400], at: [0, -100, 0], up: [0, 1, 0], half: 84 },
  jthree: { label: "Three-quarter", pos: [283, -100, 303], at: [0, -100, 20], up: [0, 1, 0], half: 84 },
  jside: { label: "Side", pos: [400, -100, 20], at: [0, -100, 20], up: [0, 1, 0], half: 84 },
  jbelow: { label: "From below", pos: [0, -330, 150], at: [0, -100, 20], up: [0, 0, 1], half: 84 },
  // the brow and forehead reach from 40 mm below the eyes to 100 mm above them, and the cheekbones and cheeks from 95 mm below to 30 mm above
  bfront: { label: "Front", pos: [0, 30, 400], at: [0, 30, 0], up: [0, 1, 0], half: 90 },
  bthree: { label: "Three-quarter", pos: [283, 30, 303], at: [0, 30, 20], up: [0, 1, 0], half: 115 },
  bside: { label: "Side", pos: [400, 30, 20], at: [0, 30, 20], up: [0, 1, 0], half: 90 },
  babove: { label: "From above", pos: [0, 330, 40], at: [0, 40, -10], up: [0, 0, 1], half: 100 },
  cfront: { label: "Front", pos: [0, -32, 400], at: [0, -32, 0], up: [0, 1, 0], half: 90 },
  cthree: { label: "Three-quarter", pos: [283, -32, 303], at: [0, -32, 20], up: [0, 1, 0], half: 115 },
  cside: { label: "Side", pos: [400, -32, 20], at: [0, -32, 20], up: [0, 1, 0], half: 90 },
  cbelow: { label: "From below", pos: [0, -330, 150], at: [0, -50, 20], up: [0, 0, 1], half: 90 },
  // the eyes sit at y = 0 with their centres 31.5 mm either side of the middle: one eye from the front, both eyes from the front, and one eye from the three-quarter side
  efront: { label: "One eye from the front", pos: [31.5, 0, 400], at: [31.5, 0, 0], up: [0, 1, 0], half: 22 },
  ewide: { label: "Both eyes from the front", pos: [0, 0, 400], at: [0, 0, 0], up: [0, 1, 0], half: 55 },
  ethree: { label: "Three-quarter", pos: [303, 0, 285], at: [25, 0, 10], up: [0, 1, 0], half: 40 },
};
// the views a feature offers, when it does not offer the usual four
const VIEW_SLOTS = { eyes: ["front", "wide", "three"] };
const VIEW_ITEMS = ["front", "three", "side", "below"].map((k) => [k, VIEWS[k].label]);
// the view of a feature: the nose's own framing, or the jaw's
const VIEW_PREFIX = { jaw: "j", brow: "b", cheeks: "c", eyes: "e" };
// the fourth view is from above for the brow and from below for the others
const viewKey = (feature, view) => viewKeyOf(feature, VIEW_SLOTS[feature] ? (VIEW_SLOTS[feature].includes(view) ? view : "front") : view === "wide" ? "three" : view);
const viewKeyOf = (feature, view) => (VIEW_PREFIX[feature] || "") + (feature === "brow" && view === "below" ? "above" : feature === "cheeks" && view === "above" ? "below" : view);
const viewItems = (feature) => (VIEW_SLOTS[feature] || ["front", "three", "side", "below"]).map((k) => [k, VIEWS[viewKey(feature, k)].label]);
function camera(view, aspect = 1) {
  const v = VIEWS[view];
  const c = new THREE.OrthographicCamera(-v.half * aspect, v.half * aspect, v.half, -v.half, 1, 2000);
  c.position.set(...v.pos); c.up.set(...v.up); c.lookAt(...v.at); c.updateMatrixWorld(); c.updateProjectionMatrix();
  return c;
}
function lights(scene, cam) {
  scene.add(new THREE.HemisphereLight(0xffffff, 0x4a4540, 0.35));
  const d = new THREE.DirectionalLight(0xffffff, 0.85);
  const dir = new THREE.Vector3().subVectors(cam.position, new THREE.Vector3(...VIEWS.front.at)).normalize();
  d.position.copy(dir.multiplyScalar(100).add(new THREE.Vector3(-40, 80, 0)));
  scene.add(d);
}

const SIZE = 300;
const BG = 0x8c8a86;
const renderers = {};
function rendererOf(size) {
  if (renderers[size]) return renderers[size];
  const r = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
  r.setSize(size, size);
  r.setPixelRatio(1);
  r.outputEncoding = THREE.sRGBEncoding;
  return (renderers[size] = r);
}

// draws objects into a canvas from a view, then the marks (points and lines in the frame of the eyes)
function snap(canvas, objects, view, overlay, size = SIZE) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(BG);
  const cam = camera(view);
  lights(scene, cam);
  for (const o of objects) scene.add(o);
  const renderer = rendererOf(size);
  renderer.render(scene, cam);
  canvas.width = size; canvas.height = size;
  const ctx = canvas.getContext("2d");
  ctx.drawImage(renderer.domElement, 0, 0);
  for (const o of objects) scene.remove(o);
  if (overlay) drawOverlay(ctx, cam, overlay, size);
}
function project(cam, p, size) {
  const v = new THREE.Vector3(...p).project(cam);
  return [(v.x + 1) / 2 * size, (1 - v.y) / 2 * size];
}
function drawOverlay(ctx, cam, ov, size) {
  ctx.lineWidth = 2;
  for (const line of ov.lines || []) {
    ctx.strokeStyle = line.color || "#ffd34d";
    ctx.beginPath();
    line.points.forEach((p, i) => { const [x, y] = project(cam, p, size); i ? ctx.lineTo(x, y) : ctx.moveTo(x, y); });
    ctx.stroke();
  }
  for (const p of ov.points || []) {
    const [x, y] = project(cam, p.at, size);
    ctx.fillStyle = p.color || "#ffd34d"; ctx.strokeStyle = "#000";
    ctx.beginPath(); ctx.arc(x, y, p.r || 4, 0, 7); ctx.fill(); ctx.stroke();
  }
}

// what the measure finds, for one proportion, from a measured nose
function overlayFor(key, nose) {
  if (!nose) return null;
  const m = nose.marks, Y = "#ffd34d", C = "#5be0ff";
  const val = (k) => nose.proportions[P.findIndex((p) => p.key === k)];
  const prof = (nose.outlines?.[0] || []).map((q) => [0, q[0], q[1]]);
  switch (key) {
    case "width_wings": case "flare": return { points: m.wings.map((w) => ({ at: w })), lines: [{ points: m.wings }] };
    case "width_bridge": return { points: m.bridge.map((b) => ({ at: [b[0], b[1], 60] })), lines: [{ points: m.bridge.map((b) => [b[0], b[1], 60]) }] };
    case "width_tip": case "round_below": case "round_profile": return { points: [{ at: m.tip }] };
    case "length": return { points: [{ at: m.base }], lines: [{ points: [[0, 0, 40], [0, 0, -10]], color: C }, { points: [[0, 0, 30], [0, m.base[1], 30]], color: Y }] };
    case "projection": return { points: [{ at: m.tip }, { at: [0, m.wings[1][1], m.tip[2] - val("projection")], color: C }] };
    case "root": return { points: [{ at: [0, 0, val("root")] }], lines: [{ points: prof, color: C }] };
    case "tip_turn": return { points: [{ at: m.tip }, { at: m.base }], lines: [{ points: [m.base, m.tip] }] };
    case "dorsum": case "bridge_height": return { lines: [{ points: prof, color: C }], points: [{ at: m.tip }] };
    default: return null;
  }
}

// draws a list of jobs one at a time as their cards come into view
const queue = [];
let busy = false;
const watcher = new IntersectionObserver((es) => { for (const e of es) if (e.isIntersecting && e.target._job) { queue.push(e.target._job); e.target._job = null; watcher.unobserve(e.target); } pump(); }, { rootMargin: "300px" });
function lazy(el, job) { el._job = job; watcher.observe(el); }
async function pump() {
  if (busy) return;
  busy = true;
  while (queue.length) { try { await queue.shift()(); } catch (e) { console.error(e); } await new Promise((r) => setTimeout(r, 0)); }
  busy = false;
}

// ---------- tabs ----------
let currentTab = "wall";
for (const b of $("tabs").querySelectorAll("button")) {
  b.onclick = () => {
    for (const x of $("tabs").querySelectorAll("button")) x.classList.toggle("on", x === b);
    for (const s of document.querySelectorAll("main > section")) s.hidden = s.id !== b.dataset.tab;
    currentTab = b.dataset.tab;
    if (currentTab === "live") liveStart();
    if (currentTab === "ask") askStart();
    if (currentTab === "made") drawMade();
  };
}
function seg(el, options, value, onChange) {
  el.innerHTML = "";
  for (const [k, text] of options) {
    const b = document.createElement("button");
    b.textContent = text; b.classList.toggle("on", k === value);
    b.onclick = () => { for (const x of el.children) x.classList.toggle("on", x === b); onChange(k); };
    el.appendChild(b);
  }
}

// ---------- 1. the wall ----------
const wallState = { feature: "nose" };
seg($("wallFeature"), FEATURE_ITEMS, "nose", (k) => { wallState.feature = k; drawWall(); });
$("wallSort").onchange = drawWall;
$("wallMarks").onchange = drawWall;
// the page of results whose measure a feature is shown with
async function featurePage(feature) {
  if (feature === "nose") return page;
  const j = await round(feature).catch((e) => { console.error(e); return null; });
  return j?.page || null;
}
async function drawWall() {
  const feature = wallState.feature, F = FEATURES[feature];
  const pg = await featurePage(feature);
  $("wallNose").hidden = feature !== "nose";
  $("wallJaw").hidden = feature === "nose";
  if (!pg) { $("wallGrid").innerHTML = `<p class="muted">The results for the ${F.one} are not written yet.</p>`; return; }
  const PP = pg.measure.proportions;
  if ($("wallSort").dataset.feature !== feature) {
    $("wallSort").innerHTML = "";
    PP.forEach((p, i) => $("wallSort").add(new Option(p.label, i)));
    $("wallSort").dataset.feature = feature;
  }
  const i = +$("wallSort").value, p = PP[i], key = p.key, view = viewOf(feature, key);
  const hs = pg.heads;
  const val = (h) => h.kotor.proportions?.[i];
  const readable = hs.filter((h) => val(h) != null), hidden = hs.filter((h) => val(h) == null);
  const order = [...readable.sort((a, b) => val(a) - val(b)), ...hidden];
  $("wallLow").textContent = "← " + p.low;
  $("wallHigh").textContent = p.high + " →";
  $("wallNote").textContent = `Seen ${/^(From|One|Both) /.test(VIEWS[viewKey(feature, view)].label) ? VIEWS[viewKey(feature, view)].label.toLowerCase() : "from the " + VIEWS[viewKey(feature, view)].label.toLowerCase()}. The provisional pass mark is ${fmtP(pg.measure.marks.proportions[i], p)}${unit(p)}, and KOTOR's ${F.plural} spread by ${fmtP(pg.measure.spread_proportions[i], p)}${unit(p)} (one standard deviation).${hidden.length ? ` The measure cannot read this on ${hidden.length} heads, because hair, cloth or metal covers the ${F.one}. They are at the end.` : ""}`;
  const grid = $("wallGrid");
  grid.innerHTML = "";
  for (const h of order) {
    const card = document.createElement("div");
    card.className = "card";
    const c = document.createElement("canvas");
    card.append(c);
    const v = val(h);
    card.insertAdjacentHTML("beforeend", `<div class="name" title="${esc(label(h))}">${esc(h.title || h.name)}${h.mod ? " · mod" : ""}</div><div class="val">${v == null ? "cannot be read" : fmtP(v, p) + unit(p)}</div>`);
    grid.append(card);
    lazy(card, async () => snap(c, [await kotorGroup(h.name, true, F.crops)], viewKey(feature, view), $("wallMarks").checked ? (feature === "nose" ? overlayFor(key, h.kotor) : overlayJaw(key, h.kotor)) : null));
  }
  if (feature !== "nose") jawChecks(pg, feature);
}
// the view that shows each jaw measure best
const JAW_VIEW_FOR = { jaw_width: "front", jaw_square: "front", chin_width: "front", jaw_slope: "side", chin_bottom: "side", chin_projection: "side", under_angle: "below" };
const jawViewFor = (key) => JAW_VIEW_FOR[key] || "side";
// the same for the brow and forehead (the fourth view is from above) and for the cheekbones and cheeks
const BROW_VIEW_FOR = { temple_width: "front", brow_slant: "front", brow_line: "front", brow_projection: "side", brow_middle: "side", brow_bulge: "side", forehead_slope: "side", forehead_profile: "side", brow_height: "side", forehead_curve: "below", brow_sweep: "below" };
const CHEEKS_VIEW_FOR = { cheek_width: "front", cheek_side: "front", face_front: "front", cheekbone_projection: "side", cheek_fullness: "side", cheek_profile: "side" };
const EYES_VIEW_FOR = { width: "wide", tilt: "wide", height: "front", upper_rise: "front", lower_drop: "front", upper_peak: "front", lower_low: "front", inner_angle: "front", outer_angle: "front", upper_depth: "three", lower_depth: "three", ball: "front", upper_inner: "front", upper_outer: "front", lower_inner: "front", lower_outer: "front", inner_height: "front", outer_lift: "front" };
const viewOf = (feature, key) => (feature === "eyes" ? EYES_VIEW_FOR[key] || "front" : feature === "jaw" ? jawViewFor(key) : feature === "brow" ? BROW_VIEW_FOR[key] || "side" : feature === "cheeks" ? CHEEKS_VIEW_FOR[key] || "three" : viewFor(key));
// the points the jaw measure finds on a head: the outline of the jaw on both sides, the chin profile and the underside of the chin,
// with the lowest point of the chin and the most forward point of the chin drawn larger. KOTOR's are yellow and ours are blue.
function overlayJaw(key, kotor, ours) {
  const points = [];
  for (const [m, dot, big] of [[kotor?.marks, "#ffd34d", "#ff7a3d"], [ours?.marks, "#5be0ff", "#2f6bff"]]) {
    if (!m) continue;
    for (const d of m.dots || []) points.push({ at: d, color: dot, r: 2.5 });
    if (m.bottom) points.push({ at: m.bottom, color: big, r: 5 });
    if (m.pogonion) points.push({ at: m.pogonion, color: big, r: 5 });
  }
  return points.length ? { points } : null;
}
// what the jaw's measure can and cannot be trusted with, shown under the wall: how far the jaw sliders move the parts of the face
// that must stay still, and the heads whose jaw the measure cannot read
async function jawChecks(pg, feature = "jaw") {
  const box = $("wallJaw"), F = FEATURES[feature];
  const mm = (x) => `${fmt(x, 2)} mm`;
  const rows = (pg.guards || []).map((g) => `<tr><td>${esc(cap(g.label))}</td><td>${mm(g.largest)}</td><td>${mm(g.bent)}</td></tr>`).join("");
  const labelOf = (name) => {
    const m = pg.measure.proportions.find((p) => p.key === name) || pg.measure.outlines.find((o) => o.key === name);
    return m ? m.label : name;
  };
  const hs = Object.fromEntries(pg.heads.map((h) => [h.name, h]));
  const list = (pg.unreadable || []).filter((u) => u.name !== "n_jedimalek").map((u) => {
    const all = u.rows.length >= pg.measure.proportions.length + pg.measure.outlines.length;
    return `<li><b>${esc(hs[u.name]?.title || u.name)}</b>: ${all ? "none of the measures can be read." : "these measures cannot be read: " + u.rows.map((k) => esc(labelOf(k).toLowerCase())).join(", ") + "."}</li>`;
  }).join("");
  box.innerHTML = `<h2>Checking the ${F.one} sliders</h2>
  ${(pg.guards || []).length ? `<h3>The sliders leave the rest of the face alone</h3>
  <p>Each ${F.one} slider was moved to each end of its range on the base head. The table gives, for ${(pg.guards || []).length} parts of the face that must stay where they are, the largest movement of any vertex over every slider end. It also gives how much of that movement is bent rather than carried. A part that is carried moves as a whole, and a part that is bent changes shape.</p>
  <table><tr><th>Part of the face</th><th>Largest movement</th><th>Of which bent</th></tr>${rows}</table>` : ""}
  <h3>The heads whose ${F.one} cannot be read</h3>
  <p>Hair, a hood or a beard modelled on KOTOR's head hides some parts of the ${F.one}, so the measure cannot read every head. A measure that cannot be read on a head is left out of that head's verdict.${feature === "jaw" ? " Malak's jaw is metal, so he is left out of the jaw's measures and is not listed below." : ""}</p>
  ${list ? `<ul>${list}</ul>` : `<p class="muted">Every head's ${F.one} can be read.</p>`}`;
}

function checksSection() {
  const keys = measureLabels(page);
  const units = [...P.map((p) => unit(p)), ...O.map(() => " mm")];
  const sum = Object.fromEntries(page.check1.summary.map((s) => [s.what, s]));
  const spreads = [...page.measure.spread_proportions, ...page.measure.spread_outlines];
  const m = [...marks.proportions, ...marks.outlines];
  let h = `<h3>Check 1: it ignores what nobody notices</h3>
  <p>Each KOTOR head was measured again in four other forms. "Subdivided" is KOTOR's mesh cut into 16 times as many triangles by a rule that keeps every vertex where it is, so the facets go and the shape stays. "Rounded" is one step of Loop's subdivision, which also rounds the nose a little, so it is a harsher test. "Bump" adds a 0.5 mm bump on the cheek beside the wing, 28 mm from the middle and 40 mm below the eyes. "Our wrap" is our own mesh laid onto the KOTOR head. The table gives how far each measure moved, as the median and the 95th percentile over the 57 heads, next to the pass mark and the spread of KOTOR's noses.</p>
  <table><tr><th>Measure</th><th>Subdivided</th><th>Rounded</th><th>Bump</th><th>Our wrap</th><th>Pass mark</th><th>KOTOR spread</th><th>Tells noses apart</th></tr>`;
  keys.forEach((k, i) => {
    const cell = (w) => { const s = sum[w]?.stats[i]; return s ? `<td class="${s.p95 > m[i] ? "bad" : ""}">${fmt(s.median, 2)} / ${fmt(s.p95, 2)}</td>` : "<td>–</td>"; };
    h += `<tr><td>${k} (${units[i].trim()})</td>${cell("subdivided")}${cell("rounded")}${cell("bump")}${cell("wrap")}<td>${fmt(m[i], 2)}</td><td>${fmt(spreads[i], 2)}</td><td class="${m[i] < 0.75 * spreads[i] ? "ok" : "bad"}">${m[i] < 0.5 * spreads[i] ? "yes" : m[i] < 0.75 * spreads[i] ? "roughly" : "no"}</td></tr>`;
  });
  h += `</table><p class="muted">Red means the 95th percentile is above the pass mark. The provisional mark of each measure is a quarter of KOTOR's spread, or twice the 95th percentile of the subdivided test if that is larger, so a measure whose wobble is near KOTOR's spread cannot tell noses apart. The last column says "yes" when the mark is under half of KOTOR's spread, "roughly" under three quarters, and "no" otherwise.</p>`;
  const c3 = page.check3;
  h += `<h3>Check 2: it sees what you notice</h3><p>This is the tab called "Same or different nose". Your answers there replace the provisional marks.</p>`;
  h += `<h3>Check 3: it tells KOTOR's heads apart correctly</h3><p>Copies of a test head in the other game, and heads that BioWare made by moving a few vertices of a test head, measured against that head. A copy whose nose is the same (a largest gap near the nose of 0 mm) must pass everything. A head whose nose was rebuilt or edited may differ.</p><table><tr><th>Head</th><th>Copy of</th><th>Median gap between meshes</th><th>Largest gap near the nose</th><th>Measures within the marks</th><th>What differs</th></tr>`;
  for (const t of c3.twins) {
    const v = t.verdict;
    const n = v ? v.pass.filter((x) => x).length : 0;
    h += `<tr><td>${t.name}</td><td>${t.of}</td><td>${fmt(t.gap_mm, 2)} mm</td><td>${fmt(t.nose_gap_mm, 1)} mm</td><td class="${v && v.made ? "ok" : "bad"}">${v ? n + " of " + v.pass.length : t.error}</td><td style="text-align:left">${v ? v.words.join(" ") : ""}</td></tr>`;
  }
  for (const t of c3.pairs) {
    const v = t.verdict, n = v.pass.filter((x) => x).length;
    h += `<tr><td>${t.name}</td><td>${t.of}</td><td>a mod or a near copy</td><td></td><td class="${v.made ? "ok" : "bad"}">${n} of ${v.pass.length}</td><td style="text-align:left">${v.words.join(" ")}</td></tr>`;
  }
  h += `</table><p>By this measure and its provisional marks, KOTOR's 57 test heads hold <b>${c3.kinds.length} kinds of nose</b>. Two noses are the same kind when every proportion and every outline is within its pass mark of every other nose of that kind. The kinds with more than one head are: ${c3.kinds.filter((k) => k.length > 1).map((k) => k.join(", ")).join("; ") || "none"}.</p>`;
  h += `<h3>Check 4: its extremes look right</h3><p>The pairs of KOTOR noses the measure calls most alike, then most different, from the front and the side.</p><div class="pairs" id="pairsAlike"></div><h3>Most different</h3><div class="pairs" id="pairsDifferent"></div>`;
  $("checks").innerHTML = h;
  for (const [id, list] of [["pairsAlike", page.check4.alike], ["pairsDifferent", page.check4.different]]) {
    for (const pr of list.slice(0, 6)) {
      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `<div class="pair"></div><div class="name">${pr.a} and ${pr.b}</div><div class="val">Distance ${fmt(pr.distance, 2)} pass marks</div>`;
      const box = card.querySelector(".pair");
      const cs = [0, 1, 2, 3].map(() => { const c = document.createElement("canvas"); box.append(c); return c; });
      $(id).append(card);
      lazy(card, async () => {
        snap(cs[0], [await kotorGroup(pr.a)], "front");
        snap(cs[1], [await kotorGroup(pr.b)], "front");
        snap(cs[2], [await kotorGroup(pr.a)], "side");
        snap(cs[3], [await kotorGroup(pr.b)], "side");
      });
    }
  }
}

// ---------- the noticing strip, kept as a record ----------
let answers = {};
try { answers = JSON.parse(localStorage.getItem("noseNoticing") || "{}"); } catch { answers = {}; }
fetch(SAVE + "/load/noticing").then((r) => r.json()).then((a) => { if (a) { answers = { ...a, ...answers }; showAnswers(); } }).catch(() => {});
for (const [i, s] of (page.strips || []).entries()) $("stripKey").add(new Option(s.label, i));
let stripView = null;
$("stripKey").onchange = () => { stripView = null; drawStrip(); };
function drawStrip() {
  const s = page.strips?.[+$("stripKey").value];
  if (!s) return;
  stripView ??= viewFor(s.key);
  seg($("stripView"), VIEW_ITEMS, stripView, (k) => { stripView = k; drawStrip(); });
  const i = P.findIndex((p) => p.key === s.key);
  $("stripInfo").textContent = `Each step adds ${fmt(s.step, 2)} ${s.unit} of ${s.label.toLowerCase()}. Under each picture is the change the measure finds on that head. The other ${P.length - 1} proportions move by less than the numbers in the hover text.`;
  const grid = $("stripGrid");
  grid.innerHTML = "";
  for (const st of s.steps) {
    const card = document.createElement("div");
    card.className = "card" + (answers[s.key]?.step === st.step ? " pick" : "");
    const c = document.createElement("canvas");
    card.append(c);
    const got = st.measured_change ? st.measured_change[i] : null;
    const others = st.measured_change ? st.measured_change.map((v, k) => (k === i || !P[k] ? null : `${P[k].label}: ${fmt(v, 2)}`)).filter(Boolean).join("\n") : "";
    card.title = others;
    card.insertAdjacentHTML("beforeend", `<div class="val">Step ${st.step}<br>${fmt(got, 2)}${unit(P[i] || {})}</div>`);
    card.onclick = () => saveAnswer(s, st, got);
    grid.append(card);
    const world = R1.builder.build(base, new Float64Array(st.values));
    const g = new THREE.Mesh(ourGeometry(world, page.base_frame), clayMat());
    snap(c, [g], stripView);
  }
  showAnswers();
}
$("stripNone").onclick = () => { const s = page.strips[+$("stripKey").value]; saveAnswer(s, null, null); };
async function saveAnswer(s, st, got) {
  answers[s.key] = st ? { label: s.label, step: st.step, asked: st.asked, measured: got, unit: s.unit, view: stripView, when: new Date().toISOString() } : { label: s.label, step: null, note: "no difference seen up to step 10", when: new Date().toISOString() };
  try { localStorage.setItem("noseNoticing", JSON.stringify(answers)); } catch {}
  try {
    const r = await fetch(SAVE + "/save/noticing", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(answers) });
    $("stripSaved").textContent = r.ok ? "Saved to the answers file." : "Could not save to the file; kept in this browser.";
  } catch { $("stripSaved").textContent = "The answers server is not running; your answers are kept in this browser only."; }
  drawStrip();
}
function showAnswers() {
  const rows = Object.entries(answers).map(([k, a]) => `<tr><td>${a.label}</td><td>${a.step == null ? "none seen" : "step " + a.step}</td><td>${a.step == null ? "" : fmt(a.measured, 2) + " " + a.unit}</td></tr>`);
  $("stripAnswers").innerHTML = rows.length ? `<table><tr><th>Measure</th><th>First step seen</th><th>Measured change there</th></tr>${rows.join("")}</table>` : `<p class="muted">No answers yet.</p>`;
}

// ---------- the pass marks from David's answers ----------
// Every answer of the same-or-different questions. The mark of a measure is the largest change still called "Same nose".
const ASK_KEY = "noseSameOrDifferent";
let ask = { plan: [], index: 0, stairs: {}, answers: [] };
try { ask = { ...ask, ...JSON.parse(localStorage.getItem(ASK_KEY) || "{}") }; } catch {}
function myMarks() {
  const out = {};
  for (const a of ask.answers) {
    const o = (out[a.measure] ??= { mark: null, n: 0, same: 0, smallestDifferent: null });
    const size = Math.abs(Number.isFinite(a.measured) ? a.measured : a.change);
    o.n++;
    if (a.answer === "same") { o.same++; o.mark = Math.max(o.mark ?? 0, size); }
    else o.smallestDifferent = Math.min(o.smallestDifferent ?? Infinity, size);
  }
  return out;
}
// David's marks for the proportions of a page, or null where he has none. Only for a page measured like round 2.
function marksOver(pg, r2page) {
  if (!pg || !r2page) return null;
  if (measureKeys(pg).join() !== measureKeys(r2page).join()) return null;
  const my = myMarks();
  const over = pg.measure.proportions.map((p) => my[p.key]?.mark ?? null);
  return over.some((x) => x != null) ? over : null;
}
const madeWith = (v, over) => v.pass.every((p, k) => (over && k < over.length && over[k] != null ? Math.abs(v.gap[k]) <= over[k] + 1e-9 : p));

// ---------- 3. live sliders ----------
let live = null;
const GROUPS = [["root", "Root"], ["bridge", "Bridge"], ["tip", "Tip"], ["wings", "Wings"], ["base", "Base"], ["jaw", "Jaw"], ["chin", "Chin"], ["brow", "Brow"], ["forehead", "Forehead"], ["temples", "Temples"], ["cheekbones", "Cheekbones"], ["cheeks", "Cheeks"], ["whole", "Whole eye"], ["size", "Size"], ["lids", "Lids"], ["corners", "Corners"], ["caruncle", "Caruncle"]];
// one head with every feature at once: the sets of the round 2 nose and of the jaw and chin, merged by the Rust code (nose sliders
// first, then the jaw's), on the stand-in faces that have both features taken out
let allCache = null;
function allKit() {
  return (allCache ??= (async () => {
    const [r2, jaw, brow, cheeks, eyes] = await Promise.all([round("r2"), round("jaw"), round("brow"), round("cheeks"), round("eyes")]);
    if (!r2.page || !jaw.page || !brow.page || !cheeks.page || !eyes.page) throw new Error("the results of a feature are not written yet");
    const merged = wasm.merge_sets(JSON.stringify([JSON.parse(r2.text), JSON.parse(jaw.text), JSON.parse(brow.text), JSON.parse(cheeks.text), JSON.parse(eyes.text)]));
    const mergedMeta = JSON.parse(merged).sliders;
    const meta = [...r2.meta, ...jaw.meta, ...brow.meta, ...cheeks.meta, ...eyes.meta];
    if (mergedMeta.length !== meta.length) console.warn(`the merged set has ${mergedMeta.length} sliders, the two sets have ${meta.length}`);
    const [builder, prepMs] = timed(() => new wasm.Builder(base, tris, keep, merged));
    return { key: "all", label: "Nose, jaw, brow, cheek and eye sliders", meta, builder, prepMs, parts: [r2, jaw, brow, cheeks, eyes], startsFile: "starts_all.bin", solvedMode: "full" };
  })());
}
const liveKit = (k) => (k === "all" ? allKit() : round(k));
const liveParts = (r) => r.parts || [r];
async function liveStart() {
  if (live) return;
  const canvas = $("liveCanvas");
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.outputEncoding = THREE.sRGBEncoding;
  renderer.setPixelRatio(window.devicePixelRatio);
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(BG);
  const cam = new THREE.PerspectiveCamera(22, 1, 10, 3000);
  const controls = new THREE.OrbitControls(cam, canvas);
  scene.add(new THREE.HemisphereLight(0xffffff, 0x4a4540, 0.35));
  const sun = new THREE.DirectionalLight(0xffffff, 0.85);
  cam.add(sun); sun.position.set(-60, 120, 0); scene.add(cam);
  let first = R1;
  try { first = await allKit(); } catch (e) { console.warn("The live view starts with round 1, because " + e.message); }
  live = { lid: "open", view: "three", focus: first.key === "all" ? "jaw" : "nose", renderer, scene, cam, controls, head: -1, round: first, values: new Float64Array(first.meta.length), paint: "clay", mesh: null, kotor: null };
  // the camera looks at the middle of the chosen feature from one of the four presets
  const setCam = () => {
    const v = VIEWS[viewKey(live.focus, live.view)];
    const dir = new THREE.Vector3(...v.pos).sub(new THREE.Vector3(...v.at)).normalize();
    controls.target.set(...v.at);
    cam.position.copy(new THREE.Vector3(...v.at).add(dir.multiplyScalar(live.focus === "jaw" ? 460 : live.focus === "nose" ? 240 : v.half * 5.5)));
    cam.up.set(...v.up);
    controls.update();
  };
  live.setCam = setCam;
  const camSeg = () => seg($("liveCams"), viewItems(live.focus), live.view, (k) => { live.view = k; setCam(); liveKotor(); });
  camSeg();
  seg($("liveFocus"), FEATURE_ITEMS, live.focus, (k) => { live.focus = k; if (!viewItems(k).some(([x]) => x === live.view)) live.view = "three"; camSeg(); setCam(); liveKotor(); });
  setCam();
  seg($("livePaint"), [["clay", "Clay"], ["paint", "KOTOR's paint carried over"]], "clay", (k) => { live.paint = k; liveMaterial(); });
  seg($("liveRound"), [["all", "Nose, jaw, brow, cheeks and eyes"], ["r2", "Round 2 nose only"], ["r1", "Round 1 nose only"]], first.key, async (k) => { live.round = await liveKit(k); live.showEyesBar(); liveSliders(); await liveHead(live.head, live.head >= 0); });
  $("liveHead").add(new Option("The base head", -1));
  heads.forEach((h, i) => $("liveHead").add(new Option(label(h), i)));
  $("liveHead").onchange = () => liveHead(+$("liveHead").value, true);
  $("liveSolved").onclick = () => liveHead(live.head, true);
  $("liveReset").onclick = () => { $("liveHead").value = -1; liveHead(-1, false); };
  $("liveKotor").onchange = liveKotor;
  liveSliders();
  // the eyes' own controls: the lid, and the proposed default eyes
  seg($("liveLid"), [["open", "Open"], ["closed", "Closed"]], "open", (k) => { live.lid = k; liveBuild(); });
  const setEyes = (values) => { live.round.meta.forEach((s, k) => { if (s.name in values) { live.values[k] = values[s.name]; $("s" + k).value = values[s.name]; } }); liveBuild(); };
  $("liveProposed").onclick = () => setEyes(PROPOSED_EYES);
  $("liveDefault").onclick = () => setEyes(Object.fromEntries(Object.keys(PROPOSED_EYES).map((n) => [n, 0])));
  live.showEyesBar = () => { $("liveEyesBar").hidden = !liveHasEyes(live.round); };
  live.showEyesBar();
  lidMaker().then(() => liveBuild()).catch((e) => console.error("the eye pieces", e));
  const resize = () => { const w = canvas.clientWidth, h = canvas.clientHeight; renderer.setSize(w, h, false); cam.aspect = w / h; cam.updateProjectionMatrix(); };
  new ResizeObserver(resize).observe(canvas);
  resize();
  const loop = () => { renderer.render(scene, cam); requestAnimationFrame(loop); };
  loop();
  await liveHead(-1, false);
}
// the slider panel of the chosen kit, grouped by part of the face when the sliders have groups
function liveSliders() {
  const r = live.round, box = $("liveSliders");
  live.values = new Float64Array(r.meta.length);
  const row = (s, k) => {
    const lo = Math.min(0, s.low), hi = Math.max(0, s.high);
    return `<label><div class="row"><span>${esc(s.name)}</span><span id="sv${k}"></span></div>${s.hint ? `<div class="hint">${esc(s.hint)}</div>` : ""}<input type="range" id="s${k}" min="${lo}" max="${hi}" step="${(hi - lo) / 400}" value="0"><div class="row muted" style="font-size:11px"><span>${fmtVal(lo, s)}</span><span>${s.measure === "ball" ? "radius" : unitShort(s)}</span><span>${fmtVal(hi, s)}</span></div></label>`;
  };
  const items = r.meta.map((s, k) => [s, k]);
  let h = "";
  if (r.meta.some((s) => s.group)) {
    const known = new Set(GROUPS.map((g) => g[0]));
    for (const [g, title] of [...GROUPS, [null, "Other"]]) {
      const list = items.filter(([s]) => (g ? s.group === g : !known.has(s.group)));
      if (list.length) h += `<h4>${title}</h4>` + list.map(([s, k]) => row(s, k)).join("");
    }
  } else h = items.map(([s, k]) => row(s, k)).join("");
  box.innerHTML = h;
  r.meta.forEach((s, k) => { $("s" + k).oninput = (e) => { live.values[k] = +e.target.value; liveBuild(); }; });
}
function solvedValues(r, i, mode, count = r.counts[r.counts.length - 1]) {
  const h = r.byName[heads[i].name];
  const s = h?.solved?.[mode] || h?.solved?.held_out;
  return s?.counts?.[count]?.values || null;
}
// the solved values of a head for every slider of a kit (for the merged kit, the nose's then the jaw's), or null when it has none
function liveSolved(r, i) {
  const out = [];
  let any = false;
  for (const q of liveParts(r)) { const got = solvedValues(q, i, q.solvedMode); if (got) any = true; out.push(...q.toSet(got)); }
  return any ? Float64Array.from(out) : null;
}
async function liveHead(i, solved) {
  const r = live.round;
  live.head = i;
  live.start = i < 0 ? base : await startOf(i, r.startsFile || "nose/starts.bin");
  live.frame = i < 0 ? page.base_frame : heads[i].frame;
  live.eyes = i < 0 ? baseEyes : new Float64Array(heads[i].eyes.flat());
  live.values.fill(0);
  const got = i >= 0 && solved ? liveSolved(r, i) : null;
  if (got) live.values.set(got);
  r.meta.forEach((s, k) => ($("s" + k).value = live.values[k]));
  $("livePaint").querySelectorAll("button")[1].disabled = i < 0;
  if (i < 0 && live.paint === "paint") { live.paint = "clay"; seg($("livePaint"), [["clay", "Clay"], ["paint", "KOTOR's paint carried over"]], "clay", (k) => { live.paint = k; liveMaterial(); }); }
  const how = r.key === "r1"
    ? (heads[i]?.mod ? "This is a mod head, so the sliders were never learned from it." : "The values loaded are those solved with the sliders learned from all game heads, this one included.")
    : `The values loaded are those solved with the largest number of sliders in the results, ${liveParts(r).map((q) => `${q.counts[q.counts.length - 1]} for the ${FEATURES[q.feature].one}`).join(" and ")}. Their ranges were set from all 57 heads.`;
  const which = r.key === "all" ? "The nose, the jaw and chin, the brow and forehead, the cheekbones and cheeks, and the eyes all come from the sliders. The rest of the face is a stand-in taken from this head's earlier fit, with that fit's own nose, jaw, brow, cheeks and eyes taken out." : "Only the nose comes from the sliders. The rest of the face is a stand-in taken from this head's earlier fit, with that fit's own nose taken out.";
  $("liveNote").textContent = i < 0
    ? `The base head, with every slider of ${r.label.toLowerCase()} at 0. Its whole face is the base head's.`
    : `${label(heads[i])}. ${which} ${solved ? (got ? how : "These results have no solved values for this head yet, so every slider is at 0.") : ""}`;
  if (live.mesh) { live.scene.remove(live.mesh); live.mesh.geometry.dispose(); live.mesh = null; }
  liveBuild();
  await liveMaterial();
  await liveKotor();
}
async function liveMaterial() {
  if (!live.mesh) return;
  live.mesh.material = live.paint === "paint" && live.head >= 0 ? await ourPaint(heads[live.head].name) : clayMat();
  if (liveHasEyes(live.round)) liveBuild();
}
async function liveKotor() {
  const c = $("liveKotorCanvas");
  c.hidden = !$("liveKotor").checked || live.head < 0;
  if (c.hidden) return;
  snap(c, [await kotorGroup(heads[live.head].name, true, FEATURES[live.focus].crops)], viewKey(live.focus, live.view));
}
// the page whose measure matches what the module measures: round 2's, or round 1's once it is measured the same way
function measurePage(n) {
  for (const pg of [r2Loaded?.page, page]) if (pg && pg.measure.proportions.length === n) return pg;
  return null;
}
let r2Loaded = null;
round("r2").then((r) => (r2Loaded = r)).catch((e) => console.error(e));
// the table of one feature's proportions on the live head, next to KOTOR's head and the pass marks
function measureTable(title, props, error, mp, byName) {
  const MP = mp ? mp.measure.proportions : props.map((_, i) => ({ key: "", label: `Proportion ${i + 1}`, unit: "mm" }));
  const t = live.head >= 0 && mp ? byName(heads[live.head].name)?.kotor : null;
  const my = myMarks();
  const anyMine = mp && MP.some((p) => my[p.key]?.mark != null);
  let h = `${title ? `<h3>${title}</h3>` : ""}<table><tr><th>Proportion</th><th>Ours</th>${t ? "<th>KOTOR</th><th>Gap</th>" : ""}${mp ? "<th>Provisional pass mark</th>" : ""}${anyMine ? "<th>Your pass mark</th>" : ""}</tr>`;
  MP.forEach((p, i) => {
    const o = props[i], kv = t?.proportions?.[i];
    const g = t && o != null && kv != null ? o - kv : null;
    const mark = my[p.key]?.mark ?? mp?.measure.marks.proportions[i];
    h += `<tr><td>${esc(p.label)}</td><td>${fmtP(o, p)}${unit(p)}</td>${t ? `<td>${fmtP(kv, p)}</td><td class="${g == null ? "" : Math.abs(g) <= mark ? "ok" : "bad"}">${fmtP(g, p)}</td>` : ""}${mp ? `<td>${fmtP(mp.measure.marks.proportions[i], p)}</td>` : ""}${anyMine ? `<td>${fmtP(my[p.key]?.mark, p)}</td>` : ""}</tr>`;
  });
  return h + "</table>" + (anyMine ? `<p class="muted">The gap is coloured with your pass mark where you have one, and with the provisional mark otherwise.</p>` : "") + (error ? `<p class="bad">The measure could not read this ${title ? title.toLowerCase() : "nose"}: ${esc(error)}.</p>` : "");
}
const liveHasEyes = (r) => liveParts(r).some((q) => q.feature === "eyes");
// the eyeballs and lid pieces of the live head, rebuilt every time the head changes. Gives the time it took in ms, or null when the kit has no eyes.
// the value of the Caruncle width slider of the live head, 0 when the set does not have it
function caruncleValues(r) {
  const a = r.meta.findIndex((m) => m.name === "Caruncle width");
  return a >= 0 ? live.values[a] || 0 : 0;
}
// the value of the Upper lid cover slider of the live head (mm the open lid piece is lowered in the middle), 0 when the set does not have it
function coverValues(r) {
  const a = r.meta.findIndex((m) => m.name === "Upper lid cover");
  return a >= 0 ? live.values[a] || 0 : 0;
}
function liveEyes(world, r) {
  if (live.eyeRoot) { live.scene.remove(live.eyeRoot); disposeEyes(live.eyeRoot); live.eyeRoot = null; }
  if (!liveHasEyes(r)) { showAllSkin(live.mesh.geometry); return null; }
  if (!lidKitReady) return null;
  const map = live.paint === "paint" && live.head >= 0 ? live.mesh.material.map || null : null;
  const [root, ms] = timed(() => eyeObjects(lidKitReady, world, live.frame, live.mesh, live.lid === "closed", map, caruncleValues(r), coverValues(r)));
  live.eyeRoot = root;
  live.scene.add(root);
  return ms;
}
let pending = false;
function liveBuild() {
  if (pending) return;
  pending = true;
  requestAnimationFrame(() => {
    pending = false;
    const r = live.round;
    const [world, ms] = timed(() => r.builder.build(live.start, live.values));
    const geo = ourGeometry(world, live.frame, live.mesh ? live.mesh.geometry : null);
    if (!live.mesh) { live.mesh = new THREE.Mesh(geo, clayMat()); live.scene.add(live.mesh); liveMaterial(); }
    const eyesMs = liveEyes(world, r);
    let html = "", mms = 0;
    if (r.parts) {
      for (const q of r.parts) {
        const [nj, t1] = timed(() => r.builder.measure_feature(q.feature, world, live.eyes));
        mms += t1;
        const n = JSON.parse(nj);
        html += measureTable(FEATURES[q.feature].title, n.proportions || [], n.error, q.page, (name) => q.byName[name]);
      }
    } else {
      const [nj, t1] = timed(() => r.builder.measure(world, live.eyes));
      mms = t1;
      const n = JSON.parse(nj);
      const props = n.proportions || [];
      const mp = measurePage(props.length);
      html = measureTable("", props, n.error, mp, (name) => (mp === page ? heads.find((h) => h.name === name) : r2Loaded.byName[name]));
    }
    $("liveTime").textContent = `Built in ${fmt(ms, 0)} ms and measured in ${fmt(mms, 0)} ms by the Rust code in this page.${eyesMs == null ? "" : ` The eyeballs and lid pieces took ${fmt(eyesMs, 0)} ms.`}`;
    $("liveMeasure").innerHTML = html;
    r.meta.forEach((s, k) => ($("sv" + k).textContent = fmtVal(live.values[k], s)));
  });
}

// ---------- 4. made against original ----------
const standinCache = {};
async function madeWorld(r, i, mode, count) {
  const h = heads[i];
  const jaw = r.feature !== "nose";
  standinCache[r.key + i] ??= jaw ? r.builder.build(await startOf(i, r.startsFile), new Float64Array(r.meta.length)) : R1.builder.build(await startOf(i), new Float64Array(0));
  const full = standinCache[r.key + i].slice();
  r.madeIndex ??= await getJSON(D + r.made + ".json" + VER);
  const e = r.madeIndex[h.name]?.[mode];
  if (!e) return null;
  r.madeBin ??= await fetchTyped(D + r.made + ".bin" + VER, Float32Array);
  const off = e.offsets[count];
  const bin = r.madeBin;
  e.vertices.forEach((v, k) => { full[3 * v] = bin[3 * (off + k)]; full[3 * v + 1] = bin[3 * (off + k) + 1]; full[3 * v + 2] = bin[3 * (off + k) + 2]; });
  return full;
}
const madeState = { lid: "open", feature: "nose", round: "r1", count: "12", mode: "held_out", view: "three", paint: "paint", missMode: "held_out", missCount: "14" };
// the kit the cards show: the jaw's, or one of the nose's two rounds
const madeKit = () => round(madeState.feature === "nose" ? madeState.round : madeState.feature);
$("madeCount").onchange = () => { madeState.count = $("madeCount").value; drawMade(); };
seg($("madeFeature"), FEATURE_ITEMS, madeState.feature, (k) => { madeState.feature = k; $("madeRound").hidden = k !== "nose"; if (!viewItems(k).some(([x]) => x === madeState.view)) madeState.view = "three"; madeViewSeg(); drawMade(); });
seg($("madeRound"), [["r1", ROUNDS.r1.label], ["r2", ROUNDS.r2.label]], madeState.round, (k) => { madeState.round = k; madeState.mode = "held_out"; madeState.count = k === "r2" ? "14" : "12"; drawMade(); });
const madeViewSeg = () => seg($("madeView"), viewItems(madeState.feature), madeState.view, (k) => { madeState.view = k; drawMade(); });
madeViewSeg();
seg($("madePaint"), [["paint", "KOTOR's paint"], ["clay", "Clay"]], madeState.paint, (k) => { madeState.paint = k; drawMade(); });
seg($("madeLid"), [["open", "Lid open"], ["closed", "Lid closed"]], madeState.lid, (k) => { madeState.lid = k; drawMade(); });
const countLabel = (r, c) => (r.key === "r1" ? `${c} sliders (${Math.min(4, +c)} proportion, ${+c - 4} learned)` : `${c} sliders, the ${c} used most`);
const cap = (t) => t.charAt(0).toUpperCase() + t.slice(1);

// ---------- the marks on the cards ----------
// One mark per head and feature: "same" or "not_same". A head without a mark is not in the object.
// Saved as {nose: {head: mark}, jaw: {head: mark}, shown: {feature: {head: {round, mode, count}}}, updated}.
const MARKS_KEY = "noseCardMarks";
let cardMarks = { nose: {}, jaw: {}, brow: {}, cheeks: {}, eyes: {}, shown: { nose: {}, jaw: {}, brow: {}, cheeks: {}, eyes: {} }, updated: null };
const cleanMarks = (m) => { const out = { shown: {}, updated: m?.updated || null }; for (const [f] of FEATURE_ITEMS) { out[f] = { ...(m?.[f] || {}) }; out.shown[f] = { ...(m?.shown?.[f] || {}) }; } return out; };
try { cardMarks = cleanMarks(JSON.parse(localStorage.getItem(MARKS_KEY) || "{}")); } catch {}
const marksLoaded = fetch(SAVE + "/load/card_marks").then((r) => r.json()).then((saved) => {
  if (saved && (!cardMarks.updated || (saved.updated || "") >= cardMarks.updated)) cardMarks = cleanMarks(saved);
}).catch(() => {});
let marksSaving = Promise.resolve();
function saveMarks() {
  cardMarks.updated = new Date().toISOString();
  try { localStorage.setItem(MARKS_KEY, JSON.stringify(cardMarks)); } catch {}
  const body = JSON.stringify(cardMarks);
  marksSaving = marksSaving.then(async () => {
    try {
      const r = await fetch(SAVE + "/save/card_marks", { method: "POST", headers: { "Content-Type": "application/json" }, body });
      $("marksSaved").textContent = r.ok ? "Your marks are saved to the answers file." : "Could not save to the answers file. Your marks are kept in this browser.";
    } catch { $("marksSaved").textContent = "The answers server is not running. Your marks are kept in this browser only."; }
  });
}
const MARK_STATES = [[null, "Not marked"], ["same", "Same"], ["not_same", "Not the same"]];
function markControl(feature, name, r) {
  const box = document.createElement("span");
  box.className = "seg markbar";
  const show = () => [...box.children].forEach((b, k) => { b.classList.toggle("on", MARK_STATES[k][0] === (cardMarks[feature][name] || null)); });
  MARK_STATES.forEach(([state, text]) => {
    const b = document.createElement("button");
    b.textContent = text;
    b.dataset.state = state || "none";
    b.title = state === "same" ? `The made ${FEATURES[feature].one} ${feature === "eyes" ? "look" : "looks"} the same as KOTOR's.` : state === "not_same" ? `The made ${FEATURES[feature].one} ${feature === "eyes" ? "do" : "does"} not look the same as KOTOR's.` : "No mark.";
    b.onclick = () => {
      if (state) { cardMarks[feature][name] = state; cardMarks.shown[feature][name] = { round: r.key, mode: madeState.mode, count: madeState.count }; }
      else { delete cardMarks[feature][name]; delete cardMarks.shown[feature][name]; }
      show();
      saveMarks();
      drawMarkSummary();
      drawScores().catch((e) => console.error("scorecards", e));
    };
    box.append(b);
  });
  show();
  return box;
}
const markedCount = (feature) => Object.keys(cardMarks[feature]).length;
// the verdict of a head at a mode and count, with the card marks laid over it: Same counts as made, Not the same as missed
const hasReadable = (v) => !!v && v.pass.some((p) => p !== null);
const madeWithMark = (feature, name, v) => (cardMarks[feature][name] === "same" ? true : cardMarks[feature][name] === "not_same" ? false : !!v.made);

// per measure, the largest miss among the cards marked Same and the smallest miss among the cards marked Not the same
function drawMarkSummary() {
  const cur = madeCurrent, box = $("marksSummary");
  if (!cur) { box.innerHTML = ""; return; }
  const { r, list, feature } = cur, F = FEATURES[feature], pg = r.page;
  const labels = measureLabels(pg);
  const us = [...pg.measure.proportions.map((p) => unit(p)), ...pg.measure.outlines.map(() => " mm")];
  const mk = [...pg.measure.marks.proportions, ...pg.measure.marks.outlines];
  const sc = [...pg.measure.proportions.map(scaleOf), ...pg.measure.outlines.map(() => 1)];
  const mine = cardMarks[feature];
  const sameN = list.filter((x) => mine[x.h.name] === "same").length, diffN = list.filter((x) => mine[x.h.name] === "not_same").length;
  let h = `<h3>What your marks say about each measure</h3><p>${sameN + diffN} of ${list.length} cards are marked: ${sameN} as "Same" and ${diffN} as "Not the same". A miss is how far the made ${F.one} is from KOTOR's on one measure, in the unit of that measure, whichever way it is off. When the largest miss among the "Same" cards is smaller than the smallest miss among the "Not the same" cards, a pass mark between the two matches your marks.</p>`;
  if (!sameN && !diffN) return void (box.innerHTML = h + `<p class="muted">Mark some cards to see the table.</p>`);
  h += `<table><tr><th>Measure</th><th>Provisional pass mark</th><th>Largest miss among "Same"</th><th>Smallest miss among "Not the same"</th><th>Your marks</th></tr>`;
  labels.forEach((lab, k) => {
    const miss = (state) => list.filter((x) => mine[x.h.name] === state).map((x) => x.s.verdict?.gap?.[k]).filter((g) => g != null && Number.isFinite(g)).map(Math.abs);
    const a = miss("same"), d = miss("not_same");
    const hi = a.length ? Math.max(...a) : null, lo = d.length ? Math.min(...d) : null;
    const verdict = hi == null || lo == null ? "needs both kinds of mark" : hi < lo ? "a mark between them fits" : "the marks overlap";
    h += `<tr><td>${esc(lab)}</td><td>${fmt(mk[k] * sc[k], 2)}${us[k]}</td><td>${hi == null ? "–" : fmt(hi * sc[k], 2) + us[k]}</td><td>${lo == null ? "–" : fmt(lo * sc[k], 2) + us[k]}</td><td class="${hi == null || lo == null ? "" : hi < lo ? "ok" : "bad"}">${verdict}</td></tr>`;
  });
  box.innerHTML = h + `</table>`;
}

// one kit's scorecard: heads made at each slider count, per mode, with the provisional marks, with David's answers to the questions, or with the card marks
function scoreTable(r, over, withCards = false, strict = false) {
  const pg = r.page;
  const cell = (mode, c) => {
    if (!over && !withCards) { const x = pg.scorecard.find((s) => s.mode === mode && s.count === +c); return x ? `${strict ? x.made_strict : x.made} of ${x.of}` : "–"; }
    let made = 0, of = 0;
    for (const h of pg.heads) {
      const v = h.solved?.[mode]?.counts?.[c]?.verdict;
      if (!hasReadable(v)) continue;
      of++;
      if (withCards ? madeWithMark(r.feature, h.name, v) : madeWith(v, over)) made++;
    }
    return of ? `${made} of ${of}` : "–";
  };
  const alone = !over && !withCards && pg.scorecard.find((s) => s.mode === "standin");
  const aloneRow = alone ? `<tr><td>The stand-in faces alone, with no sliders</td><td colspan="${r.counts.length}">${strict ? alone.made_strict : alone.made} of ${alone.of}</td></tr>` : "";
  return `<table><tr><th>Sliders</th>${r.counts.map((c) => `<th>${c}</th>`).join("")}</tr>${aloneRow}${r.modes.map(([m, t]) => `<tr><td>${t}</td>${r.counts.map((c) => `<td>${cell(m, c)}</td>`).join("")}</tr>`).join("")}</table>`;
}
async function drawScores() {
  const feature = madeState.feature, F = FEATURES[feature];
  const box = $("madeScore");
  const side = (r, r2) => {
    if (!r.page) return `<div><h3>${r.label}</h3><p class="muted">The results of this kit are not written yet.</p></div>`;
    const over = r.feature === "nose" ? marksOver(r.page, r2.page) : null;
    const n = markedCount(r.feature);
    const provisional = n ? `The provisional pass marks, which still decide every head you have not marked.` : `The pass marks are provisional. They were not set from what you see, and they stay in use until you mark cards below.`;
    let h = `<div><h3>${r.label}</h3><p class="muted">${provisional}</p>${scoreTable(r, null)}`;
    if (r.page.scorecard.some((x) => x.made_strict != null)) {
      h += `<p class="muted">The first row of the table above counts the ${F.plural} that the stand-in faces make with no sliders at all. The rows below it count the ${F.plural} the sliders make. The sliders only help when they make more than the stand-ins do.</p>
      <p class="muted">The table below counts again with strict pass marks. The strict pass mark of a measure is a quarter of KOTOR's spread of that measure, even where the measure wobbles by more than that. The strict pass marks are provisional too.</p>${scoreTable(r, null, false, true)}`;
    }
    if (over) h += `<p class="muted">With your pass marks for the ${over.filter((x) => x != null).length} measures you have answered in the questions, and the provisional marks for the others.</p>${scoreTable(r, over)}`;
    else if (r.key === "r1" && r2.page && measureKeys(r.page).join() !== measureKeys(r2.page).join()) h += `<p class="muted">Round 1 still uses the old measure, with ${r.page.measure.proportions.length} proportions. Your pass marks will apply to it once round 1 is measured again.</p>`;
    h += n ? `<p class="muted">From your marks on the cards. A head counts as made when you marked it "Same", as missed when you marked it "Not the same", and by the provisional verdict when it is not marked. ${n} of ${r.page.heads.filter((x) => hasReadable(x.solved?.[r.solvedMode]?.counts?.[r.counts[r.counts.length - 1]]?.verdict)).length} heads are marked.</p>${scoreTable(r, null, true)}` : `<p class="muted">Once you mark cards, a second table here counts a head as made when you marked it "Same".</p>`;
    return h + "</div>";
  };
  let r2 = null;
  try { r2 = await round("r2"); } catch {}
  let kits;
  try { kits = feature !== "nose" ? [await round(feature)] : [R1, r2]; } catch { box.innerHTML = `<p class="muted">The results for the ${F.one} are not written yet.</p>`; return; }
  box.innerHTML = `<h2>Scorecards</h2><p>A ${F.one} is made when every measure that can be read is within its pass mark. The tables count the ${F.plural} made out of ${feature === "jaw" ? "56, because Malak's jaw is metal and is left out" : feature === "nose" ? "57" : kits[0].page.scorecard.find((x) => x.mode === kits[0].solvedMode)?.of ?? 57}, at each number of sliders. Until you mark cards or answer the questions in the tab "Same or different nose", every pass mark is provisional.</p><div class="cards2">${kits.map((r) => side(r, r2)).join("")}</div><div id="misses"></div>`;
  drawMisses(kits[kits.length - 1]);
}
function drawMisses(r) {
  const box = $("misses"), F = FEATURES[r.feature];
  if (!r.page) { box.innerHTML = ""; return; }
  if (!r.counts.includes(madeState.missCount)) madeState.missCount = r.counts[r.counts.length - 1];
  box.innerHTML = `<h3>${F.title} misses, and what is wrong with each</h3><div class="bar"><span class="seg" id="missMode"></span><label>Sliders <select id="missCount">${r.counts.map((c) => `<option value="${c}">${c}</option>`).join("")}</select></label></div><div id="missList"></div>`;
  $("missCount").value = madeState.missCount;
  $("missCount").onchange = () => { madeState.missCount = $("missCount").value; drawMisses(r); };
  seg($("missMode"), r.modes, madeState.missMode, (k) => { madeState.missMode = k; drawMisses(r); });
  const sc = r.page.scorecard.find((s) => s.mode === madeState.missMode && s.count === +madeState.missCount);
  const list = sc?.misses || [];
  const mk = measureKeys(r.page);
  const per = sc?.per_measure ? `<p class="muted">How many of the ${sc.of} ${F.plural} pass each measure: ${sc.per_measure.map((n, k) => `${SHORT[mk[k]] || mk[k]} ${n}`).join(", ")}.</p>` : "";
  $("missList").innerHTML = `<p>${sc ? `${sc.made} of ${sc.of} ${F.plural} are made, and ${list.length} are missed.` : ""} The words below use the provisional pass marks.</p>${per}<ul class="words">${list.map((m) => `<li><b>${esc(m.title ? `${m.title} (${m.name})` : m.name)}.</b> ${(m.words || []).map(esc).join(" ")}</li>`).join("")}</ul>`;
}

let madeDrawn = null, madeCurrent = null;
async function drawMade() {
  await marksLoaded;
  const feature = madeState.feature, F = FEATURES[feature];
  $("madeRound").hidden = feature !== "nose";
  $("madeLid").hidden = feature !== "eyes";
  $("madeTitle").textContent = `The ${F.plural}, one card per head`;
  $("madeIntro").textContent = `For each of the 57 heads, KOTOR's ${F.one} on the left and the ${F.one} the sliders made on the right, from the same camera. Misses come first. The face around our ${F.one} is a stand-in taken from the head's earlier fit, and only the ${F.one} comes from the sliders. Under each card you can mark whether the two look the same.`;
  let r;
  try { r = await madeKit(); } catch (e) { madeCurrent = null; $("madeGrid").innerHTML = `<p class="muted">The results for the ${F.one} are not written yet.</p>`; $("madeScore").innerHTML = ""; $("marksSummary").innerHTML = ""; $("variety").innerHTML = ""; return; }
  const key = JSON.stringify(madeState) + ask.answers.length;
  if (madeDrawn === key) return;
  madeDrawn = key;
  drawScores().catch((e) => console.error("scorecards", e));
  if (!r.page) { madeCurrent = null; $("madeGrid").innerHTML = `<p class="muted">The results of ${r.label.toLowerCase()} are not written yet.</p>`; $("marksSummary").innerHTML = ""; drawVariety(r); return; }
  $("madeCount").innerHTML = "";
  for (const c of r.counts) $("madeCount").add(new Option(countLabel(r, c), c));
  if (!r.counts.includes(madeState.count)) madeState.count = r.counts[r.counts.length - 1];
  $("madeCount").value = madeState.count;
  if (!r.modes.some(([m]) => m === madeState.mode)) madeState.mode = r.modes[0][0];
  seg($("madeMode"), r.modes, madeState.mode, (k) => { madeState.mode = k; drawMade(); });
  const { count, mode, view, paint } = madeState;
  const pg = r.page;
  const keys = measureKeys(pg), labels = measureLabels(pg);
  const list = heads.map((h, i) => ({ h, i, s: r.byName[h.name]?.solved?.[mode]?.counts?.[count] })).filter((x) => x.s && (!x.s.verdict || hasReadable(x.s.verdict)));
  const misses = (s) => (s.verdict ? s.verdict.pass.filter((p) => p === false).length : 99);
  list.sort((a, b) => (a.s.verdict?.made === b.s.verdict?.made ? (b.s.cost ?? misses(b.s)) - (a.s.cost ?? misses(a.s)) : a.s.verdict?.made ? 1 : -1));
  madeCurrent = { r, list, feature };
  const made = list.filter((x) => x.s.verdict?.made).length;
  $("madeNow").innerHTML = `<p class="big">${cap(F.plural)}: ${made} of ${list.length} made by ${r.label.toLowerCase()} with ${count} sliders, ${r.modes.find((m) => m[0] === mode)[1].toLowerCase()}.</p><p class="muted">The chips and the words on the cards use the provisional pass marks. A red chip is a measure outside its mark. A grey chip is a measure that cannot be read on KOTOR's head, because hair, cloth or metal covers it.</p>`;
  const grid = $("madeGrid");
  grid.innerHTML = "";
  for (const { h, i, s } of list) {
    const v = s.verdict;
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML = `<div class="pair"><canvas></canvas><canvas></canvas><div class="lab">KOTOR</div><div class="lab">Made by ${count} sliders</div></div>
      <div class="name">${v && v.made ? "✓" : "✗"} ${esc(label(h))}${h.mod ? " · mod" : ""}</div>
      <div class="chips">${(v ? v.pass : []).map((p, k) => `<span class="${p === false ? "bad" : p === null ? "unread" : ""}" title="${esc(labels[k] || "")}: ${p === null ? "cannot be read" : fmtP(v.gap[k], pg.measure.proportions[k], 2)}">${esc(SHORT[keys[k]] || keys[k] || k)}</span>`).join("")}</div>
      ${v && v.words.length ? `<ul class="words">${v.words.map((w) => `<li>${esc(w)}</li>`).join("")}</ul>` : ""}
      <div class="markrow"></div>`;
    card.querySelector(".markrow").append(markControl(feature, h.name, r));
    card.dataset.find = (label(h) + " " + h.name).toLowerCase();
    const [ck, cm] = card.querySelectorAll("canvas");
    grid.append(card);
    lazy(card, async () => {
      snap(ck, [await kotorGroup(h.name, paint === "paint", F.crops)], viewKey(feature, view));
      const w = await madeWorld(r, i, mode, count);
      if (!w) return;
      const mat = paint === "paint" ? await ourPaint(h.name) : clayMat();
      const mesh = new THREE.Mesh(ourGeometry(w, h.frame), mat);
      const kit = feature === "eyes" ? await lidMaker() : null;
      const eyes = kit ? eyeObjects(kit, w, h.frame, mesh, madeState.lid === "closed", paint === "paint" ? mat.map : null) : null;
      snap(cm, eyes ? [mesh, eyes] : [mesh], viewKey(feature, view));
      if (eyes) disposeEyes(eyes);
    });
  }
  $("madeFind").oninput();
  drawMarkSummary();
  drawVariety(r);
}

$("madeFind").oninput = function () {
  const q = $("madeFind").value.toLowerCase().split(",").map((x) => x.trim()).filter(Boolean);
  for (const c of $("madeGrid").children) c.hidden = q.length > 0 && !q.some((x) => (c.dataset.find || "").includes(x));
};

// variety, safety and the leak table of one round, below the cards
let varietyOf = null;
function drawVariety(r) {
  const box = $("variety");
  if (varietyOf === r.key) return;
  varietyOf = r.key;
  const pg = r.page, F = FEATURES[r.feature], jaw = r.feature !== "nose";
  if (!pg) { box.innerHTML = ""; return; }
  const faultsOf = (f) => f.total ?? (f.flipped || 0) + (f.folded || 0) + (f.nostrils_flipped || 0);
  const kinds = [["like KOTOR", `Like the KOTOR ${F.plural}`], ["box", "Each slider anywhere in its range"]];
  const ofKind = (k) => (pg.samples || []).filter((s) => s.kind === k);
  const count = (k) => ofKind(k).filter((s) => faultsOf(s.faults) > 0).length;
  const kbad = (pg.kotor_faults || []).filter((s) => faultsOf(s.faults) > 0);
  const leaks = pg.ends || pg.leaks || [];
  const pairs = pg.pairs && pg.pairs.list ? `<p>Each pair of sliders was also set to the four corners of their two ranges at once, on the base head. ${pg.pairs.faulty_pairs} of the ${pg.pairs.list.length} pairs have a fault at one corner or more, ${pg.pairs.faulty_corners} corners in all.</p>` : "";
  box.innerHTML = `<h2>Variety and safety of ${r.label.toLowerCase()}</h2>
  <p>${cap(F.plural)} drawn at random inside the slider ranges, on the base head, in two ways. "Like the KOTOR ${F.plural}" draws the sliders together, with the spread and the links between sliders that the 57 solved ${F.plural} have, kept inside the ranges. "Each slider anywhere in its range" draws every slider on its own, anywhere in its range, which reaches corners no KOTOR ${F.one} is near. Each shape was checked for triangles turned over and for folds (two neighbouring triangles that met at under 45 degrees now meeting at more than 90)${jaw ? "" : ", and for nostril triangles turned over"}.</p>
  <p class="big">${count("like KOTOR")} of ${ofKind("like KOTOR").length} ${F.plural} drawn like the KOTOR ${F.plural} have at least one fault, and ${count("box")} of ${ofKind("box").length} drawn anywhere in the ranges.</p>
  <p>On the 57 solved KOTOR ${F.plural}, ${kbad.length} have at least one fault, ${kbad.reduce((a, k) => a + faultsOf(k.faults), 0)} faults in all.</p>${pairs}
  <div class="bar"><span class="seg" id="sampleKind"></span></div>
  <div class="wall" id="sampleGrid"></div>
  <h2>Leak table of ${r.label.toLowerCase()}</h2><p>Each slider at each end of its range on the base head, and how far each zone of the face moves, the most and the average, in mm. Only vertices outside the zone of the ${F.one} count in the zone columns. The zone of the ${F.one} fades out over 7 mm into the rest of the face, and that soft edge has its own column.${pg.ends ? " The two columns after the value count the faults on the base head, and how many of the 57 stand-in faces get at least one fault with the slider at that end." : ""}</p><div id="leaks"></div>`;
  const drawSamples = (kind) => {
    $("sampleGrid").innerHTML = "";
    ofKind(kind).forEach((s) => {
      const card = document.createElement("div");
      card.className = "card";
      const c = document.createElement("canvas");
      card.append(c);
      const f = s.faults;
      card.insertAdjacentHTML("beforeend", `<div class="val ${faultsOf(f) ? "bad" : ""}">${faultsOf(f) ? `${f.flipped ?? 0} turned over, ${f.folded ?? 0} folds${jaw ? "" : `, ${f.nostrils_flipped ?? 0} in a nostril`}` : "no fault"}</div>`);
      $("sampleGrid").append(card);
      lazy(card, async () => {
        const w = r.builder.build(base, r.toSet(s.values)), mesh = new THREE.Mesh(ourGeometry(w, page.base_frame), clayMat());
        const kit = r.feature === "eyes" ? await lidMaker() : null;
        const eyes = kit ? eyeObjects(kit, w, page.base_frame, mesh, false, null) : null;
        snap(c, eyes ? [mesh, eyes] : [mesh], viewKey(r.feature, "three"));
        if (eyes) disposeEyes(eyes);
      });
    });
  };
  seg($("sampleKind"), kinds, "like KOTOR", drawSamples);
  drawSamples("like KOTOR");
  const zoneOrder = ["soft edge", "around the eyes", "cheeks", "mouth", "brows", "forehead", "chin", "jaw", "nose"];
  const seen = new Set(leaks.flatMap((l) => Object.keys(l.zones || {})));
  const zones = [...zoneOrder.filter((z) => seen.has(z)), ...[...seen].filter((z) => !zoneOrder.includes(z))];
  const extra = pg.ends ? "<th>Faults on the base head</th><th>Stand-ins with a fault</th>" : "";
  let t = `<table><tr><th>Slider</th><th>End</th><th>Value</th>${extra}${zones.map((z) => `<th>${z}</th>`).join("")}</tr>`;
  for (const l of leaks) {
    const ex = pg.ends ? `<td class="${faultsOf(l.faults) ? "bad" : ""}">${faultsOf(l.faults)}</td><td class="${l.heads_with_faults ? "bad" : ""}">${l.heads_with_faults ?? "–"} of 57</td>` : "";
    t += `<tr><td>${esc(l.slider)}</td><td>${l.end}</td><td>${fmt(l.value, 2)}</td>${ex}${zones.map((z) => { const v = l.zones[z]; return `<td class="${v && z !== "soft edge" && v.max > 0.5 ? "bad" : ""}">${v ? fmt(v.max, 2) + " / " + fmt(v.mean, 2) : "–"}</td>`; }).join("")}</tr>`;
  }
  $("leaks").innerHTML = t + `</table><p class="muted">Each zone cell is the most / the average movement in mm. Red marks a zone outside the ${F.one} that moves by more than 0.5 mm somewhere. The column called nose is the rest of the old nose zone of the fits, which reaches a little past ours.</p>`;
}

// ---------- 2. same or different nose ----------
const PAINTED = ["p_carthh", "p_bastilah", "pmha01"].filter((n) => n in headIndex);
const PER_MEASURE = 6;
const shuffle = (a) => { for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; };
// 6 questions per measure, half on the clay base head and half on a painted KOTOR head, mixed so one measure rarely comes twice in a row
function makePlan(r2) {
  const keys = r2.page.measure.proportions.map((p) => p.key).filter((k) => r2.meta.some((s) => s.measure === k));
  const left = {};
  for (const key of keys) left[key] = shuffle([...Array(PER_MEASURE)].map((_, i) => (i % 2 || !PAINTED.length ? "base" : PAINTED[Math.floor(Math.random() * PAINTED.length)])));
  const out = [];
  while (Object.values(left).some((l) => l.length)) {
    const last = out[out.length - 1]?.measure;
    let pool = keys.filter((k) => left[k].length && k !== last);
    if (!pool.length) pool = keys.filter((k) => left[k].length);
    let x = Math.random() * pool.reduce((a, k) => a + left[k].length, 0), pick = pool[0];
    for (const k of pool) { x -= left[k].length; if (x < 0) { pick = k; break; } }
    out.push({ measure: pick, head: left[pick].pop() });
  }
  return out;
}
let askReady = null, askCur = null, askBusy = false;
function askSaveLocal() { try { localStorage.setItem(ASK_KEY, JSON.stringify(ask)); } catch {} }
async function askSave() {
  askSaveLocal();
  try {
    const r = await fetch(SAVE + "/save/same_or_different", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ ...ask, updated: new Date().toISOString() }) });
    $("askSaved").textContent = r.ok ? "Every answer is saved to the answers file." : "Could not save to the answers file. Your answers are kept in this browser.";
  } catch { $("askSaved").textContent = "The answers server is not running. Your answers are kept in this browser only."; }
}
function askStart() {
  return (askReady ??= (async () => {
    const r2 = await round("r2");
    r2Loaded = r2;
    if (!r2.page) { $("askBox").innerHTML = `<p class="muted">The round 2 results are not written yet, so there is nothing to ask about.</p>`; return; }
    try {
      const saved = await (await fetch(SAVE + "/load/same_or_different")).json();
      if (saved && (saved.answers?.length || 0) > ask.answers.length) ask = { ...ask, ...saved };
    } catch {}
    if (!ask.plan?.length) { ask.plan = makePlan(r2); ask.index = 0; askSaveLocal(); }
    seg($("askView"), VIEW_ITEMS, "side", (k) => askTurn(k));
    $("askSame").onclick = () => askAnswer("same");
    $("askDiff").onclick = () => askAnswer("different");
    $("askStop").onclick = () => { askCur = null; $("askBox").hidden = true; $("askResume").hidden = false; $("askProgress").textContent = `Stopped after question ${ask.index} of ${ask.plan.length}. The page remembers where you stopped.`; };
    $("askResume").onclick = () => { $("askBox").hidden = false; $("askResume").hidden = true; askQuestion(); };
    $("askMore").onclick = () => { ask.plan.push(...makePlan(r2)); askSaveLocal(); $("askBox").hidden = false; $("askMore").hidden = true; askQuestion(); };
    document.addEventListener("keydown", (e) => {
      if (currentTab !== "ask" || !askCur || e.target.closest?.("input, select, textarea")) return;
      const k = e.key.toLowerCase();
      if (k === "s") askAnswer("same");
      else if (k === "d") askAnswer("different");
      else if ("1234".includes(k) && k.length === 1) askTurn(Object.keys(VIEWS)[+k - 1]);
    });
    askMarksTable();
    await askQuestion();
  })());
}
function askTurn(view) {
  if (!askCur) return;
  askCur.view = view;
  if (!askCur.viewsSeen.includes(view)) askCur.viewsSeen.push(view);
  [...$("askView").children].forEach((b, i) => b.classList.toggle("on", Object.keys(VIEWS)[i] === view));
  askDraw();
}
function askDraw() {
  const c = askCur;
  snap($("askA"), [c.meshes[0]], c.view, null, 520);
  snap($("askB"), [c.meshes[1]], c.view, null, 520);
}
async function askQuestion() {
  const r2 = await round("r2");
  const pg = r2.page;
  if (ask.index >= ask.plan.length) {
    askCur = null;
    $("askBox").hidden = true;
    $("askMore").hidden = false;
    $("askProgress").textContent = `All ${ask.plan.length} questions are answered. Thank you.`;
    askMarksTable();
    return;
  }
  askBusy = true;
  const q = ask.plan[ask.index];
  const key = q.measure;
  const mi = pg.measure.proportions.findIndex((p) => p.key === key);
  const si = r2.meta.findIndex((s) => s.measure === key);
  const s = r2.meta[si];
  const spread = s.spread ?? pg.measure.spread_proportions[mi];
  const step = ask.stairs[key] ?? spread;
  const hi = q.head !== "base" ? headIndex[q.head] : -1;
  const ref = r2.toSet(hi >= 0 ? solvedValues(r2, hi, "full") : null);
  // the change goes up or down at random, and stays inside the slider's range
  const room = { up: s.high - ref[si], down: ref[si] - s.low };
  let sign = Math.random() < 0.5 ? 1 : -1;
  const roomOf = (sg) => Math.max(0, sg > 0 ? room.up : room.down);
  if (roomOf(sign) < step && roomOf(-sign) > roomOf(sign)) sign = -sign;
  const change = sign * Math.min(step, roomOf(sign));
  const moved = ref.slice();
  moved[si] += change;
  const start = hi >= 0 ? await startOf(hi) : base;
  const frame = hi >= 0 ? heads[hi].frame : page.base_frame;
  const eyes = hi >= 0 ? new Float64Array(heads[hi].eyes.flat()) : baseEyes;
  const wRef = r2.builder.build(start, ref), wMoved = r2.builder.build(start, moved);
  const nRef = JSON.parse(r2.builder.measure(wRef, eyes)), nMoved = JSON.parse(r2.builder.measure(wMoved, eyes));
  const measured = nRef.proportions && nMoved.proportions ? nMoved.proportions[mi] - nRef.proportions[mi] : null;
  const mat = hi >= 0 ? await ourPaint(q.head) : clayMat();
  const side = Math.random() < 0.5 ? "left" : "right";
  const mRef = new THREE.Mesh(ourGeometry(wRef, frame), mat), mMoved = new THREE.Mesh(ourGeometry(wMoved, frame), mat);
  const view = viewFor(key);
  askCur = { q, key, si, mi, change, step, measured, side, view, firstView: view, viewsSeen: [view], meshes: side === "left" ? [mMoved, mRef] : [mRef, mMoved], unit: pg.measure.proportions[mi].unit };
  [...$("askView").children].forEach((b, i) => b.classList.toggle("on", Object.keys(VIEWS)[i] === view));
  askDraw();
  $("askProgress").textContent = `Question ${ask.index + 1} of ${ask.plan.length}`;
  askBusy = false;
}
async function askAnswer(answer) {
  if (!askCur || askBusy) return;
  askBusy = true;
  const c = askCur;
  ask.answers.push({ measure: c.key, unit: c.unit, change: c.change, size: Math.abs(c.change), sign: Math.sign(c.change), measured: c.measured, head: c.q.head, side: c.side, view: c.firstView, views_seen: c.viewsSeen, answer, question: ask.index + 1, when: new Date().toISOString() });
  const r2 = await round("r2");
  const s = r2.meta[c.si], spread = s.spread ?? r2.page.measure.spread_proportions[c.mi];
  ask.stairs[c.key] = Math.min(s.high - s.low, Math.max(0.05 * spread, c.step * (answer === "different" ? 0.6 : 1.5)));
  ask.index++;
  for (const m of c.meshes) m.geometry.dispose();
  askCur = null;
  askSave();
  askMarksTable();
  madeDrawn = null;
  await askQuestion();
}
function askMarksTable() {
  const pg = r2Loaded?.page;
  if (!pg) return;
  const my = myMarks();
  let h = `<table><tr><th>Measure</th><th>Provisional pass mark</th><th>KOTOR spread</th><th>Questions answered</th><th>Called the same</th><th>Your pass mark</th><th>Next change asked</th></tr>`;
  pg.measure.proportions.forEach((p, i) => {
    const m = my[p.key], s = r2Loaded.meta.find((x) => x.measure === p.key);
    const next = ask.stairs[p.key] ?? s?.spread ?? pg.measure.spread_proportions[i];
    const mine = m?.mark != null ? `${fmt(m.mark, 2)} ${unitWord(p)}` : m?.n ? `under ${fmt(m.smallestDifferent, 2)} ${unitWord(p)}` : "no answer yet";
    h += `<tr><td>${p.label}</td><td>${fmt(pg.measure.marks.proportions[i], 2)} ${unitWord(p)}</td><td>${fmt(pg.measure.spread_proportions[i], 2)} ${unitWord(p)}</td><td>${m?.n ?? 0}</td><td>${m?.same ?? 0}</td><td>${mine}</td><td>${fmt(next, 2)} ${unitWord(p)}</td></tr>`;
  });
  $("askMarks").innerHTML = h + `</table><p class="muted">The provisional marks come from how much KOTOR's noses vary and how much the measure wobbles. Your pass mark for a measure is the largest change you still called "Same nose", as the measure finds it on the head shown. When you have called every change of a measure different, your mark is below the smallest of them, and the provisional mark stays in use. The scorecards under "Made against original" use your marks where you have them.</p>`;
}

console.log(`wasm loaded in ${fmt(wasm.loadMs, 0)} ms, mesh prepared in ${fmt(R1.prepMs, 0)} ms`);
checksSection();
drawWall();
drawStrip();
window.noseTimings = { load: wasm.loadMs, prepare: R1.prepMs };
window.nosePage = { round, ask: () => ask, myMarks };
