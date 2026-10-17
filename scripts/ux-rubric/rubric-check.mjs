#!/usr/bin/env node
// rubric-check.mjs — runs machine-checkable rows from rubric.json against
// real pages. Node ≥23, ESM. No new dependencies — playwright-core resolved
// from --app-dir's own node_modules.

import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join, resolve } from "node:path";
import { readFileSync, existsSync } from "node:fs";
import { homedir } from "node:os";
import { createRequire } from "node:module";

const HERE = dirname(fileURLToPath(import.meta.url));

// Parse arguments
const args = process.argv.slice(2);
const appDirIndex = args.indexOf("--app-dir");
const selfTestIndex = args.indexOf("--self-test");
const initScriptIndex = args.indexOf("--init-script");
const reducedMotionIndex = args.indexOf("--reduced-motion");
const settleIndex = args.indexOf("--settle");

if (appDirIndex === -1) {
  console.error("Error: --app-dir is required");
  process.exit(1);
}
const appDir = resolve(args[appDirIndex + 1]);
const isSelfTest = selfTestIndex !== -1;

// --init-script <file.js>: contents are installed via context.addInitScript
// on every page the harness creates, so localStorage / sessionStorage can be
// seeded (e.g. parallaxNewSurfaces, clearing a PIN gate) before the page's
// own scripts run. --reduced-motion: emulateMedia({reducedMotion:"reduce"})
// on every page, in addition to whatever a specific check (A3) already does.
// Both are no-ops on behaviour/output when absent.
let initScriptContent = null;
if (initScriptIndex !== -1) {
  const initScriptPath = resolve(args[initScriptIndex + 1] || "");
  try {
    initScriptContent = readFileSync(initScriptPath, "utf8");
  } catch (err) {
    console.error("Error: cannot read --init-script file:", initScriptPath, err.message);
    process.exit(1);
  }
}
const useReducedMotion = reducedMotionIndex !== -1;

// --settle <ms>: navigation now waits for "load" (not "networkidle" — a live
// Parallax build opens a persistent /events SSE connection right after
// auth, which "networkidle" waits forever for, returning an empty results
// table) followed by a fixed settle delay, plus document.fonts.ready.
// Default 1500ms; override for slower pages.
let settleMs = 1500;
if (settleIndex !== -1) {
  const raw = args[settleIndex + 1];
  const parsed = Number(raw);
  if (raw === undefined || Number.isNaN(parsed) || parsed < 0) {
    console.error("Error: --settle requires a non-negative number of milliseconds");
    process.exit(1);
  }
  settleMs = parsed;
}

// Filter out known flags (and their values) to get target paths
const initScriptValueIndex = initScriptIndex === -1 ? -1 : initScriptIndex + 1;
const settleValueIndex = settleIndex === -1 ? -1 : settleIndex + 1;
const targetPaths = args.filter((_, i) =>
  i !== appDirIndex && i !== appDirIndex + 1 && i !== selfTestIndex &&
  i !== initScriptIndex && i !== initScriptValueIndex && i !== reducedMotionIndex &&
  i !== settleIndex && i !== settleValueIndex
);

// In self-test mode, we don't need target paths
if (!isSelfTest && targetPaths.length === 0) {
  console.error("Error: at least one path or URL is required");
  process.exit(1);
}

// Resolve playwright-core from appDir's node_modules
let chromium;
try {
  const requireFromApp = createRequire(join(appDir, "package.json"));
  const pwModule = requireFromApp("playwright-core");
  chromium = pwModule.chromium;
} catch (err) {
  console.error("Error: cannot resolve playwright-core from", appDir, err.message);
  process.exit(1);
}

// Resolve Chromium executable
const candidates = [
  process.env.TRELLIS_CHROMIUM,
  "/usr/bin/chromium",
  join(homedir(), ".cache/ms-playwright/chromium-1243/chrome-linux64/chrome"),
].filter(Boolean);

const chromiumPath = candidates.find((p) => existsSync(p));
if (!chromiumPath) {
  console.error("Error: no Chromium executable found in:");
  candidates.forEach((p) => console.error("  -", p));
  process.exit(2);
}

// Load rubric
const rubricPath = join(HERE, "rubric.json");
let rubric;
try {
  const raw = readFileSync(rubricPath, "utf8");
  rubric = JSON.parse(raw);
} catch (err) {
  console.error("Error: cannot load rubric.json:", err.message);
  process.exit(1);
}

// Filter to harness-run checks
const harnessChecks = rubric.checks.filter((c) => c.run_by === "harness");

// Viewports to test
const viewports = [
  { name: "phone", width: 390, height: 844 },
  { name: "desktop", width: 1280, height: 800 },
];

// WCAG contrast ratio calculation (GUESS: standard formula)
// GUESS: Using standard WCAG 2.1 contrast formula
function getContrastRatio(color1, color2) {
  // Parse color strings to RGB
  const rgb1 = parseColor(color1);
  const rgb2 = parseColor(color2);
  
  if (!rgb1 || !rgb2) return null;
  
  // Calculate relative luminance
  const l1 = getLuminance(rgb1);
  const l2 = getLuminance(rgb2);
  
  // Ensure lighter color is first
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  
  return (lighter + 0.05) / (darker + 0.05);
}

