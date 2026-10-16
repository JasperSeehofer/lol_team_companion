// Surface loader: ?t=t1|t2|t3 picks art/<t>.json, which names the SVG and the
// overlay anchors (team label, marks, tick labels, crops) in its 100x100
// units. Marks are fetched from public/marks/ at load, so a fix there shows
// up in every still on the next capture. Sets <html data-ready> when done.
(async () => {
  const t = new URLSearchParams(location.search).get("t") || "t1";
  document.documentElement.dataset.t = t;
  const MARKS = new URL("../../../../../public/marks/", location.href);
  const cfg = await (await fetch(`art/${t}.json`)).json();
  if (cfg.css) document.head.append(Object.assign(document.createElement("link"), { rel: "stylesheet", href: `art/${cfg.css}` }));
  const svgText = await (await fetch(`art/${cfg.svg}`)).text();

  const el = (tag, cls, text) => { const n = document.createElement(tag); if (cls) n.className = cls; if (text) n.textContent = text; return n; };
  const place = (n, box, vb, x, y) => {
    n.style.left = `${((x - vb[0]) / vb[2]) * 100}%`;
    n.style.top = `${((y - vb[1]) / vb[3]) * 100}%`;
    box.append(n);
  };

  for (const box of document.querySelectorAll("[data-art]")) {
    const role = box.dataset.art;
    box.insertAdjacentHTML("afterbegin", svgText);
    const svg = box.querySelector("svg");
    if (cfg.crops?.[role]) { svg.setAttribute("viewBox", cfg.crops[role]); svg.setAttribute("preserveAspectRatio", "xMidYMid slice"); }
    const vb = svg.getAttribute("viewBox").split(/\s+/).map(Number);
    if (role === "hero") {
      if (box.dataset.label && cfg.label) place(el("span", "art-label eyebrow", box.dataset.label), box, vb, cfg.label.x, cfg.label.y);
      for (const m of cfg.marks || []) {
        const n = el("span", "art-mark");
        const mk = el("span", "mark");
        mk.dataset.mark = m.name;
        n.append(mk, el("span", "word", m.word));
        place(n, box, vb, m.x, m.y);
      }
      if (cfg.ticks) {
        [...cfg.ticks.cols].forEach((c, i) => place(el("span", "art-tick", c), box, vb, 2 + 96 * (i + 0.5) / cfg.ticks.cols.length, -2.6));
        [...cfg.ticks.rows].forEach((r, i) => place(el("span", "art-tick", r), box, vb, -2.6, 2 + 96 * (i + 0.5) / cfg.ticks.rows.length));
      }
    }
    if (role === "share" && box.dataset.label && cfg.shareLabel) place(el("span", "art-label halo eyebrow", box.dataset.label), box, vb, cfg.shareLabel.x, cfg.shareLabel.y);
  }

  const cache = {};
  await Promise.all([...document.querySelectorAll("[data-mark]")].map(async (slot) => {
    const name = slot.dataset.mark;
    cache[name] ??= fetch(new URL(`${name}.svg`, MARKS)).then((r) => r.text());
    slot.innerHTML = (await cache[name]).replace(/<title>[^<]*<\/title>/, "");
    const s = slot.querySelector("svg");
    s.setAttribute("aria-hidden", "true");
    s.setAttribute("focusable", "false");
  }));
  document.documentElement.dataset.ready = "1";
})();
