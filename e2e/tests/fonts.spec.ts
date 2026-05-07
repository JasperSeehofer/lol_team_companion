/**
 * Phase 17 plan 17-01 — Wave 0 G-01 enforcement spec.
 *
 * Hard NO from `[[guardrails#G-01]]`: zero requests to fonts.googleapis.com
 * or fonts.gstatic.com. All 5 font families are self-hosted under
 * `/fonts/{family}/*.woff2` (see plan 17-01 task 1).
 */
import { test, expect } from "@playwright/test";

test("no Google Fonts CDN requests on landing page", async ({ page }) => {
  const googleFontRequests: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    if (url.includes("fonts.googleapis.com") || url.includes("fonts.gstatic.com")) {
      googleFontRequests.push(url);
    }
  });
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  expect(googleFontRequests).toEqual([]);
});

test("no Google Fonts CDN requests on auth pages", async ({ page }) => {
  const googleFontRequests: string[] = [];
  page.on("request", (req) => {
    const url = req.url();
    if (url.includes("fonts.googleapis.com") || url.includes("fonts.gstatic.com")) {
      googleFontRequests.push(url);
    }
  });
  await page.goto("/auth/login");
  await page.waitForLoadState("networkidle");
  await page.goto("/auth/register");
  await page.waitForLoadState("networkidle");
  expect(googleFontRequests).toEqual([]);
});

test("self-hosted fonts respond with 200 status", async ({ page }) => {
  const fontResponses: { url: string; status: number }[] = [];
  page.on("response", (resp) => {
    const url = resp.url();
    if (url.includes("/fonts/") && url.endsWith(".woff2")) {
      fontResponses.push({ url, status: resp.status() });
    }
  });
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  // Browser may not request all 19 weights — only assert that any responses
  // we did see came back 200 (no 404 / 5xx).
  for (const { url, status } of fontResponses) {
    expect(status, `font ${url} returned ${status}`).toBe(200);
  }
});