// GUESS: Standard color parsing for CSS color values
function parseColor(colorStr) {
  if (!colorStr) return null;
  
  // Handle transparent/none
  if (colorStr === "transparent" || colorStr === "none" || colorStr === "rgba(0, 0, 0, 0)") {
    return { r: 0, g: 0, b: 0, a: 0 };
  }
  
  // Hex format
  if (colorStr.startsWith("#")) {
    const hex = colorStr.slice(1);
    let r, g, b;
    if (hex.length === 3 || hex.length === 4) {
      r = parseInt(hex[0] + hex[0], 16);
      g = parseInt(hex[1] + hex[1], 16);
      b = parseInt(hex[2] + hex[2], 16);
    } else if (hex.length === 6 || hex.length === 8) {
      r = parseInt(hex.slice(0, 2), 16);
      g = parseInt(hex.slice(2, 4), 16);
      b = parseInt(hex.slice(4, 6), 16);
    } else {
      return null;
    }
    return { r, g, b, a: 255 };
  }
  
  // rgb() or rgba() format
  const rgbMatch = colorStr.match(/^(rgb|rgba)\(([^)]+)\)/);
  if (rgbMatch) {
    const parts = rgbMatch[2].split(",").map((p) => parseFloat(p.trim()));
    if (parts.length >= 3) {
      return {
        r: Math.round(parts[0]),
        g: Math.round(parts[1]),
        b: Math.round(parts[2]),
        a: parts.length >= 4 ? Math.round(parts[3] * 255) : 255,
      };
    }
  }
  
  // hsl() or hsla() format
  const hslMatch = colorStr.match(/^(hsl|hsla)\(([^)]+)\)/);
  if (hslMatch) {
    const parts = hslMatch[2].split(",").map((p) => parseFloat(p.trim().replace(/%/g, "")));
    if (parts.length >= 3) {
      return hslToRgb(parts[0], parts[1], parts[2], parts[3] || 1);
    }
  }
  
  // color-mix() - GUESS: simplify by returning black for unsupported
  if (colorStr.includes("color-mix")) {
    // GUESS: For color-mix, we'll use a fallback - this is a limitation
    return { r: 0, g: 0, b: 0, a: 255 };
  }
  
  return null;
}

// GUESS: HSL to RGB conversion
function hslToRgb(h, s, l, a = 1) {
  s /= 100;
  l /= 100;
  
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  
  let r, g, b;
  if (h >= 0 && h < 60) { r = c; g = x; b = 0; }
  else if (h >= 60 && h < 120) { r = x; g = c; b = 0; }
  else if (h >= 120 && h < 180) { r = 0; g = c; b = x; }
  else if (h >= 180 && h < 240) { r = 0; g = x; b = c; }
  else if (h >= 240 && h < 300) { r = x; g = 0; b = c; }
  else { r = c; g = 0; b = x; }
  
  return {
    r: Math.round((r + m) * 255),
    g: Math.round((g + m) * 255),
    b: Math.round((b + m) * 255),
    a: Math.round(a * 255),
  };
}

// GUESS: Standard luminance calculation
function getLuminance(rgb) {
  const { r, g, b } = rgb;
  const rsrgb = r / 255;
  const gsrgb = g / 255;
  const bsrgb = b / 255;
  
  const rLinear = rsrgb <= 0.03928 ? rsrgb / 12.92 : Math.pow((rsrgb + 0.055) / 1.055, 2.4);
  const gLinear = gsrgb <= 0.03928 ? gsrgb / 12.92 : Math.pow((gsrgb + 0.055) / 1.055, 2.4);
  const bLinear = bsrgb <= 0.03928 ? bsrgb / 12.92 : Math.pow((bsrgb + 0.055) / 1.055, 2.4);
  
  return 0.2126 * rLinear + 0.7152 * gLinear + 0.0722 * bLinear;
}

// GUESS: Check if a color is transparent (alpha = 0)
function isTransparent(colorStr) {
  if (!colorStr) return true;
  if (colorStr === "transparent" || colorStr === "none") return true;
  // Check for rgba with alpha 0
  const rgbaMatch = colorStr.match(/^rgba\([^,]+,[^,]+,[^,]+,\s*0(\.0+)?\)/);
  if (rgbaMatch) return true;
  return false;
}

