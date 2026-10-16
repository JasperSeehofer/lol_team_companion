#!/usr/bin/env node
// capture.mjs: screenshot a gallery page with headless Chromium.
// Uses playwright from e2e/node_modules (the repo's existing devDependency;
// run `cd e2e && npm ci` first). No new dependencies.
//
//   node capture.mjs <page.html|http-url> <out.png> [--width 390|1280] [--height N]
//        [--full] [--reduced-motion] [--og] [--base http://127.0.0.1:PORT]
//        [--selector CSS]  (shoot one element, scrolled into view; padding 16px)
//        [--frames 6 --duration 220 [--trigger "<js expr>"] [--settle MS]]
//
// A local path is turned into a URL under --base (default http://127.0.0.1:8151,
// a server started at the repo root) so fonts load same-origin; pass an
// http(s) URL to skip that. --og = 1200x630. Frames mode: run --trigger,
// pause every document.getAnimations(), seek them to N evenly spaced times
// across --duration ms (0 .. duration), shoot each to <out>-fN.png, then
// compose <out>-strip.png (frames side by side, labelled with the time).
import { createRequire } from "node:module";
import { existsSync, readFileSync, writeFileSync, unlinkSync } from "node:fs";
import { dirname, join, resolve, relative, basename } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { homedir, tmpdir } from "node:os";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, "..", "..", "..", "..", "..");
const require = createRequire(join(REPO, "e2e", "package.json"));
let chromium;
try { ({ chromium } = require("playwright")); }
catch { ({ chromium } = require("playwright-core")); }

const argv = process.argv.slice(2);
const flag = (n) => argv.includes(`--${n}`);
const opt = (n, d) => { const i = argv.indexOf(`--${n}`); return i < 0 ? d : argv[i + 1]; };
const valued = new Set(["width", "height", "frames", "duration", "trigger", "base", "settle", "selector"]);
const pos = argv.filter((a, i) => !a.startsWith("--") && !(i > 0 && argv[i - 1].startsWith("--") && valued.has(argv[i - 1].slice(2))));
if (pos.length < 2) {
  console.error("usage: node capture.mjs <page.html|url> <out.png> [--width 390|1280] [--height N] [--full] [--reduced-motion] [--og] [--frames N --duration MS --trigger EXPR]");
  process.exit(1);
}
const [target, outArg] = pos;
const out = resolve(outArg);
const og = flag("og");
const width = og ? 1200 : Number(opt("width", 1280));
const height = og ? 630 : Number(opt("height", width <= 480 ? 844 : 800));
const frames = Number(opt("frames", 0));
const duration = Number(opt("duration", 220));
const trigger = opt("trigger", null);
const settle = Number(opt("settle", 300));
const selector = opt("selector", null);
const base = opt("base", process.env.GALLERY_BASE || "http://127.0.0.1:8151");

const url = /^(https?|file):/.test(target) ? target
  : `${base.replace(/\/$/, "")}/${relative(REPO, resolve(target)).split("\\").join("/")}`;

const candidates = [
  process.env.GALLERY_CHROMIUM,
  join(homedir(), ".cache/ms-playwright/chromium-1243/chrome-linux64/chrome"),
  join(homedir(), ".cache/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-linux64/chrome-headless-shell"),
  "/snap/bin/chromium",
].filter(Boolean);
const executablePath = candidates.find((p) => existsSync(p));
if (!executablePath) { console.error("no Chromium found; set GALLERY_CHROMIUM"); process.exit(2); }

const browser = await chromium.launch({ executablePath, args: ["--no-sandbox", "--font-render-hinting=none"] });
try {
  const ctx = await browser.newContext({
    viewport: { width, height }, deviceScaleFactor: 1,
    reducedMotion: flag("reduced-motion") ? "reduce" : "no-preference",
  });
  const page = await ctx.newPage();
  const failed = [];
  page.on("requestfailed", (r) => failed.push(r.url()));
  page.on("response", (r) => { if (r.status() >= 400) failed.push(`${r.status()} ${r.url()}`); });
  const resp = await page.goto(url, { waitUntil: "load" });
  if (!resp || !resp.ok()) throw new Error(`page load failed: ${resp?.status()} ${url}`);
  await page.evaluate(() => document.fonts.ready);
  const fonts = await page.evaluate(() => [...document.fonts].filter((f) => f.status === "loaded").map((f) => `${f.family} ${f.style}`));
  await page.waitForTimeout(settle);
  if (failed.length) console.warn("WARN failed requests:\n  " + failed.join("\n  "));

  const shoot = async (path) => {
    if (!selector) return page.screenshot({ path, fullPage: flag("full") });
    const el = page.locator(selector).first();
    await el.scrollIntoViewIfNeeded();
    const b = await el.boundingBox();
    if (!b) throw new Error(`--selector ${selector}: no box`);
    const pad = 16, vw = page.viewportSize().width;
    return page.screenshot({ path, fullPage: true, clip: { x: Math.max(0, b.x - pad), y: Math.max(0, b.y - pad + await page.evaluate(() => scrollY)), width: Math.min(vw, b.width + 2 * pad), height: b.height + 2 * pad } });
  };
  if (!frames) {
    await shoot(out);
    console.log(`${out} (${width}x${height}${flag("full") ? " full" : ""}) fonts loaded: ${[...new Set(fonts)].join(", ") || "NONE"}`);
  } else {
    if (trigger) await page.evaluate((expr) => (0, eval)(expr), trigger);
    // Let the trigger's style change start its transitions, then freeze.
    await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
    const count = await page.evaluate(() => { const a = document.getAnimations(); a.forEach((x) => x.pause()); return a.length; });
    if (!count) console.warn("WARN no running animations found after trigger");
    const shots = [];
    for (let i = 0; i < frames; i++) {
      const t = frames === 1 ? duration : Math.round((duration * i) / (frames - 1));
      await page.evaluate((ms) => document.getAnimations().forEach((a) => { a.currentTime = ms; }), t);
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
      const p = out.replace(/\.png$/, "") + `-f${i + 1}.png`;
      await shoot(p);
      shots.push({ p, t });
    }
    // Strip: a tiny local HTML page with the frames inline, screenshotted.
    const label = "font: 500 12px monospace; color: #ccc; padding: 4px 0;"; // compose page only, not gallery UI
    const cells = shots.map(({ p, t }) =>
      `<figure style="margin:0"><img src="data:image/png;base64,${readFileSync(p).toString("base64")}" style="display:block"><figcaption style="${label}">${t} ms</figcaption></figure>`).join("");
    const html = `<!doctype html><body style="margin:0;background:#111;display:flex;gap:8px;padding:8px;width:max-content">${cells}</body>`;
    const tmp = join(tmpdir(), `strip-${process.pid}.html`);
    writeFileSync(tmp, html);
    const sp = await ctx.newPage();
    await sp.setViewportSize({ width: 400, height: 40 }); // fullPage grows it to the content
    await sp.goto(pathToFileURL(tmp).href);
    const strip = out.replace(/\.png$/, "") + "-strip.png";
    await sp.screenshot({ path: strip, fullPage: true });
    unlinkSync(tmp);
    console.log(`${shots.length} frames over ${duration} ms (${count} animations) -> ${basename(strip)}`);
  }
} finally {
  await browser.close();
}
