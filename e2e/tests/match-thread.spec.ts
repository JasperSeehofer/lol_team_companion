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

async function createOpponent(page: Page, name: string) {
  await navigateTo(page, "/opponents");
  await page.getByTestId("new-opponent").click();
  const dialog = page.getByRole("dialog", { name: "New opponent" });
  await dialog.locator("#opp-new-name").fill(name);
  await dialog.getByRole("button", { name: "Add opponent" }).click();
  await expect(page.getByTestId("dossier").getByRole("heading", { level: 2, name })).toBeVisible({ timeout: 10000 });
}

/**
 * M159 item 3: "Start a draft" in the match thread carries the match, the
 * opponent and the game; the draft opens prefilled (opponent chosen, name
 * "vs <opponent> · <day time> · G1") and the first save stamps the match.
 * A match id that is not the team's leaves an ordinary new draft.
 *
 * Frames: draft-from-thread-390.png, draft-from-thread-1280.png and
 * draft-from-thread-1280-flow.png when $MISSION_FRAMES_DIR is set.
 */
test("match thread: the draft opens prefilled from the thread link", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  const opp = `Rooks ${Date.now()}`;
  await createOpponent(page, opp);

  await navigateTo(page, "/");
  await page.getByTestId("add-next-match").click();
  const dialog = page.getByRole("dialog", { name: "Add the next match" });
  await expect(dialog).toBeVisible();
  await dialog.locator("#match-date").fill(inDays(1));
  await dialog.locator("#match-time").fill("19:00");
  await dialog.locator("#match-opponent").selectOption({ label: opp });
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });
  await expect(page.getByRole("heading", { level: 1, name: `Scrim vs ${opp}` })).toBeVisible({ timeout: 10000 });
  await page.getByTestId("mark-brief-read").click();
  await expect(page.getByTestId("brief-read")).toBeVisible({ timeout: 10000 });

  // The link carries the match, the opponent and game 1 (a best of 3).
  const start = page.locator("[data-prep]").getByRole("link", { name: "Start a draft" });
  await expect(start).toBeVisible({ timeout: 10000 });
  const href = (await start.getAttribute("href")) ?? "";
  expect(href).toMatch(/^\/draft\?match_id=scheduled_match:[^&]+&opponent_id=opponent:[^&]+&game=1$/);
  await start.click();
  await page.waitForURL((u) => new URL(u).pathname === "/draft", { timeout: 10000 });

  // Prefilled: the opponent is chosen and the draft is named for the match.
  await expect(page.locator("#draft-opponent")).toHaveValue(/^opponent:/, { timeout: 15000 });
  const escaped = opp.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  await expect(page.locator("#draft-name")).toHaveValue(new RegExp(`^vs ${escaped} · \\S+ \\d{2}:\\d{2} · G1$`));
  await expect(page.getByTestId("draft-thread")).toBeVisible();
  await settle(page);
  await shot(page, "draft-from-thread-1280", false);
  await shot(page, "draft-from-thread-1280-flow", true);
  await page.setViewportSize({ width: 390, height: 844 });
  await settle(page);
  await shot(page, "draft-from-thread-390", false);
  await page.setViewportSize({ width: 1280, height: 800 });

  // The first save stamps the match: Match Day counts the draft.
  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });
  await navigateTo(page, "/");
  const steps = page.locator("[data-prep] li");
  await expect(steps.nth(1)).toContainText("1 draft", { timeout: 10000 });

  expect(filterRealErrors(errors)).toHaveLength(0);

  // A match id that is not the team's: the server answers "Not found" (a
  // failed request in the console, so checked after the error capture)
  // and the page is an ordinary new draft, said plainly.
  await navigateTo(page, "/draft?match_id=scheduled_match:not-ours&game=1");
  await expect(page.getByTestId("draft-thread-unknown")).toBeVisible({ timeout: 15000 });
  await expect(page.locator("#draft-name")).toHaveValue("");
});

/**
 * M159 item 4: Match Day counts only work stamped to the match. A draft
 * made on the draft page without the thread is earlier work: shown as
 * "Earlier work vs <opponent> (1)" and not counted, until "Use for this
 * match" stamps it.
 *
 * Frames: match-day-prep-bo3-390.png and match-day-prep-bo3-1280.png when
 * $MISSION_FRAMES_DIR is set (a Bo3, the editor's default format).
 */
test("match thread: earlier work is offered, not counted, and used for this match", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  const opp = `Rooks ${Date.now()}`;
  await createOpponent(page, opp);

  // A draft against the opponent made without the match thread.
  await navigateTo(page, "/draft");
  await page.locator("#draft-opponent").selectOption({ label: opp }, { timeout: 15000 });
  await page.locator("#draft-name").fill("Old plan vs them");
  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });

  await navigateTo(page, "/");
  await page.getByTestId("add-next-match").click();
  const dialog = page.getByRole("dialog", { name: "Add the next match" });
  await expect(dialog).toBeVisible();
  await dialog.locator("#match-date").fill(inDays(1));
  await dialog.locator("#match-time").fill("19:00");
  await dialog.locator("#match-opponent").selectOption({ label: opp });
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });
  await expect(page.getByRole("heading", { level: 1, name: `Scrim vs ${opp}` })).toBeVisible({ timeout: 10000 });

  // Not counted: the draft step is still to do; the work is offered.
  const steps = page.locator("[data-prep] li");
  await expect(steps.nth(1)).toContainText("None yet", { timeout: 10000 });
  const earlier = page.locator("[data-earlier]");
  await expect(earlier).toContainText(`Earlier work vs ${opp} (1)`);
  await expect(earlier.getByRole("link", { name: /Old plan vs them/ })).toHaveAttribute("href", /^\/draft\?draft_id=draft:/);
  await settle(page);
  await shot(page, "match-day-prep-bo3-1280", true);
  await page.setViewportSize({ width: 390, height: 844 });
  await settle(page);
  await shot(page, "match-day-prep-bo3-390", true);
  await page.setViewportSize({ width: 1280, height: 800 });

  // "Use for this match" stamps it: it counts and is no longer offered.
  await earlier.getByTestId("use-for-match").click();
  await expect(page.locator("[data-earlier]")).toHaveCount(0, { timeout: 10000 });
  await expect(steps.nth(1)).toContainText("1 draft", { timeout: 10000 });

  expect(filterRealErrors(errors)).toHaveLength(0);
});
