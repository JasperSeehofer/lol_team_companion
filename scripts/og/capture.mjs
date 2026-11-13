#!/usr/bin/env node
// Link-preview cards (M152 item 5, recipe card F): builds the three pages with
// build.py, serves them with the self-hosted fonts on a loopback port, captures
// each at 1200x630 and writes public/og/{landing,share,default}.png.
//
//   PLAYWRIGHT_CORE=<path to playwright-core> [CHROMIUM=<executable>] node scripts/og/capture.mjs
//
// Then each PNG is reduced to a 64-colour palette with ImageMagick (`convert`)
// when that is installed. Fails if a card is not 1200x630 or not under 100 KB.
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { mkdtempSync, readFileSync, statSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..", "..");
const OUT = join(ROOT, "public", "og");
const FONTS = join(ROOT, "public", "fonts");
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_CORE || "playwright-core");

const pages = mkdtempSync(join(tmpdir(), "og-pages-"));
execFileSync("python3", ["-I", join(HERE, "build.py"), pages], { stdio: "inherit" });

const types = { ".html": "text/html; charset=utf-8", ".woff2": "font/woff2" };
const server = createServer((req, res) => {
  const url = decodeURIComponent(new URL(req.url, "http://x").pathname);
  const [base, rel] = url.startsWith("/fonts/") ? [FONTS, url.slice(7)] : [pages, url.slice(1)];
  const file = normalize(join(base, rel));
  if (!file.startsWith(base) || !existsSync(file)) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": types[file.slice(file.lastIndexOf("."))] || "application/octet-stream" });
  res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(0, "127.0.0.1", ok));
const base = `http://127.0.0.1:${server.address().port}/`;

const browser = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {});
let failed = false;
for (const kind of ["landing", "share", "default"]) {
  const ctx = await browser.newContext({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const external = [];
  page.on("request", (r) => {
    if (!r.url().startsWith(base)) external.push(r.url());
  });
  await page.goto(base + kind + ".html", { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
  const fonts = await page.evaluate(() => Array.from(document.fonts).filter((f) => f.status === "loaded").map((f) => f.family));
  const file = join(OUT, `${kind}.png`);
  await page.screenshot({ path: file });
  await ctx.close();
  try {
    execFileSync("convert", [file, "-colors", "64", "-strip", "-define", "png:compression-level=9", `PNG8:${file}`]);
  } catch {
    console.error("convert (ImageMagick) not found: the PNG keeps its full colours");
  }
  const bytes = readFileSync(file);
  const w = bytes.readUInt32BE(16);
  const h = bytes.readUInt32BE(20);
  const size = statSync(file).size;
  const ok = w === 1200 && h === 630 && size < 100 * 1024 && external.length === 0 && fonts.length >= 3;
  if (!ok) failed = true;
  console.log(`${ok ? "ok" : "FAIL"} ${file}: ${w}x${h}, ${size} bytes, fonts ${[...new Set(fonts)].join(", ")}${external.length ? `, external ${external.join(" ")}` : ""}`);
}
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
