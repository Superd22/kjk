// The live paint of head viewer 2. It paints our head's texture and the procedural eyeballs with head_paint_gpu (Rust compiled to
// WebAssembly, run on the GPU through WebGPU), as the live paint page ../paint/ does, and builds one tab of sliders per paint layer.
// viewer2.js calls it through a few hooks: start() when the page loads, texture() and withHair() where it picks the skin's texture,
// eyes() where it picks the eyeballs' texture, skinButton() and skinUI() for the Skin group, and shapeMoved() after a shape slider moved.
// The textures are canvases, so that the painted hair layer and the lid pieces of sliders.js can read their pixels as they read a baked
// paint's image. A canvas texture is flipped as a loaded picture is, so the painter's rows from the top fit viewer 2's own texture
// coordinates without the flip that the paint page needs.
// While the group "Light from maps" is on, the painter also writes a normal map, a gloss mask and the detail tile's mask, and the head's
// material takes a highlight from the viewer's directional lights and a small repeated tile of pores and furrows (see "light from maps"
// below). A hook in viewer2.js calls material() after the skin is put on, and gives the scenes so that the main light can turn.
window.HeadPaint = (function () {
  const LIVE = "/paint/live/";
  const FULL = 2048;
  // How long the head must keep its shape before the paint is computed again from it, when no slider was let go (a button moved them).
  const SHAPE_WAIT_MS = 250;
  // The eye paint of a head made by the sliders reads the lids' edges from the eye curve's rim, the visible edge of the opening, instead
  // of the lid edge loop, which the curve lays under the lid. The lid pieces' texture is painted with the shading layer at no more than
  // LID_SHADING_MAX.
  const $ = (id) => document.getElementById(id);
  const getJSON = async (url) => { const r = await fetch(url, { cache: "no-cache" }); if (!r.ok) throw new Error(`${url}: ${r.status}`); return r.json(); };
  const typed = async (url, Type) => { const r = await fetch(url); if (!r.ok) throw new Error(`${url}: ${r.status}`); return new Type(await r.arrayBuffer()); };

  let ctx = null, mod = null, painter = null, recipes = null, groups = [], list = [], unpainted = [];
  let failed = null, readyP = null, prepared = false;
  // what is wanted and what is painted
  let values = null, fitted = null, switches = [], valuesHead = null;
  let want = { h: null, shapeGen: 0 }, seq = 0, doneSeq = 0, running = false, waiters = [];
  let shapeGen = 0, shapeTimer = 0;
  let headKey = null, eyeKey = null, last = { ms: 0, headMs: 0 };
  const base = { canvas: document.createElement("canvas"), tex: null, h: null, old: [] };
  const hair = { canvas: document.createElement("canvas"), tex: null, from: null, layer: null };
  const eye = { canvas: document.createElement("canvas"), tex: null, h: null, old: null };
  // The lid pieces of sliders.js copy their colour from the texels of the lid's edge, which the eye paint fills with the lash line and
  // the liner. So the pieces get a second, smaller paint of the same head with those controls at 0, and keep the crease, the socket,
  // the eye shadow and the lid's own colours. The piece's own lash strip draws the lashes.
  const LID_SIZE = 1024;
  const LID_OFF = ["eye_upper_lash", "eye_lower_lash", "eye_liner", "eye_liner_lower"];
  // With the eye rim on, the pieces' paint takes the shading layer at no more than this strength, the shared pattern of the KOTOR heads.
  // The pieces wear the skin just above the lid's edge, deep in the fold under the brow, where a strong shading paints it near black,
  // while the piece itself stands in front of the fold and the viewer's light shades it.
  const LID_SHADING_MAX = 1;
  const lid = { canvas: document.createElement("canvas"), tex: null, h: null, prepared: false };
  // The maps of the last paint made with the light from maps on, and the detail tile, shared by every head.
  const lm = { tex: null, h: null, tile: null, sizes: null, group: -1, ms: 0 };

  // --- starting ---
  // Returns whether the browser offers WebGPU. The module, the layout and the recipes then load in the background.
  function start(c) {
    ctx = c;
    if (!navigator.gpu) { failed = "This browser has no WebGPU, so the live paint is off and the head wears our newest baked paint."; return false; }
    readyP = load().catch((e) => {
      console.error("the live paint", e);
      failed = "The live paint could not start on this browser's GPU, so the head wears our newest baked paint. " + (e && e.message ? "The error was: " + e.message : "");
      if (ctx.skin === "live") ctx.setSkin(ctx.bakedKey());
      ctx.toast(failed);
      markButton();
    });
    return true;
  }
  async function load() {
    const L = await import("/wasm/load.js");
    const V = await import("/paint/gpu_version.js?t=" + Date.now());
    const [m, r, uv, tris, vmap, meshTris] = await Promise.all([
      L.loadCrate("head_paint_gpu", "/wasm", V.GPU_MODULE), getJSON(LIVE + "recipes.json"),
      typed(LIVE + "layout/uv.f64", Float64Array), typed(LIVE + "layout/tris.u32", Uint32Array),
      typed(LIVE + "layout/vmap.u32", Uint32Array), typed(LIVE + "layout/mesh_tris.u32", Uint32Array),
    ]);
    mod = m;
    painter = await m.GpuPainter.create(uv, tris, vmap, meshTris);
    // ?beardgrid=<name> lays values written straight onto the facial hair's grid, from live/grids/<name>.json, over the sliders' preset,
    // as the grid editor will. A grid made on another head's grid size is ignored.
    const beardGrid = new URLSearchParams(location.search).get("beardgrid");
    if (beardGrid && painter.set_hair_edit) painter.set_hair_edit(await (await fetch(LIVE + "grids/" + beardGrid + ".json")).text());
    // ?hairgroups=<name> loads hair groups from live/groups/<name>.json, and ?hairpreset=<id> adds a preset once a head is set.
    const hairGroups = new URLSearchParams(location.search).get("hairgroups");
    if (hairGroups && painter.set_hair_groups_json) painter.set_hair_groups_json(await (await fetch(LIVE + "groups/" + hairGroups + ".json")).text());
    hg.preset = new URLSearchParams(location.search).get("hairpreset");
    hg.fromUrl = !!hairGroups;
    recipes = r.heads;
    const spec = JSON.parse(m.controls_json());
    groups = spec.groups;
    list = spec.controls;
    switches = groups.map(() => 1);
    lm.group = groups.findIndex((g) => g.id === "maps");
    unpainted = JSON.parse(m.unpainted_controls_json());
    if (lm.group >= 0) await makeTile();
    if (!ctx.THUMBS) buildPanel();
  }

  // --- painting ---
  function fittedOf(name) { return list.map((c) => (recipes[name] && recipes[name][c.id] !== undefined ? recipes[name][c.id] : c.default)); }
  function request() { seq++; pump(); return seq; }
  async function positionsOf(h) {
    const w = ctx.world(h);
    if (w) return { from: "sliders", p: Float64Array.from(w) };
    return { from: "fit", p: await typed(LIVE + "heads/" + h.name + ".f64", Float64Array) };
  }
  async function pump() {
    if (running) return;
    running = true;
    try {
      while (doneSeq < seq) {
        const job = { seq, h: want.h, shapeGen: want.shapeGen, values: Float64Array.from(values), switches: Uint8Array.from(switches) };
        if (!job.h) { doneSeq = job.seq; continue; }
        const t0 = performance.now();
        const key = job.h.name + "|" + job.shapeGen + "|" + (ctx.world(job.h) ? "s" : "f");
        if (key !== headKey) {
          const pos = await positionsOf(job.h);
          if (painter.set_eye_rim) painter.set_eye_rim(pos.from === "sliders");
          painter.set_head(pos.p);
          if (hg.head !== job.h.name) await switchGroups(job.h.name);
          addPendingPreset();
          if (!prepared) { await painter.prepare(FULL); prepared = true; }
          headKey = key;
          last.from = pos.from;
          last.headMs = performance.now() - t0;
        }
        const t1 = performance.now();
        painter.paint(FULL, job.values, job.switches);
        const rgba = await painter.read(FULL);
        last.ms = performance.now() - t1;
        const mapsOn = mapsWanted(job.switches);
        let mapsBytes = null;
        if (mapsOn) { const t2 = performance.now(); mapsBytes = await painter.read_maps(FULL); lm.ms = performance.now() - t2; }
        const ek = key + "|" + list.map((c, k) => (c.group === "eyes" ? job.values[k] : "")).join(",") + "|" + switches.join("") + "|" + (mapsOn ? view.catchKept : "");
        let eyesNew = false;
        if (ek !== eyeKey) {
          painter.paint_eyeballs(mapsOn ? catchlightCut(job.values) : job.values);
          const eb = await painter.read_eyeballs();
          showEyes(job.h, eb, painter.eyeball_size());
          irisPlaces(job.values);
          eyeKey = ek;
          eyesNew = true;
        }
        if (ctx.world(job.h)) await paintLid(job);
        showBase(job.h, rgba);
        showGrid(job.h, job.values);
        showHandles(job.h);
        showMaps(job.h, mapsBytes);
        last.total = performance.now() - t0;
        doneSeq = job.seq;
        const mine = waiters.filter((w) => w.seq <= doneSeq);
        waiters = waiters.filter((w) => w.seq > doneSeq);
        for (const w of mine) w.resolve();
        // A repaint that nobody waits for (a slider move, a new shape) is put on the head here.
        if (!mine.length && doneSeq === seq && ctx.skin === "live" && ctx.ours === job.h) await ctx.applySkin();
        flushOld();
        status(eyesNew);
      }
    } catch (e) {
      console.error("the live paint", e);
      ctx.toast("The live paint failed: " + (e && e.message ? e.message : e));
      for (const w of waiters) w.resolve();
      waiters = [];
      doneSeq = seq;
    } finally {
      running = false;
    }
  }
  function opaque(rgba) { for (let i = 3; i < rgba.length; i += 4) rgba[i] = 255; return rgba; }
  function onCanvas(canvas, rgba, w, h) {
    if (canvas.width !== w || canvas.height !== h) { canvas.width = w; canvas.height = h; }
    canvas.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray(opaque(rgba).buffer, rgba.byteOffset, rgba.length), w, h), 0, 0);
  }
  function canvasTexture(canvas) {
    const t = new THREE.CanvasTexture(canvas);
    t.anisotropy = ctx.maxAnisotropy;
    return t;
  }
  function showBase(h, rgba) {
    onCanvas(base.canvas, rgba, FULL, FULL);
    if (base.tex) base.old.push(base.tex);
    if (hair.tex) base.old.push(hair.tex);
    base.tex = canvasTexture(base.canvas);
    base.h = h;
    hair.tex = null; hair.from = null;
  }
  function showEyes(h, rgba, n) {
    onCanvas(eye.canvas, rgba, 2 * n, n);
    if (eye.tex) base.old.push(eye.tex);
    eye.tex = canvasTexture(eye.canvas);
    eye.h = h;
  }
  async function paintLid(job) {
    if (!lid.prepared) { await painter.prepare(LID_SIZE); lid.prepared = true; }
    const v = Float64Array.from(job.values);
    for (const id of LID_OFF) { const k = list.findIndex((c) => c.id === id); if (k >= 0) v[k] = 0; }
    const strength = list.findIndex((c) => c.id === "shading_strength");
    if (strength >= 0) v[strength] = Math.min(v[strength], LID_SHADING_MAX);
    painter.paint(LID_SIZE, v, job.switches);
    onCanvas(lid.canvas, await painter.read(LID_SIZE), LID_SIZE, LID_SIZE);
    if (lid.tex) base.old.push(lid.tex);
    lid.tex = canvasTexture(lid.canvas);
    lid.h = job.h;
  }
  // The texture the lid pieces take their colour from, or null to let them use the head's own texture.
  function lidMap(h, skin) { return skin === "live" && !failed && lid.tex && lid.h === h ? lid.tex : null; }
  // Textures that the head no longer wears are freed after the head took the new ones.
  function flushOld() {
    const keep = new Set([base.tex, hair.tex, eye.tex, lm.tex, lid.tex]);
    const s = ctx.ours;
    const used = s ? [s.mat && s.mat.map, s.emat && s.emat.map] : [];
    base.old = base.old.filter((t) => { if (keep.has(t) || used.includes(t)) return true; t.dispose(); return false; });
  }

  // --- the hooks of viewer2.js ---
  // The live texture of a head, painted for it if it was not yet. A head with no fitted paint wears our newest baked paint.
  async function texture(h) {
    if (failed) return ctx.baked(h);
    await readyP;
    if (failed) return ctx.baked(h);
    if (!recipes[h.name]) return ctx.baked(h);
    if (valuesHead !== h.name) { valuesHead = h.name; fitted = fittedOf(h.name); values = fitted.slice(); syncPanel(); }
    if (want.h !== h || (base.h !== h && doneSeq === seq)) { want = { h, shapeGen }; const s = request(); await new Promise((resolve) => waiters.push({ seq: s, resolve })); }
    else if (doneSeq < seq) { const s = seq; await new Promise((resolve) => waiters.push({ seq: s, resolve })); }
    noteShape();
    return base.h === h ? base.tex : null;
  }
  // The live texture with the painted hair layer drawn over it, as withPaintedHair draws it over a baked paint.
  function withHair(h, tex) {
    if (!tex || tex !== base.tex || !h.hairLayer) return tex;
    if (hair.tex && hair.from === tex && hair.layer === h.hairLayer) return hair.tex;
    const c = hair.canvas;
    if (c.width !== FULL) { c.width = FULL; c.height = FULL; }
    const g = c.getContext("2d");
    g.drawImage(base.canvas, 0, 0);
    g.drawImage(h.hairLayer, 0, 0, FULL, FULL);
    if (hair.tex) base.old.push(hair.tex);
    hair.tex = canvasTexture(c);
    hair.from = tex;
    hair.layer = h.hairLayer;
    return hair.tex;
  }
  // Puts the procedural eyeball texture on the head while the live paint is chosen, and the head's own eye texture otherwise.
  function eyes(h, skin) {
    if (!("bakedEyeMap" in h)) h.bakedEyeMap = h.eyeMap || null;
    h.eyeMap = skin === "live" && !failed && eye.tex && eye.h === h ? eye.tex : h.bakedEyeMap;
  }
  // A shape slider moved. The paint is computed again from the new shape when the slider is let go, or after SHAPE_WAIT_MS without a move.
  function shapeMoved(h) {
    if (failed || !painter || ctx.skin !== "live" || !h || want.h !== h) return;
    clearTimeout(shapeTimer);
    shapeTimer = setTimeout(() => shapeSettled(h), SHAPE_WAIT_MS);
  }
  function shapeSettled(h) {
    clearTimeout(shapeTimer);
    if (want.h !== h || !ctx.world(h)) return;
    shapeGen++;
    want = { h, shapeGen };
    request();
  }
  function skinButton(box, choose) {
    const b = document.createElement("button");
    b.type = "button";
    b.dataset.skinMain = "live";
    b.dataset.note = "Our paint, painted live on the GPU from this head's fitted values. The paint tabs hold its sliders.";
    b.textContent = "Live paint";
    b.addEventListener("click", () => { if (!failed) choose(); });
    box.prepend(b);
    markButton();
  }
  function markButton() {
    const b = document.querySelector('[data-skin-main="live"]');
    if (!b) return;
    b.disabled = !!failed;
    if (failed) b.dataset.note = failed;
  }
  function skinUI(tag, setHint) {
    tag.className = "tag ours";
    tag.textContent = "Ours";
    setHint("skin-hint", "Our paint, painted live on the GPU. Every paint layer has a tab of sliders after the shape tabs.");
  }
  // The shape note of the tab Look says that the paint was baked for the smoothed head. With the live paint that is no longer true.
  function shapeNote(setHint) {
    const n = $("shape-note");
    if (!n || !n.dataset.base) return;
    const baked = "The paint was baked for the smoothed head, and it follows this shape through the same texture coordinates.";
    const h = ctx.ours;
    const now = h && ctx.world(h) ? "The live paint is computed from this shape, again each time a shape slider is let go."
      : "The live paint is computed from the fitted head, the same shape as on the live paint page.";
    if (n.dataset.base.includes(baked)) setHint("shape-note", n.dataset.base.replace(baked, now), n.classList.contains("warn"));
    else if (!n.dataset.base.includes(now)) setHint("shape-note", n.dataset.base.replace(/ Our paint's shading and sheen were computed on the default smoothed head, not on this shape\./, "") + " " + now, n.classList.contains("warn"));
  }
  function failure() { return failed; }

  // --- the paint tabs: one per layer, built from the module's list of controls as the paint page builds them ---
  const tabs = [];
  let page = null, active = null, noteEls = [], statusEl = null;
  function el(tag, cls, text) { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; }
  function fmt(c, v) {
    const digits = c.step >= 1 ? 0 : c.step >= 0.1 ? 1 : c.step >= 0.01 ? 2 : c.step >= 0.001 ? 3 : 4;
    return `${Number(v).toFixed(digits)}${c.unit ? " " + c.unit : ""}`;
  }
  function buildPanel() {
    const tabBox = $("panel-tabs"), sliders = $("panel-sliders");
    if (!tabBox || !sliders) return;
    const style = el("style");
    style.textContent = ".pl-moved .sl-val { color: var(--accent); } .pl-fit { background: none; border: 0; padding: 0; font: inherit; color: var(--ink-soft); cursor: pointer; text-decoration: underline dotted; }" +
      " .pl-fit:hover { color: var(--accent); } .pl-unpainted { margin: 2px 0 0; font-size: 11.5px; color: var(--borrowed); } .pl-off .sl-group, .pl-off > .sl { opacity: 0.45; }" +
      " .pl-status { margin: 0; font-size: 12px; color: var(--muted); min-height: 1.4em; }";
    document.head.append(style);
    page = el("div", "panel-page");
    page.id = "panel-paint";
    page.hidden = true;
    sliders.after(page);
    groups.forEach((g, gi) => {
      const sec = el("div", "sl-feature");
      sec.dataset.paintGroup = g.id;
      sec.hidden = true;
      const head = el("section", "group sl-tools");
      const gh = el("div", "group-head");
      const h2 = el("h2", "section-title", g.label);
      h2.append(el("span", null, `${list.filter((c) => c.group === g.id).length} sliders`));
      gh.append(h2);
      const note = el("p", "sl-note", g.note);
      const shapeNote = el("p", "sl-note");
      noteEls.push(shapeNote);
      const rows = el("div", "rows");
      const sw = el("button", "row");
      sw.type = "button";
      sw.dataset.paintSwitch = gi;
      sw.dataset.note = `Turns the layer "${g.label}" on or off. A layer that is off paints nothing.`;
      sw.setAttribute("aria-pressed", "true");
      sw.append(el("span", "opt-label", "Layer on"), el("span", "sw-track"));
      sw.addEventListener("click", () => { switches[gi] = switches[gi] ? 0 : 1; syncSwitches(); request(); });
      rows.append(sw);
      const r = el("div", "sl-row");
      r.append(el("span", "caption", "Back to this head's fitted values"));
      const seg = el("div", "seg sl-buttons");
      const one = el("button", null, g.label), all = el("button", null, "Every layer");
      one.type = all.type = "button";
      one.dataset.note = `Sets the sliders of the layer "${g.label}" back to the values fitted to this head's KOTOR paint, and turns the layer on.`;
      all.dataset.note = "Sets every paint slider back to the values fitted to this head's KOTOR paint, and turns every layer on.";
      one.onclick = () => { list.forEach((c, i) => { if (c.group === g.id) values[i] = fitted[i]; }); switches[gi] = 1; syncPanel(); request(); };
      all.onclick = () => { values = fitted.slice(); switches = groups.map(() => 1); syncPanel(); request(); };
      seg.append(one, all);
      r.append(seg);
      const st = el("p", "pl-status");
      if (!statusEl) statusEl = [];
      statusEl.push(st);
      const hint = el("p", "hint");
      head.append(gh, note, shapeNote, rows, r, st, hint);
      head.addEventListener("pointerover", (e) => { const t = e.target.closest && e.target.closest("[data-note]"); if (t) hint.textContent = t.dataset.note; });
      head.addEventListener("pointerleave", () => { hint.textContent = ""; });
      sec.append(head);
      if (g.id === "facial_hair") {
        // The facial hair has no sliders of its own: they live in its hair groups, and the head's fitted values do not touch them.
        h2.lastChild.textContent = "Hair groups";
        r.hidden = true;
        sec.append(groupControls(), gridControls());
      }
      if (g.id === "maps") {
        sec.append(viewerControls());
        sw.querySelector(".opt-label").textContent = "Light from maps on";
        sw.dataset.note = "Turns the light from maps on or off. When it is off, the head wears exactly the colour painted without maps, and no highlight, normal map or detail tile.";
      }
      const parts = new Map();
      list.forEach((c, i) => {
        if (c.group !== g.id) return;
        let into = sec;
        const partName = c.part || "";
        if (!parts.has(partName)) {
          const grp = el("section", "sl-group");
          if (c.part) grp.append(el("h3", "sl-group-title", c.part));
          sec.append(grp);
          parts.set(partName, grp);
        }
        into = parts.get(partName);
        const row = el("div", "sl");
        row.dataset.paintControl = i;
        const top = el("div", "sl-top");
        const name = el("label", "sl-name", c.label);
        name.htmlFor = "pl-ctl-" + i;
        const val = el("output", "sl-val");
        val.id = "pl-val-" + i;
        top.append(name, val);
        const input = el("input");
        input.type = "range";
        input.id = "pl-ctl-" + i;
        input.min = c.min; input.max = c.max; input.step = c.step;
        input.title = "Double-click to go back to this head's fitted value.";
        const ends = el("div", "sl-ends");
        const fit = el("button", "pl-fit");
        fit.type = "button";
        fit.id = "pl-fit-" + i;
        fit.title = "Sets this slider back to this head's fitted value.";
        ends.append(el("span", null, fmt(c, c.min)), fit, el("span", null, fmt(c, c.max)));
        row.append(top);
        if (c.help) row.append(el("p", "sl-hint", c.help));
        row.append(input, ends);
        if (unpainted.includes(c.id)) row.append(el("p", "pl-unpainted", "The GPU painter does not paint this slider yet, so moving it changes nothing."));
        input.addEventListener("input", () => { values[i] = Number(input.value); syncControl(i); request(); });
        const back = () => { values[i] = fitted[i]; syncControl(i); request(); };
        input.addEventListener("dblclick", back);
        fit.addEventListener("click", back);
        into.append(row);
      });
      page.append(sec);
      const t = el("button", null, g.label);
      t.type = "button";
      t.setAttribute("role", "tab");
      t.dataset.paintTab = g.id;
      t.setAttribute("aria-selected", "false");
      t.addEventListener("click", () => show(g.id));
      tabs.push(t);
      tabBox.append(t);
    });
    // The shape tabs of sliders.js may be added after these. The paint tabs always come last.
    new MutationObserver(() => { if (tabBox.lastElementChild !== tabs[tabs.length - 1] || [...tabBox.children].slice(-tabs.length).some((b, k) => b !== tabs[k])) for (const b of tabs) tabBox.append(b); })
      .observe(tabBox, { childList: true });
    tabBox.addEventListener("click", (e) => {
      const b = e.target.closest && e.target.closest("[data-panel-tab]");
      if (!b) return;
      hide();
      if (b.dataset.panelTab === "look") { $("panel-look").hidden = false; b.setAttribute("aria-selected", "true"); }
    });
    syncPanel();
    let start = null;
    try { start = localStorage.getItem("viewer2.paintTab"); } catch (e) { /* storage may be blocked */ }
    if (start && groups.some((g) => g.id === start)) show(start);
  }
  function show(id) {
    active = id;
    for (const b of document.querySelectorAll("[data-panel-tab]")) b.setAttribute("aria-selected", "false");
    for (const b of tabs) b.setAttribute("aria-selected", b.dataset.paintTab === id ? "true" : "false");
    $("panel-look").hidden = true;
    $("panel-sliders").hidden = true;
    page.hidden = false;
    for (const s of page.children) s.hidden = s.dataset.paintGroup !== id;
    page.parentElement.scrollTop = 0;
    noteShape();
    showHandles();
    try { localStorage.setItem("viewer2.paintTab", id); } catch (e) { /* storage may be blocked */ }
  }
  function hide() {
    if (!page) return;
    active = null;
    page.hidden = true;
    showHandles();
    for (const b of tabs) b.setAttribute("aria-selected", "false");
    try { localStorage.removeItem("viewer2.paintTab"); } catch (e) { /* storage may be blocked */ }
  }
  function syncControl(i) {
    const input = $("pl-ctl-" + i);
    if (!input || !values) return;
    const c = list[i];
    input.value = values[i];
    $("pl-val-" + i).textContent = fmt(c, values[i]);
    $("pl-fit-" + i).textContent = "Fitted " + fmt(c, fitted[i]);
    input.closest(".sl").classList.toggle("pl-moved", values[i] !== fitted[i]);
  }
  function syncSwitches() {
    for (const b of document.querySelectorAll("[data-paint-switch]")) {
      const on = switches[Number(b.dataset.paintSwitch)] === 1;
      b.setAttribute("aria-pressed", on ? "true" : "false");
      b.closest(".sl-feature").classList.toggle("pl-off", !on);
    }
  }
  function syncPanel() {
    if (!page || !values) return;
    list.forEach((_, i) => syncControl(i));
    syncSwitches();
  }
  // Says in every paint tab which shape the paint was computed from.
  function noteShape() {
    const h = ctx && ctx.ours;
    let text;
    if (ctx && ctx.skin !== "live") text = "The live paint is not chosen under Skin in the tab Look, so these sliders change nothing on the head shown.";
    else if (h && ctx.world(h)) text = "The paint follows the shape sliders. When a shape slider is let go, the paint is computed again from the new shape.";
    else text = "The paint is computed from the fitted head, the same shape as on the live paint page, because the head shown is not the one made by the sliders.";
    for (const n of noteEls) n.textContent = text;
  }
  function status(eyesNew) {
    if (!statusEl) return;
    const t = `The last repaint took ${Math.round(last.ms)} ms on the GPU at ${FULL.toLocaleString("en")} pixels, with the read back.` +
      (mapsWanted(switches) ? ` Reading the maps back took ${Math.round(lm.ms)} ms more.` : "") +
      (eyesNew ? " The eyeballs were painted again." : "");
    for (const s of statusEl) s.textContent = t;
  }

  // --- light from maps ---
  // The settings that live only in the viewer's material and lights, as kjk's shader would hold them. The highlight is kjk's own
  // Blinn-Phong: pow(max(N.H, 0), exponent) times the light's colour and strength times `strength`, masked by the gloss.
  const VIEW = [
    { id: "turn", label: "Turn the light", part: "Light", min: -180, max: 180, step: 1, value: 0, unit: "degrees", help: "Turns the main light round the head, from where the lighting choice puts it. At 90 degrees the light comes from the head's left side." },
    { id: "raise", label: "Raise the light", part: "Light", min: -60, max: 60, step: 1, value: 0, unit: "degrees", help: "Moves the main light up or down from where the lighting choice puts it." },
    { id: "strength", label: "Highlight strength", part: "Highlight in the viewer", min: 0, max: 3, step: 0.01, value: 1.2, unit: "", help: "How bright the highlight is where the gloss mask is full, against the light's own strength on a surface that faces it. KOTOR's mod heads reach a mask of 36% on the lips." },
    { id: "broad", label: "Broad sheen", part: "Highlight in the viewer", min: 0, max: 1, step: 0.01, value: 0.3, unit: "", help: "The strength of a second, wider highlight, as skin has: its exponent is a quarter of the highlight's. A small bump tilts it as it tilts the highlight, so the grains and furrows of the normal map and the detail tile show on dark skin, where the light they take from the diffuse colour is small. At 0 there is only the sharp highlight." },
    { id: "exponent", label: "Highlight sharpness", part: "Highlight in the viewer", min: 4, max: 128, step: 1, value: 30, unit: "", help: "The highlight's exponent. A higher number makes a smaller, sharper highlight. KOTOR's bump pass used 16 and kjk's highlight of a fight's light uses 60." },
    { id: "relief", label: "Normal map strength", part: "Highlight in the viewer", min: 0, max: 3, step: 0.01, value: 1, unit: "", help: "Multiplies the slopes of the painter's normal map in the viewer. At 0 the creases have no relief." },
    { id: "tile", label: "Detail tile strength", part: "Detail tile", min: 0, max: 2, step: 0.01, value: 0.5, unit: "", help: "How strongly the repeated tile of pores and furrows tilts the skin, where the pore layer's map of regions lets it. At 0 there is no tile." },
    { id: "tileMm", label: "Detail tile size", part: "Detail tile", min: 3, max: 20, step: 0.1, value: 8, unit: "mm", help: "How much skin one tile covers. The tile is made 8 mm wide, so a larger size makes the pores and furrows larger." },
    { id: "eyeStrength", label: "Wet eye highlight", part: "Eyes", min: 0, max: 3, step: 0.01, value: 1.5, unit: "", help: "How bright the highlight on the eyeball is. The whole eyeball is wet, so it has no mask." },
    { id: "eyeExponent", label: "Wet eye sharpness", part: "Eyes", min: 8, max: 256, step: 1, value: 90, unit: "", help: "The eyeball highlight's exponent. The eye is smoother than the skin, so its highlight is smaller." },
    { id: "catchKept", label: "Painted catchlight kept", part: "Eyes", min: 0, max: 1, step: 0.01, value: 0.3, unit: "", help: "The share of the eyeball's painted reflection of a light that stays while the light from maps is on, because the eyeball now has a real highlight." },
    { id: "skinWrap", label: "Light round the shadow edge", part: "Skin light", min: 0, max: 1, step: 0.01, value: 0.25, unit: "", help: "How far each light wraps past the edge of the shadow, as light that travelled under the skin does. The skin's strongest colour wraps furthest, so the edge of a shadow turns red-brown on dark skin and pink-red on pale skin. At 0.25 the strongest colour of the skin still gets 20% of the light a facing surface gets at the edge of the shadow, and the weakest 11%." },
    { id: "skinSheen", label: "Viewer's grey sheen kept", part: "Skin light", min: 0, max: 1, step: 0.01, value: 0, unit: "", help: "The share kept of the broad grey sheen that the viewer's material adds from every light. kjk's shader has no such sheen, and on dark skin it reads as an ashy veil over the lit cheek and nose. At 1 the head looks as it did before. The highlight of the light from maps is not changed." },
    { id: "irisLight", label: "Light inside the iris", part: "Light inside the iris", min: 0, max: 3, step: 0.01, value: 2, unit: "", help: "How bright the crescent of light is that the cornea focuses onto the side of the iris away from each light. At 1 the crescent gets as much light again as an iris that faces the light, and at 2, the default, twice as much. The crescent is strongest when the light comes from the side and is gone when the light is straight in front of the eye or behind it. At 0, and with the limbus shadow at 0, the eyes look as they did before." },
    { id: "irisSize", label: "Crescent width", part: "Light inside the iris", min: 0, max: 1, step: 0.01, value: 0.7, unit: "", help: "How wide the crescent is at its middle, as a share of the iris's radius, when the light comes fully from the side. A light closer to the front gives a thinner crescent." },
    { id: "irisSoft", label: "Crescent softness", part: "Light inside the iris", min: 0, max: 1, step: 0.01, value: 0.8, unit: "", help: "How softly the crescent's inner edge fades into the rest of the iris. At 0 the edge is sharp." },
    { id: "irisGlow", label: "Crescent colour strength", part: "Light inside the iris", min: 0, max: 1, step: 0.01, value: 0.4, unit: "", help: "How much stronger the colour of the iris is in the crescent than elsewhere, as light that travelled inside the iris's tissue. At 0 the crescent has the iris's own colour. At 1 its colour strength is twice the iris's." },
    { id: "irisFibres", label: "Fibres in the crescent", part: "Light inside the iris", min: 0, max: 2, step: 0.01, value: 1, unit: "", help: "How much the light fibres of the iris stand out in the crescent, against the darker gaps between them. At 0 the crescent lights the fibres and the gaps alike." },
    { id: "irisShade", label: "Limbus shadow", part: "Light inside the iris", min: 0, max: 1, step: 0.01, value: 0.8, unit: "", help: "How much of each light the iris loses on the side nearest that light, where the rim of the eye white shades the iris, which lies a little deeper. At 1 that side of the iris gets no light from it." },
    { id: "irisFloor", label: "Dark iris lift", part: "Light inside the iris", min: 0, max: 0.2, step: 0.005, value: 0.06, unit: "", help: "The least lightness, in linear light, that the crescent's colour takes, so that a dark brown iris shows its brown where the light gathers, as real dark eyes do in sunlight. Its hue is kept. The lift is at most 16 times the iris's own lightness. At 0 a dark iris gets as little light in its crescent as its colour gives." },
  ];
  const view = Object.fromEntries(VIEW.map((v) => [v.id, v.value]));
  let circling = false, circleRaf = 0, circleLast = 0;
  // Whether the skin light is on in the head's material. It lives in the viewer only, as it would in kjk's shader.
  let skinOn = true;
  const U = {
    lmMaps: { value: null }, lmTile: { value: null }, lmRepeat: { value: 30 }, lmTileStrength: { value: view.tile }, lmReliefStrength: { value: view.relief },
    lmExponent: { value: view.exponent }, lmBroad: { value: view.broad }, lmStrength: { value: view.strength }, lmEyeExponent: { value: view.eyeExponent }, lmEyeStrength: { value: view.eyeStrength }, lmDebug: { value: 0 },
    slWrap: { value: view.skinWrap }, slSheen: { value: view.skinSheen },
    irGeo0: { value: new THREE.Vector4(0, 0, 0, 0) }, irGeo1: { value: new THREE.Vector4(0, 0, 0, 0) }, irStrength: { value: view.irisLight },
    irSize: { value: view.irisSize }, irSoft: { value: view.irisSoft }, irGlow: { value: view.irisGlow }, irFibres: { value: view.irisFibres }, irShade: { value: view.irisShade },
    irClear: { value: 1 }, irFloor: { value: view.irisFloor },
  };
  function mapsWanted(sw) { return lm.group >= 0 && sw[lm.group] === 1; }
  function catchlightCut(vals) {
    const k = list.findIndex((c) => c.id === "eye_catchlight");
    if (k < 0) return vals;
    const v = Float64Array.from(vals);
    v[k] *= view.catchKept;
    return v;
  }
  async function makeTile() {
    lm.sizes = Array.from(mod.tile_sizes());
    const n = lm.sizes[1];
    const bytes = await painter.detail_tile(n);
    const t = new THREE.DataTexture(bytes, n, n, THREE.RGBAFormat);
    t.wrapS = t.wrapT = THREE.RepeatWrapping;
    t.magFilter = THREE.LinearFilter;
    t.minFilter = THREE.LinearMipmapLinearFilter;
    t.generateMipmaps = true;
    t.anisotropy = ctx.maxAnisotropy;
    t.needsUpdate = true;
    lm.tile = t;
    U.lmTile.value = t;
    tileRepeat();
  }
  // The tile repeats so that its millimetres match the skin's where a texel covers the face's median size.
  function tileRepeat() { if (lm.sizes) U.lmRepeat.value = (FULL * lm.sizes[2]) / view.tileMm; }
  function showMaps(h, bytes) {
    if (!bytes) { lm.h = null; return; }
    const t = new THREE.DataTexture(bytes, FULL, FULL, THREE.RGBAFormat);
    t.flipY = true;
    t.magFilter = THREE.LinearFilter;
    t.minFilter = THREE.LinearMipmapLinearFilter;
    t.generateMipmaps = true;
    t.anisotropy = ctx.maxAnisotropy;
    t.needsUpdate = true;
    if (lm.tex) base.old.push(lm.tex);
    lm.tex = t;
    lm.h = h;
  }
  // Called by viewer2.js after the skin is put on: the head's material reads the maps while the live paint is chosen and they are on.
  function material(h, skin) {
    if (!h || !h.mat) return;
    const on = skin === "live" && !failed && mapsWanted(switches) && lm.tex && lm.h === h && !!h.mat.map;
    U.lmMaps.value = on ? lm.tex : null;
    patch(h.mat, on ? "skin" : null, skin === "live" && !failed && skinOn && !!h.mat.map);
    ctx.requestRender && ctx.requestRender();
  }
  function eyesOn() { const h = ctx.ours; return !!(h && ctx.skin === "live" && !failed && mapsWanted(switches) && lm.h === h); }
  // Puts the highlight into a material, or takes it out. `kind` is "skin", "eye" or null.
  // `skinLight` puts the skin light of "Skin light" in as well.
  // `iris` puts the light inside the iris in, for the eyeballs.
  function patch(mat, kind, skinLight, iris) {
    const was = mat.userData.lmKind || null;
    const sl = !!skinLight, ir = !!iris;
    if (was === kind && !!mat.userData.slOn === sl && !!mat.userData.irOn === ir) return;
    mat.userData.lmKind = kind;
    mat.userData.slOn = sl;
    mat.userData.irOn = ir;
    mat.defines = mat.defines || {};
    delete mat.defines.LIGHT_MAPS;
    delete mat.defines.LIGHT_MAPS_EYE;
    delete mat.defines.SKIN_LIGHT;
    delete mat.defines.IRIS_LIGHT;
    if (ir) mat.defines.IRIS_LIGHT = "";
    if (sl) mat.defines.SKIN_LIGHT = "";
    if (kind === "skin") mat.defines.LIGHT_MAPS = "";
    if (kind === "eye") mat.defines.LIGHT_MAPS_EYE = "";
    if (!mat.userData.lmPatched) {
      mat.userData.lmPatched = true;
      mat.onBeforeCompile = patchShader;
    }
    mat.needsUpdate = true;
  }
  function patchShader(shader) {
    Object.assign(shader.uniforms, U);
    shader.vertexShader = shader.vertexShader
      .replace("#include <common>", "#include <common>\n" + IR_VARYINGS)
      .replace("#include <fog_vertex>", "#include <fog_vertex>\n" + IR_AXES);
    shader.fragmentShader = shader.fragmentShader
      .replace("#include <common>", "#include <common>\n" + LM_PARS + IR_PARS)
      .replace("#include <normal_fragment_maps>", "#include <normal_fragment_maps>\n" + LM_NORMAL)
      .replace("#include <lights_fragment_end>", "#include <lights_fragment_end>\n" + SL_LIGHT + IR_LIGHT + LM_HIGHLIGHT)
      .replace("#include <dithering_fragment>", "#include <dithering_fragment>\n" + LM_DEBUG);
  }
  // --- the light inside the iris ---
  // A real cornea is a lens: light from the side is focused onto the far side of the iris, where a crescent glows with the iris's own
  // colour, and the near side lies in the shadow of the limbus. Here, for each directional light, the crescent is the part of the iris
  // outside a circle of the iris's own radius moved toward the light, by the crescent width times how far the light is off the eye's
  // axis. The limbus shadow is the same shape moved the other way, on the near side. The iris's place in the eyeball's texture comes from
  // the painter (eyeball_irises), and the eye's axes come from the eyeball mesh, whose x, y and z are the texture's across, up and forward.
  function irisOn() {
    return !!(ctx && ctx.skin === "live" && !failed && U.irGeo0.value.z > 0 && (view.irisLight > 0 || view.irisShade > 0));
  }
  function irisPlaces(vals) {
    if (!painter.eyeball_irises) return;
    const g = painter.eyeball_irises(vals);
    U.irGeo0.value.set(g[0], g[1], g[2], g[3]);
    U.irGeo1.value.set(g[4], g[5], g[6], g[7]);
    // a clouded iris scatters the light everywhere, so it shows no crescent
    const k = list.findIndex((c) => c.id === "eye_clouding");
    U.irClear.value = k < 0 ? 1 : (1 - Math.min(1, Math.max(0, vals[k]))) ** 2;
  }
  const IR_VARYINGS = `
#ifdef IRIS_LIGHT
varying vec3 vIrX;
varying vec3 vIrY;
varying vec3 vIrZ;
#endif`;
  const IR_AXES = `
#ifdef IRIS_LIGHT
vIrX = normalMatrix * vec3(1.0, 0.0, 0.0);
vIrY = normalMatrix * vec3(0.0, 1.0, 0.0);
vIrZ = normalMatrix * vec3(0.0, 0.0, 1.0);
#endif`;
  const IR_PARS = `
#ifdef IRIS_LIGHT
varying vec3 vIrX;
varying vec3 vIrY;
varying vec3 vIrZ;
uniform vec4 irGeo0;
uniform vec4 irGeo1;
uniform float irStrength;
uniform float irSize;
uniform float irSoft;
uniform float irGlow;
uniform float irFibres;
uniform float irShade;
uniform float irClear;
uniform float irFloor;
#endif`;
  const IR_LIGHT = `
#if defined(IRIS_LIGHT) && defined(USE_MAP) && NUM_DIR_LIGHTS > 0
{
  float irE = step(0.5, vUv.x);
  vec2 irQ = vec2((vUv.x - 0.25 - 0.5 * irE) * 4.0, (vUv.y - 0.5) * 2.0);
  vec4 irG = mix(irGeo0, irGeo1, irE);
  vec2 irP = (irQ - irG.xy) / max(irG.z, 1e-3);
  float irR = length(irP);
  float irIn = (1.0 - smoothstep(0.9, 0.99, irR)) * smoothstep(irG.w, irG.w + 0.06, irR);
  if (irIn > 0.0) {
    vec3 irX = normalize(vIrX), irY = normalize(vIrY), irZ = normalize(vIrZ);
    vec3 irAlb = material.diffuseColor;
    vec3 irW = vec3(0.2126, 0.7152, 0.0722);
    float irLum = dot(irAlb, irW);
    // a dark iris shows its own brown in the crescent: its colour is lifted to the floor's lightness, at most 16 times
    vec3 irLift = irAlb * min(max(1.0, irFloor / max(irLum, 1e-4)), 16.0);
    float irLiftLum = dot(irLift, irW);
    vec3 irTint = max(vec3(irLiftLum) + (irLift - vec3(irLiftLum)) * (1.0 + irGlow), vec3(0.0));
    // the fibres: the texel's lightness against a blurred lightness, read from a smaller level of the texture
    float irBlur = dot(mapTexelToLinear(texture2D(map, vUv, 2.0)).rgb, irW);
    float irFib = pow(clamp(dot(texelColor.rgb, irW) / max(irBlur, 1e-4), 0.0, 4.0), 1.5 * irFibres);
    for (int i = 0; i < NUM_DIR_LIGHTS; i++) {
      vec3 irL = directionalLights[i].direction;
      vec2 irL2 = vec2(dot(irL, irX), dot(irL, irY));
      float irOff = length(irL2);
      vec2 irD = irL2 / max(irOff, 1e-4);
      float irSide = smoothstep(0.08, 0.75, irOff) * smoothstep(-0.45, 0.1, dot(irL, irZ));
      float irShift = irSize * irSide;
      // 0 on the crescent's inner edge and 1 at the iris's rim facing away from the light, so the crescent is brightest at the rim
      float irT = (length(irP - irD * irShift) - 1.0) / max(irShift, 1e-3);
      float irCres = smoothstep(-0.3 * irSoft, max(0.9 * irSoft, 0.02), irT);
      float irTn = (length(irP + irD * (0.6 * irShift)) - 1.0) / max(0.6 * irShift, 1e-3);
      float irNear = smoothstep(-0.3 * irSoft, max(0.9 * irSoft, 0.02), irTn);
      float irNL = clamp(dot(normal, irL), 0.0, 1.0);
      reflectedLight.directDiffuse += irIn * irSide * irClear * (irStrength * irCres * irFib * irTint - irShade * irNear * irNL * irAlb) * directionalLights[i].color;
    }
  }
}
#endif`;
  const LM_PARS = `
#if defined(LIGHT_MAPS) || defined(LIGHT_MAPS_EYE)
uniform sampler2D lmMaps;
uniform sampler2D lmTile;
uniform float lmRepeat;
uniform float lmTileStrength;
uniform float lmReliefStrength;
uniform float lmExponent;
uniform float lmBroad;
uniform float lmStrength;
uniform float lmEyeExponent;
uniform float lmEyeStrength;
uniform float lmDebug;
#endif
#ifdef SKIN_LIGHT
uniform float slWrap;
uniform float slSheen;
#endif`;
  // The normal: the painter's slopes plus the tile's, both along the texture's u and v, put on the surface with a frame built from the
  // screen derivatives of the position and the texture coordinates, as three.js does for a normal map without tangents.
  const LM_NORMAL = `
#if defined(LIGHT_MAPS) || defined(LIGHT_MAPS_EYE)
vec3 lmGeomNormal = normal;
float lmGloss = 1.0;
float lmExp = lmEyeExponent;
float lmGain = lmEyeStrength;
float lmWide = 0.0;
#endif
#ifdef LIGHT_MAPS
vec4 lmTexel = texture2D(lmMaps, vUv);
vec2 lmSlope = (lmTexel.xy * 2.0 - 1.0) * lmReliefStrength + (texture2D(lmTile, vUv * lmRepeat).xy * 2.0 - 1.0) * (lmTileStrength * lmTexel.a);
vec3 lmN = normalize(vec3(lmSlope, 1.0));
vec3 lmQ0 = dFdx(-vViewPosition);
vec3 lmQ1 = dFdy(-vViewPosition);
vec2 lmS0 = dFdx(vUv);
vec2 lmS1 = dFdy(vUv);
vec3 lmQ1p = cross(lmQ1, normal);
vec3 lmQ0p = cross(normal, lmQ0);
vec3 lmT = lmQ1p * lmS0.x + lmQ0p * lmS1.x;
vec3 lmB = lmQ1p * lmS0.y + lmQ0p * lmS1.y;
float lmDet = max(dot(lmT, lmT), dot(lmB, lmB));
float lmScale = lmDet == 0.0 ? 0.0 : inversesqrt(lmDet);
normal = normalize(lmT * (lmN.x * lmScale) + lmB * (lmN.y * lmScale) + normal * lmN.z);
lmGloss = lmTexel.b;
lmExp = lmExponent;
lmGain = lmStrength;
lmWide = lmBroad;
#endif`;
  // The highlight of each directional light: kjk's Blinn-Phong with the mapped normal, kept off the side that faces away from the light.
  const LM_HIGHLIGHT = `
#if (defined(LIGHT_MAPS) || defined(LIGHT_MAPS_EYE)) && NUM_DIR_LIGHTS > 0
{
  vec3 lmV = normalize(vViewPosition);
  for (int i = 0; i < NUM_DIR_LIGHTS; i++) {
    vec3 lmL = directionalLights[i].direction;
    float lmFacing = clamp(4.0 * dot(lmGeomNormal, lmL), 0.0, 1.0);
    vec3 lmH = normalize(lmL + lmV);
    float lmNH = max(dot(normal, lmH), 0.0);
    float lmShine = pow(lmNH, lmExp) + lmWide * pow(lmNH, 0.25 * lmExp);
    reflectedLight.directSpecular += (lmGain * lmGloss * lmShine * lmFacing / PI) * directionalLights[i].color;
  }
}
#endif`;
  // The skin light, for every directional light. kjk's diffuse is strength * max(N.L, 0) * colour. Here each channel wraps past the
  // shadow's edge by its own width, w = skinWrap * (0.5 + 0.5 * d), where d is 1 on the texel's strongest channel and 0 on its weakest:
  // facing = clamp((N.L + w) / (1 + w), 0, 1). Only the difference to the plain diffuse is added, so at a wrap of 0 nothing changes.
  // The viewer's grey sheen: three.js's standard material adds a broad white specular (roughness 0.85) from every light, which kjk does
  // not have. Only the share skinSheen of it is kept. This runs before the highlight of the light from maps, which it leaves alone.
  const SL_LIGHT = `
#ifdef SKIN_LIGHT
{
  vec3 slAlbedo = material.diffuseColor;
  float slTop = max(max(slAlbedo.r, slAlbedo.g), slAlbedo.b);
  vec3 slDir = vec3(0.0);
  if (slTop > 1e-4) {
    vec3 slT = slAlbedo / slTop;
    float slLow = min(min(slT.r, slT.g), slT.b);
    if (slLow < 0.9999) slDir = (slT - vec3(slLow)) / (1.0 - slLow);
  }
  vec3 slW = slWrap * (0.5 + 0.5 * slDir);
#if NUM_DIR_LIGHTS > 0
  for (int i = 0; i < NUM_DIR_LIGHTS; i++) {
    float slNL = dot(normal, directionalLights[i].direction);
    vec3 slE = clamp((vec3(slNL) + slW) / (vec3(1.0) + slW), 0.0, 1.0);
    reflectedLight.directDiffuse += (slE - vec3(clamp(slNL, 0.0, 1.0))) * directionalLights[i].color * slAlbedo;
  }
#endif
  reflectedLight.directSpecular *= slSheen;
}
#endif`;
  // For tests: 1 shows the gloss mask on the head, 2 the detail tile's mask, 3 the mapped normal, instead of the lit colour.
  const LM_DEBUG = `
#ifdef LIGHT_MAPS
if (lmDebug > 0.5 && lmDebug < 1.5) gl_FragColor = vec4(vec3(lmTexel.b), 1.0);
if (lmDebug > 1.5 && lmDebug < 2.5) gl_FragColor = vec4(vec3(lmTexel.a), 1.0);
if (lmDebug > 2.5) gl_FragColor = vec4(normal * 0.5 + 0.5, 1.0);
#endif`;
  function setView(id, v) {
    view[id] = v;
    if (id === "strength") U.lmStrength.value = v;
    if (id === "exponent") U.lmExponent.value = v;
    if (id === "broad") U.lmBroad.value = v;
    if (id === "relief") U.lmReliefStrength.value = v;
    if (id === "tile") U.lmTileStrength.value = v;
    if (id === "tileMm") tileRepeat();
    if (id === "eyeStrength") U.lmEyeStrength.value = v;
    if (id === "eyeExponent") U.lmEyeExponent.value = v;
    if (id === "catchKept" && mapsWanted(switches)) request();
    if (id === "skinWrap") U.slWrap.value = v;
    if (id === "skinSheen") U.slSheen.value = v;
    const ir = { irisLight: "irStrength", irisSize: "irSize", irisSoft: "irSoft", irisGlow: "irGlow", irisFibres: "irFibres", irisShade: "irShade", irisFloor: "irFloor" }[id];
    if (ir) U[ir].value = v;
    const input = $("pl-view-" + id);
    if (input && Number(input.value) !== v) input.value = v;
    const out = $("pl-view-val-" + id);
    if (out) out.textContent = fmt(VIEW.find((x) => x.id === id), v);
    ctx.requestRender && ctx.requestRender();
  }
  // Places the main light, the strongest directional light of each scene, turned and raised from where the lighting choice put it. The
  // lighting choice makes its lights again, so each new light keeps its own first place in userData.
  function placeLights(scene) {
    let main = null;
    scene.traverse((o) => { if (o.isDirectionalLight && (!main || o.intensity > main.intensity)) main = o; });
    if (main) {
      if (!main.userData.lmBase) main.userData.lmBase = main.position.clone();
      const b = main.userData.lmBase, r = b.length();
      const az = Math.atan2(b.x, b.z) + (view.turn * Math.PI) / 180;
      const el = Math.max(-1.5, Math.min(1.5, Math.asin(b.y / r) + (view.raise * Math.PI) / 180));
      main.position.set(r * Math.cos(el) * Math.sin(az), r * Math.sin(el), r * Math.cos(el) * Math.cos(az));
      main.updateMatrixWorld();
    }
    const h = ctx.ours;
    if (h && h.group && scene === h.group.parent) {
      const on = eyesOn();
      h.group.traverse((o) => {
        if (!o.isMesh || !o.material || Array.isArray(o.material) || o.material === h.mat) return;
        const m = o.material;
        if (m.map && m.map === eye.tex) patch(m, on ? "eye" : null, false, irisOn());
      });
    }
  }
  function hookScenes() {
    if (!ctx.scenes || hookScenes.done) return;
    hookScenes.done = true;
    for (const s of ctx.scenes) {
      const before = s.onBeforeRender;
      s.onBeforeRender = function (...a) { placeLights(s); return before.apply(this, a); };
    }
  }
  function skinLight(on) {
    skinOn = on;
    const b = $("pl-skin-light");
    if (b) { b.setAttribute("aria-pressed", on ? "true" : "false"); b.textContent = on ? "Skin light on" : "Skin light off"; }
    if (ctx && ctx.ours) material(ctx.ours, ctx.skin);
  }
  function circle(on) {
    circling = on;
    const b = $("pl-circle");
    if (b) { b.setAttribute("aria-pressed", on ? "true" : "false"); b.textContent = on ? "Stop the light" : "Circle the light"; }
    cancelAnimationFrame(circleRaf);
    if (!on) return;
    circleLast = performance.now();
    const step = (now) => {
      const dt = (now - circleLast) / 1000;
      circleLast = now;
      let t = view.turn + 30 * dt;
      if (t > 180) t -= 360;
      setView("turn", Math.round(t * 10) / 10);
      if (circling) circleRaf = requestAnimationFrame(step);
    };
    circleRaf = requestAnimationFrame(step);
  }
  // --- the paint grid, drawn on the head ---
  // Each point of the grid on the skin is a dot, coloured by its density or by its zone, with a short tick for the direction its hair
  // grows. The dots sit 0.6 mm off the skin, so they show over the painted hair and hide behind the head.
  const grid = { mode: "off", obj: null, h: null, values: null };
  // The zones of head_paint::face_grid::ZONES, in their order, with their colours.
  const ZONE_NAMES = ["Bare skin", "Scalp", "Hairline", "Temple", "Sideburn", "Cheek", "Moustache", "Soul patch", "Chin", "Jaw", "Neck", "Nape"];
  const ZONE_COLOURS = ["light grey", "blue", "cyan", "violet", "brown", "green", "red", "orange", "olive", "pink", "dark grey", "navy"];
  const ZONE_RGB = [[205, 205, 205], [31, 119, 180], [23, 190, 207], [148, 103, 189], [140, 86, 75], [44, 160, 44], [214, 39, 40], [255, 127, 14], [188, 189, 34], [227, 119, 194], [110, 110, 110], [20, 20, 140]];
  function showGrid(h, values) {
    grid.h = h;
    grid.values = values;
    if (grid.obj) { grid.obj.parent && grid.obj.parent.remove(grid.obj); grid.obj.traverse((o) => { if (o.geometry) o.geometry.dispose(); if (o.material) o.material.dispose(); }); grid.obj = null; }
    if (grid.mode === "off" || !h || !h.group || !painter || !painter.hair_grid_points) { if (ctx) ctx.requestRender(); return; }
    const d = painter.hair_grid_points();
    const n = d.length / 11;
    const pos = new Float32Array(3 * n), col = new Float32Array(3 * n), seg = new Float32Array(6 * n), segCol = new Float32Array(6 * n);
    for (let i = 0; i < n; i++) {
      const o = 11 * i, dens = d[o + 6];
      for (let k = 0; k < 3; k++) {
        const p = d[o + k] + 0.0006 * d[o + 3 + k];
        pos[3 * i + k] = p;
        seg[6 * i + k] = p;
        seg[6 * i + 3 + k] = p + 0.0022 * d[o + 7 + k];
      }
      let c;
      const full = Math.min(1, dens / 2);
      if (grid.mode === "regions") {
        const r = ZONE_RGB[d[o + 10]] || ZONE_RGB[0];
        const s = dens > 0.01 ? 1 : 0.6;
        c = [s * r[0] / 255, s * r[1] / 255, s * r[2] / 255];
      } else {
        c = dens > 0.01 ? [0.25 + 0.75 * full, 0.3 + 0.55 * full, 0.6 - 0.5 * full] : [0.22, 0.22, 0.25];
      }
      for (let k = 0; k < 3; k++) { col[3 * i + k] = c[k]; segCol[6 * i + k] = segCol[6 * i + 3 + k] = dens > 0.01 ? 0.05 : 0.3; }
    }
    const g = new THREE.Group();
    const pg = new THREE.BufferGeometry();
    pg.setAttribute("position", new THREE.BufferAttribute(pos, 3));
    pg.setAttribute("color", new THREE.BufferAttribute(col, 3));
    g.add(new THREE.Points(pg, new THREE.PointsMaterial({ size: 4, sizeAttenuation: false, vertexColors: true })));
    const lg = new THREE.BufferGeometry();
    lg.setAttribute("position", new THREE.BufferAttribute(seg, 3));
    lg.setAttribute("color", new THREE.BufferAttribute(segCol, 3));
    g.add(new THREE.LineSegments(lg, new THREE.LineBasicMaterial({ vertexColors: true })));
    g.traverse((o) => { o.frustumCulled = false; });
    h.group.add(g);
    grid.obj = g;
    ctx.requestRender();
  }
  function setGridMode(mode) {
    grid.mode = mode;
    for (const b of document.querySelectorAll("[data-grid-mode]")) b.setAttribute("aria-pressed", b.dataset.gridMode === mode ? "true" : "false");
    if (grid.h) showGrid(grid.h, grid.values || Float64Array.from(values));
  }
  function gridControls() {
    const grp = el("section", "sl-group");
    grp.append(el("h3", "sl-group-title", "Show hair grid"));
    grp.append(el("p", "sl-hint", "Draws every point of the paint grid on the skin as a dot, with a short dark tick for the direction its hair grows by default. The grid covers every place hair can grow, from the neck to the crown. It has holes on the lips, the nose, the eyelids and the ears, where hair never grows. By density, a point with no hair is dark grey, and a point turns from blue to yellow as its hair grows denser, up to 2 hairs per square millimetre. By zone, each point takes the colour of its zone, a little darker where it has no hair: " + ZONE_NAMES.map((r, i) => r + " " + ZONE_COLOURS[i]).join(", ") + ". The zones set the default stubble and the default direction of growth. They are not walls: a hair group paints over any of them. Add ?beardgrid=p_carthh_goatee to the address to load Carth's goatee file onto the grid of any head."));
    const r = el("div", "sl-row");
    const seg = el("div", "seg sl-buttons");
    for (const [mode, label, note] of [["off", "Off", "Hides the grid."], ["density", "By density", "Colours each grid point by how dense its hair is."], ["regions", "By zone", "Colours each grid point by its zone."]]) {
      const b = el("button", null, label);
      b.type = "button";
      b.dataset.gridMode = mode;
      b.dataset.note = note;
      b.setAttribute("aria-pressed", mode === grid.mode ? "true" : "false");
      b.onclick = () => setGridMode(mode);
      seg.append(b);
    }
    r.append(seg);
    grp.append(r);
    return grp;
  }

  // --- hair groups (decision 61): a head's facial hair is its list of hair groups ---
  // The groups and every change to them live in the painter (head_paint::hair_groups, in Rust). This part only draws the selected
  // group's handles and curve and the other groups faintly, and passes the pointer and the keys on. Every edit keeps a step of undo.
  const hg = { spec: null, sel: 0, flow: -1, point: -1, segment: -1, edit: true, obj: null, h: null, handles: [], curve: [], ends: [], drag: null, box: null, preset: null, head: null, fromUrl: false, hint: null, sliding: false, drawing: false };
  // Each head keeps its own hair groups in this browser's storage, saved after every change, so that they come back after a reload. A
  // head with nothing saved takes live/groups/<head>.json when there is one. The first head takes ?hairgroups= or ?hairpreset= instead.
  // Groups saved before 2026-11-21, under the old key, had the stubble in sliders. They get the head's stubble from its file under them.
  const GROUPS_KEY = "kjk-viewer2-hair-groups2:", OLD_GROUPS_KEY = "kjk-viewer2-hair-groups:";
  function saveGroups() { if (!hg.head || !painter) return; try { localStorage.setItem(GROUPS_KEY + hg.head, painter.hair_groups_json()); } catch (e) { /* storage may be off */ } }
  async function fileGroups(name) { try { const r = await fetch(LIVE + "groups/" + name + ".json", { cache: "no-cache" }); return r.ok ? await r.text() : ""; } catch (e) { return ""; } }
  async function switchGroups(name) {
    const first = hg.head === null;
    hg.head = name;
    hg.sel = 0; hg.point = -1; hg.segment = -1;
    if (first && hg.fromUrl) { syncGroups(); return; }
    let text = null, old = null;
    try { text = localStorage.getItem(GROUPS_KEY + name); if (text === null) old = localStorage.getItem(OLD_GROUPS_KEY + name); } catch (e) { text = null; }
    if (text === null && old !== null) {
      const file = await fileGroups(name);
      try {
        const o = JSON.parse(old);
        const stubbles = file ? JSON.parse(file).groups.filter((g) => g.kind === "stubble") : [];
        o.groups = (o.groups || []).concat(stubbles);
        text = JSON.stringify(o);
      } catch (e) { text = file; }
    }
    if (text === null) text = await fileGroups(name);
    if (!painter.set_hair_groups_json(text || "")) painter.set_hair_groups_json("");
    if (old !== null) saveGroups();
    syncGroups(); showHandles();
  }
  function downloadGroups() {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([painter.hair_groups_json()], { type: "application/json" }));
    a.download = (hg.head || "head") + ".json";
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  }
  function groupList() { try { return painter && painter.hair_groups_json ? JSON.parse(painter.hair_groups_json()).groups : []; } catch (e) { return []; } }
  function addPendingPreset() { if (hg.preset && painter.hair_group_add(hg.preset)) { hg.preset = null; hg.sel = 0; syncGroups(); saveGroups(); } }
  // Runs one edit as one step of undo, then repaints and redraws.
  function act(f) { painter.hair_groups_checkpoint(); const r = f(); if (hg.flow >= 0) painter.hair_flow_focus(hg.sel, hg.flow); changed(); syncGroups(); return r; }
  function undo() { if (painter.hair_groups_undo()) { setFlow(-1); changed(); syncGroups(); } }
  function redo() { if (painter.hair_groups_redo()) { setFlow(-1); changed(); syncGroups(); } }
  // What the handles shape: the selected group, or its flow line being shaped, as a stroke of its own.
  function target() {
    const g = groupList()[hg.sel];
    if (!g || hg.flow < 0) return g;
    const f = (g.flow || [])[hg.flow];
    if (!f) { setFlow(-1); return g; }
    return { name: `${g.name}, flow line ${hg.flow + 1}`, kind: "stroke", points: f.points, mirror: g.mirror, hidden: g.hidden, flowLine: true };
  }
  // Shapes flow line `n` of the selected group from now on, or the group's own points with -1.
  function setFlow(n) { hg.flow = n; hg.point = -1; hg.segment = -1; if (painter && painter.hair_flow_focus) painter.hair_flow_focus(hg.sel, n); }
  function shaping() { return active === "facial_hair" && hg.edit; }
  function lift(d, s, off) { return [0, 1, 2].map((k) => d[s + k] + off * d[s + 3 + k]); }
  // Lines of a curve read from rows of six numbers (place and normal) from `s`, broken at rows of NaN.
  function curveRows(d, s, out, segs) {
    let prev = null;
    for (; s + 5 < d.length; s += 6) {
      if (Number.isNaN(d[s])) { prev = null; continue; }
      const p = lift(d, s, 0.0008);
      if (prev) { out.push(...prev, ...p); if (segs) segs.push([prev, p]); }
      prev = p;
    }
  }
  function points(list, size, colour) {
    const pos = new Float32Array(3 * list.length), col = new Float32Array(3 * list.length);
    list.forEach((q, i) => { for (let k = 0; k < 3; k++) { pos[3 * i + k] = q.p[k]; col[3 * i + k] = colour(q)[k]; } });
    const pg = new THREE.BufferGeometry();
    pg.setAttribute("position", new THREE.BufferAttribute(pos, 3));
    pg.setAttribute("color", new THREE.BufferAttribute(col, 3));
    return new THREE.Points(pg, new THREE.PointsMaterial({ size, sizeAttenuation: false, vertexColors: true }));
  }
  function lines(seg, colour, opacity) {
    const lg = new THREE.BufferGeometry();
    lg.setAttribute("position", new THREE.BufferAttribute(new Float32Array(seg), 3));
    return new THREE.LineSegments(lg, new THREE.LineBasicMaterial({ color: colour, transparent: opacity < 1, opacity }));
  }
  function showHandles(h) {
    if (h) hg.h = h;
    if (hg.obj) { hg.obj.parent && hg.obj.parent.remove(hg.obj); hg.obj.traverse((o) => { if (o.geometry) o.geometry.dispose(); if (o.material) o.material.dispose(); }); hg.obj = null; }
    hg.handles = []; hg.curve = []; hg.ends = [];
    const head = hg.h;
    const gs = groupList();
    if (!head || !head.group || !shaping() || !gs.length || hg.sel >= gs.length || gs[hg.sel].hidden) { syncHint(); if (ctx) ctx.requestRender(); return; }
    const g = new THREE.Group();
    // The other groups, faint: their curves, and their strokes' end points, onto which an end point of this stroke can be dropped.
    const o = painter.hair_group_others(hg.sel);
    let s = 0;
    for (; s + 8 < o.length && !Number.isNaN(o[s]); s += 9) hg.ends.push({ p: lift(o, s, 0.0012), group: o[s + 6], end: o[s + 7], mirror: o[s + 8] === 1 });
    const faint = [];
    curveRows(o, s + 1, faint);
    g.add(lines(faint, 0xc8d2dc, 0.75));
    if (gs[hg.sel].kind === "stroke" && hg.flow < 0 && hg.ends.length) g.add(points(hg.ends, 7, () => [0.62, 0.67, 0.72]));
    // The group's flow lines, in the group's colour, each with an arrow head where the hairs grow toward. The one being shaped is drawn
    // again below with its handles.
    if (painter.hair_flow_lines) {
      const fl = [];
      curveRows(painter.hair_flow_lines(hg.sel), 0, fl);
      if (fl.length) g.add(lines(fl, 0x40e0ff, hg.flow >= 0 ? 0.45 : 0.8));
    }
    const d = painter.hair_group_view(hg.sel);
    for (s = 0; s + 8 < d.length && !Number.isNaN(d[s]); s += 9) hg.handles.push({ p: lift(d, s, 0.0012), k: d[s + 6], mirror: d[s + 7] === 1, type: d[s + 8] });
    const seg = [];
    curveRows(d, s + 1, seg, hg.curve);
    g.add(lines(seg, 0x40e0ff, 1));
    // The edges of a stroke's width, faint, so that the width shows while it is changed.
    const edges = [];
    curveRows(painter.hair_group_widths(hg.sel), 0, edges);
    if (edges.length) g.add(lines(edges, 0x40e0ff, 0.4));
    const middles = hg.handles.filter((q) => q.type === 2), ctrl = hg.handles.filter((q) => q.type !== 2);
    g.add(points(middles, 7, (q) => (q.k === hg.segment ? [1, 0.85, 0.1] : q.mirror ? [0.15, 0.42, 0.5] : [0.25, 0.88, 1])));
    g.add(points(ctrl, 11, (q) => (q.k === hg.point ? (q.type === 1 ? [1, 0.6, 0.1] : [1, 0.85, 0.1]) : q.type === 1 ? [1, 0.5, 0.35] : q.mirror ? [0.72, 0.6, 0.9] : [1, 1, 1])));
    g.traverse((x) => { x.frustumCulled = false; });
    head.group.add(g);
    hg.obj = g;
    syncHint();
    ctx.requestRender();
  }
  // A short hint over the head while a group is shaped, for what the pointer and the keys do now.
  function syncHint() {
    if (!hg.hint) {
      const stage = $("stage-canvas") && $("stage-canvas").parentElement;
      if (!stage) return;
      hg.hint = el("div", "hg-hint");
      stage.append(hg.hint);
    }
    const g = target();
    let t = "";
    if (shaping() && g && g.kind !== "stubble" && !g.hidden) {
      const n = g.points.length, end = g.kind === "stroke" && n >= 2 && (hg.point === 0 || hg.point === n - 1);
      if (!n) t = `Click on the skin to place the first point of "${g.name}". Each click adds the next point.`;
      else if (hg.drawing) t = `Click on the skin to add point ${n + 1} of "${g.name}". Press Enter or Escape when it is done.` + (n >= 3 ? (g.kind === "area" ? " Clicking its first point also ends it." : " Click its first point to close it into a filled area.") : "");
      else if (hg.point >= 0 && g.flowLine) t = `Point ${hg.point + 1} of ${n} is selected. Drag it to move it. K makes it a ${g.points[hg.point] && g.points[hg.point].corner ? "smooth point" : "corner"}, Delete removes it.`;
      else if (hg.point >= 0) t = `Point ${hg.point + 1} of ${n} is selected. Drag it to move it. S splits the ${g.kind === "area" ? "outline open" : "stroke"} here, K makes it a ${g.points[hg.point] && g.points[hg.point].corner ? "smooth point" : "corner"}, Delete removes it.` + (g.kind === "stroke" ? " [ and ] make its width narrower or wider." : "") + (end ? " Drop this end point on another stroke's end point (grey) to join the two, or on this stroke's other end to close it into a filled area. Shift-click the skin to add a point past this end." : "");
      else if (hg.segment >= 0) t = `Piece ${hg.segment + 1} is selected. Drag its small blue handle to bend it. S splits the ${g.kind === "area" ? "outline open" : "stroke"} here, by taking this piece out.`;
      else if (g.flowLine) t = "This flow line sets the direction of the group's hairs, from its first point toward the arrow head at its last. Drag a white point to move it. Drag a small cyan handle to bend the line. Click the line to add a point. Shift-click the skin to add a point at the end.";
      else t = "Drag a white point to move it, or click it to select it for S, K and Delete. Drag a small cyan handle to bend the curve between two points. Click the curve to add a point. Shift-click the skin to add a point at the end.";
      t += " Ctrl+Z or Cmd+Z undoes.";
    }
    hg.hint.textContent = t;
    hg.hint.hidden = !t;
  }
  // Where the pointer is in the view of our head: its place in pixels, and the ray under it in the head's own frame, in metres. In the
  // side by side view our head is the right half.
  function pointerView(e) {
    const canvas = $("stage-canvas"), r = canvas.getBoundingClientRect();
    let x = e.clientX - r.left, left = 0, w = r.width;
    if (Math.abs(ctx.camera.aspect - (r.width / 2) / r.height) < 0.01 && Math.abs(ctx.camera.aspect - r.width / r.height) > 0.01) { left = r.width / 2; w = r.width / 2; }
    const ndc = new THREE.Vector2(((x - left) / w) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
    const ray = new THREE.Raycaster();
    ray.setFromCamera(ndc, ctx.camera);
    const inv = new THREE.Matrix4().copy(hg.h.group.matrixWorld).invert();
    const o = ray.ray.origin.clone().applyMatrix4(inv), dir = ray.ray.direction.clone().transformDirection(inv);
    const screen = (p) => { const v = new THREE.Vector3(p[0], p[1], p[2]).applyMatrix4(hg.h.group.matrixWorld).project(ctx.camera); return [left + (v.x + 1) / 2 * w, (1 - v.y) / 2 * r.height]; };
    return { x, y: e.clientY - r.top, origin: [o.x, o.y, o.z], dir: [dir.x, dir.y, dir.z], screen };
  }
  function snapAt(v) { const s = painter.hair_grid_snap(v.origin, v.dir); return s.length === 2 ? s : null; }
  function nearestTo(v, list, max) {
    let best = null, bd = max;
    for (const q of list) { const s = v.screen(q.p), d = Math.hypot(s[0] - v.x, s[1] - v.y); if (d < bd) { bd = d; best = q; } }
    return best;
  }
  function curveDistance(v) {
    let near = Infinity;
    for (const [a, b] of hg.curve) {
      const sa = v.screen(a), sb = v.screen(b), ex = sb[0] - sa[0], ey = sb[1] - sa[1], l2 = ex * ex + ey * ey || 1;
      const t = Math.max(0, Math.min(1, ((v.x - sa[0]) * ex + (v.y - sa[1]) * ey) / l2));
      near = Math.min(near, Math.hypot(sa[0] + t * ex - v.x, sa[1] + t * ey - v.y));
    }
    return near;
  }
  function changed() { request(); showHandles(); saveGroups(); }
  // A point just added under the pointer is dragged on from the side of the face where it was clicked.
  function dragNew(k, v, at) {
    const mine = nearestTo(v, hg.handles.filter((q) => q.type !== 2 && q.k === k), 40);
    hg.drag = { kind: "point", k, mirror: !!(mine && mine.mirror), last: at.join(), moved: true };
  }
  function onDown(e) {
    if (!shaping() || !hg.h || !hg.h.group || e.button !== 0) return;
    const g = target();
    if (!g || g.kind === "stubble" || g.hidden) return;
    const v = pointerView(e);
    const hold = () => { e.preventDefault(); e.stopImmediatePropagation(); $("stage-canvas").setPointerCapture(e.pointerId); };
    const pt = nearestTo(v, hg.handles.filter((q) => q.type !== 2), 14);
    const mid = pt ? null : nearestTo(v, hg.handles.filter((q) => q.type === 2), 11);
    if (hg.drawing && pt && pt.k === 0 && !pt.mirror && g.points.length >= 3 && !g.flowLine) {
      e.preventDefault(); e.stopImmediatePropagation();
      hg.drawing = false; hg.point = -1;
      if (g.kind === "stroke") act(() => painter.hair_group_set(hg.sel, "area", 1)); else { showHandles(); syncGroups(); }
      return;
    }
    if (pt && e.altKey) { e.preventDefault(); e.stopImmediatePropagation(); hg.point = -1; act(() => painter.hair_group_remove_point(hg.sel, pt.k)); return; }
    if (pt) {
      hold();
      painter.hair_groups_checkpoint();
      hg.point = pt.k; hg.segment = -1;
      hg.drag = { kind: "point", k: pt.k, mirror: pt.mirror, last: "", moved: false };
      showHandles(); syncGroups();
      return;
    }
    if (mid) {
      hold();
      painter.hair_groups_checkpoint();
      hg.segment = mid.k; hg.point = -1;
      hg.drag = { kind: "bend", k: mid.k, mirror: mid.mirror, last: "", moved: false };
      showHandles(); syncGroups();
      return;
    }
    const empty = g.points.length === 0;
    if (empty || hg.drawing || e.shiftKey) {
      const at = snapAt(v);
      if (!at) return;
      hold();
      const atStart = !empty && !hg.drawing && hg.point === 0 && g.kind === "stroke";
      const k = act(() => painter.hair_group_extend(hg.sel, at[0], at[1], atStart));
      hg.point = k; hg.segment = -1;
      showHandles(); syncGroups();
      dragNew(k, v, at);
      return;
    }
    const at = curveDistance(v) < 9 ? snapAt(v) : null;
    if (!at) return;
    hold();
    const k = act(() => painter.hair_group_insert(hg.sel, at[0], at[1]));
    hg.point = k; hg.segment = -1;
    showHandles(); syncGroups();
    dragNew(k, v, at);
  }
  function onMove(e) {
    if (!hg.drag) return;
    e.stopImmediatePropagation();
    const at = snapAt(pointerView(e));
    if (!at || at.join() === hg.drag.last) return;
    hg.drag.last = at.join();
    hg.drag.moved = true;
    // A point or a piece picked on the mirror image is moved through its own side: the painter mirrors the column back.
    if (hg.drag.kind === "bend") painter.hair_group_bend(hg.sel, hg.drag.k, at[0], at[1], hg.drag.mirror);
    else painter.hair_group_move(hg.sel, hg.drag.k, at[0], at[1], hg.drag.mirror);
    changed();
  }
  // An end point of a stroke dropped on another stroke's end point joins the two strokes, and dropped on its own stroke's other end it
  // closes the stroke into a filled area.
  function onUp(e) {
    const d = hg.drag;
    if (!d) return;
    e.stopImmediatePropagation();
    hg.drag = null;
    const g = target();
    const last = g ? g.points.length - 1 : -1;
    if (d.kind === "point" && d.moved && g && g.kind === "stroke" && !g.flowLine && last >= 1 && (d.k === 0 || d.k === last)) {
      const v = pointerView(e);
      const own = last >= 2 && nearestTo(v, hg.handles.filter((q) => q.type !== 2 && q.mirror === d.mirror && q.k === (d.k === 0 ? last : 0)), 14);
      const other = own ? null : nearestTo(v, hg.ends.filter((q) => q.mirror === d.mirror), 14);
      if (own) { painter.hair_group_join(hg.sel, d.k === 0, hg.sel, d.k === 0); hg.point = -1; }
      else if (other) { const n = painter.hair_group_join(hg.sel, d.k === 0, other.group, other.end === 1); if (n >= 0) { hg.sel = n; hg.point = -1; } }
      if (own || other) changed();
    }
    syncGroups();
  }
  function splitHere() {
    if (hg.point >= 0) { const k = hg.point; hg.point = -1; act(() => painter.hair_group_split(hg.sel, k, false)); }
    else if (hg.segment >= 0) { const k = hg.segment; hg.segment = -1; act(() => painter.hair_group_split(hg.sel, k, true)); }
  }
  function toggleCorner() {
    const g = target();
    if (!g || hg.point < 0 || !g.points[hg.point]) return;
    act(() => painter.hair_group_corner(hg.sel, hg.point, !g.points[hg.point].corner));
  }
  // Changes the width of the selected point, or of every point when none is selected, by `factor`.
  function widen(factor) {
    const g = target();
    if (!g || g.kind !== "stroke" || g.flowLine || !g.points.length) return;
    const p = g.points[hg.point];
    const now = p ? p.width_mm : Math.max(...g.points.map((q) => q.width_mm));
    act(() => painter.hair_group_width(hg.sel, p ? hg.point : -1, now * factor));
  }
  function onKey(e) {
    if (active !== "facial_hair") return;
    const t = e.target;
    if (t && t.matches && t.matches("input, select, textarea")) return;
    const k = e.key.toLowerCase();
    if ((e.metaKey || e.ctrlKey) && (k === "z" || k === "y")) { e.preventDefault(); if (k === "y" || e.shiftKey) redo(); else undo(); return; }
    if (e.metaKey || e.ctrlKey || e.altKey || !hg.edit) return;
    const g = target();
    if (!g || g.kind === "stubble") return;
    if ((k === "delete" || k === "backspace") && hg.point >= 0) { e.preventDefault(); const p = hg.point; hg.point = -1; act(() => painter.hair_group_remove_point(hg.sel, p)); }
    else if (k === "s" && !g.flowLine) splitHere();
    else if (k === "k") toggleCorner();
    else if (k === "[" || k === "]") { e.preventDefault(); widen(k === "]" ? 1.1 : 1 / 1.1); }
    else if (k === "escape" || (k === "enter" && hg.drawing)) { hg.drawing = false; hg.point = -1; hg.segment = -1; showHandles(); syncGroups(); }
  }
  function groupControls() {
    hg.spec = JSON.parse(painter.hair_group_spec_json());
    const canvas = $("stage-canvas");
    canvas.addEventListener("pointerdown", onDown, true);
    canvas.addEventListener("pointermove", onMove, true);
    canvas.addEventListener("pointerup", onUp, true);
    window.addEventListener("keydown", onKey);
    const style = el("style");
    style.textContent = ".hg-hint { position: absolute; right: 16px; top: 150px; max-width: 260px; padding: 8px 10px; background: rgba(6, 11, 16, 0.86); border: 1px solid var(--line-hi); color: var(--ink-soft); font: 12px/1.45 var(--body, sans-serif); pointer-events: none; z-index: 3; }" +
      " .hg-row { display: flex; align-items: center; gap: 8px; margin: 4px 0; } .hg-row .hg-name { flex: 1; min-width: 0; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .hg-row .seg { flex: none; } .hg-row .seg button { padding: 5px 7px; letter-spacing: 0; }" +
      " .hg-name { background: transparent; border: 1px solid var(--line); color: var(--ink-soft); padding: 6px 9px; font: 500 12.5px var(--title); cursor: pointer; }" +
      " .hg-name[aria-pressed=\"true\"] { border-color: var(--accent); color: var(--accent); background: var(--accent-dim); } .hg-hidden .hg-name { opacity: 0.45; text-decoration: line-through; }" +
      " .hg-kind { color: var(--muted); font-size: 11px; margin-left: 6px; } .hg-rename { width: 100%; box-sizing: border-box; background: rgba(6, 11, 16, 0.85); border: 1px solid var(--line-hi); color: var(--ink); padding: 6px 8px; font: 13px var(--body, sans-serif); }" +
      " .hg-box .sl-buttons { flex-wrap: wrap; } .hg-tools { display: flex; flex-wrap: wrap; gap: 6px; margin: 6px 0; }";
    document.head.append(style);
    const grp = el("section", "sl-group hg-box");
    grp.append(el("h3", "sl-group-title", "Hair groups"));
    grp.append(el("p", "sl-hint", "A head's facial hair is its list of hair groups, each with its own sliders. Where two groups cover the same place, the group higher in the list wins."));
    hg.box = el("div");
    grp.append(hg.box);
    syncGroups();
    return grp;
  }
  function button(label, note, onclick, pressed) {
    const b = el("button", null, label);
    b.type = "button";
    b.dataset.note = note;
    if (pressed !== undefined) b.setAttribute("aria-pressed", pressed ? "true" : "false");
    b.onclick = onclick;
    return b;
  }
  function seg(...buttons) { const s = el("div", "seg sl-buttons"); s.append(...buttons); return s; }
  const KIND_NAMES = { stroke: "stroke", area: "filled area", stubble: "stubble" };
  function syncGroups() {
    const box = hg.box;
    if (!box) return;
    box.textContent = "";
    const gs = groupList();
    if (hg.sel >= gs.length) hg.sel = Math.max(0, gs.length - 1);
    const select = (i) => { if (i !== hg.sel) { hg.drawing = false; setFlow(-1); } hg.sel = i; hg.point = -1; hg.segment = -1; syncGroups(); showHandles(); };
    // Adding: a preset, or an empty stroke or area whose points are placed by clicking on the head.
    const add = seg(...hg.spec.presets.map(([id, label]) => button(label, `Adds the preset "${label}" at the top of the list of hair groups.`, () => { if (act(() => painter.hair_group_add(id))) select(0); })));
    const r1 = el("div", "sl-row");
    r1.append(el("span", "caption", "Add a preset"), add);
    const empty = seg(
      button("New stroke", "Adds an empty stroke at the top of the list. Click on the head to place its points, one after the other, and press Enter when it is done.", () => { hg.edit = true; act(() => painter.hair_group_add_empty(false)); select(0); hg.drawing = true; syncGroups(); showHandles(); }),
      button("New filled area", "Adds an empty filled area at the top of the list. Click on the head to place the points of its outline, one after the other. It fills once it has 3 points. Press Enter when it is done.", () => { hg.edit = true; act(() => painter.hair_group_add_empty(true)); select(0); hg.drawing = true; syncGroups(); showHandles(); }),
      button("New pull", "Adds an empty pull at the top of the list. A pull grows no hair. It turns the hairs under it toward its own direction, like a comb, from its first point toward its last. A pull with only one point turns the hairs toward that point. Click on the head to place its points, and press Enter when it is done.", () => { hg.edit = true; act(() => painter.hair_group_add_pull()); select(0); hg.drawing = true; syncGroups(); showHandles(); }),
    );
    const r2 = el("div", "sl-row");
    r2.append(el("span", "caption", "Add your own"), empty);
    const hist = painter.hair_groups_history();
    const u = button("Undo", "Takes back the last change to the hair groups. Ctrl+Z or Cmd+Z does the same while this tab is open.", undo);
    const re = button("Redo", "Puts back the last change that was undone. Ctrl+Shift+Z or Cmd+Shift+Z does the same.", redo);
    u.disabled = !hist[0]; re.disabled = !hist[1];
    const r3 = el("div", "sl-row");
    r3.append(el("span", "caption", "Undo and save"), seg(u, re, button("Download", "Saves this head's hair groups as a file named after the head. Put it in live/groups/ to make it the head's groups in every browser.", () => downloadGroups())));
    box.append(r1, r2, r3);
    if (!gs.length) { box.append(el("p", "sl-hint", "This head has no hair group yet, so it has no facial hair.")); return; }
    // The list, the top group first.
    box.append(el("p", "caption", "Groups, the top one first"));
    gs.forEach((g, i) => {
      const row = el("div", "hg-row" + (g.hidden ? " hg-hidden" : ""));
      const name = el("button", "hg-name");
      name.type = "button";
      name.append(document.createTextNode(g.name), el("span", "hg-kind", g.pull ? "pull" : KIND_NAMES[g.kind] || g.kind));
      name.setAttribute("aria-pressed", i === hg.sel ? "true" : "false");
      name.dataset.note = "Selects this group, to show its sliders and its handles on the head.";
      name.onclick = () => select(i);
      const up = button("▲", "Moves this group one place up the list. A group higher in the list wins where two groups cover the same place.", () => { act(() => { hg.sel = painter.hair_group_shift(i, true); }); });
      const down = button("▼", "Moves this group one place down the list.", () => { act(() => { hg.sel = painter.hair_group_shift(i, false); }); });
      up.disabled = i === 0; down.disabled = i === gs.length - 1;
      const eye = button(g.hidden ? "Show" : "Hide", "Hides this group, so that it paints nothing, or shows it again. A hidden group keeps its shape and its sliders.", () => act(() => painter.hair_group_set(i, "hidden", g.hidden ? 0 : 1)));
      const moves = el("div", "seg");
      moves.append(up, down, eye);
      row.append(name, moves);
      box.append(row);
    });
    const g = gs[hg.sel];
    // The selected group: its name, its shape tools and its sliders.
    const title = el("h3", "sl-group-title", g.name);
    title.style.marginTop = "14px";
    box.append(title);
    const rename = el("input", "hg-rename");
    rename.type = "text"; rename.value = g.name; rename.setAttribute("aria-label", "The group's name");
    rename.addEventListener("change", () => { act(() => painter.hair_group_rename(hg.sel, rename.value)); });
    box.append(rename);
    const own = el("div", "hg-tools");
    own.append(seg(
      button("Copy", "Copies this group just above itself, with its shape and its sliders.", () => { const i = hg.sel; act(() => painter.hair_group_duplicate(i)); select(i); }),
      button("Delete group", "Removes this group. Undo brings it back.", () => { const i = hg.sel; act(() => painter.hair_group_remove(i)); select(Math.max(0, Math.min(i, gs.length - 2))); }),
    ));
    box.append(own);
    if (g.kind !== "stubble") {
      const n = g.points.length;
      box.append(el("p", "sl-hint", `A ${g.pull ? "pull" : KIND_NAMES[g.kind]} with ${n} control point${n === 1 ? "" : "s"}${g.mirror ? ", mirrored to the other side of the face" : ""}.`));
      // The flow lines: curves inside the group that set the direction of its hairs. Each is shaped with the same handles as a stroke.
      if (!g.pull) {
        const lines = g.flow || [];
        box.append(el("p", "sl-hint", lines.length
          ? `The group has ${lines.length} flow line${lines.length === 1 ? "" : "s"}. Every hair of the group takes its direction from them, blended by distance, so the nearest line wins near it. They show as cyan lines with an arrow head.`
          : "The group has no flow line, so its hairs grow the default way. Add a flow line to set their direction."));
        const fr = el("div", "hg-tools");
        fr.append(seg(
          button("Own points", "Shapes the group's own points with the handles.", () => { setFlow(-1); hg.drawing = false; showHandles(); syncGroups(); }, hg.flow < 0),
          ...lines.map((_, n) => button(`Flow line ${n + 1}`, "Shapes this flow line with the handles: drag its points, bend it, click it to add a point, and Shift-click the skin to add a point at its end.", () => { hg.edit = true; setFlow(n); hg.drawing = false; showHandles(); syncGroups(); }, hg.flow === n)),
        ));
        fr.append(seg(
          button("New flow line", "Adds a flow line to this group. Click on the skin to place its points in the direction the hairs should grow, and press Enter when it is done.", () => { hg.edit = true; const n = act(() => painter.hair_flow_add(hg.sel)); if (n >= 0) { setFlow(n); hg.drawing = true; } showHandles(); syncGroups(); }),
          ...(hg.flow >= 0 ? [button("Remove flow line", "Removes the flow line being shaped. Undo brings it back.", () => { const n = hg.flow; act(() => painter.hair_flow_remove(hg.sel, n)); setFlow(-1); hg.drawing = false; showHandles(); syncGroups(); })] : []),
        ));
        box.append(fr);
      }
      const tools = el("div", "hg-tools");
      tools.append(seg(
        button("Show handles", "Shows the group's control points as white handles, the middle of each piece of curve as a small cyan handle, and its curve or outline as a blue line. The other groups show as faint grey lines.", () => { hg.edit = !hg.edit; showHandles(); syncGroups(); }, hg.edit),
        button("Mirror", "Mirrors the group to the other side of the head. The mirror image's handles are lilac. A mirrored handle moves its own point on the other side.", () => act(() => painter.hair_group_set(hg.sel, "mirror", g.mirror ? 0 : 1)), g.mirror),
      ));
      if (!g.pull && hg.flow < 0) tools.append(seg(
        g.kind === "area"
          ? button("Open into a stroke", "Opens the outline into a stroke, by taking out the piece from the last point back to the first. To open it somewhere else, select a point or a piece and press S.", () => act(() => painter.hair_group_set(hg.sel, "area", 0)))
          : button("Close into a filled area", "Closes the stroke into a filled area, the inside of an outline through its points. It needs at least 3 points. You can also drop one end point on the other.", () => act(() => painter.hair_group_set(hg.sel, "area", 1))),
      ));
      const sel = [];
      const t = target();
      if (t && t.flowLine) {
        if (hg.point >= 0 && t.points[hg.point]) {
          sel.push(button(t.points[hg.point].corner ? "Make smooth (K)" : "Make a corner (K)", "A smooth point lets the line pass through it without a corner. A corner lets the line turn sharply there.", toggleCorner));
          sel.push(button("Remove point (Delete)", "Removes this point of the flow line. A flow line keeps at least one point.", () => { const p = hg.point; hg.point = -1; act(() => painter.hair_group_remove_point(hg.sel, p)); }));
        } else if (hg.segment >= 0) {
          sel.push(button("Straighten", "Takes the bend out of this piece of the flow line.", () => act(() => painter.hair_group_straighten(hg.sel, hg.segment))));
        }
      } else if (hg.point >= 0 && g.points[hg.point]) {
        sel.push(button(g.kind === "area" ? "Open here (S)" : "Split here (S)", g.kind === "area" ? "Opens the outline into a stroke that starts and ends at this point." : "Splits the stroke into two strokes at this point. The second one goes just under this one in the list.", splitHere));
        sel.push(button(g.points[hg.point].corner ? "Make smooth (K)" : "Make a corner (K)", "A smooth point lets the curve pass through it without a corner. A corner lets the curve turn sharply there. Corner points are drawn orange.", toggleCorner));
        sel.push(button("Remove point (Delete)", "Removes this control point. A stroke keeps at least one point, and a filled area three.", () => { const p = hg.point; hg.point = -1; act(() => painter.hair_group_remove_point(hg.sel, p)); }));
      } else if (hg.segment >= 0) {
        sel.push(button(g.kind === "area" ? "Open here (S)" : "Split here (S)", g.kind === "area" ? "Opens the outline into a stroke by taking out this piece." : "Splits the stroke into two strokes by taking out this piece.", splitHere));
        sel.push(button("Straighten", "Takes the bend out of this piece of the curve.", () => act(() => painter.hair_group_straighten(hg.sel, hg.segment))));
      }
      if (hg.drawing) sel.unshift(button("Done (Enter)", "Ends placing points by clicking. The points can still be dragged, and Shift-click adds more at the end.", () => { hg.drawing = false; showHandles(); syncGroups(); }));
      if (sel.length) tools.append(seg(...sel));
      box.append(tools);
      // The width of a stroke: of the selected point, or of every point at once when no point is selected.
      if (g.kind === "stroke" && n && hg.flow < 0) {
        const p = g.points[hg.point];
        const widest = Math.max(...g.points.map((q) => q.width_mm));
        const c = { label: p ? `Width of point ${hg.point + 1}` : "Width of every point", min: 1, max: 120, step: 0.5, unit: "mm" };
        const r = el("div", "sl");
        const top = el("div", "sl-top");
        const val = el("output", "sl-val", fmt(c, p ? p.width_mm : widest));
        top.append(el("label", "sl-name", c.label), val);
        const input = el("input");
        input.type = "range"; input.min = c.min; input.max = c.max; input.step = c.step; input.value = p ? p.width_mm : widest;
        input.addEventListener("input", () => {
          if (!hg.sliding) { painter.hair_groups_checkpoint(); hg.sliding = true; }
          painter.hair_group_width(hg.sel, p ? hg.point : -1, Number(input.value)); val.textContent = fmt(c, Number(input.value)); request(); showHandles(); saveGroups();
        });
        input.addEventListener("change", () => { hg.sliding = false; syncGroups(); });
        const help = g.pull
          ? "How wide the pull reaches, in millimetres, across its curve or round its single point. The hairs turn less toward the edge of the width. The faint blue lines on the head show the width."
          : "How wide the stroke is, in millimetres, across its curve. The width changes smoothly from one point to the next. The faint blue lines on the head show the width.";
        r.append(top, el("p", "sl-hint", help + (p ? " With no point selected, this slider changes every point's width together and keeps the differences between them." : " Select a point to change only its width.") + " The keys [ and ] make it 10% narrower or wider."), input);
        box.append(r);
      }
    } else {
      box.append(el("p", "sl-hint", "A stubble has no control points. Its sliders set the density of each region of the beard and where its edges lie."));
    }
    // The sliders, by part, each under its part's heading even when the list adds to a part after another one. A stubble has the
    // regions' and the edges' sliders too.
    const parts = new Map();
    hg.spec.sliders.forEach((c, k) => {
      if (c.stubble_only && g.kind !== "stubble") return;
      if (c.area_only && g.kind !== "area") return;
      if (!!c.pull_only !== !!g.pull) return;
      if (!parts.has(c.part)) { const part = el("div"); const h = el("p", "caption", c.part); h.style.marginTop = "10px"; part.append(h); box.append(part); parts.set(c.part, part); }
      const r = el("div", "sl");
      const top = el("div", "sl-top");
      const val = el("output", "sl-val", fmt(c, g.values[k]));
      top.append(el("label", "sl-name", c.label), val);
      const input = el("input");
      input.type = "range"; input.min = c.min; input.max = c.max; input.step = c.step; input.value = g.values[k];
      input.addEventListener("input", () => {
        if (!hg.sliding) { painter.hair_groups_checkpoint(); hg.sliding = true; }
        painter.hair_group_set(hg.sel, c.id, Number(input.value)); val.textContent = fmt(c, Number(input.value)); request(); saveGroups();
      });
      input.addEventListener("change", () => { hg.sliding = false; syncGroups(); });
      input.addEventListener("dblclick", () => act(() => painter.hair_group_set(hg.sel, c.id, c.default)));
      input.title = "Double-click to set it back to its default.";
      r.append(top, el("p", "sl-hint", c.help), input);
      parts.get(c.part).append(r);
    });
  }

  function viewerControls() {
    hookScenes();
    const box = el("div");
    const parts = new Map();
    for (const v of VIEW) {
      if (!parts.has(v.part)) {
        const grp = el("section", "sl-group");
        grp.append(el("h3", "sl-group-title", v.part));
        if (v.part === "Light") {
          grp.append(el("p", "sl-hint", "These controls live in the viewer only. They turn the main light of both heads, ours and KOTOR's."));
          const r = el("div", "sl-row");
          const seg = el("div", "seg sl-buttons");
          const b = el("button", null, "Circle the light");
          b.type = "button";
          b.id = "pl-circle";
          b.setAttribute("aria-pressed", "false");
          b.dataset.note = "Turns the main light slowly round the head, a full circle in 12 seconds, until it is pressed again.";
          b.onclick = () => circle(!circling);
          const back = el("button", null, "Light back");
          back.type = "button";
          back.dataset.note = "Puts the main light back where the lighting choice puts it.";
          back.onclick = () => { circle(false); setView("turn", 0); setView("raise", 0); };
          seg.append(b, back);
          r.append(seg);
          grp.append(r);
        } else if (v.part === "Skin light") {
          grp.append(el("p", "sl-hint", "These controls live in the viewer only, as they would in kjk's shader, and work whether the light from maps is on or off. They change how our head's material takes the light, not the paint."));
          const r = el("div", "sl-row");
          const seg = el("div", "seg sl-buttons");
          const b = el("button", null, "Skin light on");
          b.type = "button";
          b.id = "pl-skin-light";
          b.setAttribute("aria-pressed", "true");
          b.dataset.note = "Turns the skin light of our head on or off: the light round the shadow edge and the share of the viewer's grey sheen.";
          b.onclick = () => skinLight(!skinOn);
          seg.append(b);
          r.append(seg);
          grp.append(r);
        } else if (v.part === "Highlight in the viewer") {
          grp.append(el("p", "sl-hint", "These controls live in the viewer's material, as they would in kjk's shader. The painter's own sliders follow under Gloss, Relief and Pores."));
        }
        box.append(grp);
        parts.set(v.part, grp);
      }
      const row = el("div", "sl");
      const top = el("div", "sl-top");
      const name = el("label", "sl-name", v.label);
      name.htmlFor = "pl-view-" + v.id;
      const val = el("output", "sl-val", fmt(v, view[v.id]));
      val.id = "pl-view-val-" + v.id;
      top.append(name, val);
      const input = el("input");
      input.type = "range";
      input.id = "pl-view-" + v.id;
      input.min = v.min; input.max = v.max; input.step = v.step; input.value = view[v.id];
      input.title = "Double-click to go back to the start value.";
      input.addEventListener("input", () => { if (v.id === "turn") circle(false); setView(v.id, Number(input.value)); });
      input.addEventListener("dblclick", () => setView(v.id, v.value));
      const ends = el("div", "sl-ends");
      ends.append(el("span", null, fmt(v, v.min)), el("span"), el("span", null, fmt(v, v.max)));
      row.append(top, el("p", "sl-hint", v.help), input, ends);
      parts.get(v.part).append(row);
    }
    return box;
  }

  // --- a small interface for scripts that test the page in a headless browser ---
  const test = {
    ids: () => list.map((c) => c.id),
    control: (id) => list.find((c) => c.id === id),
    value: (id) => values[list.findIndex((c) => c.id === id)],
    fitted: (id) => fitted[list.findIndex((c) => c.id === id)],
    groups: () => groups.map((g) => g.id),
    hairGrid: (mode) => setGridMode(mode),
    // The selected hair group's handles in the canvas's pixels, for scripts that drag them.
    // The selected hair group's handles in the canvas's pixels, with their kind (0 a smooth point, 1 a corner, 2 the middle of a piece),
    // and the other strokes' end points.
    hairHandles: () => {
      const canvas = $("stage-canvas"), r = canvas.getBoundingClientRect();
      const at = (p) => { const v = new THREE.Vector3(...p).applyMatrix4(hg.h.group.matrixWorld).project(ctx.camera); return { x: r.left + (v.x + 1) / 2 * r.width, y: r.top + (1 - v.y) / 2 * r.height }; };
      return { handles: hg.handles.map((q) => ({ k: q.k, mirror: q.mirror, type: q.type, ...at(q.p) })), ends: hg.ends.map((q) => ({ group: q.group, end: q.end, mirror: q.mirror, ...at(q.p) })) };
    },
    // Handles show only while the facial hair tab is open, so turning them on opens it.
    hairShape: (on) => { hg.edit = on; if (on && active !== "facial_hair") show("facial_hair"); syncGroups(); showHandles(); },
    hairGroups: () => groupList(),
    hairGroupsSet: (json) => { const ok = painter.set_hair_groups_json(json); hg.sel = 0; hg.point = -1; hg.segment = -1; request(); syncGroups(); showHandles(); return ok; },
    hairSelect: (i, point = -1, segment = -1) => { hg.sel = i; hg.point = point; hg.segment = segment; syncGroups(); showHandles(); },
    // The middle of the selected group's handles, in the world's frame, for aiming the camera at it.
    hairCentre: () => { if (!hg.handles.length) return null; const c = new THREE.Vector3(); for (const q of hg.handles) c.add(new THREE.Vector3(...q.p).applyMatrix4(hg.h.group.matrixWorld)); c.multiplyScalar(1 / hg.handles.length); return [c.x, c.y, c.z]; },
    hairState: () => ({ sel: hg.sel, point: hg.point, segment: hg.segment, hint: hg.hint ? hg.hint.textContent : "" }),
    hairGroupsJson: () => painter.hair_groups_json(),
    // The painted texture as a PNG data address, for scripts that look at the texels.
    texturePng: () => base.canvas.toDataURL("image/png"),
    set(id, v) {
      const i = list.findIndex((c) => c.id === id);
      if (i < 0) throw new Error(`no control ${id}`);
      const input = $("pl-ctl-" + i);
      if (input) { input.value = v; input.dispatchEvent(new Event("input")); } else { values[i] = v; request(); }
      return values[i];
    },
    switchLayer(gi, on) { switches[gi] = on ? 1 : 0; syncSwitches(); request(); },
    view: (k, v) => { if (v !== undefined) setView(k, v); return view[k]; },
    debug: (v) => { U.lmDebug.value = v; ctx.requestRender(); },
    skinLight: (on) => { skinLight(on); return skinOn; },
    // Puts the camera `d` metres from the point `at` (three numbers), in the direction `from`, looking at the point.
    aim(at, from, d) {
      if (!ctx.camera || !ctx.target) return false;
      const f = new THREE.Vector3(...from).normalize();
      ctx.target.set(...at);
      ctx.camera.position.copy(ctx.target).addScaledVector(f, d);
      ctx.camera.lookAt(ctx.target);
      ctx.requestRender();
      return true;
    },
    get maps() { return { on: mapsWanted(switches), tex: !!lm.tex, tile: !!lm.tile, ms: lm.ms, group: lm.group }; },
    show,
    shapeSettled: () => shapeSettled(ctx.ours),
    // Resolves once every requested paint is on the head.
    async settle() {
      if (readyP) await readyP;
      for (;;) {
        if (!running && doneSeq === seq) { await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))); if (!running && doneSeq === seq) return { ...last, seq, failed }; }
        await new Promise((r) => setTimeout(r, 50));
      }
    },
    get state() { return { seq, doneSeq, running, headKey, eyeKey, failed, from: last.from, base: !!base.tex, eye: !!eye.tex }; },
  };

  return { start, texture, withHair, eyes, lidMap, material, shapeMoved, skinButton, skinUI, shapeNote, failure, test };
})();
