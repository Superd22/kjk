// The head sliders of viewer 2. The slider page's live view (../nose/nose.js, tab "3. Live sliders") is the working example. This file
// loads the same browser modules and the same files, and builds the head with the same calls. A head is a start shape (starts_all.bin,
// the head's earlier fit with its nose, jaw, brow, cheeks and eyes taken out) plus the values of every slider. Everything here is in
// the fit's frame, in metres, which is the frame of viewer 2's own head. viewer_map.bin gives, for each of viewer 2's 5,852 vertices,
// the one of our 4,750 it is a copy of (prep_nose.py writes it).
window.HeadSliders = (function () {
  const D = "/nose/data/";
  // keep these equal to VER, EYES_MODULE_VER, the lash sizes and PROPOSED_EYES in nose.js
  const VER = "?b=49";
  const EYES_MODULE_VER = "22";
  // the head_sliders module's own version: 39 adds the fold check (fold_setup, fold_marks, fold_blame); 40 the lid smoother (eyes round 6)
  const SLIDERS_MODULE_VER = "50";
  const EYE_LASH_LENGTH = 1.2, EYE_LASH_THICKNESS = 0.25;
  const PROPOSED_EYES = { "Eyeball size": 1.0 };
  // the eyeball's radius when its slider is at 0, in mm; the slider shows the radius itself
  const BALL_MM = 12.3;
  // ---------- the eye construction: the curve of 2026-11-21 ----------
  // Every head's eye opening is a curve on the eyeball set by ten handles, and the lid piece and the lash strip are built on the curve.
  // The modules come from /wasm/curve/ and the presets from the *_curve.json files. The old opening (the lid and corner sliders without
  // the curve) is kept in ~/.claude/tools/likeness-fit-data/backups/eyes-legacy-2026-11-22/. Raise EYE_CURVE_VER after a rebuild of a
  // module in /wasm/curve/ or a change of a *_curve.json file or of the curve's data, so that browsers load them again.
  const EYE_CURVE_VER = "5";
  const MOD_ROOT = "/wasm/curve";
  const FEATURES = [
    { key: "nose", title: "Nose", sliders: "nose/sliders_r3_nostril.json", page: "nose/page_r3_nostril.json" },
    { key: "jaw", title: "Jaw and chin", sliders: "jaw/sliders.json", page: "jaw/page_throat.json" },
    { key: "brow", title: "Brow and forehead", sliders: "brow/sliders.json", page: "brow/page_fold.json" },
    { key: "cheeks", title: "Cheekbones and cheeks", sliders: "cheeks/sliders.json", page: "cheeks/page_curve.json" },
    { key: "eyes", title: "Eyes", sliders: "eyes/sliders_curve.json", page: "eyes/page_curve.json" },
    { key: "lips", title: "Lip shape", sliders: "lips/sliders_r2.json", page: "lips/page_r2.json" },
    { key: "skull", title: "Throat and skull", sliders: "skull/sliders.json", page: "skull/page.json" },
  ];
  const fileVer = (f) => (/_curve\.json$/.test(f) ? VER + "c" + EYE_CURVE_VER : VER);
  const GROUPS = [["root", "Root"], ["bridge", "Bridge"], ["tip", "Tip"], ["wings", "Wings"], ["base", "Base"], ["jaw", "Jaw"], ["chin", "Chin"], ["brow", "Brow"], ["forehead", "Forehead"], ["temples", "Temples"], ["cheekbones", "Cheekbones"], ["cheeks", "Cheeks"], ["whole", "Whole eye"], ["size", "Size"], ["lids", "Lids"], ["corners", "Corners"], ["caruncle", "Caruncle"], ["throat", "Throat"], ["skull", "Skull"], ["lips", "Lips"]];
  const fetchOk = async (u) => { const r = await fetch(u, { cache: "no-cache" }); if (!r.ok) throw new Error(`${u}: ${r.status}`); return r; };
  const getJSON = async (u) => (await fetchOk(u)).json();
  const getText = async (u) => (await fetchOk(u)).text();
  const typed = async (u, Type) => new Type(await (await fetchOk(u)).arrayBuffer());
  const loader = () => import("/wasm/load.js?v=22");

  // the merged groups, the first vertex of each group and the hidden flags of the head mesh, which the curve's build reads (init_curve)
  const CD = "/wasm/head_eye_curve/data/";

  // ---------- the kit: the merged set of every feature's sliders, the start shapes and each head's solved values ----------
  let kitP = null;
  function kit() {
    return kitP || (kitP = (async () => {
      const t0 = performance.now();
      const L = await loader();
      const [wasm, base, tris, keep, heads, starts, vmap, parts, hiddenV, pageAll, eyeStarts] = await Promise.all([
        L.loadCrate("head_sliders", MOD_ROOT, SLIDERS_MODULE_VER + "c" + EYE_CURVE_VER),
        typed(D + "base.bin" + VER, Float32Array), typed(D + "triangles.bin" + VER, Uint32Array), typed(D + "keep.bin" + VER, Uint8Array),
        getJSON(D + "heads.json" + VER), typed(D + "starts_all.bin" + VER, Float32Array), typed("viewer_map.bin", Uint32Array),
        Promise.all(FEATURES.map(async (f) => ({ ...f, text: await getText(D + f.sliders + fileVer(f.sliders)), page: await getJSON(D + f.page + fileVer(f.page)) }))),
        typed(D + "fold_hidden.bin" + VER, Uint8Array), getJSON(D + "page.json" + VER), typed(D + "eyes/starts.bin" + VER, Float32Array),
      ]);
      const VC = base.length / 3;
      const merged = wasm.merge_sets(JSON.stringify(parts.map((p) => JSON.parse(p.text))));
      const builder = new wasm.Builder(base, tris, keep, merged);
      const q = "?v=" + EYE_CURVE_VER;
      const [group, first, hiddenM] = await Promise.all([typed(CD + "group.bin" + q, Uint32Array), typed(CD + "first.bin" + q, Uint32Array), typed(CD + "hidden.bin" + q, Uint8Array)]);
      builder.init_curve(group, first, hiddenM);
      // the throat construction (head_sliders chin_neck::ThroatShape): the hidden vertices' six skin neighbours and their weights, and
      // the merged vertices it may move
      const TD = D + "throat/";
      const [near, nearW, free] = await Promise.all([typed(TD + "near.bin" + VER, Uint32Array), typed(TD + "near_w.bin" + VER, Float64Array), typed(TD + "free.bin" + VER, Uint8Array)]);
      if (builder.init_throat) builder.init_throat(base, group, first, hiddenM, near, nearW, free);
      else console.error("this head_sliders module has no throat construction (init_throat); rebuild it from the worktree's head_sliders");
      // the finer triangles below the lower lids (head_eye_curve refine.rs): the same for every head, with new vertices after the 4,750
      let refine = builder.refine_parents ? { parents: builder.refine_parents(), weights: builder.refine_weights(), tris: builder.refine_triangles(), parent: builder.refine_parent(), split: builder.refine_split() } : null;
      if (refine && !refine.parents.length) refine = null;
      let lastExtra = null;
      const meta = [];
      let eyesText = null;
      for (const p of parts) {
        const sl = JSON.parse(p.text).sliders;
        p.first = meta.length;
        p.count = sl.length;
        // a curve handle's or a throat slider's rest value is its middle, the base head's, not 0, which would be no eye or no throat
        sl.forEach((s) => meta.push({ feature: p.key, name: s.name, hint: s.hint, group: s.group, unit: s.unit, kind: s.kind, measure: s.measure, low: s.low, high: s.high, zero: s.kind === "curve" || s.kind === "throat" ? s.mid : 0 }));
        // the results list their sliders in their own order, so values are matched to the set's sliders by name, as nose.js does
        const names = sl.map((s) => s.name);
        p.place = p.page.sliders ? p.page.sliders.map((s) => names.indexOf(s.name)) : null;
        p.byName = Object.fromEntries(p.page.heads.map((h) => [h.name, h]));
        const counts = p.page.counts ? p.page.counts.map(String) : [];
        p.last = counts[counts.length - 1];
        if (p.key === "eyes") eyesText = p.text;
        delete p.text;
      }
      if (JSON.parse(merged).sliders.length !== meta.length) console.warn(`the merged set has ${JSON.parse(merged).sliders.length} sliders, the feature sets have ${meta.length}`);
      const index = Object.fromEntries(heads.map((h, i) => [h.name, i]));
      // the values solved for a head with the most sliders, ranges set from all heads ("full"), as the slider page's "Load this head's solved values"
      function solved(name) {
        const out = new Float64Array(meta.length);
        let any = false;
        for (const p of parts) {
          const h = p.byName[name], s = h && h.solved && (h.solved.full || h.solved.held_out);
          const v = s && s.counts && s.counts[p.last] && s.counts[p.last].values;
          if (!v) continue;
          any = true;
          v.forEach((x, i) => { const j = p.place ? p.place[i] : i; if (j >= 0 && j < p.count) out[p.first + j] = x; });
        }
        return any ? out : null;
      }
      let lastValues = null;
      let lastName = null;
      const build = (name, values) => {
        const i = index[name]; lastValues = values; lastName = name;
        const out = builder.build(starts.subarray(i * VC * 3, (i + 1) * VC * 3), values);
        lastExtra = refine ? builder.refine_extra() : null;
        return out;
      };
      // the fold check of the kit (head_sliders check::Checker): the faults are counted against the head's smoothed start, among the
      // vertices of the eyes' window, with the head's own frame from its eye centres
      const eyesOf = {};
      for (const h of pageAll.heads) eyesOf[h.name] = new Float64Array([...h.eyes[0], ...h.eyes[1]]);
      // two ways to run the check. "kit" is the eye audit's: the head built with the eye sliders only, from the eyes' own start shapes
      // (eyes/starts.bin), so the counts equal the kit's. "shown" is the head as the page shows it, with every feature.
      const eyesPart = parts.find((p) => p.key === "eyes");
      const eyeBuilder = new wasm.Builder(base, tris, keep, eyesText);
      const modes = {
        kit: { builder: eyeBuilder, starts: eyeStarts, pick: (v) => Float64Array.from(v).slice(eyesPart.first, eyesPart.first + eyesPart.count), at: null },
        shown: { builder, starts, pick: (v) => Float64Array.from(v), at: null },
      };
      const startOf = (m, name) => { const i = index[name]; return modes[m].starts.subarray(i * VC * 3, (i + 1) * VC * 3); };
      const foldSetup = (m, name) => { const o = modes[m]; if (o.at !== name) { o.builder.fold_setup(startOf(m, name), eyesOf[name], hiddenV); o.at = name; } return o; };
      const foldMarks = (m, name, values) => { const o = foldSetup(m, name); return JSON.parse(o.builder.fold_marks(startOf(m, name), o.pick(values))); };
      // the blame comes back with slider numbers of the mode's own set; in "kit" mode they are shifted to the merged set's numbers
      const foldBlame = (m, name, values, top) => {
        const o = foldSetup(m, name), r = JSON.parse(o.builder.fold_blame(startOf(m, name), o.pick(values), top)), d = m === "kit" ? eyesPart.first : 0;
        for (const x of r.singles) x.k += d;
        for (const x of r.pairs) { x.a += d; x.b += d; }
        return r;
      };
      // the curve of the last build of each eye (head_sliders Builder.curve_info), read right after it so no other build comes between
      const curveInfo = (e) => { const t = builder.curve_info(e); return t ? JSON.parse(t) : null; };
      return { meta, metaOf: meta, parts, solved, build, curveInfo, foldMarks, foldBlame, names: Object.keys(index), current: () => ({ name: lastName, values: lastValues }), has: (name) => name in index, base, tris, VC, vmap, refine, extra: () => lastExtra, loadMs: performance.now() - t0 };
    })());
  }
  // Viewer 2's mesh with the finer triangles below the lower lids, made once from its own mesh (common.json) and the kit's refinement.
  // Its vertices are viewer 2's 5,852 first, then one per new vertex and side of a seam, then the midpoints of its edges. `vmap` takes
  // each to the build's vertices, where the new ones follow the 4,750. Each new vertex takes the texture place and the skin weights of
  // its corners by its weights, and `fixSkin` does so again after the rig gives the old vertices new weights. `cutter` leaves out the
  // triangles that lie in hidden head triangles, as skinCutter.
  function refinedTopo(common, kit) {
    const R = kit.refine, VC = kit.VC, n = common.n, E = common.edges, full = common.trisFull, shown = common.tris, vmap = kit.vmap;
    const key3 = (a, b, c) => { const k = [a, b, c].sort((x, y) => x - y); return (k[0] * VC + k[1]) * VC + k[2]; };
    const headTri = new Map();
    for (let t = 0; t < kit.tris.length / 3; t++) headTri.set(key3(kit.tris[3 * t], kit.tris[3 * t + 1], kit.tris[3 * t + 2]), t);
    // viewer 2's triangles before its split in four, from the corner quarters [a, mid(a, b), mid(a, c)] of the split ones
    const other = (m, a) => (E[2 * (m - n)] === a ? E[2 * (m - n) + 1] : E[2 * (m - n)]);
    const shownSet = new Set();
    for (let t = 0; t < shown.length / 3; t++) shownSet.add(shown[3 * t] + "," + shown[3 * t + 1] + "," + shown[3 * t + 2]);
    const base = new Map();
    for (let t = 0; t < full.length / 3; t++) {
      const a = full[3 * t], b = full[3 * t + 1], c = full[3 * t + 2];
      if (!(a < n && b >= n && c >= n)) continue;
      const tri = [a, other(b, a), other(c, a)], lo = tri.indexOf(Math.min(...tri)), k = [tri[lo], tri[(lo + 1) % 3], tri[(lo + 2) % 3]].join(",");
      const vis = shownSet.has(a + "," + b + "," + c);
      const was = base.get(k);
      base.set(k, { tri, vis: (was ? was.vis : false) || vis });
    }
    const split = new Set(R.split), byParent = new Map();
    for (let i = 0; i < R.parent.length; i++) { const t = R.parent[i]; if (!byParent.has(t)) byParent.set(t, []); byParent.get(t).push(i); }
    const vm = Array.from(vmap), newOf = new Map(), from = [];
    const tris0 = [], vis0 = [], par0 = [];
    for (const { tri, vis } of base.values()) {
      const t = headTri.get(key3(vmap[tri[0]], vmap[tri[1]], vmap[tri[2]]));
      // near the mouth viewer 2's triangles are not the head's; none of them is split
      if (t === undefined || !split.has(t)) { tris0.push(tri); vis0.push(vis); par0.push(t === undefined ? -1 : t); continue; }
      const corner = (x) => tri[[0, 1, 2].find((k) => vmap[tri[k]] === x)];
      const at = (x) => {
        if (x < VC) return corner(x);
        const j = x - VC, ps = [], ws = [];
        for (let k = 0; k < 3; k++) { const w = R.weights[3 * j + k]; if (w > 1e-6) { ps.push(corner(R.parents[3 * j + k])); ws.push(w); } }
        const key = j + ":" + ps.join(",");
        if (!newOf.has(key)) { newOf.set(key, n + from.length); from.push({ ps, ws }); vm.push(VC + j); }
        return newOf.get(key);
      };
      for (const i of byParent.get(t)) { tris0.push([at(R.tris[3 * i]), at(R.tris[3 * i + 1]), at(R.tris[3 * i + 2])]); vis0.push(vis); par0.push(t); }
    }
    const n2 = n + from.length;
    // split in four at the edges' midpoints, as viewer 2's own mesh
    const edgeAt = new Map(), edges = [];
    const mid = (a, b) => { const k = a < b ? a * n2 + b : b * n2 + a; if (!edgeAt.has(k)) { edgeAt.set(k, n2 + edges.length / 2); edges.push(a, b); } return edgeAt.get(k); };
    const tris = [], parent = [];
    tris0.forEach(([a, b, c], i) => {
      if (!vis0[i]) return;
      const ab = mid(a, b), bc = mid(b, c), ca = mid(c, a);
      tris.push(a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca);
      for (let k = 0; k < 4; k++) parent.push(par0[i]);
    });
    const total = n2 + edges.length / 2, uv = new Float32Array(2 * total), cu = common.uv;
    uv.set(cu.subarray(0, 2 * n));
    from.forEach(({ ps, ws }, i) => { for (let c = 0; c < 2; c++) uv[2 * (n + i) + c] = ps.reduce((s, p, k) => s + ws[k] * cu[2 * p + c], 0); });
    for (let k = 0; k < edges.length / 2; k++) for (let c = 0; c < 2; c++) uv[2 * (n2 + k) + c] = (uv[2 * edges[2 * k] + c] + uv[2 * edges[2 * k + 1] + c]) / 2;
    const fixSkin = (idx, w) => {
      from.forEach(({ ps, ws }, i) => {
        const acc = new Map();
        ps.forEach((p, k) => { for (let s = 0; s < 4; s++) { const b = idx[4 * p + s], x = w[4 * p + s] * ws[k]; if (x > 0) acc.set(b, (acc.get(b) || 0) + x); } });
        const top = [...acc.entries()].sort((a, b) => b[1] - a[1]).slice(0, 4), sum = top.reduce((s, x) => s + x[1], 0) || 1, v = n + i;
        for (let s = 0; s < 4; s++) { idx[4 * v + s] = top[s] ? top[s][0] : 0; w[4 * v + s] = top[s] ? top[s][1] / sum : 0; }
      });
    };
    // the normals of the new vertices and of the midpoints next to them: their corners' normals by their weights, so the fine skin
    // shades as smoothly as the head's own triangles did and no flat face of a split triangle shows
    const midNew = [];
    for (let k = 0; k < edges.length / 2; k++) if (edges[2 * k] >= n || edges[2 * k + 1] >= n) midNew.push(k);
    const fixNormals = (N) => {
      from.forEach(({ ps, ws }, i) => {
        let x = 0, y = 0, z = 0;
        ps.forEach((p, k) => { x += ws[k] * N[3 * p]; y += ws[k] * N[3 * p + 1]; z += ws[k] * N[3 * p + 2]; });
        const l = Math.hypot(x, y, z) || 1, v = 3 * (n + i);
        N[v] = x / l; N[v + 1] = y / l; N[v + 2] = z / l;
      });
      for (const k of midNew) {
        const a = 3 * edges[2 * k], b = 3 * edges[2 * k + 1], v = 3 * (n2 + k);
        const x = N[a] + N[b], y = N[a + 1] + N[b + 1], z = N[a + 2] + N[b + 2], l = Math.hypot(x, y, z) || 1;
        N[v] = x / l; N[v + 1] = y / l; N[v + 2] = z / l;
      }
    };
    const skinIndex = new common.skinIndex.constructor(4 * n2), skinWeight = new Float32Array(4 * n2);
    skinIndex.set(common.skinIndex); skinWeight.set(common.skinWeight);
    fixSkin(skinIndex, skinWeight);
    const T = new Uint32Array(tris), m = T.length / 3;
    let last = null, lastIndex = null;
    const cutter = (hidden) => {
      const sig = hidden.join(",");
      if (sig === last) return lastIndex;
      const hide = new Set(hidden), keep = [];
      for (let t = 0; t < m; t++) if (!hide.has(parent[t])) keep.push(T[3 * t], T[3 * t + 1], T[3 * t + 2]);
      last = sig;
      lastIndex = new THREE.BufferAttribute(new Uint32Array(keep), 1);
      return lastIndex;
    };
    return { n: n2, tris: T, edges: new Uint32Array(edges), uv, skinIndex, skinWeight, vmap: Uint32Array.from(vm), parent, fixSkin, fixNormals, cutter, refined: true };
  }
  // our 4,750 positions as viewer 2's 5,852
  function toViewer(world, vmap, out) {
    out = out || new Float32Array(3 * vmap.length);
    for (let i = 0; i < vmap.length; i++) { const v = 3 * vmap[i]; out[3 * i] = world[v]; out[3 * i + 1] = world[v + 1]; out[3 * i + 2] = world[v + 2]; }
    return out;
  }

  // ---------- the eyeballs and the generated upper lid piece, made by head_eyes as on the slider page ----------
  let eyeP = null;
  function eyeKit(uv4750) {
    return eyeP || (eyeP = (async () => {
      const k = await kit();
      const L = await loader();
      const [m, edges] = await Promise.all([L.loadCrate("head_eyes", MOD_ROOT, EYES_MODULE_VER + "c" + EYE_CURVE_VER), getJSON("/lids/data/lid_edges.json")]);
      const maker = new m.LidMaker(k.tris, k.base.subarray(0, 3 * edges.vertices));
      for (const key of ["L", "R"]) {
        const e = edges.eyes[key];
        maker.add_eye(new Uint32Array(e.edge), new Uint8Array(e.lid.map((l) => (l === "upper" ? 1 : 0))), e.inner_corner, e.outer_corner, new Uint32Array(e.eyeball_centre_vertices), e.temple_is_plus_x);
      }
      return { maker, edges, uv: uv4750, tris: k.tris, VC: k.VC, kit: k };
    })());
  }
  // the eye's picture used when the head has no eye texture of its own: a white eye, a brown iris and a black pupil, one eye in each half
  let drawnEye = null;
  function drawnEyeMap() {
    if (drawnEye) return drawnEye;
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
    return (drawnEye = new THREE.CanvasTexture(c));
  }
  function eyeCentreOf(edges, world, key) {
    const ids = edges.eyes[key].eyeball_centre_vertices, c = [0, 0, 0];
    for (const v of ids) for (let k = 0; k < 3; k++) c[k] += world[3 * v + k] / ids.length;
    return c;
  }
  // an eye's radius in metres, read from its 8 eyeball-centre vertices: the Eyeball size slider scales their ring (6.907 mm from its
  // mean at the radius of 12.3 mm) with the radius, as head_eyes and the paint read it
  function eyeRadiusOf(edges, world, key) {
    const ids = edges.eyes[key].eyeball_centre_vertices, c = eyeCentreOf(edges, world, key);
    let s = 0;
    for (const v of ids) for (let k = 0; k < 3; k++) s += (world[3 * v + k] - c[k]) ** 2;
    return 0.0123 * Math.sqrt(s / ids.length) / 0.006907;
  }
  // the two eyeballs, spheres round the eyeball-centre vertices at each eye's own radius, as one mesh. Their texture coordinates are laid out as viewer 2's
  // eye textures expect: a point made from the unit point (x, y, z) reads (x / 4 + 0.25 + half, y / 2 + 0.5).
  function eyeballMesh(edges, world, map, clay) {
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
    const mat = clay ? new THREE.MeshStandardMaterial({ color: new THREE.Color(0.72, 0.64, 0.58), roughness: 0.75, metalness: 0 })
      : new THREE.MeshStandardMaterial({ color: 0xffffff, map: map || drawnEyeMap(), roughness: 0.4, metalness: 0 });
    const mesh = new THREE.Mesh(g, mat);
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
  // one eye's piece and lash strip, in a group that turns about the pivot. The piece takes its colour from the paint above it, as on the
  // slider page, and is clay when the head has no paint.
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
    for (let i = 0; i < n; i++) raw.set(px ? sampleColour(px, uv[2 * i], uv[2 * i + 1], 6) : [0.72, 0.64, 0.58], 3 * i);
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
    // the piece on the curve turns about the line through the eyeball's centre parallel to the line through the tips
    const axis = new THREE.Vector3(...info.axis);
    group.quaternion.setFromAxisAngle(axis, closed ? (info.closed_angle * Math.PI) / 180 : 0);
    group.userData = { closedAngle: info.closed_angle, closed, axis };
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
  // the eyeballs and both pieces of a head (positions of our 4,750 vertices in metres), and the skin faces of our 4,750 that they replace
  function eyes(kit, world, o) {
    for (const e of [0, 1]) { const c = kit.kit.curveInfo(e); kit.maker.set_curve(e, new Float32Array(c ? c.upper : []), new Float32Array(c ? c.lower : [])); }
    kit.maker.build(world, 0, EYE_LASH_LENGTH, EYE_LASH_THICKNESS, 35, false, 0);
    const hidden = [...kit.maker.hidden_faces(0), ...kit.maker.hidden_faces(1)];
    const root = new THREE.Group();
    root.matrixAutoUpdate = false;
    root.add(eyeballMesh(kit.edges, world, o.eyeMap, o.clay));
    for (const e of [0, 1]) root.add(pieceGroup(kit, e, o.clay ? null : o.map, o.closed));
    foldOnEyes(kit.kit, root, world);
    const balls = ["L", "R"].map((key) => ({ c: eyeCentreOf(kit.edges, world, key), r: eyeRadiusOf(kit.edges, world, key) }));
    return { root, hidden, balls };
  }
  // ---------- the fold view: the faulty triangles of the shown head, and the sliders that cause them ----------
  // The check is the kit's (head_sliders check::Checker, run by the browser module): a triangle is faulty when it is turned over
  // against the head it started from, or lies beside a neighbour it now meets at more than 90 degrees where it met at under 45.
  // `count` is the kit's number (a fold of two neighbours counts once), `tris` the triangles drawn.
  const fold = { mode: "kit", on: false, hi: null, hiLabel: "", tris: [], count: null, mesh: null, root: null, world: null, kit: null, blame: null, blameKey: "", blameMs: 0, busy: false, worst: null, timer: 0, boxes: [] };
  const foldBoxes = fold.boxes;
  const FOLD_RED = [1, 0.04, 0.04], FOLD_HI = [1, 0.92, 0.1];
  const FOLD_LIFT = 0.0005;
  // the orbit controls of the page's camera, taken when viewer2.js makes them (this file loads first), for the button that frames the folds
  let orbit = null;
  if (window.THREE && THREE.OrbitControls) {
    const OC = THREE.OrbitControls;
    const W = function (o, d) { const c = new OC(o, d); orbit = c; return c; };
    W.prototype = OC.prototype;
    THREE.OrbitControls = W;
  }
  const redraw = () => window.dispatchEvent(new Event("resize"));
  function foldColours() {
    const col = new Float32Array(9 * fold.tris.length);
    fold.tris.forEach((t, i) => { const c = fold.hi && fold.hi.has(t) ? FOLD_HI : FOLD_RED; for (let v = 0; v < 3; v++) col.set(c, 9 * i + 3 * v); });
    return col;
  }
  function foldMesh(k, world, r) {
    const T = k.tris, B = k.base, n = r.tris.length, P = new Float32Array(9 * n), N = new Float32Array(3 * n);
    r.tris.forEach((t, i) => {
      const id = [T[3 * t], T[3 * t + 1], T[3 * t + 2]];
      // the push out of the skin follows the base head's outward side, as a turned-over triangle's own normal points inward
      const a = id.map((v) => [B[3 * v], B[3 * v + 1], B[3 * v + 2]]);
      const u = [0, 1, 2].map((c) => a[1][c] - a[0][c]), w = [0, 1, 2].map((c) => a[2][c] - a[0][c]);
      const nn = [u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]], l = Math.hypot(...nn) || 1;
      id.forEach((v, j) => { for (let c = 0; c < 3; c++) P[9 * i + 3 * j + c] = world[3 * v + c] + (FOLD_LIFT * nn[c]) / l; });
      N.set(nn.map((x) => x / l), 3 * i);
    });
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.BufferAttribute(P, 3));
    g.setAttribute("color", new THREE.BufferAttribute(foldColours(), 3));
    const m = new THREE.Mesh(g, new THREE.MeshBasicMaterial({ vertexColors: true, side: THREE.DoubleSide, polygonOffset: true, polygonOffsetFactor: -4, polygonOffsetUnits: -4 }));
    m.frustumCulled = false;
    m.name = "folds";
    // the faulty triangles are often under a millimetre wide, so each one also gets a dot of 7 pixels at its centre
    const cen = new Float32Array(3 * n), cg = new THREE.BufferGeometry();
    for (let i = 0; i < n; i++) for (let c = 0; c < 3; c++) cen[3 * i + c] = (P[9 * i + c] + P[9 * i + 3 + c] + P[9 * i + 6 + c]) / 3;
    cg.setAttribute("position", new THREE.BufferAttribute(cen, 3));
    { const col = g.getAttribute("color").array, c3 = new Float32Array(3 * n); for (let i = 0; i < n; i++) for (let c = 0; c < 3; c++) c3[3 * i + c] = col[9 * i + c]; cg.setAttribute("color", new THREE.BufferAttribute(c3, 3)); }
    const dots = new THREE.Points(cg, new THREE.PointsMaterial({ size: 7, sizeAttenuation: false, vertexColors: true, depthTest: false, transparent: true }));
    dots.frustumCulled = false;
    dots.renderOrder = 10;
    m.add(dots);
    m.userData.normals = N;
    return m;
  }
  // called by eyes() after every build of the head, and by the toggle
  function foldAttach(k, root, world) {
    const cur = k.current();
    if (!cur.name || !cur.values) return;
    const r = k.foldMarks(fold.mode, cur.name, cur.values);
    fold.count = r.count;
    fold.tris = r.tris;
    fold.mesh = foldMesh(k, world, r);
    root.add(fold.mesh);
    foldBoxes.forEach((b) => b.update());
  }
  function foldOnEyes(k, root, world) {
    fold.root = root; fold.world = world; fold.kit = k;
    fold.mesh = null;
    if (!fold.on) return;
    const cur = k.current();
    if (fold.hi && fold.hiHead !== cur.name) fold.hi = null;
    foldAttach(k, root, world);
    if (fold.blame && fold.blameKey !== foldKey(k)) {
      foldBoxes.forEach((b) => b.update());
      clearTimeout(fold.timer);
      fold.timer = setTimeout(() => foldBlame(), 700);
    }
  }
  const foldKey = (k) => { const c = k.current(); return fold.mode + "|" + c.name + "|" + Array.from(c.values).join(","); };
  function foldSet(on) {
    fold.on = on;
    if (fold.mesh) { fold.mesh.parent && fold.mesh.parent.remove(fold.mesh); dispose(fold.mesh); fold.mesh = null; }
    if (on && fold.kit && fold.root) foldAttach(fold.kit, fold.root, fold.world);
    foldBoxes.forEach((b) => b.update());
    redraw();
  }
  function foldHighlight(tris, label) {
    fold.hi = tris && tris.length ? new Set(tris) : null;
    fold.hiLabel = fold.hi ? label : "";
    fold.hiHead = fold.kit ? fold.kit.current().name : null;
    if (fold.mesh) {
      fold.mesh.geometry.setAttribute("color", new THREE.BufferAttribute(foldColours(), 3));
      const dots = fold.mesh.children[0], col = foldColours(), c3 = new Float32Array(3 * fold.tris.length);
      for (let i = 0; i < fold.tris.length; i++) for (let c = 0; c < 3; c++) c3[3 * i + c] = col[9 * i + c];
      dots.geometry.setAttribute("color", new THREE.BufferAttribute(c3, 3));
    }
    foldBoxes.forEach((b) => b.update());
    redraw();
  }
  function foldBlame() {
    const k = fold.kit;
    if (!k || fold.busy) return;
    fold.busy = true;
    foldBoxes.forEach((b) => b.update());
    setTimeout(() => {
      const cur = k.current(), t0 = performance.now();
      fold.blame = k.foldBlame(fold.mode, cur.name, cur.values, 10);
      fold.blameMs = performance.now() - t0;
      fold.blameKey = foldKey(k);
      fold.busy = false;
      foldBoxes.forEach((b) => b.update());
    }, 20);
  }
  // the presets of all heads, counted with the same check
  function foldWorst() {
    const k = fold.kit;
    if (!k || fold.worstBusy) return;
    fold.worstBusy = true;
    const names = k.names.filter((n) => k.solved(n)), rows = [], t0 = performance.now();
    let i = 0;
    const step = () => {
      for (let n = 0; n < 4 && i < names.length; n++, i++) rows.push([names[i], k.foldMarks(fold.mode, names[i], k.solved(names[i])).count]);
      fold.worstProgress = `${i} of ${names.length}`;
      if (i < names.length) { foldBoxes.forEach((b) => b.update()); setTimeout(step, 0); return; }
      rows.sort((a, b) => b[1] - a[1]);
      fold.worst = { rows, ms: performance.now() - t0, heads: rows.length, bad: rows.filter((r) => r[1] > 0).length, total: rows.reduce((s, r) => s + r[1], 0) };
      fold.worstBusy = false;
      foldBoxes.forEach((b) => b.update());
    };
    step();
  }
  function foldFrame() {
    if (!fold.mesh || !orbit || !orbit.object) return false;
    const m = fold.mesh, pos = m.geometry.attributes.position, nrm = m.userData.normals;
    const pick = fold.hi && fold.hi.size ? fold.tris.map((t, i) => [t, i]).filter(([t]) => fold.hi.has(t)).map((x) => x[1]) : fold.tris.map((t, i) => i);
    if (!pick.length) return false;
    m.updateWorldMatrix(true, false);
    const pts = [], nsum = new THREE.Vector3();
    for (const i of pick) {
      for (let v = 0; v < 3; v++) pts.push(new THREE.Vector3().fromBufferAttribute(pos, 3 * i + v).applyMatrix4(m.matrixWorld));
      nsum.add(new THREE.Vector3(nrm[3 * i], nrm[3 * i + 1], nrm[3 * i + 2]).transformDirection(m.matrixWorld));
    }
    const c = pts.reduce((a, p) => a.add(p), new THREE.Vector3()).multiplyScalar(1 / pts.length);
    const radius = Math.max(0.012, ...pts.map((p) => p.distanceTo(c)));
    const cam = orbit.object, dir = cam.position.clone().sub(orbit.target).normalize();
    if (nsum.lengthSq() > 0) dir.copy(nsum.normalize());
    orbit.minDistance = Math.min(orbit.minDistance, 0.05);
    const d = Math.min(orbit.maxDistance, Math.max(orbit.minDistance, (radius / Math.tan((cam.fov * Math.PI) / 360)) * 1.5));
    orbit.target.copy(c);
    cam.position.copy(c).addScaledVector(dir, d);
    cam.lookAt(c);
    orbit.update();
    redraw();
    return true;
  }
  const foldStyle = document.createElement("style");
  foldStyle.textContent = `.fold-box .fold-list { margin: 6px 0 0; padding: 0; list-style: none; display: grid; gap: 3px; }
.fold-box .fold-list button { box-sizing: border-box; width: 100%; flex-wrap: wrap; display: flex; justify-content: space-between; gap: 10px; text-align: left; background: transparent; border: 1px solid var(--line-hi); color: var(--ink-soft); padding: 4px 8px; font: 500 12px var(--title); cursor: pointer; border-radius: 2px; }
.fold-box .fold-list button:hover { border-color: var(--accent); color: var(--ink); }
.fold-box .fold-list button[aria-pressed="true"] { background: var(--accent-dim); border-color: #ffe81a; color: var(--ink); }
.fold-box .fold-list .n { white-space: nowrap; color: var(--accent); }
.fold-box h3 { margin: 10px 0 0; font: 600 12px var(--title); letter-spacing: 0.04em; }
.fold-box .fold-key { display: inline-block; width: 10px; height: 10px; margin-right: 5px; vertical-align: -1px; }`;
  document.head.appendChild(foldStyle);
  function foldBox() {
    const root = el("section", "group sl-tools fold-box");
    const gh = el("div", "group-head");
    gh.append(el("h2", "section-title", "Skin folds"));
    const status = el("p", "sl-note");
    const row = el("div", "sl-row");
    const seg = el("div", "seg sl-buttons");
    const mk = (label, note, fn) => { const b = el("button", null, label); b.type = "button"; b.dataset.note = note; b.onclick = fn; seg.append(b); return b; };
    const tog = mk("Show folds", "Draws every faulty triangle of the shown head in red, a little in front of the skin, and keeps it up to date as a slider moves.", () => foldSet(!fold.on));
    const frame = mk("Frame the folds", "Moves the camera close to the faulty triangles, or to the highlighted ones.", () => { if (!foldFrame()) status.textContent = "There is no fold to frame."; });
    row.append(el("span", "caption", "Folds"), seg);
    const row2 = el("div", "sl-row");
    const seg2 = el("div", "seg sl-buttons");
    const mk2 = (label, note, fn) => { const b = el("button", null, label); b.type = "button"; b.dataset.note = note; b.onclick = fn; seg2.append(b); return b; };
    const why = mk2("Which sliders cause it", "Sets each slider back to 0 on its own, and each pair among the ten sliders that matter most, and counts the faulty triangles that go away.", () => foldBlame());
    const worst = mk2("Worst heads", "Counts the faulty triangles of every head's preset and lists the heads with the most.", () => foldWorst());
    row2.append(el("span", "caption", "Causes"), seg2);
    const row3 = el("div", "sl-row");
    const seg3 = el("div", "seg sl-buttons");
    const modeBtns = {};
    for (const [m, label, note] of [["kit", "Eye sliders only", "Counts the faults as the kit does: the head built from the eye sliders alone on the eyes' own start shape. These counts equal the kit's."], ["shown", "Head as shown", "Counts the faults of the head as it is shown, with the values of every feature."]]) {
      const b = el("button", null, label); b.type = "button"; b.dataset.note = note;
      b.onclick = () => { if (fold.mode === m) return; fold.mode = m; fold.blame = null; fold.worst = null; fold.hi = null; clearTimeout(fold.timer); foldSet(fold.on); };
      seg3.append(b); modeBtns[m] = b;
    }
    row3.append(el("span", "caption", "Check"), seg3);
    const out = el("div", "fold-out");
    const hint = el("p", "hint");
    root.addEventListener("pointerover", (e) => { const t = e.target.closest && e.target.closest("[data-note]"); if (t) hint.textContent = t.dataset.note; });
    root.addEventListener("pointerleave", () => { hint.textContent = ""; });
    root.append(gh, status, row, row2, row3, out, hint);
    const label = (k) => { const m = fold.kit.current && foldMeta()[k]; return m ? `${m.name} (${m.feature})` : "slider " + k; };
    const listRow = (text, n, tris, tag, extra) => {
      const b = el("button");
      b.type = "button";
      const key = tag;
      b.dataset.key = key;
      b.setAttribute("aria-pressed", fold.hi && fold.hiLabel === key ? "true" : "false");
      b.append(el("span", null, text), el("span", "n", extra ? `${n} go away (${extra})` : `${n} go away`));
      b.onclick = () => foldHighlight(fold.hiLabel === key ? null : tris, key);
      const li = el("li");
      li.append(b);
      return li;
    };
    const update = () => {
      for (const m in modeBtns) modeBtns[m].setAttribute("aria-pressed", fold.mode === m ? "true" : "false");
      tog.setAttribute("aria-pressed", fold.on ? "true" : "false");
      tog.textContent = fold.on ? "Hide folds" : "Show folds";
      frame.disabled = !fold.on || !fold.tris.length;
      why.disabled = !fold.on || fold.busy;
      worst.disabled = !fold.kit || !!fold.worstBusy;
      if (!fold.kit) status.textContent = "The fold view needs the head made by the sliders, shown under Shape in the tab Look.";
      else if (!fold.on) status.textContent = "The folds are off. Show folds draws each faulty triangle of the head in red.";
      else status.textContent = `${fold.count} faults, drawn as ${fold.tris.length} red triangles.${fold.hi ? ` The ${fold.hi.size} yellow ones are what "${fold.hiLabel.replace(/^[a-z]+:/, "")}" causes.` : ""} A fault is a triangle turned over against the head it started from, or one of two neighbours that now meet at more than 90 degrees.`;
      out.replaceChildren();
      if (fold.on && (fold.busy || fold.blame)) {
        const h = el("h3", null, "Sliders that cause the folds");
        out.append(h);
        if (fold.busy) out.append(el("p", "sl-note", "Counting, with 100 or so heads built…"));
        else if (fold.blame) {
          const stale = fold.blameKey !== (fold.kit ? foldKey(fold.kit) : "");
          out.append(el("p", "sl-note", `${stale ? "The values changed, so this is out of date and is counted again in a moment. " : ""}Each number is how many of the ${fold.blame.count} faults go away when those sliders are set to 0 and the others stay. Counted in ${Math.round(fold.blameMs)} ms.`));
          if (!fold.blame.singles.length) out.append(el("p", "sl-note", fold.blame.count ? "No single slider removes a fault." : "This head has no fault."));
          const ul = el("ul", "fold-list");
          for (const s of fold.blame.singles.slice(0, 10)) ul.append(listRow(label(s.k), s.saved, s.tris, "one:" + label(s.k)));
          out.append(ul);
          const pairs = fold.blame.pairs.filter((p) => p.extra > 0).sort((x, y) => y.saved - x.saved || y.extra - x.extra).slice(0, 10);
          if (!pairs.length && fold.blame.singles.length) out.append(el("p", "sl-note", "No pair of sliders removes more faults than the better one of the two alone."));
          if (pairs.length) {
            out.append(el("h3", null, "Pairs of sliders (only pairs that remove more than the better one alone)"));
            const pl = el("ul", "fold-list");
            for (const p of pairs) pl.append(listRow(`${label(p.a)} and ${label(p.b)}`, p.saved, p.tris, `pair:${label(p.a)} and ${label(p.b)}`, p.extra ? `${p.extra} more than the better one alone` : "no more than one alone"));
            out.append(pl);
          }
        }
      }
      if (fold.worstBusy) { out.append(el("h3", null, "Worst heads")); out.append(el("p", "sl-note", `Counting the presets, ${fold.worstProgress}…`)); }
      else if (fold.worst) {
        const w = fold.worst;
        out.append(el("h3", null, "Worst heads"));
        out.append(el("p", "sl-note", `${w.bad} of ${w.heads} presets have a fault, ${w.total} faults in all. Counted in ${Math.round(w.ms)} ms. Click a head to load its preset.`));
        const ul = el("ul", "fold-list");
        for (const [name, n] of w.rows.slice(0, 10)) {
          const b = el("button");
          b.type = "button";
          const info = window.HEAD_INFO && window.HEAD_INFO[name];
          b.append(el("span", null, `${info ? info.label + " " : ""}(${name})`), el("span", "n", `${n} faults`));
          b.onclick = () => { const p = document.querySelector(`.portrait[data-head="${name}"]`); if (p) p.click(); };
          const li = el("li");
          li.append(b);
          ul.append(li);
        }
        out.append(ul);
      }
    };
    update();
    return { root, update };
  }
  const foldMeta = () => fold.kit.metaOf;
  const dispose = (root) => root.traverse((x) => { if (x.geometry) x.geometry.dispose(); if (x.material) x.material.dispose(); });
  // Viewer 2 draws our head split once more, each triangle cut in four at its edges' midpoints. Each of its triangles is keyed by the
  // triangle of our 4,750 it lies in, so the faces the lid piece replaces can be left out of the drawn index.
  function skinCutter(topo, vmap, kit) {
    const { tris, VC } = kit, T = topo.tris, n = topo.n, E = topo.edges, m = T.length / 3, keys = new Float64Array(m);
    for (let t = 0; t < m; t++) {
      const s = new Set();
      for (let c = 0; c < 3; c++) {
        const v = T[3 * t + c];
        if (v < n) s.add(vmap[v]); else { s.add(vmap[E[2 * (v - n)]]); s.add(vmap[E[2 * (v - n) + 1]]); }
      }
      const k = [...s].sort((a, b) => a - b);
      keys[t] = k.length === 3 ? (k[0] * VC + k[1]) * VC + k[2] : -1;
    }
    let last = null, lastIndex = null;
    return (hidden) => {
      const sig = hidden.join(",");
      if (sig === last) return lastIndex;
      const hide = new Set();
      for (const f of hidden) { const k = [tris[3 * f], tris[3 * f + 1], tris[3 * f + 2]].sort((a, b) => a - b); hide.add((k[0] * VC + k[1]) * VC + k[2]); }
      const keep = [];
      for (let t = 0; t < m; t++) if (!hide.has(keys[t])) keep.push(T[3 * t], T[3 * t + 1], T[3 * t + 2]);
      last = sig;
      lastIndex = new THREE.BufferAttribute(new Uint32Array(keep), 1);
      return lastIndex;
    };
  }

  // ---------- the panel: one tab per feature, each slider with its name, hint, value, range and the head's preset ----------
  const unitOf = (s) => { const m = /^1 (mm|degrees)\b/.exec(s.unit || ""); return m ? m[1] : /^1 spread/.test(s.unit || "") ? "spreads" : s.kind === "proportion" ? "mm" : ""; };
  function fmtVal(v, s, signed) {
    if (s.measure === "ball") return `${(BALL_MM + v).toFixed(2)} mm`;
    const u = unitOf(s), d = u === "degrees" ? 1 : 2, x = Math.abs(v) < 0.5 * Math.pow(10, -d) ? 0 : v;
    const t = (signed && x > 0 ? "+" : "") + x.toFixed(d).replace("-", "−");
    return u === "degrees" ? t + "°" : u ? t + " " + u : t;
  }
  function el(tag, cls, text) { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; }
  // o: { tabs, look, page, kit, onInput(k, v), onPreset(feature or null), onZero(feature or null), onProposed(values), onLid(state), lid }
  function panel(o) {
    const { kit: K } = o;
    const rows = [];
    const sections = {};
    const notes = [];
    let tab = "look";
    for (const p of K.parts) {
      const sec = el("div", "sl-feature");
      sec.dataset.feature = p.key;
      sec.hidden = true;
      const head = el("section", "group sl-tools");
      const gh = el("div", "group-head");
      const h2 = el("h2", "section-title", p.title);
      h2.append(el("span", null, `${p.count} sliders`));
      gh.append(h2);
      const note = el("p", "sl-note");
      notes.push(note);
      const toolRow = (label, feature) => {
        const r = el("div", "sl-row");
        r.append(el("span", "caption", label));
        const seg = el("div", "seg sl-buttons");
        const a = el("button", null, "Back to the preset"), z = el("button", null, "Set to zero");
        a.type = z.type = "button";
        a.dataset.note = feature ? `Sets the ${p.title.toLowerCase()} sliders back to the values solved for this head.` : "Sets every slider of every feature back to the values solved for this head.";
        z.dataset.note = feature ? `Sets the ${p.title.toLowerCase()} sliders to the base head's values. The ${p.title.toLowerCase()} then has the base head's shape.` : "Sets every slider of every feature to the base head's values.";
        a.onclick = () => o.onPreset(feature);
        z.onclick = () => o.onZero(feature);
        seg.append(a, z);
        r.append(seg);
        return r;
      };
      head.append(gh, note, toolRow(p.title, p.key), toolRow("Every feature", null));
      foldBoxes.push(foldBox());
      if (p.key === "eyes") {
        const lidRow = el("div", "sl-row");
        lidRow.append(el("span", "caption", "Upper lid"));
        const seg = el("div", "seg sl-buttons");
        seg.setAttribute("role", "group");
        seg.setAttribute("aria-label", "Upper lid");
        for (const [k, label] of [["open", "Open"], ["closed", "Closed"]]) {
          const b = el("button", null, label);
          b.type = "button";
          b.dataset.lid = k;
          b.dataset.note = k === "open" ? "The generated upper lid piece is open." : "The generated upper lid piece turns down over the eyeball, as in a blink.";
          b.setAttribute("aria-pressed", k === o.lid ? "true" : "false");
          b.onclick = () => { for (const x of seg.children) x.setAttribute("aria-pressed", x === b ? "true" : "false"); o.onLid(k); };
          seg.append(b);
        }
        lidRow.append(seg);
        const prop = el("button", "toggle sl-proposed");
        prop.type = "button";
        prop.append(el("span", "opt-label", "Proposed default eyes"));
        const words = Object.entries(PROPOSED_EYES).map(([n, v]) => (n === "Eyeball size" ? `eyeball's radius to ${(BALL_MM + v).toFixed(1)} mm` : `${n.toLowerCase()} to ${v} mm`)).join(" and the ");
        prop.dataset.note = `Sets the ${words}, the values proposed as the default for every head. The other eye sliders keep their values.`;
        prop.onclick = () => o.onProposed(PROPOSED_EYES);
        head.append(lidRow, prop);
      }
      const hint = el("p", "hint");
      head.append(hint);
      head.addEventListener("pointerover", (e) => { const t = e.target.closest && e.target.closest("[data-note]"); if (t) hint.textContent = t.dataset.note; });
      head.addEventListener("pointerleave", () => { hint.textContent = ""; });
      sec.append(head);
      sec.append(foldBoxes[foldBoxes.length - 1].root);
      const items = K.meta.map((s, k) => [s, k]).filter(([s]) => s.feature === p.key);
      const known = new Set(GROUPS.map((g) => g[0]));
      for (const [g, title] of [...GROUPS, [null, "Other"]]) {
        const list = items.filter(([s]) => (g ? s.group === g : !known.has(s.group)));
        if (!list.length) continue;
        const grp = el("section", "sl-group");
        grp.append(el("h3", "sl-group-title", title));
        for (const [s, k] of list) {
          const own = s.kind === "curve" || s.kind === "throat";
          const lo = own ? s.low : Math.min(0, s.low), hi = own ? s.high : Math.max(0, s.high);
          const row = el("div", "sl");
          const top = el("div", "sl-top");
          const name = el("label", "sl-name", s.name);
          name.htmlFor = "sl" + k;
          const val = el("output", "sl-val");
          top.append(name, val);
          const input = el("input");
          input.type = "range";
          input.id = "sl" + k;
          input.min = lo; input.max = hi; input.step = (hi - lo) / 400; input.value = 0;
          const ends = el("div", "sl-ends");
          const pre = el("span", "sl-preset");
          ends.append(el("span", null, fmtVal(lo, s)), pre, el("span", null, fmtVal(hi, s)));
          input.addEventListener("input", () => { val.textContent = fmtVal(+input.value, s, true); o.onInput(k, +input.value); });
          row.append(top);
          if (s.hint) row.append(el("p", "sl-hint", s.hint));
          row.append(input, ends);
          grp.append(row);
          rows[k] = { input, val, pre, s };
        }
        sec.append(grp);
      }
      o.page.append(sec);
      sections[p.key] = sec;
      const t = el("button", null, p.title);
      t.type = "button";
      t.setAttribute("role", "tab");
      t.dataset.panelTab = p.key;
      t.setAttribute("aria-selected", "false");
      o.tabs.append(t);
    }
    const scroller = o.page.parentElement;
    function show(key) {
      tab = sections[key] ? key : "look";
      for (const b of o.tabs.querySelectorAll("[data-panel-tab]")) b.setAttribute("aria-selected", b.dataset.panelTab === tab ? "true" : "false");
      o.look.hidden = tab !== "look";
      o.page.hidden = tab === "look";
      for (const [k, s] of Object.entries(sections)) s.hidden = k !== tab;
      scroller.scrollTop = 0;
      try { localStorage.setItem("viewer2.panelTab", tab); } catch (e) { /* storage may be blocked */ }
    }
    for (const b of o.tabs.querySelectorAll("[data-panel-tab]")) b.addEventListener("click", () => show(b.dataset.panelTab));
    let start = "look";
    try { start = localStorage.getItem("viewer2.panelTab") || "look"; } catch (e) { /* storage may be blocked */ }
    show(start);
    return {
      show,
      get tab() { return tab; },
      // puts the values on the sliders, and the head's preset under each one
      sync(values, preset) {
        rows.forEach((r, k) => {
          if (!r) return;
          r.input.value = values[k];
          r.val.textContent = fmtVal(values[k], r.s, true);
          r.pre.textContent = preset ? "Preset " + fmtVal(preset[k], r.s, true) : "No preset";
        });
      },
      note(text) { for (const n of notes) n.textContent = text; },
    };
  }

  return { kit, eyeKit, toViewer, refinedTopo, eyes, dispose, skinCutter, panel, FEATURES, PROPOSED_EYES, fold, get orbit() { return orbit; } };
})();
