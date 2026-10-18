#!/usr/bin/env node
// Capture the RiftSketch preview stills (M152 item 2).
//   python3 -m http.server 8152 --directory <repo root>   (fonts load same-origin)
//   PLAYWRIGHT_CORE=<path to playwright-core> node capture-rift.mjs
// Writes .planning/missions/M152/gallery/out/rift-*.png at DPR 1 (and -dpr2 for the sketches).
import { createRequire } from "node:module";
import { mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const GAL = resolve(HERE, "..");
const OUT = join(GAL, "out");
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_CORE || "playwright-core");
const base = process.env.BASE || "http://127.0.0.1:8152/.planning/missions/M152/gallery/";
const exe = process.env.CHROMIUM;

const shots = [
  ["rift-sketch-1280", 1280, 800, 1],
  ["rift-sketch-1280", 1280, 800, 2],
  ["rift-sketch-390", 390, 852, 1],
  ["rift-sketch-390", 390, 852, 2],
  ["rift-survey-1280", 1280, 800, 1],
  ["rift-survey-4x-1280", 1280, 800, 1],
  ["rift-share-390", 390, 160, 1],
  ["rift-share-390", 390, 160, 2],
  ["rift-share-1280", 1280, 200, 1],
  ["rift-inset-390", 390, 160, 1],
  ["rift-inset-390", 390, 160, 2],
];

mkdirSync(OUT, { recursive: true });
const browser = await chromium.launch(exe ? { executablePath: exe } : {});
for (const [name, width, height, dpr] of shots) {
  const ctx = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: dpr });
  const page = await ctx.newPage();
  const failed = [];
  page.on("requestfailed", (r) => failed.push(r.url()));
  await page.goto(base + name + ".html", { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
  const file = join(OUT, `${name}${dpr > 1 ? "-dpr" + dpr : ""}.png`);
  await page.screenshot({ path: file });
  if (failed.length) console.error("failed requests:", failed);
  console.log(file);
  await ctx.close();
}
await browser.close();