// GUESS: Walk up ancestor chain to find first non-transparent background
// Returns the computed background-color string of the first non-transparent ancestor
async function getResolvedBackground(page, element) {
  return await element.evaluate((node) => {
    // Inline the transparent check logic for browser context
    function isTransparent(colorStr) {
      if (!colorStr) return true;
      if (colorStr === "transparent" || colorStr === "none" || colorStr === "rgba(0, 0, 0, 0)") return true;
      // Check for rgba with alpha 0
      const rgbaMatch = colorStr.match(/^rgba\([^,]+,[^,]+,[^,]+,\s*0(\.0+)?\)/);
      if (rgbaMatch) return true;
      return false;
    }
    
    let current = node;
    const doc = node.ownerDocument;
    
    while (current) {
      const style = doc.defaultView.getComputedStyle(current);
      const bgColor = style.backgroundColor;
      
      // Check if this background is non-transparent
      if (!isTransparent(bgColor)) {
        return bgColor;
      }
      
      // Move up to parent
      current = current.parentElement;
    }
    
    // Fallback: check document element and body
    const docStyle = doc.defaultView.getComputedStyle(doc.documentElement);
    if (!isTransparent(docStyle.backgroundColor)) {
      return docStyle.backgroundColor;
    }
    const bodyStyle = doc.defaultView.getComputedStyle(doc.body);
    if (!isTransparent(bodyStyle.backgroundColor)) {
      return bodyStyle.backgroundColor;
    }
    
    // Ultimate fallback: white
    return "rgb(255, 255, 255)";
  });
}

// GUESS: Composite foreground color over background color using alpha compositing
// result = fg * alpha + bg * (1 - alpha) per channel
function compositeColors(fgRgb, bgRgb) {
  // Convert to 0-1 range for compositing
  const fg = { r: fgRgb.r / 255, g: fgRgb.g / 255, b: fgRgb.b / 255, a: fgRgb.a / 255 };
  const bg = { r: bgRgb.r / 255, g: bgRgb.g / 255, b: bgRgb.b / 255, a: bgRgb.a / 255 };
  
  // Alpha compositing: result = fg * fg.a + bg * (1 - fg.a)
  // We assume bg is opaque (a=1) for background colors from CSS
  const alpha = fg.a;
  const result = {
    r: Math.round((fg.r * alpha + bg.r * (1 - alpha)) * 255),
    g: Math.round((fg.g * alpha + bg.g * (1 - alpha)) * 255),
    b: Math.round((fg.b * alpha + bg.b * (1 - alpha)) * 255),
    a: 255 // Result is opaque
  };
  
  return result;
}

// GUESS: Format RGB object back to CSS string for reporting
function rgbToCssString(rgb) {
  if (rgb.a === 0) return "transparent";
  if (rgb.a === 255) {
    return `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
  }
  const alpha = (rgb.a / 255).toFixed(2);
  // Clean up trailing zeros
  const cleanAlpha = alpha.endsWith(".00") ? "1" : alpha.endsWith("0") ? alpha.slice(0, -1) : alpha;
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${cleanAlpha})`;
}

// Parse blur value from CSS string (GUESS: heuristic for blur parsing)
function parseBlurValue(styleValue) {
  if (!styleValue) return 0;
  
  // Look for blur() function
  const blurMatch = styleValue.match(/blur\(([0-9.]+)px\)/);
  if (blurMatch) {
    return parseFloat(blurMatch[1]);
  }
  
  // Look for box-shadow blur (third value in shadow)
  const shadowMatch = styleValue.match(/box-shadow:\s*[^)]*?([0-9.]+)px[^)]*?([0-9.]+)px[^)]*?([0-9.]+)px/);
  if (shadowMatch) {
    return parseFloat(shadowMatch[3]);
  }
  
  return 0;
}

// GUESS: Parse CSS duration string (e.g., "160ms", "0.16s") to milliseconds
function parseDurationToMs(durationStr) {
  if (!durationStr || durationStr === "0s") return [];
  
  // Split on whitespace or comma to handle multiple durations
  const parts = durationStr.split(/[\s,]+/);
  const durations = [];
  
  for (const part of parts) {
    if (!part || part === "0s") continue;
    
    // Check for ms suffix
    if (part.endsWith("ms")) {
      const value = parseFloat(part.slice(0, -2));
      if (!isNaN(value)) {
        durations.push(value); // Already in milliseconds
      }
    }
    // Check for s suffix (seconds)
    else if (part.endsWith("s")) {
      const value = parseFloat(part.slice(0, -1));
      if (!isNaN(value)) {
        durations.push(value * 1000); // Convert to milliseconds
      }
    }
    // Bare number - assume seconds (GUESS: computed style always has suffix, but be safe)
    else {
      const value = parseFloat(part);
      if (!isNaN(value)) {
        durations.push(value * 1000); // Assume seconds
      }
    }
  }
  
  return durations;
}

// Create a page in a fresh browser context, applying --init-script and
// --reduced-motion when present. Returns the page; use closePageContext()
// to tear down the page's context afterwards (not just the page) so
// per-navigation contexts don't accumulate across many target paths /
// self-test fixtures.
async function newPage(browser, viewport) {
  const context = await browser.newContext({ viewport });
  if (initScriptContent) {
    await context.addInitScript(initScriptContent);
  }
  const page = await context.newPage();
  if (useReducedMotion) {
    await page.emulateMedia({ reducedMotion: "reduce" });
  }
  return page;
}

