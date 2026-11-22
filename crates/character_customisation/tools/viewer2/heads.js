// Readable names, groups and sexes for the 57 test heads. The page falls back to the file name for a head that is not listed here.
window.HEAD_INFO = (function () {
  const info = {};
  const add = (name, label, group, sex) => { info[name] = { label, group, sex }; };
  const sexWord = { f: "female", m: "male" };
  for (const n of ["pfha02", "pfha05", "pfha06", "pfhb04", "pfhc02", "pmba01", "pmha01", "pmha02", "pmha04", "pmha05", "pmha06", "pmhc01", "pmhc03", "pmhc04"]) {
    const sex = n[1];
    const letter = n === "pmba01" ? "B" : n[3].toUpperCase();
    add(n, `${sex === "f" ? "Female" : "Male"} ${letter}${+n.slice(4)}`, "player", sex);
  }
  for (const [n, people, sex, k] of [["comm_a_f", "A", "f", ""], ["comm_a_f2", "A", "f", " 2"], ["comm_a_m", "A", "m", ""], ["comm_b_f", "B", "f", ""], ["comm_b_m", "B", "m", ""], ["comm_w_f", "W", "f", ""], ["comm_w_m", "W", "m", ""]])
    add(n, `Commoner ${people}${k}, ${sexWord[sex]}`, "common", sex);
  for (const n of ["old_a_f", "old_a_m", "old_b_f", "old_b_m", "old_w_f", "old_w_m"])
    add(n, `Elder ${n[4].toUpperCase()}, ${sexWord[n[6]]}`, "common", n[6]);
  for (const [n, label, sex] of [
    ["n_admoffh", "Admiralty officer", "m"], ["n_daraalah", "Daraala", "f"], ["n_dodonnah", "Admiral Dodonna", "f"], ["n_dorakh", "Master Dorak", "m"],
    ["n_hussef", "Hussef", "m"], ["n_jedimalek", "Malak as a Jedi", "m"], ["n_mastkavarh", "Master Kavar", "m"], ["n_terenaadareh", "Terena Adare", "f"],
    ["n_traskh", "Trask Ulgo", "m"], ["n_zezkaiellh", "Zez-Kai Ell", "m"], ["n_zherronh", "Zherron", "m"],
    ["p_baodurh", "Bao-Dur", "m"], ["p_bastilah", "Bastila", "f"], ["p_carthh", "Carth", "m"], ["p_handmaidenbb", "The Handmaiden", "f"],
    ["p_joleeh", "Jolee", "m"], ["p_kreiah", "Kreia", "f"], ["sith_app", "Sith apprentice", "m"]]) add(n, label, "character", sex);
  for (const [n, label, sex] of [
    ["mod_exile_canon", "The Exile", "f"], ["mod_hp_pfha01", "Female A1, high detail", "f"], ["mod_hp_pfhb02", "Female B2, high detail", "f"],
    ["mod_improved_bastila", "Bastila, improved", "f"], ["mod_jaesa", "Jaesa", "f"], ["mod_keanu", "Keanu", "m"], ["mod_kenobi", "Kenobi", "m"],
    ["mod_kyle_katarn", "Kyle Katarn", "m"], ["mod_layne", "Layne", "m"], ["mod_meetra", "Meetra", "f"], ["mod_narve_canderous", "Canderous", "m"],
    ["mod_tcs_bastila", "Bastila, TCS", "f"]]) add(n, label, "mod", sex);
  return info;
})();
