// Head viewer, version 2. It shows our fitted head as a character creator would, with the KOTOR original one key away.
// The loading, skinning, clip and sound code is copied from ../viewer/viewer.js, which is the checked version.
(function () {
  const DATA = "data/";
  const params = new URLSearchParams(location.search);
  const THUMBS = params.has("thumbs");
  const MUTE = params.has("mute");
  // The shape variants file. ?shapes=<address> points at another one, for testing.
  const SHAPES_URL = params.get("shapes") || DATA + "shapes.json";
  const SHAPES = ["Rest", "IH IY", "EH ER EY", "AA AE AH", "OW OY", "UH UW W", "D DH S Y Z", "CH JH SH ZH", "F V", "G HH K NG", "T TH", "B M P", "L N", "R", "AW AY", "AO"];
  const CLIPS = [["pause1", "Idle"], ["tlknorm", "Talk"], ["tlklaugh", "Laugh"], ["tlksad", "Sad"], ["tlkplead", "Plead"], ["tlkforce", "Persuade"]];
  const CLAY = new THREE.Color(0.72, 0.64, 0.58);
  const GROUPS = { player: "Player heads", common: "Commoners and elders", character: "Characters", mod: "Mod heads" };
  const $ = (id) => document.getElementById(id);
  const toast = (t) => { const s = $("status"); if (s) s.textContent = t; };

  function bytes(b64) { const s = atob(b64); const u = new Uint8Array(s.length); for (let i = 0; i < s.length; i++) u[i] = s.charCodeAt(i); return u.buffer; }
  // These take a base64 string, or an array that is already typed (the live rig's are).
  const f32 = (b) => ArrayBuffer.isView(b) ? b : new Float32Array(bytes(b));
  const u16 = (b) => ArrayBuffer.isView(b) ? b : new Uint16Array(bytes(b));
  const u8 = (b) => ArrayBuffer.isView(b) ? b : new Uint8Array(bytes(b));
  const getJSON = async (url) => { const r = await fetch(url, { cache: "no-cache" }).catch(() => null); return r && r.ok ? r.json() : null; };
  const texLoader = new THREE.TextureLoader();
  const loadTex = (url) => new Promise((res) => texLoader.load(url, res, undefined, () => res(null)));

  // --- the renderer: one canvas, one camera, two scenes (KOTOR's head and ours) ---
  const canvas = $("stage-canvas");
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true, preserveDrawingBuffer: THUMBS });
  renderer.setPixelRatio(THUMBS ? 1 : Math.min(window.devicePixelRatio || 1, 2));
  renderer.setClearColor(0x000000, 0);
  const camera = new THREE.PerspectiveCamera(22, 1, 0.01, 5);
  const target = new THREE.Vector3();
  const sceneK = new THREE.Scene(), sceneO = new THREE.Scene();
  const controls = new THREE.OrbitControls(camera, canvas);
  controls.target = target;
  controls.enableDamping = false;
  controls.minDistance = 0.12;
  controls.maxDistance = 1.6;
  controls.addEventListener("change", () => { activePreset(null); requestRender(); });

  // Lighting presets. "In game" keeps the old viewer's light, a soft sky and one key from the upper left, which is close to how KOTOR
  // lights a face in conversation. "Studio" adds a fill and a rim so that the shape reads. "Flat" has no direction, to judge colour alone.
  const LIGHTS = {
    game: [["hemi", 0xffffff, 0x444444, 0.75], ["dir", 0xffffff, 0.75, [-0.5, 0.6, 1.0]]],
    studio: [["hemi", 0xdde6ff, 0x221c18, 0.35], ["dir", 0xfff1e0, 0.95, [-0.8, 0.7, 0.9]], ["dir", 0xbfd4ff, 0.35, [0.9, 0.1, 0.6]], ["dir", 0xffffff, 0.7, [0.3, 0.6, -1.0]]],
    flat: [["amb", 0xffffff, 1.25]],
  };
  const rigs = [new THREE.Group(), new THREE.Group()];
  sceneK.add(rigs[0]); sceneO.add(rigs[1]);
  let lighting = "game";
  function applyLights() {
    for (const rig of rigs) {
      rig.clear();
      for (const l of LIGHTS[lighting]) {
        if (l[0] === "hemi") rig.add(new THREE.HemisphereLight(l[1], l[2], l[3]));
        else if (l[0] === "amb") rig.add(new THREE.AmbientLight(l[1], l[2]));
        else { const d = new THREE.DirectionalLight(l[1], l[2]); d.position.set(...l[3]); rig.add(d); }
      }
    }
    requestRender();
  }

  // --- state ---
  let common = null, oursPaints = [], shapeVariants = [], fit = null, heads = [], eyeTex = null;
  let current = null, kotor = null, kotorGroup = null, ours = null;
  // Which layers of a layered paint are on, by layer id. ?layers=101 sets them in the paint's order, for testing.
  const layerOn = {};
  let skin = null, withHair = true, compare = "ours", peek = false, split = 0.5, shape = new URLSearchParams(location.search).get("shape") || (THUMBS || !window.HeadSliders ? "fit" : "sliders");
  const cache = {}, hairCache = {}, skinCache = {}, shapeCache = {};
  // The face rig of our head, made by head_rig's mouth_rig tool from the animation calibration (head_rig's calibrate tool) and listed in
  // rigs.json. calibration.json holds the fingerprint of each head's fitted positions that the calibration learned from.
  let rigVariants = [], rig = null, calibration = null;
  // The parts round the eye that the "Eye parts" switches hide, so the eye's shape can be compared alone: the lid pieces (KOTOR's, and
  // ours with the lash strip), our caruncle, and the eye's paint (the eyeballs then take one plain light colour). ?eyes=shape starts
  // with all three hidden. kotor_eye_tris.json lists, per KOTOR head and per material, the triangles of its lid pieces and eyeballs
  // (matched to sliders/eyes/kotor_eyes.json by the eye shape session).
  const eyeParts = { lids: true, caruncle: true, paint: true };
  if (new URLSearchParams(location.search).get("eyes") === "shape") Object.assign(eyeParts, { lids: false, caruncle: false, paint: false });
  const PLAIN_EYE = new THREE.Color(0.86, 0.87, 0.9);
  let kotorEyeTris = null;
  const rigCache = {}, rigSkin = {};
  // Our own hair (roadmap part D), listed in ourhair.json by head_hair's make_hair tool. ?hair=kotor|ours|none picks the hair, and
  // ?hairpaint=0 turns off the painted hair layer.
  const HAIRS = ["kotor", "ours", "none"];
  let hairChoice = HAIRS.includes(params.get("hair")) ? params.get("hair") : "ours", hairPaint = params.get("hairpaint") !== "0", ourHairList = {};
  const ourHairCache = {};
  // The head made by the sliders (sliders.js) is the shape "sliders": the head's start shape plus the values of every slider, rebuilt
  // live. Choosing a head loads its preset, the values solved for it. Its eyeballs and upper lid pieces are head_eyes', as on the
  // slider page, and they replace our own eyeballs for this shape only.
  const slider = { kit: null, eyeKit: null, values: null, preset: null, head: null, world: null, lid: "open", panel: null, pending: false, cut: null };

  // --- skinning, as rig_preview.Rig does it (copied from viewer.js) ---
  function quat(q) { return new THREE.Quaternion(q[0], q[1], q[2], q[3]); }
  function sample(keys, t, n) {
    if (!keys || !keys.length) return null;
    if (t <= keys[0][0]) return keys[0].slice(1, 1 + n);
    const last = keys[keys.length - 1];
    if (t >= last[0]) return last.slice(1, 1 + n);
    for (let i = 0; i < keys.length - 1; i++) {
      const a = keys[i], b = keys[i + 1];
      if (b[0] > t) {
        const f = (t - a[0]) / Math.max(b[0] - a[0], 1e-6);
        let vb = b.slice(1, 1 + n);
        const va = a.slice(1, 1 + n);
        if (n === 4 && va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2] + va[3] * vb[3] < 0) vb = vb.map((x) => -x);
        const v = va.map((x, k) => x + (vb[k] - x) * f);
        if (n === 4) { const l = Math.hypot(...v); return v.map((x) => x / l); }
        return v;
      }
    }
  }
  function clipPose(clip, t) {
    const pose = {};
    for (const [name, tr] of Object.entries(clip.tracks)) pose[name] = { p: sample(tr.p, t, 3), r: sample(tr.r, t, 4) };
    return pose;
  }
  function blendPose(a, b, f) {
    const out = {};
    for (const name of new Set([...Object.keys(a), ...Object.keys(b)])) {
      const pa = a[name] || {}, pb = b[name] || {};
      const p = pa.p && pb.p ? pa.p.map((x, k) => x + (pb.p[k] - x) * f) : pa.p || pb.p;
      let r = pa.r || pb.r;
      if (pa.r && pb.r) { const q = quat(pa.r).slerp(quat(pb.r), f); r = [q.x, q.y, q.z, q.w]; }
      out[name] = { p, r };
    }
    return out;
  }
  function worlds(head, pose) {
    const out = [];
    common.bones.forEach((b, i) => {
      const hb = head.bones[i];
      let r = hb.rot;
      const ps = pose[b.name];
      // the lids stay at rest: the old lid weights tear FLAME's lids, and the new blink is not in this rig
      if (ps && ps.r && b.name !== "eyellid" && b.name !== "eyerlid") r = ps.r;
      const p = hb.local.slice();
      if (ps && ps.p) for (let k = 0; k < 3; k++) p[k] += head.keyScale[k] * ps.p[k];
      const M = new THREE.Matrix4().makeRotationFromQuaternion(quat(r));
      M.setPosition(p[0], p[1], p[2]);
      out.push(b.parent >= 0 ? new THREE.Matrix4().multiplyMatrices(out[b.parent], M) : M);
    });
    return out;
  }
  function skinMats(head, pose) {
    const A = common.toOur;
    const S = new THREE.Matrix4().set(A[0][0], A[0][1], A[0][2], 0, A[1][0], A[1][1], A[1][2], 0, A[2][0], A[2][1], A[2][2], 0, 0, 0, 0, 1);
    const W = worlds(head, pose);
    return W.map((w, i) => new THREE.Matrix4().multiplyMatrices(S, w).multiply(head.restInv[i]).multiply(S));
  }
  function smoothGroups(p) {
    const ids = new Map(), group = new Int32Array(p.length / 3);
    for (let i = 0; i < p.length; i += 3) {
      const k = `${Math.round(p[i] * 1e5)},${Math.round(p[i + 1] * 1e5)},${Math.round(p[i + 2] * 1e5)}`;
      if (!ids.has(k)) ids.set(k, ids.size);
      group[i / 3] = ids.get(k);
    }
    return { group, count: ids.size };
  }
  function smoothInto(p, sm, out) {
    const sum = new Float32Array(3 * sm.count);
    const a = new THREE.Vector3(), b = new THREE.Vector3(), c = new THREE.Vector3(), n = new THREE.Vector3(), e = new THREE.Vector3();
    for (let t = 0; t < p.length; t += 9) {
      a.fromArray(p, t); b.fromArray(p, t + 3); c.fromArray(p, t + 6);
      n.subVectors(b, a); e.subVectors(c, a); n.cross(e);
      for (let k = 0; k < 3; k++) { const g = 3 * sm.group[t / 3 + k]; sum[g] += n.x; sum[g + 1] += n.y; sum[g + 2] += n.z; }
    }
    for (let i = 0; i < p.length / 3; i++) {
      const g = 3 * sm.group[i];
      const l = Math.hypot(sum[g], sum[g + 1], sum[g + 2]) || 1;
      out[3 * i] = sum[g] / l; out[3 * i + 1] = sum[g + 1] / l; out[3 * i + 2] = sum[g + 2] / l;
    }
  }
  function smoothNormals(p) {
    const sm = smoothGroups(p), out = new Float32Array(p.length);
    smoothInto(p, sm, out);
    return out;
  }
  function posedKotor(pose) {
    const k = kotor;
    if (!k) return;
    const mats = skinMats(k, pose).map((m) => new THREE.Matrix4().multiplyMatrices(k.toFit, m).multiply(k.fromFit).elements);
    mats.push(new THREE.Matrix4().elements);
    for (const m of k.meshes) {
      const out = m.mesh.geometry.attributes.position.array, base = m.rest, n = base.length / 3;
      for (let v = 0; v < n; v++) {
        const x = base[3 * v], y = base[3 * v + 1], z = base[3 * v + 2];
        let ox = 0, oy = 0, oz = 0, ws = 0;
        for (let j = 0; j < 4; j++) {
          const w = m.w[4 * v + j];
          if (w <= 0) continue;
          const e = mats[m.idx[4 * v + j] === 255 ? mats.length - 1 : m.idx[4 * v + j]];
          ox += w * (e[0] * x + e[4] * y + e[8] * z + e[12]);
          oy += w * (e[1] * x + e[5] * y + e[9] * z + e[13]);
          oz += w * (e[2] * x + e[6] * y + e[10] * z + e[14]);
          ws += w;
        }
        if (ws <= 0) { out[3 * v] = x; out[3 * v + 1] = y; out[3 * v + 2] = z; continue; }
        out[3 * v] = ox / ws; out[3 * v + 1] = oy / ws; out[3 * v + 2] = oz / ws;
      }
      m.mesh.geometry.attributes.position.needsUpdate = true;
      smoothInto(out, m.smooth, m.mesh.geometry.attributes.normal.array);
      m.mesh.geometry.attributes.normal.needsUpdate = true;
    }
  }
  function posed(pose) {
    lastPose = pose;
    posedKotor(pose);
    const h = ours;
    if (!h) return;
    const topo = h.topo || common;
    const n = topo.n, base = h.pos, out = h.geo.attributes.position.array;
    const mats = skinMats(h, pose).map((m) => m.elements);
    const idx = h.skinIndex || topo.skinIndex, wts = h.skinWeight || topo.skinWeight;
    for (let v = 0; v < n; v++) {
      const x = base[3 * v], y = base[3 * v + 1], z = base[3 * v + 2];
      let ox = 0, oy = 0, oz = 0, ws = 0;
      for (let k = 0; k < 4; k++) {
        const w = wts[4 * v + k];
        if (w <= 0) continue;
        const e = mats[idx[4 * v + k]];
        ox += w * (e[0] * x + e[4] * y + e[8] * z + e[12]);
        oy += w * (e[1] * x + e[5] * y + e[9] * z + e[13]);
        oz += w * (e[2] * x + e[6] * y + e[10] * z + e[14]);
        ws += w;
      }
      out[3 * v] = ox / ws; out[3 * v + 1] = oy / ws; out[3 * v + 2] = oz / ws;
    }
    keepOutOfEyes(base, out, n, h.sliderEyes ? h.sliderBalls : h.eyes.userData.balls, mats[common.headBone]);
    const E = topo.edges;
    for (let k = 0; k < E.length / 2; k++) {
      const a = E[2 * k], b = E[2 * k + 1], o = 3 * (n + k);
      out[o] = (out[3 * a] + out[3 * b]) / 2; out[o + 1] = (out[3 * a + 1] + out[3 * b + 1]) / 2; out[o + 2] = (out[3 * a + 2] + out[3 * b + 2]) / 2;
    }
    h.geo.attributes.position.needsUpdate = true;
    h.geo.computeVertexNormals();
    weldNormals(h.geo);
    if (topo.fixNormals) topo.fixNormals(h.geo.attributes.normal.array);
    h.eyes.matrix.fromArray(mats[common.headBone]);
    h.headMat = mats[common.headBone];
    if (h.sliderEyes) h.sliderEyes.matrix.fromArray(h.headMat);
    turnEyes(h, mats, pose);
    if (h.hair) h.hair.matrix.fromArray(mats[common.headBone]);
    if (h.ourHair) h.ourHair.matrix.fromArray(mats[common.headBone]);
    for (const t of h.teeth || []) t.mesh.matrix.fromArray(mats[t.bone]);
    if (h.tongue) {
      const a = mats[boneIndex("f_jaw_g")], b = mats[boneIndex("f_tonguetip_g")], rest = h.tongue.rest, wt = h.tongue.weight;
      const out = h.tongue.mesh.geometry.attributes.position.array;
      for (let i = 0; i < wt.length; i++) {
        const x = rest[3 * i], y = rest[3 * i + 1], z = rest[3 * i + 2], w = wt[i];
        for (let k = 0; k < 3; k++) out[3 * i + k] = (1 - w) * (a[k] * x + a[4 + k] * y + a[8 + k] * z + a[12 + k]) + w * (b[k] * x + b[4 + k] * y + b[8 + k] * z + b[12 + k]);
      }
      h.tongue.mesh.geometry.attributes.position.needsUpdate = true;
      h.tongue.mesh.geometry.computeVertexNormals();
    }
    requestRender();
  }
  // The two eyeballs of the fitted shape as spheres: each half of the eyes' mesh is one sphere, with its points' mean as the centre.
  function eyeballSpheres(p) {
    const half = p.length / 6;
    return [0, 1].map((e) => {
      const c = [0, 0, 0];
      for (let i = e * half; i < (e + 1) * half; i++) for (let k = 0; k < 3; k++) c[k] += p[3 * i + k] / half;
      let r = 0;
      for (let i = e * half; i < (e + 1) * half; i++) r += Math.hypot(p[3 * i] - c[0], p[3 * i + 1] - c[1], p[3 * i + 2] - c[2]) / half;
      return { c, r };
    });
  }
  // The face's bones can pull the lids into the eyeball, as the lower lid in the laugh. No posed skin point may come nearer an eyeball's
  // centre than it was at rest, and a point outside the eyeball at rest stays at least 0.5 mm outside it, which is more than a flat triangle near the eye dips into the curved eyeball. A point that comes too near is
  // put back along its line from the centre. So the skin never covers less of the eyeball than at rest, and the eyeball never shows
  // through it. The eyeballs ride the head bone, whose matrix is `head`.
  function keepOutOfEyes(rest, out, n, balls, head) {
    if (!balls) return;
    for (const { c, r } of balls) {
      const cx = head[0] * c[0] + head[4] * c[1] + head[8] * c[2] + head[12];
      const cy = head[1] * c[0] + head[5] * c[1] + head[9] * c[2] + head[13];
      const cz = head[2] * c[0] + head[6] * c[1] + head[10] * c[2] + head[14];
      const reach = r + 0.0005;
      for (let v = 0; v < n; v++) {
        const dx = out[3 * v] - cx, dy = out[3 * v + 1] - cy, dz = out[3 * v + 2] - cz, d = Math.hypot(dx, dy, dz);
        if (d >= reach || d === 0) continue;
        const floor = Math.min(reach, Math.hypot(rest[3 * v] - c[0], rest[3 * v + 1] - c[1], rest[3 * v + 2] - c[2]));
        if (d >= floor) continue;
        out[3 * v] = cx + dx * floor / d; out[3 * v + 1] = cy + dy * floor / d; out[3 * v + 2] = cz + dz * floor / d;
      }
    }
  }
  // KOTOR's clips turn each eyeball with its eye bone and close the upper lid with its lid bone. The eyeballs turn about their own
  // centres, because our rig's eye bones are not placed on them. The slider head's lid piece turns about its hinge from open to its
  // closing angle as KOTOR's lid turns from rest to 30 degrees closed. A lid set to "Closed" in the panel stays closed.
  const EYE_BONES = [["eyela", "eyellid"], ["eyera", "eyerlid"]], LID_CLOSED_DEG = 30;
  const boneAt = {};
  function boneIndex(name) {
    if (!(name in boneAt)) boneAt[name] = common.bones.findIndex((b) => b.name === name);
    return boneAt[name];
  }
  function turnEyes(h, mats, pose) {
    const balls = h.sliderEyes ? h.sliderBalls : h.eyes.userData.balls;
    const geo = h.sliderEyes ? h.sliderEyes.children[0].geometry : h.eyes.geometry;
    if (!balls || !geo) return;
    const p = geo.attributes.position.array, rest = geo.userData.rest || (geo.userData.rest = p.slice()), half = rest.length / 6;
    const headInv = new THREE.Matrix4().fromArray(mats[common.headBone]).invert();
    EYE_BONES.forEach(([eye, lid], e) => {
      const m = new THREE.Matrix4().multiplyMatrices(headInv, new THREE.Matrix4().fromArray(mats[boneIndex(eye)])).elements, c = balls[e].c;
      for (let i = e * half; i < (e + 1) * half; i++) {
        const x = rest[3 * i] - c[0], y = rest[3 * i + 1] - c[1], z = rest[3 * i + 2] - c[2];
        p[3 * i] = c[0] + m[0] * x + m[4] * y + m[8] * z;
        p[3 * i + 1] = c[1] + m[1] * x + m[5] * y + m[9] * z;
        p[3 * i + 2] = c[2] + m[2] * x + m[6] * y + m[10] * z;
      }
      const piece = h.sliderEyes && h.sliderEyes.children[1 + e];
      if (!piece || piece.userData.closedAngle === undefined) return;
      const q = pose[lid] && pose[lid].r;
      let shut = 0;
      if (q) {
        const rel = quat(h.bones[boneIndex(lid)].rot).invert().multiply(quat(q));
        if (rel.w < 0) rel.set(-rel.x, -rel.y, -rel.z, -rel.w);
        shut = Math.min(1, Math.max(0, (-2 * Math.atan2(rel.x, rel.w) * 180) / Math.PI / LID_CLOSED_DEG));
      }
      const turn = (piece.userData.closedAngle * Math.PI / 180) * (piece.userData.closed ? 1 : shut);
      piece.quaternion.setFromAxisAngle(piece.userData.axis, turn);
    });
    geo.attributes.position.needsUpdate = true;
    geo.computeVertexNormals();
  }
  // The viewer mesh splits vertices on both layouts' seams, so several vertices share one place. computeVertexNormals() gives each copy only
  // its own side's normal, which shows as facets. Each group of copies, found once from the rest positions, gets the mean of its normals.
  function weldGroups(p) {
    const at = new Map();
    for (let i = 0; i < p.length / 3; i++) {
      const k = `${Math.round(p[3 * i] * 1e6)},${Math.round(p[3 * i + 1] * 1e6)},${Math.round(p[3 * i + 2] * 1e6)}`;
      const g = at.get(k);
      if (g) g.push(i); else at.set(k, [i]);
    }
    return [...at.values()].filter((g) => g.length > 1);
  }
  function weldNormals(geo) {
    const n = geo.attributes.normal.array;
    for (const g of geo.userData.weld || []) {
      let x = 0, y = 0, z = 0;
      for (const i of g) { x += n[3 * i]; y += n[3 * i + 1]; z += n[3 * i + 2]; }
      const l = Math.hypot(x, y, z) || 1;
      for (const i of g) { n[3 * i] = x / l; n[3 * i + 1] = y / l; n[3 * i + 2] = z / l; }
    }
    geo.attributes.normal.needsUpdate = true;
  }
  let lastPose = {};

  // --- loading ---
  async function loadCommon() {
    const c = await getJSON(DATA + "common.json");
    if (!c) throw new Error("common.json is missing");
    c.tris = u16(c.tris); c.edges = u16(c.edges); c.skinIndex = u8(c.skinIndex); c.skinWeight = f32(c.skinWeight);
    c.headBone = c.bones.findIndex((b) => b.name === "head_g");
    if (c.uv) c.uv = f32(c.uv);
    if (c.uvOld) c.uvOld = f32(c.uvOld);
    common = c;
    // Our own paints, listed by the tool that bakes them. The default is the variant marked "default", or else the last one, which is the newest.
    const o = await getJSON(DATA + "ours.json");
    oursPaints = o && o.variants ? o.variants : [];
    const best = oursPaints.find((v) => v.default) || oursPaints[oursPaints.length - 1];
    skin = best ? "ours:" + best.id : "kotor";
    // The live paint (paint_live.js) is the default where the browser has WebGPU. Without it the head keeps our newest baked paint.
    if (!THUMBS && window.HeadPaint && window.HeadPaint.start({
      THUMBS, toast, applySkin, requestRender, scenes: [sceneK, sceneO], camera, target, maxAnisotropy: renderer.capabilities.getMaxAnisotropy(),
      get skin() { return skin; }, get ours() { return ours; },
      setSkin: (k) => { skin = k; applySkin(); },
      bakedKey: () => (bestPaint() ? "ours:" + bestPaint().id : "kotor"),
      baked: (h) => paintTexture(h, bestPaint() ? "ours:" + bestPaint().id : "kotor"),
      world: (h) => (h && h.slider && slider.world && slider.head === h.name ? slider.world : null),
    }) && params.get("skin") !== "baked") skin = "live";
    if (!eyeParts.paint && skin !== "none") { eyeParts.skinBefore = skin; skin = "none"; }
    if (!THUMBS && window.HeadPaint && window.HeadPaint.failure()) toast(window.HeadPaint.failure());
    const startLayers = params.get("layers");
    if (best && best.layers && startLayers) best.layers.forEach((l, i) => { layerOn[l.id] = startLayers[i] !== "0"; });
    // Other versions of our head's shape, when an agent has written them. Without the file the Shape choice stays hidden.
    const s = await getJSON(SHAPES_URL);
    shapeVariants = s && s.variants ? s.variants : [];
    const oh = await getJSON(DATA + "ourhair.json");
    ourHairList = oh && oh.heads ? oh.heads : {};
    // Our eyeballs' textures (eye_texture.py). Without the file the eyeballs keep their colour per vertex.
    eyeTex = await getJSON(DATA + "eyes.json");
    kotorEyeTris = await getJSON("kotor_eye_tris.json");
    const rj = await getJSON(DATA + "rigs.json");
    rigVariants = rj && rj.variants ? rj.variants : [];
    rig = rigVariants.length ? rigVariants[0].id : null;
    calibration = await getJSON(DATA + "calibration.json");
    fit = await getJSON("fit.json");
    heads = c.heads.map((h) => {
      const i = (window.HEAD_INFO || {})[h.name] || {};
      return { name: h.name, label: i.label || h.title || h.name, group: i.group || "character", sex: i.sex || "", number: h.number };
    });
  }

  async function headData(name) {
    if (!cache[name]) cache[name] = await getJSON(DATA + "heads/" + name + ".json");
    if (!(name in hairCache)) hairCache[name] = await getJSON(DATA + "hair/" + name + ".json");
    if (!(name in ourHairCache)) ourHairCache[name] = ourHairList[name] ? await getJSON(DATA + ourHairList[name].file) : null;
    if (!(name in skinCache)) skinCache[name] = await getJSON(DATA + "kskin/" + name + ".json");
    return cache[name];
  }
  async function shapeData(name) {
    if (shape === "fit") return null;
    const v = shapeVariants.find((x) => x.id === shape);
    if (!v) return null;
    const key = shape + "/" + name;
    if (!(key in shapeCache)) shapeCache[key] = await getJSON(new URL(v.dir.replace(/\/?$/, "/") + name + ".json", new URL(SHAPES_URL, location.href)).href);
    return shapeCache[key];
  }
  // A rig variant gives each head its bones, key scale and teeth, and replaces the skin weights of some vertices of common.json.
  async function rigData(name, topo) {
    const v = rigVariants.find((x) => x.id === rig);
    if (!v) return null;
    const overrides = (s) => {
      const idx = topo.skinIndex.slice(), w = topo.skinWeight.slice(), vs = u16(s.vertices), si = u8(s.skinIndex), sw = f32(s.skinWeight);
      vs.forEach((vert, i) => { for (let k = 0; k < 4; k++) { idx[4 * vert + k] = si[4 * i + k]; w[4 * vert + k] = sw[4 * i + k]; } });
      if (topo.fixSkin) topo.fixSkin(idx, w);
      return { idx, w };
    };
    // A variant whose weights are computed per head ("perHead") carries them in each head's file, not in one skin.json.
    const skinKey = v.id + (topo === common ? "" : "|" + topo.n);
    if (!(skinKey in rigSkin)) {
      const s = v.perHead ? null : await getJSON(DATA + v.dir + "/skin.json");
      rigSkin[skinKey] = s && s.n === common.n ? overrides(s) : null;
    }
    const key = v.id + "/" + name;
    if (!(key in rigCache)) rigCache[key] = await getJSON(DATA + v.dir + "/" + name + ".json");
    const r = rigCache[key];
    if (!r) return null;
    return { ...r, skin: r.skin ? overrides(r.skin) : rigSkin[skinKey] };
  }

  // The rig of a head made by the sliders is computed from its own shape, with head_rig in WebAssembly: the same code and the same
  // calibration files (data/calibration/) as the mouth_rig tool that writes rigs/calibrated. It is computed again whenever the head is
  // rebuilt. The shapes other than "sliders" keep the rig from the file.
  let liveKitP = null, liveKit = null;
  const liveBones = {};
  function liveRigKit() {
    return liveKitP || (liveKitP = (async () => {
      const [L, w, r, l] = await Promise.all([import("/wasm/load.js?v=22"), ...["weights_model", "regressor", "lips"].map((f) => fetch(DATA + "calibration/" + f + ".json", { cache: "no-cache" }).then((x) => { if (!x.ok) throw new Error(f + ".json: " + x.status); return x.text(); }))]);
      const m = await L.loadCrate("head_rig", "/wasm", "2");
      liveKit = new m.RigMaker(JSON.stringify(common.bones.map((b) => ({ name: b.name, parent: b.parent }))), w, r, l);
      return liveKit;
    })().catch((e) => { console.error("the live rig", e); toast("The rig could not be computed from the sliders, so the rig from the file is used: " + e.message); return null; }));
  }
  // The bones of the head's fit, which the regressor moves: the bones it does not place keep their world places.
  function liveBase(name) {
    if (!liveBones[name]) liveBones[name] = Float32Array.from(cache[name].ours.bones.flatMap((b) => [...b.local, ...b.rot]));
    return liveBones[name];
  }
  // The wasm rig of the head whose positions are `pos` (before the lip line is opened).
  function liveRigRaw(name, pos) {
    const t0 = performance.now();
    const r = liveKit.rig(pos.subarray(0, 3 * common.n), liveBase(name));
    slider.rigMs = performance.now() - t0;
    return r;
  }
  function liveSkinInto(idx, w, r) {
    const vs = r.skin_vertices(), si = r.skin_index(), sw = r.skin_weight();
    vs.forEach((vert, i) => { for (let k = 0; k < 4; k++) { idx[4 * vert + k] = si[4 * i + k]; w[4 * vert + k] = sw[4 * i + k]; } });
  }
  function liveBoneList(r) {
    const b = r.bones(), out = [];
    for (let i = 0; i < b.length; i += 7) out.push({ local: [b[i], b[i + 1], b[i + 2]], rot: [b[i + 3], b[i + 4], b[i + 5], b[i + 6]] });
    return out;
  }
  function liveLift(r) {
    const up = r.lift_up(), delta = new Float32Array(3 * up.length);
    up.forEach((d, i) => { delta[3 * i + 1] = d; });
    return { vertices: r.lift_vertices(), delta };
  }
  // The rig of a slider head in the shape of a rig file, for loadHead. `file` is the head's rig from the file, which names the teeth texture.
  async function liveRigData(name, pos, file, topo) {
    if (!(await liveRigKit())) return null;
    const r = liveRigRaw(name, pos);
    const key = r.key_scale();
    const idx = topo.skinIndex.slice(), w = topo.skinWeight.slice();
    liveSkinInto(idx, w, r);
    if (topo.fixSkin) topo.fixSkin(idx, w);
    const texture = file && file.teeth && file.teeth[0] ? file.teeth[0].texture : "mouth/teeth_v2.png";
    const out = {
      bones: liveBoneList(r), keyScale: [key, key, key], skin: { idx, w }, posDelta: liveLift(r), shade: { vertices: r.shade_vertices(), colour: r.shade_colour() },
      teeth: [0, 1].map((i) => ({ bone: r.teeth_bone(i), pos: r.teeth_positions(i), uv: r.teeth_uv(i), texture })),
      tongue: r.tongue_positions().length ? { pos: r.tongue_positions(), weight: r.tongue_weight(), colour: r.tongue_colour() } : null,
    };
    r.free();
    return out;
  }
  // Puts a new rig of the slider head on it: the bones, the weights, the lip line, the shade of the mouth's inside and the teeth.
  function applyLiveRig(h, pos) {
    const r = liveRigRaw(h.name, pos), key = r.key_scale();
    h.bones = liveBoneList(r); h.keyScale = [key, key, key];
    liveSkinInto(h.skinIndex, h.skinWeight, r);
    if (h.topo.fixSkin) h.topo.fixSkin(h.skinIndex, h.skinWeight);
    h.posDelta = liveLift(r);
    const E = h.topo.edges, vs = r.shade_vertices(), cs = r.shade_colour(), col = h.geo.attributes.color;
    if (col) {
      const c = col.array;
      vs.forEach((v, i) => { c[3 * v] = cs[3 * i]; c[3 * v + 1] = cs[3 * i + 1]; c[3 * v + 2] = cs[3 * i + 2]; });
      for (let k = 0; k < E.length / 2; k++) for (let j = 0; j < 3; j++) c[3 * (h.topo.n + k) + j] = (c[3 * E[2 * k] + j] + c[3 * E[2 * k + 1] + j]) / 2;
      col.needsUpdate = true;
    }
    (h.teeth || []).forEach((t, i) => {
      const g = t.mesh.geometry;
      g.attributes.position.array.set(r.teeth_positions(i));
      g.attributes.position.needsUpdate = true;
      g.computeVertexNormals();
    });
    if (h.tongue && r.tongue_positions().length) { h.tongue.rest = r.tongue_positions(); h.tongue.weight = r.tongue_weight(); }
    r.free();
    h.restInv = worlds(h, {}).map((m) => new THREE.Matrix4().copy(m).invert());
    h.liveRig = true;
  }

  // A layered paint has one baked texture for every combination of its layers. Its file name holds a 1 or a 0 for each layer, in order.
  function skinVariant() { return oursPaints.find((p) => "ours:" + p.id === skin); }
  function layerBits(v) { return v.layers.map((l) => (layerOn[l.id] === false ? "0" : "1")).join(""); }
  function paintKey() { const v = skinVariant(); return v && v.layers ? skin + "|" + layerBits(v) : skin; }
  function paintFile(key) {
    if (key === "kotor") return common.oursTextureNew || common.oursTexture;
    if (key.startsWith("ours:")) {
      const [id, bits] = key.slice(5).split("|");
      const v = oursPaints.find((p) => p.id === id);
      return v ? v.file.replace("{layers}", bits || "") : null;
    }
    return null;
  }
  async function paintTexture(h, key) {
    if (key === "none") return null;
    if (key === "live") return window.HeadPaint ? window.HeadPaint.texture(h) : null;
    if (!(key in h.paints)) {
      const file = paintFile(key);
      h.paints[key] = file ? loadTex(DATA + "tex/" + file.replace("{name}", h.name)).then((t) => { if (t) t.anisotropy = renderer.capabilities.getMaxAnisotropy(); return t; }) : Promise.resolve(null);
    }
    return h.paints[key];
  }

  // --- the body under the head (the "Body" switch) ---
  // body/<name>.json holds the KOTOR body the head hangs on, at rest and in the head's frame (kotor_viewer_body.py). One copy goes in each
  // scene, so that KOTOR's head and ours both stand on it.
  let bodyOn = false, bodyGroups = [], bodyToken = 0;
  async function loadBody(name) {
    const token = ++bodyToken;
    const b = await getJSON(DATA + "body/" + name + ".json");
    const textures = b ? await Promise.all(b.materials.map((m) => m.texture ? loadTex(DATA + "tex/" + m.texture) : null)) : [];
    if (token !== bodyToken) return;
    for (const g of bodyGroups) g.parent && g.parent.remove(g);
    bodyGroups = [];
    if (!b) return;
    for (const scene of [sceneK, sceneO]) {
      const g = new THREE.Group();
      b.materials.forEach((m, i) => {
        const geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(f32(m.pos), 3));
        geo.setAttribute("uv", new THREE.BufferAttribute(f32(m.uv), 2));
        geo.setAttribute("normal", new THREE.BufferAttribute(smoothNormals(geo.attributes.position.array), 3));
        const mesh = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, side: THREE.DoubleSide, map: textures[i] || null }));
        mesh.frustumCulled = false;
        g.add(mesh);
      });
      g.visible = bodyOn;
      scene.add(g);
      bodyGroups.push(g);
    }
    requestRender();
  }
  function setBody(on) {
    bodyOn = !!on;
    for (const g of bodyGroups) g.visible = bodyOn;
    const b = document.querySelector("[data-body]");
    if (b) b.setAttribute("aria-pressed", bodyOn ? "true" : "false");
    requestRender();
  }

  let loadToken = 0;
  async function loadHead(name) {
    const token = ++loadToken;
    document.body.classList.add("loading");
    const d = await headData(name);
    const sd = await shapeData(name);
    const slk = shape === "sliders" ? await sliderKit() : null;
    // a slider head with the finer triangles below the lower lids has its own mesh (sliders.js refinedTopo)
    const topo = slk && slk.has(name) && slider.rtopo ? slider.rtopo : common;
    let rd = await rigData(name, topo);
    if (token !== loadToken || !d) return;
    // KOTOR's head, with its own skin weights when kskin/<name>.json has them
    const kg = new THREE.Group();
    const ktex = [];
    for (const m of d.kotor.materials) {
      const g = new THREE.BufferGeometry();
      g.setAttribute("position", new THREE.BufferAttribute(f32(m.pos), 3));
      g.setAttribute("uv", new THREE.BufferAttribute(f32(m.uv), 2));
      g.setAttribute("normal", new THREE.BufferAttribute(smoothNormals(g.attributes.position.array), 3));
      const mat = new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, side: THREE.DoubleSide });
      if (m.texture) ktex.push(loadTex(DATA + "tex/" + m.texture).then((t) => { mat.userData.map = t; }));
      const mesh = new THREE.Mesh(g, mat);
      mesh.frustumCulled = false;
      kg.add(mesh);
    }
    let k = null;
    const sk = skinCache[name];
    if (sk && sk.materials.length === kg.children.length) {
      const B = sk.toFit;
      const toFit = new THREE.Matrix4().set(B[0][0], B[0][1], B[0][2], B[0][3], B[1][0], B[1][1], B[1][2], B[1][3], B[2][0], B[2][1], B[2][2], B[2][3], 0, 0, 0, 1);
      k = { bones: sk.bones, keyScale: sk.keyScale, toFit, fromFit: new THREE.Matrix4().copy(toFit).invert(), meshes: [] };
      k.restInv = worlds(k, {}).map((m) => new THREE.Matrix4().copy(m).invert());
      kg.children.forEach((mesh, i) => {
        const rest = mesh.geometry.attributes.position.array.slice();
        k.meshes.push({ mesh, rest, idx: u8(sk.materials[i].idx), w: f32(sk.materials[i].w), smooth: smoothGroups(rest) });
      });
    }
    // KOTOR's lid pieces and eyeballs, for the Eye parts switches. A plain eyeball mesh shares the material's positions and normals,
    // so it follows the pose.
    const kparts = kotorEyeTris && kotorEyeTris[name];
    if (kparts && kparts.length === kg.children.length) {
      [...kg.children].forEach((mesh, i) => {
        const g = mesh.geometry, n = g.attributes.position.count / 3, lid = new Set(kparts[i].lid), ball = new Set(kparts[i].ball);
        const tri = (t) => [3 * t, 3 * t + 1, 3 * t + 2];
        g.setIndex([...Array(n).keys()].flatMap(tri));
        mesh.userData.eyeTris = { n, lid, ball };
        if (!ball.size) return;
        const pg = new THREE.BufferGeometry();
        pg.setAttribute("position", g.attributes.position);
        pg.setAttribute("normal", g.attributes.normal);
        pg.setIndex([...ball].flatMap(tri));
        const plain = new THREE.Mesh(pg, new THREE.MeshStandardMaterial({ roughness: 0.5, metalness: 0, side: THREE.DoubleSide }));
        plain.frustumCulled = false;
        plain.userData.plainEye = true;
        plain.visible = false;
        kg.add(plain);
      });
    }
    // ours
    const o = d.ours;
    const pos = slk && slk.has(name) ? sliderPos(slk, name) : sd && sd.pos ? f32(sd.pos).slice() : f32(o.pos).slice();
    let live = false;
    if (slk && slk.has(name)) {
      const lr = await liveRigData(name, pos, rd, topo);
      if (token !== loadToken) return;
      if (lr) { rd = lr; live = true; }
    }
    // A rig variant may open the lip line: small moves added to whichever shape is shown.
    if (rd && rd.posDelta) {
      const vs = u16(rd.posDelta.vertices), dl = f32(rd.posDelta.delta);
      vs.forEach((v, i) => { for (let c = 0; c < 3; c++) pos[3 * v + c] += dl[3 * i + c]; });
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(3 * (topo.n + topo.edges.length / 2)), 3));
    geo.setAttribute("uv", new THREE.BufferAttribute(topo.uv || common.uv, 2));
    geo.setIndex(new THREE.BufferAttribute(topo.tris, 1));
    {
      const rest = new Float32Array(3 * (topo.n + topo.edges.length / 2)), E = topo.edges;
      rest.set(pos.subarray(0, 3 * topo.n));
      for (let k = 0; k < E.length / 2; k++) for (let c = 0; c < 3; c++) rest[3 * (topo.n + k) + c] = (rest[3 * E[2 * k] + c] + rest[3 * E[2 * k + 1] + c]) / 2;
      geo.userData.weld = weldGroups(rest);
    }
    const mat = new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, side: THREE.DoubleSide });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.frustumCulled = false;
    const eg = new THREE.BufferGeometry();
    eg.setAttribute("position", new THREE.BufferAttribute(f32(o.eyes.pos), 3));
    const ec = u8(o.eyes.colour);
    eg.setAttribute("color", new THREE.BufferAttribute(new Float32Array(ec.length).map((_, i) => ec[i] / 255), 3));
    eg.setIndex(new THREE.BufferAttribute(u16(o.eyes.tris), 1));
    eg.computeVertexNormals();
    eg.computeBoundingBox();
    // Each eyeball is a sphere of `rings` rings with 2 * rings points each, made by kotor_paint.eyeballs. Its texture (eye_texture.py) is the
    // eyeball seen from the front, left eye in the left half: a point made from the unit point (x, y, z) reads (x / 4 + 0.25 + half, y / 2 + 0.5).
    let eyeMap = null;
    const R = eyeTex && eyeTex.rings, per = R ? (R + 1) * 2 * R : 0;
    if (R && eg.attributes.position.count === 2 * per) {
      const uv = new Float32Array(4 * per);
      for (let i = 0; i < 2 * per; i++) {
        const j = i % per, a = Math.floor(j / (2 * R)) * Math.PI / R, b = (j % (2 * R)) * Math.PI / R;
        uv[2 * i] = Math.sin(a) * Math.cos(b) / 4 + 0.25 + (i < per ? 0 : 0.5);
        uv[2 * i + 1] = Math.sin(a) * Math.sin(b) / 2 + 0.5;
      }
      eg.setAttribute("uv", new THREE.BufferAttribute(uv, 2));
      eyeMap = loadTex(DATA + "tex/" + eyeTex.file.replace("{name}", name)).then((t) => { if (t) t.anisotropy = renderer.capabilities.getMaxAnisotropy(); return t; });
    }
    const emat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.4, metalness: 0 });
    const eyes = new THREE.Mesh(eg, emat);
    eyes.userData.balls = eyeballSpheres(f32(o.eyes.pos));
    eyes.matrixAutoUpdate = false;
    eyes.frustumCulled = false;
    const group = new THREE.Group();
    group.add(mesh, eyes);
    let hair = null;
    const hd = hairCache[name];
    const htex = [];
    if (hd && hd.parts.length) {
      hair = new THREE.Group();
      hair.matrixAutoUpdate = false;
      for (const m of hd.parts) {
        const g = new THREE.BufferGeometry();
        g.setAttribute("position", new THREE.BufferAttribute(f32(m.pos), 3));
        g.setAttribute("uv", new THREE.BufferAttribute(f32(m.uv), 2));
        g.setAttribute("normal", new THREE.BufferAttribute(smoothNormals(g.attributes.position.array), 3));
        const hm = new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, side: THREE.DoubleSide });
        if (m.texture) htex.push(loadTex(DATA + "tex/" + m.texture).then((t) => { hm.userData.map = t; }));
        const hmesh = new THREE.Mesh(g, hm);
        hmesh.frustumCulled = false;
        hair.add(hmesh);
      }
      group.add(hair);
    }
    // Our hair: the cap and the cards, moved by the head bone like KOTOR's hair, and the painted hair layer, put over our paint.
    let ourHair = null, hairLayer = null;
    const ohd = ourHairCache[name];
    if (ohd && ohd.parts.length) {
      ourHair = new THREE.Group();
      ourHair.matrixAutoUpdate = false;
      for (const m of ohd.parts) {
        const g = new THREE.BufferGeometry();
        g.setAttribute("position", new THREE.BufferAttribute(f32(m.pos), 3));
        g.setAttribute("uv", new THREE.BufferAttribute(f32(m.uv), 2));
        g.setAttribute("normal", new THREE.BufferAttribute(m.normal ? f32(m.normal) : smoothNormals(g.attributes.position.array), 3));
        const hm = new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, side: THREE.DoubleSide, alphaTest: m.alpha ? 0.5 : 0 });
        if (m.texture) htex.push(loadTex(DATA + "tex/" + m.texture).then((t) => { hm.userData.map = t; }));
        const hmesh = new THREE.Mesh(g, hm);
        hmesh.frustumCulled = false;
        ourHair.add(hmesh);
      }
      group.add(ourHair);
      if (ohd.paintLayer) htex.push(loadTex(DATA + "tex/" + ohd.paintLayer).then((t) => { hairLayer = t ? t.image : null; }));
    }
    const h = { name, paints: {}, pos, geo, mat, emat, eyes, group, hair, hairData: hd, ourHair, ourHairData: ohd, get hairLayer() { return hairLayer; }, bones: (sd && sd.bones) || o.bones, keyScale: (sd && sd.keyScale) || o.keyScale, topo };
    if (rd) {
      h.bones = rd.bones; h.keyScale = rd.keyScale;
      if (rd.skin) { h.skinIndex = rd.skin.idx; h.skinWeight = rd.skin.w; }
      // The inside of the mouth, darkened by its depth behind the lips: a colour per vertex that multiplies the paint.
      if (rd.shade) {
        const E = topo.edges, total = topo.n + E.length / 2, col = new Float32Array(3 * total).fill(1);
        const vs = u16(rd.shade.vertices), cs = f32(rd.shade.colour);
        vs.forEach((v, i) => { col[3 * v] = cs[3 * i]; col[3 * v + 1] = cs[3 * i + 1]; col[3 * v + 2] = cs[3 * i + 2]; });
        for (let k = 0; k < E.length / 2; k++) for (let c = 0; c < 3; c++) col[3 * (topo.n + k) + c] = (col[3 * E[2 * k] + c] + col[3 * E[2 * k + 1] + c]) / 2;
        geo.setAttribute("color", new THREE.BufferAttribute(col, 3));
        mat.vertexColors = true;
      }
      h.teeth = [];
      for (const t of rd.teeth || []) {
        const g = new THREE.BufferGeometry();
        g.setAttribute("position", new THREE.BufferAttribute(f32(t.pos), 3));
        g.setAttribute("uv", new THREE.BufferAttribute(f32(t.uv), 2));
        g.computeVertexNormals();
        const tm = new THREE.MeshStandardMaterial({ roughness: 0.5, metalness: 0, side: THREE.DoubleSide });
        htex.push(loadTex(DATA + "tex/" + t.texture).then((tx) => { tm.map = tx; tm.needsUpdate = true; }));
        const mesh = new THREE.Mesh(g, tm);
        mesh.matrixAutoUpdate = false;
        mesh.frustumCulled = false;
        group.add(mesh);
        h.teeth.push({ mesh, bone: common.bones.findIndex((b) => b.name === t.bone) });
      }
      // The tongue: its back follows the jaw and its front the tongue bone, blended per corner in posed().
      if (rd.tongue) {
        const rest = f32(rd.tongue.pos), g = new THREE.BufferGeometry();
        g.setAttribute("position", new THREE.BufferAttribute(rest.slice(), 3));
        g.setAttribute("color", new THREE.BufferAttribute(f32(rd.tongue.colour), 3));
        g.computeVertexNormals();
        const mesh = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.45, metalness: 0, side: THREE.DoubleSide }));
        mesh.frustumCulled = false;
        group.add(mesh);
        h.tongue = { mesh, rest, weight: f32(rd.tongue.weight) };
      }
    }
    h.restInv = worlds(h, {}).map((m) => new THREE.Matrix4().copy(m).invert());
    h.liveRig = live;
    if (slk && slk.has(name)) { h.slider = true; h.posDelta = rd && rd.posDelta ? rd.posDelta : null; eyes.visible = false; }
    const L = common.lipLine;
    h.mouth = new THREE.Vector3(pos[3 * L], pos[3 * L + 1], pos[3 * L + 2]);
    h.headCentre = new THREE.Vector3(0, h.mouth.y + 0.07, h.mouth.z - 0.07);
    h.eyeCentre = eg.boundingBox.getCenter(new THREE.Vector3());
    if (eyeMap) htex.push(eyeMap.then((t) => { h.eyeMap = t; }));
    // Wait for every texture before the swap, so the new head never shows half painted.
    await Promise.all([...ktex, ...htex, paintTexture(h, paintKey())]);
    if (token !== loadToken) return;
    if (kotorGroup) sceneK.remove(kotorGroup);
    if (ours) sceneO.remove(ours.group);
    kotorGroup = kg; kotor = k; ours = h;
    sceneK.add(kg); sceneO.add(group);
    loadBody(name);
    const first = !current;
    current = name;
    showRigStatus();
    await applySkin();
    applyHair();
    posed(poseAt(anim, playing ? clock() : 0));
    if (first) setPreset(THUMBS ? "portrait" : "three");
    document.body.classList.remove("loading");
    if (!THUMBS) { updateHeadUI(); prefetchNeighbours(); }
  }

  function prefetchNeighbours() {
    const list = visibleHeads();
    const i = list.findIndex((h) => h.name === current);
    for (const j of [i + 1, i - 1]) if (list[j]) setTimeout(() => headData(list[j].name), 400);
  }

  // --- what our head wears ---
  async function applySkin() {
    if (!ours) return;
    const h = ours, bare = skin === "none";
    const plain = await paintTexture(h, paintKey());
    if (h !== ours) return;
    const tex = !bare && plain && hairPaint && hairChoice === "ours" && h.ourHair && h.hairLayer ? withPaintedHair(h, paintKey(), plain) : plain;
    h.mat.map = bare ? null : tex;
    h.mat.color = bare || !tex ? CLAY.clone() : new THREE.Color(1, 1, 1);
    h.mat.needsUpdate = true;
    // With no paint the eyes are clay too, so that only the shape shows.
    if (window.HeadPaint) window.HeadPaint.eyes(h, skin);
    if (window.HeadPaint && window.HeadPaint.material) window.HeadPaint.material(h, skin);
    h.emat.vertexColors = !bare && !h.eyeMap;
    h.emat.map = bare ? null : h.eyeMap || null;
    h.emat.color = bare ? CLAY.clone() : new THREE.Color(1, 1, 1);
    h.emat.needsUpdate = true;
    const clayAll = (group) => {
      if (!group) return;
      for (const m of group.children) {
        m.material.map = bare ? null : m.material.userData.map || null;
        m.material.color = bare ? CLAY.clone() : new THREE.Color(1, 1, 1);
        m.material.needsUpdate = true;
      }
    };
    clayAll(h.hair);
    clayAll(h.ourHair);
    clayAll(kotorGroup);
    if (h.slider) sliderEyes(h);
    applyEyeParts();
    requestRender();
    if (!THUMBS) updateSkinUI();
  }
  // Our paint with the painted hair layer put over it, drawn once per paint on a canvas. The layer has straight alpha, so the canvas's
  // "over" gives the same colours as head_hair's own composite.
  function withPaintedHair(h, key, base) {
    if (key === "live") return window.HeadPaint.withHair(h, base);
    const k = key + "|hair";
    if (!h.paints[k]) {
      const img = base.image, c = document.createElement("canvas");
      c.width = img.width; c.height = img.height;
      const ctx = c.getContext("2d");
      ctx.drawImage(img, 0, 0);
      ctx.drawImage(h.hairLayer, 0, 0, c.width, c.height);
      const t = new THREE.CanvasTexture(c);
      t.anisotropy = renderer.capabilities.getMaxAnisotropy();
      h.paints[k] = t;
    }
    return h.paints[k];
  }
  function applyHair() {
    withHair = hairChoice !== "none";
    const mine = !!(ours && ours.ourHair);
    if (ours && ours.hair) ours.hair.visible = hairChoice === "kotor" || (hairChoice === "ours" && !mine);
    if (ours && ours.ourHair) ours.ourHair.visible = hairChoice === "ours";
    applySkin();
    requestRender();
    if (!THUMBS) updateHairUI();
  }

  // --- the head made by the sliders (sliders.js) ---
  let sliderKitP = null;
  function sliderKit() {
    if (!window.HeadSliders) return Promise.resolve(null);
    return sliderKitP || (sliderKitP = window.HeadSliders.kit().then((kit) => {
      slider.kit = kit;
      // the paint's texture coordinates for our 4,750 vertices, read from viewer 2's own, so the lid pieces take the colour of the paint
      const vm = kit.vmap, uv = new Float32Array(2 * kit.VC);
      for (let i = vm.length - 1; i >= 0; i--) { uv[2 * vm[i]] = common.uv[2 * i]; uv[2 * vm[i] + 1] = common.uv[2 * i + 1]; }
      slider.cut = window.HeadSliders.skinCutter(common, vm, kit);
      if (kit.refine) { slider.rtopo = window.HeadSliders.refinedTopo({ ...common, trisFull: u16(common.trisFull) }, kit); slider.cut = slider.rtopo.cutter; }
      window.HeadSliders.eyeKit(uv).then((k) => { slider.eyeKit = k; if (ours && ours.slider) { sliderEyes(ours); posed(lastPose); } })
        .catch((e) => { console.error("the eye pieces", e); toast("The eyeballs and lid pieces of the sliders could not load: " + e.message); });
      if (!THUMBS) buildSliderPanel();
      return kit;
    }).catch((e) => { console.error(e); toast("The sliders could not load, so the smoothed head is shown: " + e.message); return null; }));
  }
  // the values belong to one head; a new head starts from its preset, the values solved for it
  function sliderEnsure(name) {
    if (slider.head === name) return;
    slider.head = name;
    slider.preset = slider.kit.solved(name);
    slider.values = slider.preset ? slider.preset.slice() : Float64Array.from(slider.kit.meta, (m) => m.zero || 0);
    syncSliderPanel();
  }
  function sliderPos(kit, name) {
    sliderEnsure(name);
    slider.world = kit.build(name, slider.values);
    if (!slider.rtopo) return window.HeadSliders.toViewer(slider.world, kit.vmap);
    const x = kit.extra(), w = new Float32Array(slider.world.length + x.length);
    w.set(slider.world); w.set(x, slider.world.length);
    return window.HeadSliders.toViewer(w, slider.rtopo.vmap);
  }
  function sliderRebuild() {
    if (slider.pending) return;
    slider.pending = true;
    requestAnimationFrame(() => {
      slider.pending = false;
      const h = ours;
      if (!h || !h.slider || h.name !== slider.head) return;
      const t0 = performance.now();
      const pos = sliderPos(slider.kit, h.name);
      if (h.liveRig && liveKit) applyLiveRig(h, pos);
      if (h.posDelta) { const vs = u16(h.posDelta.vertices), dl = f32(h.posDelta.delta); vs.forEach((v, i) => { for (let c = 0; c < 3; c++) pos[3 * v + c] += dl[3 * i + c]; }); }
      h.pos.set(pos);
      sliderEyes(h);
      if (window.HeadPaint) window.HeadPaint.shapeMoved(h);
      posed(lastPose);
      slider.ms = performance.now() - t0;
    });
  }
  // the eyeballs and upper lid pieces of the slider head, made again from its shape, its paint and the lid's state
  function sliderEyes(h) {
    if (!h.slider || !slider.eyeKit || !slider.world || h.name !== slider.head) return;
    if (h.sliderEyes) { h.group.remove(h.sliderEyes); window.HeadSliders.dispose(h.sliderEyes); h.sliderEyes = null; }
    const r = window.HeadSliders.eyes(slider.eyeKit, slider.world, { closed: slider.lid === "closed", map: (window.HeadPaint && window.HeadPaint.lidMap(h, skin)) || h.mat.map, eyeMap: h.eyeMap, clay: skin === "none" });
    if (h.headMat) r.root.matrix.fromArray(h.headMat);
    h.group.add(r.root);
    h.sliderEyes = r.root;
    h.sliderBalls = r.balls;
    h.sliderHidden = r.hidden;
    h.geo.setIndex(slider.cut(r.hidden));
    applyEyeParts();
    requestRender();
  }
  // Shows or hides the lid pieces, the caruncles and the eyes' paint as the Eye parts switches say, on KOTOR's head and on ours. With
  // our lid pieces hidden, the skin faces they replace are drawn again, so the opening is our own mesh's.
  function applyEyeParts() {
    if (kotorGroup) for (const m of kotorGroup.children) {
      if (m.userData.plainEye) { m.visible = !eyeParts.paint; if (!eyeParts.paint) { m.material.map = null; m.material.color = PLAIN_EYE.clone(); m.material.needsUpdate = true; } continue; }
      const et = m.userData.eyeTris;
      if (!et) continue;
      const keep = [];
      for (let t = 0; t < et.n; t++) if (!(et.lid.has(t) && !eyeParts.lids) && !(et.ball.has(t) && !eyeParts.paint)) keep.push(3 * t, 3 * t + 1, 3 * t + 2);
      if (keep.length !== m.geometry.index.count) m.geometry.setIndex(keep);
    }
    const h = ours;
    if (!h) return;
    if (!eyeParts.paint && !h.sliderEyes) { h.emat.map = null; h.emat.vertexColors = false; h.emat.color = PLAIN_EYE.clone(); h.emat.needsUpdate = true; }
    if (h.sliderEyes) {
      const ch = h.sliderEyes.children;
      ch[1].visible = ch[2].visible = eyeParts.lids;
      for (const c of ch) if (c.name === "caruncles") c.visible = eyeParts.caruncle;
      const ball = ch[0];
      if (!ball.userData.painted) ball.userData.painted = ball.material;
      if (!eyeParts.paint && !ball.userData.plain) ball.userData.plain = new THREE.MeshStandardMaterial({ color: PLAIN_EYE.clone(), roughness: 0.5, metalness: 0 });
      ball.material = eyeParts.paint ? ball.userData.painted : ball.userData.plain;
      if (h.sliderHidden) h.geo.setIndex(slider.cut(eyeParts.lids ? h.sliderHidden : []));
    }
    if (!THUMBS) for (const b of document.querySelectorAll("[data-eye-part]")) b.setAttribute("aria-pressed", eyeParts[b.dataset.eyePart] ? "true" : "false");
    requestRender();
  }
  function forFeature(f, fn) { slider.kit.meta.forEach((s, k) => { if (!f || s.feature === f) fn(k); }); }
  function sliderChanged(sync) {
    if (sync) syncSliderPanel();
    if (shape !== "sliders") { setShape("sliders"); return; }
    sliderRebuild();
  }
  function buildSliderPanel() {
    const page = $("panel-sliders");
    if (!page || slider.panel) return;
    slider.panel = window.HeadSliders.panel({
      tabs: $("panel-tabs"), look: $("panel-look"), page, kit: slider.kit, lid: slider.lid,
      onInput: (k, v) => { sliderEnsure(current); slider.values[k] = v; sliderChanged(false); },
      onPreset: (f) => { sliderEnsure(current); forFeature(f, (k) => { slider.values[k] = slider.preset ? slider.preset[k] : slider.kit.meta[k].zero || 0; }); sliderChanged(true); },
      onZero: (f) => { sliderEnsure(current); forFeature(f, (k) => { slider.values[k] = slider.kit.meta[k].zero || 0; }); sliderChanged(true); },
      onProposed: (vals) => { sliderEnsure(current); slider.kit.meta.forEach((s, k) => { if (s.feature === "eyes" && s.name in vals) slider.values[k] = vals[s.name]; }); sliderChanged(true); },
      onLid: (state) => { slider.lid = state; if (ours && ours.slider) sliderEyes(ours); },
    });
    if (current) sliderEnsure(current);
    syncSliderPanel();
  }
  function syncSliderPanel() {
    if (!slider.panel) return;
    if (slider.values) slider.panel.sync(slider.values, slider.preset);
    const h = heads.find((x) => x.name === slider.head);
    slider.panel.note(shape !== "sliders" ? "The head shown now has another shape, chosen under Shape in the tab Look. Moving a slider shows the head made by the sliders instead."
      : !h ? "" : slider.preset ? `${h.label}'s preset is loaded. The preset is the set of values solved for this head. Moving a slider changes the head at once.`
      : `${h.label} has no solved values, so every slider starts at 0.`);
  }
  function setShape(id) {
    shape = id;
    for (const x of $("shape-options").children) x.setAttribute("aria-pressed", x.dataset.shape === shape ? "true" : "false");
    syncSliderPanel();
    loadHead(current);
  }

  // --- camera ---
  let preset = "three";
  let fovFixed = 0;
  function setPreset(which) {
    if (!ours) return;
    const P = {
      portrait: { c: ours.headCentre.clone().add(new THREE.Vector3(0, -0.004, 0)), d: 0.6, yaw: 16, lift: 0.04 },
      front: { c: ours.headCentre, d: 1.0, yaw: 0, lift: 0.02 },
      three: { c: ours.headCentre, d: 1.0, yaw: 35, lift: 0.04 },
      side: { c: ours.headCentre, d: 1.0, yaw: 90, lift: 0.02 },
      eyes: { c: ours.eyeCentre.clone().add(new THREE.Vector3(0, 0, 0.01)), d: 0.42, yaw: 0, lift: 0.0 },
      mouth: { c: ours.mouth.clone().add(new THREE.Vector3(0, -0.006, 0)), d: 0.4, yaw: 0, lift: 0.0 },
      neck: { c: ours.mouth.clone().add(new THREE.Vector3(0, -0.13, 0)), d: 0.55, yaw: 35, lift: 0.03 },
    }[which];
    fovFixed = 0;
    if (which === "dialogue") {
      // KOTOR's close shot of the speaker (kjk play_view::dialog_shot): the hook is at the eyes, the target 0.2 m ahead of it and 0.04 m under it,
      // the eye 0.5 m from the target, turned 30 degrees off the line of sight, at the target's height, filmed with a vertical field of view of 55 degrees.
      const hook = new THREE.Vector3(0, ours.eyeCentre.y, ours.headCentre.z);
      const yaw = 30 * Math.PI / 180;
      target.set(hook.x, hook.y - 0.04, hook.z + 0.2);
      camera.position.set(target.x + 0.5 * Math.sin(yaw), target.y, target.z + 0.5 * Math.cos(yaw));
      fovFixed = 55;
      camera.lookAt(target);
      controls.update();
      activePreset(which);
      requestRender();
      return;
    }
    const yaw = P.yaw * Math.PI / 180;
    target.copy(P.c);
    camera.position.set(P.c.x + P.d * Math.sin(yaw), P.c.y + P.lift, P.c.z + P.d * Math.cos(yaw));
    camera.lookAt(target);
    controls.update();
    activePreset(which);
    requestRender();
  }
  function activePreset(which) {
    preset = which;
    if (THUMBS) return;
    for (const b of document.querySelectorAll("[data-cam]")) b.setAttribute("aria-pressed", b.dataset.cam === which ? "true" : "false");
  }

  // --- drawing ---
  let pending = false;
  function requestRender() { if (!pending) { pending = true; requestAnimationFrame(draw); } }
  function size() {
    const w = canvas.clientWidth, h = canvas.clientHeight;
    const c = renderer.domElement;
    if (c.width !== Math.round(w * renderer.getPixelRatio()) || c.height !== Math.round(h * renderer.getPixelRatio())) renderer.setSize(w, h, false);
    return [w, h];
  }
  // The field of view is vertical. In a tall or narrow view it widens, so the head keeps the width it has at an aspect of 0.85.
  function aspectTo(a) {
    camera.aspect = a;
    camera.fov = fovFixed ? fovFixed : a >= 0.85 ? 22 : 2 * Math.atan(Math.tan(11 * Math.PI / 180) * 0.85 / a) * 180 / Math.PI;
    camera.updateProjectionMatrix();
  }
  function draw() {
    pending = false;
    const [w, h] = size();
    if (!w || !h) return;
    const mode = peek ? "original" : compare;
    renderer.setScissorTest(true);
    if (mode === "side") {
      aspectTo((w / 2) / h);
      const half = Math.floor(w / 2);
      renderer.setViewport(0, 0, half, h); renderer.setScissor(0, 0, half, h); renderer.render(sceneK, camera);
      renderer.setViewport(half, 0, w - half, h); renderer.setScissor(half, 0, w - half, h); renderer.render(sceneO, camera);
    } else {
      aspectTo(w / h);
      renderer.setViewport(0, 0, w, h);
      renderer.setScissor(0, 0, w, h);
      renderer.render(mode === "original" ? sceneK : sceneO, camera);
      if (mode === "wipe") {
        const x = Math.round(split * w);
        renderer.setScissor(0, 0, x, h);
        renderer.render(sceneK, camera);
      }
    }
    renderer.setScissorTest(false);
  }

  // --- animation and the voiced line ---
  let anim = "rest", playing = false, start = 0, audio = null, audioRunning = false, raf = 0;
  function clock() { return anim === "line" && audio ? (audioRunning ? audio.currentTime : 0) : (performance.now() - start) / 1000; }
  function poseAt(value, t) {
    if (!common) return {};
    if (value === "rest") return clipPose(common.clips.talk, 0);
    if (value.startsWith("shape:")) return clipPose(common.clips.talk, +value.slice(6) / 30);
    if (value.startsWith("clip:")) { const c = common.clips[value.slice(5)]; return clipPose(c, t % c.length); }
    if (value === "line") {
      const keys = common.line.lip.keys;
      if (!keys.length || t <= keys[0][0]) return clipPose(common.clips.talk, 0);
      const i = keys.findIndex((k) => k[0] > t);
      if (i < 0) return clipPose(common.clips.talk, keys[keys.length - 1][1] / 30);
      const a = keys[i - 1], b = keys[i], f = (t - a[0]) / Math.max(b[0] - a[0], 1e-6);
      return blendPose(clipPose(common.clips.talk, a[1] / 30), clipPose(common.clips.talk, b[1] / 30), f);
    }
    return {};
  }
  function lineLength() { const k = common.line.lip.keys; return k.length ? k[k.length - 1][0] : 1; }
  function frame() {
    const t = clock();
    posed(poseAt(anim, t));
    const bar = $("perf-progress");
    if (bar) {
      const len = anim === "line" ? (audio && audio.duration) || lineLength() : anim.startsWith("clip:") ? common.clips[anim.slice(5)].length : 1;
      bar.style.transform = `scaleX(${Math.min(1, (t % (anim === "line" ? Infinity : len)) / len)})`;
    }
    if (anim === "line" && audio && audio.ended) { stop(); return; }
    raf = requestAnimationFrame(frame);
  }
  function playable() { return anim === "line" || anim.startsWith("clip:"); }
  function play() {
    if (!playable()) return;
    playing = true;
    start = performance.now();
    if (anim === "line") {
      // The lips follow the sound's own clock, so they wait for the sound to start and cannot drift from it.
      const a = audio = new Audio(DATA + common.line.audio + "?pcm");
      a.muted = MUTE || $("mute").getAttribute("aria-pressed") === "true";
      audioRunning = false;
      a.addEventListener("playing", () => { audioRunning = true; });
      a.play().catch(() => {
        toast("The sound could not play. The mouth still moves.");
        if (audio === a) { audio = null; start = performance.now(); }
      });
      $("subtitle").hidden = false;
    }
    raf = requestAnimationFrame(frame);
    updatePerfUI();
  }
  function stop() {
    playing = false;
    cancelAnimationFrame(raf);
    if (audio) { audio.pause(); audio = null; }
    if ($("subtitle")) $("subtitle").hidden = true;
    if ($("perf-progress")) $("perf-progress").style.transform = "scaleX(0)";
    if (common) posed(poseAt(anim, 0));
    updatePerfUI();
  }
  function setAnim(value, autoplay) {
    stop();
    anim = value;
    posed(poseAt(anim, 0));
    if (autoplay && playable()) play();
    updatePerfUI();
  }

  // ======================= the page around the head =======================
  const filters = { group: "all", sex: "all", q: "" };
  function visibleHeads() {
    const q = filters.q.trim().toLowerCase();
    return heads.filter((h) => (filters.group === "all" || h.group === filters.group) && (filters.sex === "all" || h.sex === filters.sex) &&
      (!q || h.label.toLowerCase().includes(q) || h.name.includes(q)));
  }
  let thumbs = null;
  function buildPicker() {
    const grid = $("portraits");
    grid.textContent = "";
    for (const key of Object.keys(GROUPS)) {
      const list = visibleHeads().filter((h) => h.group === key);
      if (!list.length) continue;
      const head = document.createElement("div");
      head.className = "group-title";
      head.textContent = `${GROUPS[key]}`;
      const count = document.createElement("span");
      count.textContent = list.length;
      head.appendChild(count);
      grid.appendChild(head);
      for (const h of list) {
        const b = document.createElement("button");
        b.type = "button";
        b.className = "portrait";
        b.dataset.head = h.name;
        b.setAttribute("aria-pressed", h.name === current ? "true" : "false");
        const pic = document.createElement("span");
        pic.className = "pic";
        const t = thumbs && thumbs.heads[h.name];
        if (t) {
          pic.style.backgroundImage = `url(thumbs/${thumbs.file})`;
          pic.style.backgroundSize = `${thumbs.cols * 100}% ${thumbs.rows * 100}%`;
          pic.style.backgroundPosition = `${thumbs.cols > 1 ? (t[0] / (thumbs.cols - 1)) * 100 : 0}% ${thumbs.rows > 1 ? (t[1] / (thumbs.rows - 1)) * 100 : 0}%`;
        } else pic.textContent = h.label.slice(0, 1);
        const name = document.createElement("span");
        name.className = "pname";
        name.textContent = h.label;
        const code = document.createElement("span");
        code.className = "pcode";
        code.textContent = h.name;
        b.append(pic, name, code);
        b.addEventListener("click", () => selectHead(h.name));
        grid.appendChild(b);
      }
    }
    const n = visibleHeads().length;
    $("picker-count").textContent = n === heads.length ? `${n} heads` : `${n} of ${heads.length} heads`;
    if (!n) { const p = document.createElement("p"); p.className = "empty"; p.textContent = "No head matches this search."; grid.appendChild(p); }
  }
  function selectHead(name, scroll) {
    if (name === current) return;
    stopIfLine();
    for (const b of document.querySelectorAll(".portrait")) b.setAttribute("aria-pressed", b.dataset.head === name ? "true" : "false");
    const sel = document.querySelector(`.portrait[data-head="${name}"]`);
    if (sel && scroll) reveal(sel, false);
    const u = new URL(location.href); u.searchParams.set("head", name); history.replaceState(null, "", u);
    loadHead(name).catch((e) => toast("This head could not load: " + e.message));
  }
  // Scrolls the portrait list only, never the page, so a narrow window keeps the head in view.
  function reveal(el, centre) {
    const grid = $("portraits");
    const top = el.offsetTop, bottom = top + el.offsetHeight;
    if (centre) grid.scrollTop = top - (grid.clientHeight - el.offsetHeight) / 2;
    else if (top < grid.scrollTop) grid.scrollTop = top - 30;
    else if (bottom > grid.scrollTop + grid.clientHeight) grid.scrollTop = bottom - grid.clientHeight + 8;
  }
  function stopIfLine() { if (anim === "line" && playing) stop(); }
  function step(d) {
    const list = visibleHeads();
    if (!list.length) return;
    const i = list.findIndex((h) => h.name === current);
    selectHead(list[(i + d + list.length) % list.length].name, true);
  }

  function updateHeadUI() {
    const h = heads.find((x) => x.name === current);
    $("head-name").textContent = h.label;
    $("head-meta").textContent = `${GROUPS[h.group]} · ${h.name}`;
    document.title = `${h.label} · Head creator preview`;
    const f = fit && fit.heads[current];
    const box = $("fit-facts");
    box.textContent = "";
    if (f && shape === "fit") {
      const worst = Object.entries(f.zones).sort((a, b) => b[1] - a[1])[0];
      const pass = f.face <= 0.5 && worst[1] <= 0.75;
      const dl = document.createElement("dl");
      dl.className = "facts";
      dl.innerHTML = `<dt>Whole face, average</dt><dd>${f.face.toFixed(2)} mm</dd><dt>Worst zone, ${worst[0]}</dt><dd>${worst[1].toFixed(2)} mm</dd>` +
        `<dt>Mark of 0.5 mm and 0.75 mm</dt><dd><span class="verdict ${pass ? "pass" : "fail"}">${pass ? "Passes" : "Misses"}</span></dd>`;
      const p = document.createElement("p");
      p.className = "hint";
      p.style.minHeight = "0";
      p.textContent = "Each number is the average distance from our face to the KOTOR face. A head passes when the whole face is within 0.5 mm and every zone is within 0.75 mm.";
      box.append(dl, p);
    } else if (shape !== "fit") {
      const p = document.createElement("p");
      p.textContent = "The fit numbers belong to the default smoothed head, so they are hidden while another shape is shown.";
      box.append(p);
    }
    updateSkinUI();
    updateHairUI();
    if (slider.kit) sliderEnsure(current);
    syncSliderPanel();
  }

  // Each group of the panel has one hint line. It shows the note of the control under the pointer or the keyboard focus, and otherwise the group's own note.
  function setHint(id, text, warn) {
    const el = $(id);
    if (!el) return;
    el.dataset.base = text || "";
    el.classList.toggle("warn", !!warn);
    if (!el.dataset.hover) el.textContent = text || "";
  }
  function wireHints() {
    for (const group of document.querySelectorAll(".panel .group")) {
      const hint = group.querySelector(".hint");
      if (!hint) continue;
      const show = (e) => { const t = e.target.closest && e.target.closest("[data-note]"); if (!t || !group.contains(t)) return; hint.dataset.hover = "1"; hint.textContent = t.dataset.note; };
      const back = () => { delete hint.dataset.hover; hint.textContent = hint.dataset.base || ""; };
      group.addEventListener("pointerover", show);
      group.addEventListener("focusin", show);
      group.addEventListener("pointerleave", back);
      group.addEventListener("focusout", back);
    }
    for (const id of ["data-fold", "keys-fold"]) {
      const d = $(id);
      try { d.open = localStorage.getItem("viewer2." + id) === "1"; } catch (e) { /* storage may be blocked */ }
      d.addEventListener("toggle", () => { try { localStorage.setItem("viewer2." + id, d.open ? "1" : "0"); } catch (e) { /* storage may be blocked */ } });
    }
  }
  function updateLightHint() {
    const b = document.querySelector(`[data-light="${lighting}"]`);
    setHint("light-hint", b ? b.dataset.note : "");
  }

  function updateShapeNote() {
    if (!$("shape-note")) return;
    const v = shapeVariants.find((x) => x.id === shape);
    const pv = skinVariant();
    const paintNote = v && pv && pv.layers ? " Our paint's shading and sheen were computed on the default smoothed head, not on this shape." : "";
    if (shape === "sliders") {
      const made = ours && ours.slider;
      setHint("shape-note", made ? "Our head built live from a start shape and the values of the sliders in the tabs Nose, Jaw and chin, Brow and forehead, Cheekbones and cheeks, Eyes, and Throat and skull. The start shape is this head's earlier fit with the first five features taken out. Choosing a head loads its preset, the slider values solved for it. The paint was baked for the smoothed head, and it follows this shape through the same texture coordinates."
        : "The sliders are not loaded, so the smoothed head is shown.", !made);
      return;
    }
    if (!v) { setHint("shape-note", "Our head as fitted to the KOTOR head and then smoothed, as the last step of building it." + (window.HeadSliders ? "" : " This is the default shape.")); return; }
    const has = shapeCache[shape + "/" + current];
    setHint("shape-note", (has ? (v.note || `This head is shown with the shape "${v.label}".`) : `The shape "${v.label}" has no file for this head, so the default smoothed head is shown.`) + paintNote, !has);
  }
  function cleanLabel(l) { const s = l.replace(/^Our paint,\s*/i, ""); return s.charAt(0).toUpperCase() + s.slice(1); }
  const SKIN_NOTES = {
    kotor: "A stand-in. KOTOR's own texture is baked onto our head. It shows what our shape can carry, not what our paint can do.",
    none: "Both heads and the eyes turn to plain clay, so only the shape shows.",
  };
  let lastOurs = null;
  function bestPaint() { return oursPaints.find((v) => v.default) || oursPaints[oursPaints.length - 1]; }
  function buildSkinOptions() {
    const box = $("skin-options");
    box.textContent = "";
    const best = bestPaint();
    lastOurs = best ? "ours:" + best.id : null;
    for (const [key, label, cls, note] of [["ours", "Our paint", "", "Our own paint. Its newest version is the default."], ["kotor", "KOTOR's", "borrowed", SKIN_NOTES.kotor], ["none", "No paint", "none", SKIN_NOTES.none]]) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = cls;
      b.dataset.skinMain = key;
      b.dataset.note = note;
      b.textContent = label;
      if (key === "ours" && !best) b.disabled = true;
      b.addEventListener("click", () => { skin = key === "ours" ? lastOurs : key; applySkin(); });
      box.appendChild(b);
    }
    if (window.HeadPaint) window.HeadPaint.skinButton(box, () => { skin = "live"; applySkin(); });
    const older = oursPaints.filter((v) => v !== best).reverse();
    $("skin-older").hidden = !older.length;
    $("skin-older-count").textContent = older.length;
    const list = $("skin-older-list");
    list.textContent = "";
    for (const v of older) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "row";
      b.dataset.skin = "ours:" + v.id;
      b.dataset.note = "An older version of our paint, kept to compare with the newest.";
      b.innerHTML = `<span class="radio"></span><span class="opt-label"></span>`;
      b.querySelector(".opt-label").textContent = cleanLabel(v.label);
      b.addEventListener("click", () => { skin = skin === b.dataset.skin && best ? "ours:" + best.id : b.dataset.skin; applySkin(); });
      list.appendChild(b);
    }
  }
  function buildLayerSwitches() {
    const box = $("layer-switches");
    box.textContent = "";
    const v = skinVariant();
    box.hidden = !(v && v.layers);
    if (box.hidden) return;
    for (const l of v.layers) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "row";
      b.dataset.layer = l.id;
      b.dataset.note = l.note || "";
      b.innerHTML = `<span class="opt-label"></span><span class="sw-track"></span>`;
      b.querySelector(".opt-label").textContent = l.label;
      b.addEventListener("click", () => { layerOn[l.id] = layerOn[l.id] === false; applySkin(); });
      box.append(b);
    }
  }
  function updateSkinUI() {
    const isOurs = !!(skin && skin.startsWith("ours:"));
    if (isOurs) lastOurs = skin;
    const main = isOurs ? "ours" : skin;
    for (const b of document.querySelectorAll("[data-skin-main]")) b.setAttribute("aria-pressed", b.dataset.skinMain === main ? "true" : "false");
    for (const b of document.querySelectorAll("[data-skin]")) b.setAttribute("aria-pressed", b.dataset.skin === skin ? "true" : "false");
    const best = bestPaint();
    const v = skinVariant();
    const isOlder = isOurs && best && v !== best;
    if (isOlder) $("skin-older").open = true;
    const tag = $("skin-tag");
    tag.className = "tag " + (isOurs ? "ours" : skin === "kotor" ? "borrowed" : "none");
    tag.textContent = isOurs ? "Ours" : skin === "kotor" ? "Stand-in" : "Shape only";
    $("skin-caption").textContent = isOurs && v ? cleanLabel(v.label) + (isOlder ? ", an older version" : "") : "";
    $("skin-caption").hidden = !isOurs;
    setHint("skin-hint", isOurs ? (isOlder ? "An older version of our paint. Choose it again to go back to the newest." : "Our newest paint. Point at a layer to read what it adds.") : SKIN_NOTES[skin] || "");
    if (skin === "live" && window.HeadPaint) window.HeadPaint.skinUI(tag, setHint);
    if (($("layer-switches").dataset.for || "") !== skin) { buildLayerSwitches(); $("layer-switches").dataset.for = skin; }
    for (const b of document.querySelectorAll("[data-layer]")) b.setAttribute("aria-pressed", layerOn[b.dataset.layer] === false ? "false" : "true");
    updateShapeNote();
    if (skin === "live" && window.HeadPaint) window.HeadPaint.shapeNote(setHint);
    const box = $("tone");
    box.textContent = "";
    const d = v && current && v.heads[current];
    box.hidden = !d;
    $("tone-none").hidden = !!d;
    if (!d) return;
    const sw = document.createElement("div");
    sw.className = "swatches";
    sw.innerHTML = `<span class="sw"><i style="background:${d.colour}"></i>Ours</span><span class="sw"><i style="background:${d.kotor_colour}"></i>KOTOR</span>`;
    const target = v.target_label || "KOTOR's base tone";
    const p = document.createElement("p");
    const de = d.delta_e;
    p.textContent = d.text || (`Our base tone differs from ${target} by ${de.toFixed(1)} (CIEDE2000). ` +
      (de < 2 ? "A difference under 2 is hard to see." : de < 5 ? "A difference from 2 to 5 shows side by side." : "A difference over 5 is plain to see."));
    const rec = document.createElement("dl");
    rec.className = "recipe";
    for (const [k, val] of d.recipe) { const dt = document.createElement("dt"); dt.textContent = k; const dd = document.createElement("dd"); dd.textContent = val; rec.append(dt, dd); }
    box.append(sw, rec, p);
  }
  function updateHairUI() {
    const has = !!(ours && ours.hair), mine = !!(ours && ours.ourHair);
    for (const b of document.querySelectorAll("[data-hair]")) {
      b.setAttribute("aria-pressed", b.dataset.hair === hairChoice ? "true" : "false");
      b.disabled = (b.dataset.hair === "kotor" && !has) || (b.dataset.hair === "ours" && !mine && !has);
    }
    const showsOurs = hairChoice === "ours" && mine;
    const showsKotor = (hairChoice === "kotor" || (hairChoice === "ours" && !mine)) && has;
    const st = ours && ours.ourHairData && ours.ourHairData.stats;
    let note = "";
    if (ours) {
      if (showsOurs) note = `Our own hair, made from a few curves by head_hair: a cap and ${st ? st.triangles.card_count : "a few"} cards, ${st ? st.triangles.total : "?"} triangles against ${st ? st.kotor_triangles.hair_objects : "?"} in KOTOR's hair, and hair painted into the skin.`;
      else if (hairChoice === "ours" && !mine) note = has ? "We have no hair of our own for this head yet, so KOTOR's hair stands in." : "We have no hair of our own for this head yet, and this KOTOR head has no hair.";
      else if (showsKotor) note = "A stand-in. The hair, hood or helmet is KOTOR's own, taken from this head's model.";
      else if (hairChoice === "kotor") note = "This KOTOR head has no hair, hood or helmet, so there is nothing to show.";
      else note = "No hair is shown.";
    }
    setHint("hair-note", note);
    const tag = $("hair-tag");
    if (tag) {
      tag.hidden = !(showsOurs || showsKotor);
      tag.className = "tag " + (showsOurs ? "ours" : "borrowed");
      tag.textContent = showsOurs ? "Ours" : "Stand-in";
    }
    const row = $("hair-paint-row");
    if (row) row.hidden = !showsOurs;
    const pt = $("hair-paint-toggle");
    if (pt) pt.setAttribute("aria-pressed", hairPaint ? "true" : "false");
  }
  function buildShapeOptions() {
    const sec = $("shape-section");
    const made = window.HeadSliders ? [{ id: "sliders", label: "Made by the sliders (default)", note: "Our head built live from a start shape and the values of every slider. Choosing a head loads its preset." }] : [];
    sec.hidden = !shapeVariants.length && !made.length;
    if (sec.hidden) return;
    const box = $("shape-options");
    box.textContent = "";
    for (const v of [...made, { id: "fit", label: made.length ? "Smoothed" : "Smoothed (default)", note: "Our head as fitted to the KOTOR head and then smoothed, as the last step of building it." + (made.length ? "" : " This is the default shape.") }, ...shapeVariants]) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "row";
      b.dataset.shape = v.id;
      b.dataset.note = v.note || v.label;
      b.innerHTML = `<span class="radio"></span><span class="opt-label"></span>`;
      b.querySelector(".opt-label").textContent = v.label;
      b.setAttribute("aria-pressed", v.id === shape ? "true" : "false");
      b.addEventListener("click", () => setShape(v.id));
      box.appendChild(b);
    }
  }

  // FNV-1a over the bytes of a head's fitted positions, as head_rig's calibrate tool computes it.
  function fingerprint(b64) {
    const bytes = atob(b64);
    let h = 0xcbf29ce484222325n;
    for (let i = 0; i < bytes.length; i++) h = ((h ^ BigInt(bytes.charCodeAt(i))) * 0x100000001b3n) & 0xffffffffffffffffn;
    return h.toString(16).padStart(16, "0");
  }

  // The rig always follows the animation calibration. A head fitted again after the calibration still animates, with weights learned
  // from its older fit, and the section says so until the calibration is run again.
  function showRigStatus() {
    const sec = $("rig-section");
    if (!sec) return;
    sec.hidden = !rigVariants.length;
    const d = cache[current], h = heads.find((x) => x.name === current);
    if (!rigVariants.length || !d || !h) return;
    const live = ours && ours.name === current && ours.liveRig;
    const print = calibration && calibration.heads ? calibration.heads[current] : null;
    if (live) {
      const old = print && fingerprint(d.ours.pos) !== print ? ` The weights rule was learned from an older fit of ${h.label}, so run head_rig's calibrate tool to update it.` : "";
      setHint("rig-note", `The bones, the skin weights, the teeth and the lips of this head are computed from the shape the sliders make, each time a slider moves.` + old, !!old);
    } else if (!print) setHint("rig-note", `The animation calibration has no record of ${h.label}. Run head_rig's calibrate tool, then mouth_rig, to rig this head.`, true);
    else if (fingerprint(d.ours.pos) !== print) setHint("rig-note", `${h.label} was fitted again after the animation calibration. The face still animates, with weights learned from the older fit. Run head_rig's calibrate tool, then mouth_rig, to update the rig.`, true);
    else setHint("rig-note", `The mouth, brows and cheeks follow the animation calibration, which was learned from this fit of ${h.label}.`);
  }

  function buildPerf() {
    const box = $("perf-chips");
    const add = (value, label, cls) => {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "chip" + (cls ? " " + cls : "");
      b.dataset.anim = value;
      b.textContent = label;
      b.addEventListener("click", () => setAnim(value, value !== "rest"));
      box.appendChild(b);
    };
    add("rest", "Still");
    add("line", "Voiced line", "accent");
    for (const [k, label] of CLIPS) if (common.clips[k]) add("clip:" + k, label);
    const sel = $("shape-select");
    SHAPES.forEach((s, i) => { if (!i) return; const o = document.createElement("option"); o.value = "shape:" + i; o.textContent = s; sel.appendChild(o); });
    sel.addEventListener("change", () => { if (sel.value) setAnim(sel.value, false); });
    $("subtitle").textContent = common.line.text;
  }
  function updatePerfUI() {
    if (THUMBS || !$("perf-chips")) return;
    for (const b of document.querySelectorAll("[data-anim]")) b.setAttribute("aria-pressed", b.dataset.anim === anim ? "true" : "false");
    const sel = $("shape-select");
    sel.value = anim.startsWith("shape:") ? anim : "";
    const p = $("play");
    p.disabled = !playable();
    p.setAttribute("aria-label", playing ? "Stop" : "Play");
    p.classList.toggle("playing", playing);
  }

  function setCompare(mode) {
    compare = mode;
    for (const b of document.querySelectorAll("[data-compare]")) b.setAttribute("aria-pressed", b.dataset.compare === mode ? "true" : "false");
    updateCompareUI();
  }
  function updateCompareUI() {
    const mode = peek ? "original" : compare;
    const stage = $("stage");
    stage.dataset.mode = mode;
    $("wipe-handle").hidden = mode !== "wipe";
    $("wipe-handle").style.left = split * 100 + "%";
    const big = $("view-badge");
    big.textContent = mode === "original" ? "KOTOR original" : mode === "ours" ? "Ours" : "";
    big.className = "view-badge " + (mode === "original" ? "borrowed" : "ours");
    big.hidden = mode === "wipe" || mode === "side";
    requestRender();
  }

  function wire() {
    $("search").addEventListener("input", (e) => { filters.q = e.target.value; buildPicker(); });
    for (const b of document.querySelectorAll("[data-filter]")) b.addEventListener("click", () => {
      const [k, v] = b.dataset.filter.split(":");
      filters[k] = v;
      for (const x of document.querySelectorAll(`[data-filter^="${k}:"]`)) x.setAttribute("aria-pressed", x === b ? "true" : "false");
      buildPicker();
    });
    $("prev").addEventListener("click", () => step(-1));
    $("next").addEventListener("click", () => step(1));
    for (const b of document.querySelectorAll("[data-cam]")) b.addEventListener("click", () => setPreset(b.dataset.cam));
    for (const b of document.querySelectorAll("[data-body]")) b.addEventListener("click", () => setBody(!bodyOn));
    for (const b of document.querySelectorAll("[data-compare]")) b.addEventListener("click", () => setCompare(b.dataset.compare));
    for (const b of document.querySelectorAll("[data-light]")) b.addEventListener("click", () => {
      lighting = b.dataset.light;
      for (const x of document.querySelectorAll("[data-light]")) x.setAttribute("aria-pressed", x === b ? "true" : "false");
      applyLights();
      updateLightHint();
    });
    wireHints();
    updateLightHint();
    for (const b of document.querySelectorAll("[data-hair]")) b.addEventListener("click", () => { hairChoice = b.dataset.hair; applyHair(); });
    $("hair-paint-toggle").addEventListener("click", () => { hairPaint = !hairPaint; applyHair(); });
    for (const b of document.querySelectorAll("[data-eye-part]")) b.addEventListener("click", () => { eyeParts[b.dataset.eyePart] = !eyeParts[b.dataset.eyePart]; applySkin(); });
    // "Shape only" also takes the paint off the skin, whose painted lashes and lid shadows draw a dark band round the opening; "Show all"
    // puts the earlier skin back.
    for (const b of document.querySelectorAll("[data-eye-preset]")) b.addEventListener("click", () => {
      const on = b.dataset.eyePreset === "all";
      Object.assign(eyeParts, { lids: on, caruncle: on, paint: on });
      if (!on && skin !== "none") { eyeParts.skinBefore = skin; skin = "none"; }
      if (on && eyeParts.skinBefore) { skin = eyeParts.skinBefore; eyeParts.skinBefore = null; }
      applySkin();
    });
    $("play").addEventListener("click", () => (playing ? stop() : play()));
    $("mute").addEventListener("click", () => {
      const m = $("mute").getAttribute("aria-pressed") !== "true";
      $("mute").setAttribute("aria-pressed", m ? "true" : "false");
      $("mute").setAttribute("aria-label", m ? "Sound off" : "Sound on");
      if (audio) audio.muted = m;
    });
    if (MUTE) { $("mute").setAttribute("aria-pressed", "true"); $("mute").setAttribute("aria-label", "Sound off"); }
    // The hold button and the C key show the original only while held.
    const hold = $("hold");
    const on = (e) => { e.preventDefault(); peek = true; updateCompareUI(); }, off = () => { if (peek) { peek = false; updateCompareUI(); } };
    hold.addEventListener("pointerdown", on);
    for (const ev of ["pointerup", "pointerleave", "pointercancel"]) hold.addEventListener(ev, off);
    // the wipe line
    const handle = $("wipe-handle");
    handle.addEventListener("pointerdown", (e) => {
      handle.setPointerCapture(e.pointerId);
      const move = (ev) => {
        const r = canvas.getBoundingClientRect();
        split = Math.min(0.98, Math.max(0.02, (ev.clientX - r.left) / r.width));
        updateCompareUI();
      };
      const up = () => { handle.removeEventListener("pointermove", move); handle.removeEventListener("pointerup", up); };
      handle.addEventListener("pointermove", move);
      handle.addEventListener("pointerup", up);
    });
    window.addEventListener("keydown", (e) => {
      if ((e.target.matches && e.target.matches("input, select, textarea")) || e.metaKey || e.ctrlKey || e.altKey) return;
      const k = e.key.toLowerCase();
      if (k === "arrowright" || k === "arrowdown") { e.preventDefault(); step(1); }
      else if (k === "arrowleft" || k === "arrowup") { e.preventDefault(); step(-1); }
      else if (k === "c" && !e.repeat) { peek = true; updateCompareUI(); }
      else if (k === "v") { const m = ["ours", "wipe", "side", "original"]; setCompare(m[(m.indexOf(compare) + 1) % m.length]); }
      else if (k === " ") { e.preventDefault(); if (playing) stop(); else if (playable()) play(); else setAnim("line", true); }
      else if (k === "h") { hairChoice = HAIRS[(HAIRS.indexOf(hairChoice) + 1) % HAIRS.length]; applyHair(); }
      else if ("123456".includes(k) && k.length === 1) setPreset(["front", "three", "side", "eyes", "mouth", "portrait"][+k - 1]);
      else if (k === "/") { e.preventDefault(); $("search").focus(); }
    });
    window.addEventListener("keyup", (e) => { if (e.key.toLowerCase() === "c") { peek = false; updateCompareUI(); } });
    window.addEventListener("blur", () => { if (peek) { peek = false; updateCompareUI(); } });
    window.addEventListener("resize", requestRender);
    new ResizeObserver(requestRender).observe(canvas);
  }

  // ======================= thumbnails =======================
  // ?thumbs renders every head as a portrait into one sprite sheet and writes it into the page as a data URL, for make_thumbs.sh to save.
  async function renderThumbs() {
    const W = 160, H = 200, cols = 8, rows = Math.ceil(heads.length / cols);
    canvas.style.width = W + "px"; canvas.style.height = H + "px";
    renderer.setSize(W, H, false);
    const sheet = document.createElement("canvas");
    sheet.width = W * cols; sheet.height = H * rows;
    const ctx = sheet.getContext("2d");
    const map = {};
    for (let i = 0; i < heads.length; i++) {
      current = null;
      await loadHead(heads[i].name);
      setPreset("portrait");
      pending = false; draw();
      const x = i % cols, y = Math.floor(i / cols);
      ctx.drawImage(renderer.domElement, x * W, y * H, W, H);
      map[heads[i].name] = [x, y];
    }
    const out = document.createElement("pre");
    out.id = "thumbs-out";
    out.textContent = JSON.stringify({ cols, rows, w: W, h: H, paint: skin, heads: map, image: sheet.toDataURL("image/webp", 0.86) });
    document.body.appendChild(out);
  }

  // ======================= start =======================
  applyLights();
  loadCommon().then(async () => {
    if (THUMBS) { await renderThumbs(); return; }
    thumbs = await getJSON("thumbs/thumbs.json");
    buildSkinOptions();
    buildShapeOptions();
    buildPerf();
    buildPicker();
    wire();
    setCompare("ours");
    const first = params.get("head");
    await loadHead(heads.some((h) => h.name === first) ? first : "p_bastilah");
    const sel = document.querySelector(`.portrait[data-head="${current}"]`);
    if (sel) { sel.setAttribute("aria-pressed", "true"); reveal(sel, true); }
    updatePerfUI();
  }).catch((e) => toast("The viewer could not load its data: " + e.message));

  // hold(value, t) stops playback and shows the pose of an animation choice ("shape:4", "clip:tlklaugh", "line") at t seconds, so that
  // pictures can be taken at the same moment on every head.
  function hold(value, t) { stop(); anim = value; posed(poseAt(value, t || 0)); updatePerfUI(); }
  // closeUp(d, yaw) puts the camera d metres from the mouth, turned yaw degrees, for close pictures of the lips.
  function closeUp(d, yaw) {
    if (!ours) return;
    const c = ours.mouth.clone().add(new THREE.Vector3(0, -0.004, 0)), a = (yaw || 0) * Math.PI / 180;
    target.copy(c);
    camera.position.set(c.x + d * Math.sin(a), c.y, c.z + d * Math.cos(a));
    camera.lookAt(target);
    controls.update();
    requestRender();
  }
  // eyeCloseUp(d, yaw, pitch, side) puts the camera d metres from one eyeball's centre ("L", "R", or both eyes when side is omitted),
  // turned yaw degrees and raised pitch degrees, for close pictures of the eyes.
  function eyeCloseUp(d, yaw, pitch, side) {
    if (!ours) return;
    const p = ours.eyes.geometry.attributes.position.array, half = p.length / 6, c = new THREE.Vector3();
    const from = side === "R" ? half : 0, to = side === "L" ? half : 2 * half;
    for (let i = from; i < to; i++) { c.x += p[3 * i]; c.y += p[3 * i + 1]; c.z += p[3 * i + 2]; }
    c.multiplyScalar(1 / (to - from)).add(new THREE.Vector3(0, 0, 0.012));
    const a = (yaw || 0) * Math.PI / 180, b = (pitch || 0) * Math.PI / 180;
    target.copy(c);
    camera.position.set(c.x + d * Math.sin(a) * Math.cos(b), c.y + d * Math.sin(b), c.z + d * Math.cos(a) * Math.cos(b));
    camera.lookAt(target);
    controls.update();
    requestRender();
  }
  // setSlider(name, value) moves one slider by its name, as a person would, for tests and pictures.
  function setSlider(name, value) {
    const k = slider.kit ? slider.kit.meta.findIndex((s) => s.name === name) : -1;
    if (k < 0) return false;
    sliderEnsure(current);
    slider.values[k] = value;
    sliderChanged(true);
    return true;
  }
  window.headViewer2 = { get camera() { return camera; }, get scene() { return sceneO; }, get slider() { return slider; }, setSlider, setShape, get ours() { return ours; }, get kotor() { return kotor; }, setPreset, setCompare, setBody, bodyMeshes: () => (bodyGroups[0] ? bodyGroups[0].children.map((m) => m.geometry.attributes.position.array) : []), selectHead, hold, closeUp, eyeCloseUp, get state() { return { current, skin, withHair, hairChoice, hairPaint, compare, anim, playing, lighting, shape, rig, preset }; } };
})();