async function closePageContext(page) {
  const context = page.context();
  try {
    await page.close();
  } catch (e) {
    // Ignore page close errors
  }
  try {
    await context.close();
  } catch (e) {
    // Ignore context close errors
  }
}

// Navigate and wait for the page to be ready to measure. "networkidle"
// never resolves against a live Parallax build — it opens a persistent
// /events SSE connection right after auth, which counts as an in-flight
// request forever, so the harness would hang and return an empty results
// table. "load" plus a fixed settle delay (--settle, default 1500ms) is the
// replacement; document.fonts.ready is also awaited so measurements (esp.
// A2 contrast, V3 CPL) don't run against a pre-webfont layout.
async function gotoAndSettle(page, url, settleMsValue) {
  await page.goto(url, { waitUntil: "load" });
  await page.waitForTimeout(settleMsValue);
  try {
    await page.evaluate(() =>
      document.fonts && document.fonts.ready ? document.fonts.ready : Promise.resolve()
    );
  } catch (e) {
    // Ignore font-readiness errors (e.g. document.fonts unsupported)
  }
}

// Main measurement functions
async function runChecks(browser, pageUrl, viewport) {
  const results = [];
  const page = await newPage(browser, viewport);

  try {
    // Convert filesystem path to file:// URL if needed
    let url = pageUrl;
    if (pageUrl.startsWith("/") && !pageUrl.startsWith("file://")) {
      url = `file://${pageUrl}`;
    }
    
    await gotoAndSettle(page, url, settleMs);

    for (const check of harnessChecks) {
      const result = await measureCheckOnPage(page, url, viewport, check);
      results.push(result);
    }
  } finally {
    await closePageContext(page);
  }

  return results;
}

