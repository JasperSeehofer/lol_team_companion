/**
 * M159 item 2 (atelier card 1): Match Day lists the team's other upcoming
 * matches under the current one. A fresh team adds two matches with the
 * editor; the later one shows in the list, and its row opens it as the
 * current view (`/?match_id=`). An id that is not on the team's schedule
 * falls back to the next match with a neutral line. Runs on a fresh account
 * and team (CI-safe: no seed data needed).
 *
 * Frames: when $MISSION_FRAMES_DIR is set, writes match-day-upcoming-390.png,
 * match-day-upcoming-1280.png (first viewport) and
 * match-day-upcoming-1280-flow.png (full page) to its top level.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

const DIR = process.env.MISSION_FRAMES_DIR;

function inDays(days: number): string {
  const d = new Date(Date.now() + days * 24 * 3600 * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

async function height(loc: import("@playwright/test").Locator): Promise<number> {
  return loc.evaluate((el) => el.getBoundingClientRect().height);
}

async function shot(page: Page, name: string, fullPage: boolean) {
  const sw = await page.evaluate(() => document.documentElement.scrollWidth);
  const vw = page.viewportSize()?.width ?? 0;
  expect(sw, `${name}: ${sw}px wide in a ${vw}px viewport`).toBeLessThanOrEqual(vw);
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
  await page.waitForTimeout(200);
  await page.screenshot({ path: path.join(DIR, `${name}.png`), fullPage });
}

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(1000);
}

test("match day: a second match shows in the upcoming list and opens as the current view", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await navigateTo(page, "/");
  await expect(page.getByRole("heading", { level: 2, name: "No upcoming match" })).toBeVisible();
  // One match: no list yet.
  await page.getByTestId("add-next-match").click();
  let dialog = page.getByRole("dialog", { name: "Add the next match" });
  await expect(dialog).toBeVisible();
  await dialog.locator("#match-date").fill(inDays(1));
  await dialog.locator("#match-time").fill("19:00");
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });
  await expect(page.locator("[data-prep]")).toBeVisible({ timeout: 10000 });
  await expect(page.getByTestId("upcoming-matches")).toHaveCount(0);

  // A second, later match (an official best of 5) through "Add a match".
  await page.getByTestId("add-another-match").click();
  dialog = page.getByRole("dialog", { name: "Add a match" });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("radiogroup", { name: "Kind of match" }).getByRole("radio", { name: "Official match" }).click();
  await dialog.getByRole("radiogroup", { name: "Format" }).getByRole("radio", { name: "Best of 5" }).click();
  await dialog.locator("#match-date").fill(inDays(3));
  await dialog.locator("#match-time").fill("18:30");
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });

  // The current match is still the first one (a scrim); the second is listed.
  await expect(page.getByRole("heading", { level: 1, name: "Scrim" })).toBeVisible({ timeout: 10000 });
  const list = page.getByTestId("upcoming-matches");
  await expect(list).toBeVisible();
  const rows = list.getByTestId("upcoming-match");
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText("Official");
  await expect(rows.first()).toContainText("Bo5");
  const href = await rows.first().getAttribute("href");
  expect(href).toMatch(/^\/\?match_id=scheduled_match:/);
  await settle(page);
  await shot(page, "match-day-upcoming-1280", false);
  await shot(page, "match-day-upcoming-1280-flow", true);

  // At 390 the row is a real touch target and the page does not scroll sideways.
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(rows.first()).toBeVisible();
  expect(await height(rows.first())).toBeGreaterThanOrEqual(44);
  await settle(page);
  await shot(page, "match-day-upcoming-390", false);
  await page.setViewportSize({ width: 1280, height: 800 });

  // The row opens that match as the current view.
  await rows.first().click();
  await page.waitForURL((u) => new URL(u).pathname === "/" && new URL(u).searchParams.has("match_id"), {
    timeout: 10000,
  });
  expect(page.url()).toContain(`match_id=`);
  await expect(page.getByRole("heading", { level: 1, name: "Official match" })).toBeVisible({ timeout: 10000 });
  await expect(page.getByTestId("back-to-next-match")).toBeVisible();
  // Now the first match is in the list instead.
  await expect(page.getByTestId("upcoming-match")).toHaveCount(1);
  await expect(page.getByTestId("upcoming-match").first()).toContainText("Scrim");

  // An id that is not on the team's schedule: the next match, said plainly.
  await navigateTo(page, "/?match_id=scheduled_match:not-ours");
  await expect(page.getByTestId("unknown-match")).toBeVisible({ timeout: 10000 });
  await expect(page.getByRole("heading", { level: 1, name: "Scrim" })).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});