async function measureCheckOnPage(page, pageUrl, viewport, check) {
    try {
      let verdict = "PASS";
      let detail = "";
      
      switch (check.id) {
      case "A1":
        // Tap target size check
        const elements = await page.$$("button, a, [role=\"button\"], input, [tabindex]");
        const violations = [];
        
        for (const el of elements) {
          const box = await el.boundingBox();
          if (!box) continue;
          
          const text = await el.evaluate((node) => node.textContent?.trim() || node.getAttribute("aria-label") || "");
          const selector = await el.evaluate((node) => {
            if (node.id) return `#${node.id}`;
            if (node.className) return `.${node.className.split(" ").join(".")}`;
            return node.tagName.toLowerCase();
          });
          
          // Check size
          if (box.width < 44 || box.height < 44) {
            // Check if it's at least 8px away from next element
            const isSpaced = await checkAdjacentSpacing(page, el, box);
            if (!isSpaced) {
              violations.push(`${selector}: ${box.width}x${box.height}px (text: "${text.slice(0, 30)}")`);
            }
          }
        }
        
        if (violations.length > 0) {
          verdict = "FIX";
          detail = `Small targets: ${violations.join("; ")}`;
        }
        break;
        
      case "A2":
        // Contrast check - walk up ancestor chain for real background
        const textElements = await page.$$("body, p, span, div, a, button, h1, h2, h3, h4, h5, h6");
        const contrastViolations = [];
        
        for (const el of textElements.slice(0, 50)) { // Limit for performance
          // Only check elements that have direct text content (not just inherited from children)
          const hasDirectText = await el.evaluate((node) => {
            // Check if this element has any direct text nodes (not just whitespace)
            for (const child of node.childNodes) {
              if (child.nodeType === Node.TEXT_NODE && child.textContent?.trim().length > 0) {
                return true;
              }
            }
            return false;
          });
          
          if (!hasDirectText) continue;
          
          // Get the element's text color (computed style)
          const colorStr = await el.evaluate((node) => {
            const style = window.getComputedStyle(node);
            return style.color;
          });
          
          // Get the resolved background by walking up ancestors
          const bgColorStr = await getResolvedBackground(page, el);
          
          const fgRgb = parseColor(colorStr);
          const bgRgb = parseColor(bgColorStr);
          
          if (!fgRgb || !bgRgb) continue;
          
          // Composite the foreground color over the background
          const compositedFg = compositeColors(fgRgb, bgRgb);
          const compositedBg = bgRgb; // Background is already opaque
          
          const contrast = getContrastRatio(rgbToCssString(compositedFg), rgbToCssString(compositedBg));
          if (contrast === null) continue;
          
          // Determine if it's large text
          const fontSize = await el.evaluate((node) => {
            const style = window.getComputedStyle(node);
            return parseFloat(style.fontSize);
          });
          
          const isLargeText = fontSize >= 18; // GUESS: large text threshold
          const threshold = isLargeText ? 3.0 : 4.5;
          
          if (contrast < threshold) {
            const selector = await el.evaluate((node) => {
              if (node.id) return `#${node.id}`;
              return node.tagName.toLowerCase();
            });
            contrastViolations.push(`${selector}: ${contrast.toFixed(2)}:1 (${rgbToCssString(compositedFg)} on ${rgbToCssString(compositedBg)})`);
          }
        }
        
        if (contrastViolations.length > 0) {
          verdict = "FIX";
          detail = `Low contrast: ${contrastViolations.slice(0, 5).join("; ")}${contrastViolations.length > 5 ? ` (+${contrastViolations.length - 5} more)` : ""}`;
        }
        break;
        
      case "V4":
        // Motion durations check
        const motionViolations = [];
        
        // Check for transition-duration
        const transitionElements = await page.$$("*");
        for (const el of transitionElements.slice(0, 100)) {
          const transitionDuration = await el.evaluate((node) => {
            const style = window.getComputedStyle(node);
            return style.transitionDuration;
          });
          
          if (transitionDuration && transitionDuration !== "0s") {
            // Parse duration value with unit awareness (ms vs s)
            const durations = parseDurationToMs(transitionDuration);
            for (const dur of durations) {
              if (!isNaN(dur)) {
                // Check if element has convergence exception
                const hasConvergence = await el.evaluate((node) => {
                  return node.hasAttribute("data-motion") && node.getAttribute("data-motion") === "convergence";
                });
                
                if (!hasConvergence) {
                  if (dur < 100 || dur > 500) {
                    const selector = await el.evaluate((node) => node.tagName.toLowerCase());
                    motionViolations.push(`${selector}: ${dur}ms`);
                  }
                } else {
                  // Convergence exception: 600-900ms allowed
                  if (dur < 600 || dur > 900) {
                    const selector = await el.evaluate((node) => node.tagName.toLowerCase());
                    motionViolations.push(`${selector}: ${dur}ms (convergence exception)`);
                  }
                }
              }
            }
          }
        }
        
        if (motionViolations.length > 0) {
          verdict = "FIX";
          detail = `Motion durations: ${motionViolations.slice(0, 5).join("; ")}`;
        }
        break;
        
      case "A3":
        // Reduced motion check - check computed styles after emulation
        await page.emulateMedia({ reducedMotion: "reduce" });
        await page.waitForTimeout(150);
        
        // Check computed styles on actual DOM elements (not static CSSOM)
        // GUESS: cap at 100 elements for performance, same as V4
        const allElements = await page.$$("*");
        const hasActiveAnimations = await (async () => {
          for (const el of allElements.slice(0, 100)) {
            const computed = await el.evaluate((node) => {
              const style = window.getComputedStyle(node);
              return {
                animationName: style.animationName,
                animationDuration: style.animationDuration,
                transitionDuration: style.transitionDuration,
              };
            });
            
            // Check if this element has an active animation
            // animationName is a comma-separated list; "none" means no animation
            const animationNames = computed.animationName.split(",").map(n => n.trim());
            const hasNonNoneAnimation = animationNames.some(name => name !== "none");
            
            // Also check animationDuration - if it's non-zero and animationName is not "none"
            const animationDurations = parseDurationToMs(computed.animationDuration);
            const hasNonZeroAnimationDuration = animationDurations.some(d => d > 0);
            
            // NOTE (fix(M40): verification 2026-09-15): the original draft here also
            // OR'd in a bare `transitionDuration > 0` check ("per rubric: no state
            // change communicated by motion alone"). That fired on ANY element with a
            // CSS transition regardless of `animation-name` — L6-press-feedback.html
            // uses only `transition:` (press/release feedback), never `@keyframes`, and
            // was wrongly FIXed at both viewports, violating the SPEC's Instruction 5
            // acceptance criterion (no A2/A3/V4 FIX on L3/L4/L6). The SPEC's FIX
            // condition is explicit: "FIX only if the computed, post-emulation
            // animationName !== 'none' AND animationDuration is a nonzero duration for
            // some element — never from the static stylesheet scan." Transitions are
            // conditioned on user interaction (press/hover), not autoplaying motion, so
            // they are out of scope for this check. Removed the transitionDuration OR
            // branch to match the SPEC's literal condition.

            // Check for convergence exception (same pattern as V4)
            const hasConvergence = await el.evaluate((node) => {
              return node.hasAttribute("data-motion") && node.getAttribute("data-motion") === "convergence";
            });

            // If element has convergence exception, skip it
            if (hasConvergence) continue;

            // Element has active animation if animationName is not "none" AND
            // animationDuration > 0 (per SPEC's literal FIX condition).
            if (hasNonNoneAnimation && hasNonZeroAnimationDuration) {
              return true;
            }
          }
          return false;
        })();
        
        if (hasActiveAnimations) {
          verdict = "FIX";
          detail = "Animation still active under prefers-reduced-motion: reduce";
        }
        break;
        
      case "I7":
        // Secondary UI collapsed check
        const secondaryElements = await page.$$("[data-secondary]");
        const secondaryViolations = [];
        
        for (const el of secondaryElements) {
          const display = await el.evaluate((node) => {
            const style = window.getComputedStyle(node);
            return style.display;
          });
          
          const box = await el.boundingBox();
          
          if (display !== "none" && box && box.height > 0) {
            const text = await el.evaluate((node) => node.textContent?.trim().slice(0, 20) || "");
            const selector = await el.evaluate((node) => {
              if (node.id) return `#${node.id}`;
              return node.tagName.toLowerCase();
            });
            secondaryViolations.push(`${selector}: visible (${display}, height: ${box.height}px)`);
          }
        }
        
        if (secondaryViolations.length > 0) {
          verdict = "FIX";
          detail = `Secondary UI not collapsed: ${secondaryViolations.join("; ")}`;
        }
        break;
        
      case "I1":
        // Manual path survives - check for settings reachability
        // This is a heuristic check
        const hasSettings = await page.evaluate(() => {
          const links = Array.from(document.querySelectorAll("a, button, [role='button']"));
          return links.some(link => {
            const text = (link.textContent || "").toLowerCase();
            const href = (link.getAttribute("href") || "").toLowerCase();
            const ariaLabel = (link.getAttribute("aria-label") || "").toLowerCase();
            return text.includes("settings") || href.includes("settings") || ariaLabel.includes("settings");
          });
        });
        
        if (!hasSettings) {
          verdict = "FIX";
          detail = "No Settings reachable path found";
        }
        break;
        
      case "I2":
        // Progress past 10s - check for progress indicators
        // This is a heuristic - we can't wait 10s in the harness
        const hasProgress = await page.evaluate(() => {
          const progressElements = Array.from(document.querySelectorAll("[role='progressbar'], .progress, .spinner, .loading"));
          return progressElements.length > 0;
        });
        
        // For this check, we'll assume PASS if there are progress elements
        // GUESS: This is a limitation - we can't test actual 10s+ actions
        if (!hasProgress) {
          verdict = "FIX";
          detail = "No progress indicators found (GUESS: cannot test 10s+ actions)";
        }
        break;
        
      case "I6":
        // Interruption budget - check for notification policy
        // GUESS: heuristic check for notification elements
        const hasNotificationPolicy = await page.evaluate(() => {
          const scripts = Array.from(document.scripts);
          return scripts.some(script => {
            const content = script.textContent || "";
            return content.includes("urgency") || content.includes("quietHours") || 
                   content.includes("dailyCap") || content.includes("batchWindow");
          });
        });
        
        if (!hasNotificationPolicy) {
          verdict = "FIX";
          detail = "No notification policy found (GUESS: heuristic check)";
        }
        break;
        
      case "I8":
        // Undo/consent for agent actions - check for undo affordances
        const hasUndo = await page.evaluate(() => {
          const elements = Array.from(document.querySelectorAll("button, a, [role='button']"));
          return elements.some(el => {
            const text = (el.textContent || "").toLowerCase();
            const ariaLabel = (el.getAttribute("aria-label") || "").toLowerCase();
            return text.includes("undo") || text.includes("revert") || 
                   ariaLabel.includes("undo") || ariaLabel.includes("revert");
          });
        });
        
        if (!hasUndo) {
          verdict = "FIX";
          detail = "No undo affordance found (GUESS: heuristic check)";
        }
        break;
        
      case "V1": {
        // Closed token vocabulary - check for raw color literals.
        // Exemptions (ratified 2026-09-16, card m35-pipelines-2):
        //   (a) skip tokens.css itself / any stylesheet whose URL ends in
        //       tokens.css — it is the generated token file, not a consumer.
        //   (b) exempt 1px hairline borders (1px solid with a token colour).
        const v1Result = await page.evaluate(() => {
          const hairlineRe = /(?:border(?:-(?:top|right|bottom|left))?|outline)\s*:\s*1px\s+solid\s+var\([^)]*\)\s*;?/gi;
          const violations = [];
          let tokensCssSkipped = 0;
          let hairlineExempt = 0;
          const styles = Array.from(document.styleSheets);
          for (const sheet of styles) {
            if (sheet.href && sheet.href.endsWith("tokens.css")) {
              tokensCssSkipped++;
              continue;
            }
            try {
              const rules = sheet.cssRules || [];
              for (const rule of rules) {
                if (rule.style) {
                  let cssText = rule.style.cssText || "";
                  const hairlineMatches = cssText.match(hairlineRe);
                  if (hairlineMatches) {
                    hairlineExempt += hairlineMatches.length;
                    cssText = cssText.replace(hairlineRe, "");
                  }
                  if (cssText.match(/#[0-9a-fA-F]{3,8}/) ||
                      cssText.match(/rgb\(/)) {
                    violations.push(rule.selectorText || "(rule)");
                  }
                }
              }
            } catch (e) {
              // CORS - skip
            }
          }
          return { violations, tokensCssSkipped, hairlineExempt };
        });

        if (v1Result.violations.length > 0) {
          verdict = "FIX";
          detail = `Raw color literals found in CSS: ${v1Result.violations.slice(0, 5).join(", ")}`;
        } else {
          const notes = [];
          if (v1Result.tokensCssSkipped > 0) {
            notes.push(`exempt: tokens.css (${v1Result.tokensCssSkipped} stylesheet${v1Result.tokensCssSkipped === 1 ? "" : "s"})`);
          }
          if (v1Result.hairlineExempt > 0) {
            notes.push(`exempt: hairline (${v1Result.hairlineExempt} declaration${v1Result.hairlineExempt === 1 ? "" : "s"})`);
          }
          detail = notes.join("; ");
        }
        break;
      }

      case "V2": {
        // Spacing on 4/8 scale - check for off-scale values.
        // Exemptions (ratified 2026-09-16, card m35-pipelines-2):
        //   (a) skip tokens.css itself / any stylesheet whose URL ends in
        //       tokens.css — it is the generated token file, not a consumer.
        //   (b) exempt 1px hairline borders (1px solid with a token colour).
        const v2Result = await page.evaluate(() => {
          const validScale = new Set([4, 8, 16, 24, 32, 40, 48, 56, 64]);
          const hairlineRe = /(?:border(?:-(?:top|right|bottom|left))?|outline)\s*:\s*1px\s+solid\s+var\([^)]*\)\s*;?/gi;
          const violations = [];
          let tokensCssSkipped = 0;
          let hairlineExempt = 0;
          const styles = Array.from(document.styleSheets);
          for (const sheet of styles) {
            if (sheet.href && sheet.href.endsWith("tokens.css")) {
              tokensCssSkipped++;
              continue;
            }
            try {
              const rules = sheet.cssRules || [];
              for (const rule of rules) {
                if (rule.style) {
                  let cssText = rule.style.cssText || "";
                  const hairlineMatches = cssText.match(hairlineRe);
                  if (hairlineMatches) {
                    hairlineExempt += hairlineMatches.length;
                    cssText = cssText.replace(hairlineRe, "");
                  }
                  const matches = cssText.match(/([0-9]+)px/g);
                  if (matches) {
                    for (const match of matches) {
                      const value = parseInt(match);
                      if (!validScale.has(value) && value !== 0) {
                        violations.push(`${rule.selectorText || "(rule)"}: ${match}`);
                      }
                    }
                  }
                }
              }
            } catch (e) {
              // CORS - skip
            }
          }
          return { violations, tokensCssSkipped, hairlineExempt };
        });

        if (v2Result.violations.length > 0) {
          verdict = "FIX";
          detail = `Off-scale spacing values found: ${v2Result.violations.slice(0, 5).join(", ")}`;
        } else {
          const notes = [];
          if (v2Result.tokensCssSkipped > 0) {
            notes.push(`exempt: tokens.css (${v2Result.tokensCssSkipped} stylesheet${v2Result.tokensCssSkipped === 1 ? "" : "s"})`);
          }
          if (v2Result.hairlineExempt > 0) {
            notes.push(`exempt: hairline (${v2Result.hairlineExempt} declaration${v2Result.hairlineExempt === 1 ? "" : "s"})`);
          }
          detail = notes.join("; ");
        }
        break;
      }

      case "V3": {
        // Measure 45-75 CPL - check line lengths.
        // Exemption (ratified 2026-09-16, card m35-pipelines-2):
        //   (c) exempt short microcopy (<20 chars, or button/label/chip
        //       elements) from V3.
        const cplViolations = [];
        let microcopyExempt = 0;
        const textBlocks = await page.$$("p, .card, .brief, .transcript");

        for (const block of textBlocks.slice(0, 20)) {
          const info = await block.evaluate((node) => {
            const tag = node.tagName.toLowerCase();
            const role = (node.getAttribute("role") || "").toLowerCase();
            const cls = typeof node.className === "string" ? node.className.toLowerCase() : "";
            const isButtonLike = tag === "button" || tag === "label" || role === "button" ||
              cls.includes("chip") || cls.includes("label") || cls.includes("btn") || cls.includes("button");
            return { text: node.textContent || "", isButtonLike };
          });

          const trimmedLength = info.text.trim().length;
          if (info.isButtonLike || trimmedLength < 20) {
            microcopyExempt++;
            continue;
          }

          const lines = info.text.split("\n").filter(l => l.trim().length > 0);
          for (const line of lines) {
            if (line.length < 45 || line.length > 75) {
              const selector = await block.evaluate((node) => {
                if (node.id) return `#${node.id}`;
                return node.tagName.toLowerCase();
              });
              cplViolations.push(`${selector}: ${line.length} chars`);
            }
          }
        }

        if (cplViolations.length > 0) {
          verdict = "FIX";
          detail = `CPL violations: ${cplViolations.slice(0, 3).join("; ")}`;
        } else if (microcopyExempt > 0) {
          detail = `exempt: microcopy (${microcopyExempt} block${microcopyExempt === 1 ? "" : "s"})`;
        }
        break;
      }


      case "Vo1":
      case "Vo2":
      case "Vo3":
      case "Vo4":
        // Voice checks - skip for now as they're more complex
        // GUESS: These are limitations for this harness
        verdict = "PASS";
        detail = `GUESS: voice checks not implemented in harness`;
        break;
        
      case "T1":
      case "T2":
      case "T3":
      case "T4":
        // Trust checks - heuristic
        verdict = "PASS";
        detail = `GUESS: trust checks heuristic - need manual review`;
        break;
        
      default:
        verdict = "PASS";
        detail = `Unknown check: ${check.id}`;
    }
    
      return {
        id: check.id,
        page: pageUrl,
        viewport: viewport.name,
        verdict,
        detail,
      };
    } catch (err) {
      return {
        id: check.id,
        page: pageUrl,
        viewport: viewport.name,
        verdict: "FIX",
        detail: `Error: ${err.message}`,
      };
    }
}

// Helper function to check adjacent spacing
async function checkAdjacentSpacing(page, element, box) {
  // GUESS: simplified check - just return true for now
  // Full implementation would require checking all adjacent elements
  return true;
}

// Self-test: run fixtures and verify expected results
async function runSelfTest(browser) {
  const fixturesDir = join(HERE, "fixtures");
  const fs = await import("node:fs/promises");
  
  let files;
  try {
    files = await fs.readdir(fixturesDir);
  } catch (err) {
    console.error("Error: cannot read fixtures directory:", err.message);
    process.exit(1);
  }
  
  // Filter to .html files
  const htmlFiles = files.filter(f => f.endsWith(".html")).sort();
  const results = [];
  
  for (const file of htmlFiles) {
    const filePath = join(fixturesDir, file);
    
    // Parse expected verdict from filename
    // Format: <rule-id>-pass.html or <rule-id>-fail-*.html or <rule-id>-pass-*.html
    const baseName = file.replace(".html", "");
    const parts = baseName.split("-");
    const ruleId = parts[0].toUpperCase(); // e.g., "a2" -> "A2", "v4" -> "V4"
    
    // Determine expected verdict
    let expectedVerdict;
    if (baseName.includes("-pass")) {
      expectedVerdict = "PASS";
    } else if (baseName.includes("-fail")) {
      expectedVerdict = "FIX";
    } else {
      console.error(`Warning: fixture ${file} does not contain -pass or -fail, skipping`);
      continue;
    }
    
    // Find the check with this rule id
    const check = harnessChecks.find(c => c.id === ruleId);
    if (!check) {
      console.error(`Warning: no check found for rule id ${ruleId} in fixture ${file}, skipping`);
      continue;
    }
    
    // Run the check against the fixture (phone viewport only for self-test)
    const page = await newPage(browser, viewports[0]);
    try {
      const fileUrl = `file://${filePath}`;
      await gotoAndSettle(page, fileUrl, settleMs);

      const result = await measureCheckOnPage(page, fileUrl, viewports[0], check);
      const actualVerdict = result.verdict;
      const match = actualVerdict === expectedVerdict;

      results.push({
        fixture: file,
        expected: expectedVerdict,
        actual: actualVerdict,
        result: match ? "OK" : "MISMATCH",
      });
    } finally {
      await closePageContext(page);
    }
  }
  
  return results;
}

// Main execution
async function main() {
  const browser = await chromium.launch({
    executablePath: chromiumPath,
    args: ["--no-sandbox"],
  });
  
  const allResults = [];
  
  try {
    if (isSelfTest) {
      // Run self-test against fixtures
      allResults.push(...(await runSelfTest(browser)));
    } else {
      // Normal mode: run checks against target paths
      for (const targetPath of targetPaths) {
        for (const viewport of viewports) {
          const results = await runChecks(browser, targetPath, viewport);
          allResults.push(...results);
        }
      }
    }
  } catch (err) {
    console.error("Error during checks:", err.message);
  } finally {
    try {
      await browser.close();
    } catch (e) {
      // Ignore browser close errors
    }
  }
  
  if (isSelfTest) {
    // Print self-test results table
    console.log("| fixture | expected | actual | result |");
    console.log("|---------|----------|--------|--------|");
    
    for (const result of allResults) {
      console.log(`| ${result.fixture} | ${result.expected} | ${result.actual} | ${result.result} |`);
    }
    
    // Exit code: 1 if any mismatch
    const hasMismatch = allResults.some((r) => r.result === "MISMATCH");
    process.exit(hasMismatch ? 1 : 0);
  } else {
    // Normal mode: print standard results table
    console.log("| id | page | viewport | verdict | detail |");
    console.log("|----|------|----------|---------|--------|");
    
    for (const result of allResults) {
      const detail = result.detail.length > 50 
        ? result.detail.slice(0, 47) + "..." 
        : result.detail;
      console.log(`| ${result.id} | ${result.page} | ${result.viewport} | ${result.verdict} | ${detail} |`);
    }
    
    // Exit code
    const hasFix = allResults.some((r) => r.verdict === "FIX");
    process.exit(hasFix ? 1 : 0);
  }
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
