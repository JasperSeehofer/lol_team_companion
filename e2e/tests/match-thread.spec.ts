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

/**
 * Toasts are app-wide and outlive a client-side navigation by design (a
 * "Match saved" stays its 5 s on the next page); a frame is about the page,
 * so dismiss them before shooting.
 */
async function clearToasts(page: Page) {
  const region = page.locator('section[aria-label="Notifications"]');
  const dismiss = region.getByRole("button", { name: "Dismiss" });
  for (let i = 0; i < 6 && (await dismiss.count()) > 0; i++) {
    await dismiss.first().click({ timeout: 2000 }).catch(() => {});
  }
  await expect(region.locator("li")).toHaveCount(0, { timeout: 6000 });
}

async function shot(page: Page, name: string, fullPage: boolean) {
  await clearToasts(page);
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
  // The whole page at 390, so the "For the match" line under the name shows.
  await expect(page.getByTestId("draft-thread")).toBeVisible();
  await shot(page, "draft-from-thread-390-flow", true);
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
 * For the Bo3 readout, game 2 gets a draft from the thread (so it counts as
 * played) and game 1 a review from the thread: the review step then reads
 * "G1 done · G2 to do · G3 not played" and "1 of 2 reviewed" while the match
 * is still ahead.
 *
 * Frames: match-day-prep-bo3-390.png and match-day-prep-bo3-1280.png when
 * $MISSION_FRAMES_DIR is set (a Bo3, the editor's default format): the
 * readout and the earlier work in one state.
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

  // Game 2 played (a draft from the thread), game 1 reviewed (from the thread).
  const draftHref = (await page.locator('[data-prep] a[href^="/draft?match_id="]').first().getAttribute("href")) ?? "";
  expect(draftHref).toMatch(/&game=1$/);
  const matchId = new URL(draftHref, "http://local").searchParams.get("match_id") ?? "";
  expect(matchId).toMatch(/^scheduled_match:/);
  await navigateTo(page, draftHref.replace(/&game=1$/, "&game=2"));
  await expect(page.getByTestId("draft-thread")).toBeVisible({ timeout: 15000 });
  await expect(page.locator("#draft-name")).toHaveValue(/ · G2$/, { timeout: 15000 });
  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });

  await navigateTo(page, `/post-game?match_id=${matchId}&game=1`);
  await expect(page.getByTestId("review-thread")).toContainText("game 1", { timeout: 15000 });
  const custom = page.getByTestId("review-game-picker").getByRole("radio", { name: "Not in the list (scrim / custom)" });
  await expect(async () => {
    await custom.check();
    await expect(custom).toBeChecked({ timeout: 1000 });
  }).toPass({ timeout: 15000 });
  await page.locator("#review-well").fill("Game 1: early dragon setups worked");
  await page.getByTestId("review-save").click();
  await expect(page.getByTestId("review-save-state")).toHaveText("Saved", { timeout: 10000 });

  await navigateTo(page, "/");
  await expect(steps.nth(1)).toContainText("1 draft", { timeout: 10000 });
  await expect(steps.nth(4)).toContainText("G1 done \u00b7 G2 to do \u00b7 G3 not played", { timeout: 10000 });
  await expect(steps.nth(4)).toContainText("1 of 2 reviewed");
  await expect(earlier).toContainText(`Earlier work vs ${opp} (1)`);
  await settle(page);
  await shot(page, "match-day-prep-bo3-1280", true);
  await page.setViewportSize({ width: 390, height: 844 });
  await settle(page);
  await shot(page, "match-day-prep-bo3-390", true);
  await page.setViewportSize({ width: 1280, height: 800 });

  // "Use for this match" stamps it: it counts and is no longer offered.
  await earlier.getByTestId("use-for-match").click();
  await expect(page.locator("[data-earlier]")).toHaveCount(0, { timeout: 10000 });
  await expect(steps.nth(1)).toContainText("2 drafts", { timeout: 10000 });

  expect(filterRealErrors(errors)).toHaveLength(0);
});

/**
 * M159 item 5 (atelier card 2a): the review's game picker. A fresh team
 * has no synced games (CI never syncs from Riot), so the picker says so
 * honestly and "Not in the list (scrim / custom)" stays usable: a review
 * saves through it without a game. A `riot_match_id` in the address that
 * is not one of the team's games preselects nothing.
 */
test("review picker: a scrim review saves through Not in the list", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await navigateTo(page, "/post-game?riot_match_id=EUW1_7000000001");
  const picker = page.getByTestId("review-game-picker");
  await expect(picker.getByRole("radiogroup", { name: "Which game is this review of?" })).toBeVisible({ timeout: 10000 });
  await expect(picker).toContainText("No synced game where three or more of your members played together yet");
  // The foreign id is ignored: no game row, nothing chosen, never shown.
  await expect(picker.getByTestId("review-game")).toHaveCount(0);
  await expect(picker.getByTestId("review-game-stored")).toHaveCount(0);
  await expect(picker.locator("input[type=radio]:checked")).toHaveCount(0);
  await expect(page.locator("main")).not.toContainText("EUW1_7000000001");
  // No <select> for the game any more.
  await expect(page.locator("#review-match")).toHaveCount(0);

  const custom = picker.getByRole("radio", { name: "Not in the list (scrim / custom)" });
  await expect(async () => {
    await custom.check();
    await expect(custom).toBeChecked({ timeout: 1000 });
  }).toPass({ timeout: 15000 });
  await page.locator("#review-well").fill("Early dragon setups in the scrim");
  await page.getByTestId("review-save").click();
  await expect(page.getByTestId("review-save-state")).toHaveText("Saved", { timeout: 10000 });
  await expect(page).toHaveURL(/\/post-game\?review_id=post_game_learning:/);
  // The saved review keeps "Not in the list" as its choice.
  await expect(custom).toBeChecked();
  expect(filterRealErrors(errors)).toHaveLength(0);
});

const DEMO_GAME = "SEED1_1000000001";
/** The seed's second game, played a few hours before the seed ran. */
const RECENT_GAME = "SEED1_1000000002";

/** The viewer's local date and time `hours` from now, as the editor takes them. */
async function localInputs(page: Page, hours: number): Promise<{ date: string; time: string }> {
  return page.evaluate((h) => {
    const d = new Date(Date.now() + h * 3600 * 1000);
    const pad = (n: number) => String(n).padStart(2, "0");
    return {
      date: `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`,
      time: `${pad(d.getHours())}:${pad(d.getMinutes())}`,
    };
  }, hours);
}

async function signInSeedTop(page: Page): Promise<boolean> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", "seed-alpha-top@example.com");
  await page.fill("input[name=password]", "seedpass123");
  await page.click("main button[type=submit]");
  return page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 15000 })
    .then(() => true)
    .catch(() => false);
}

/**
 * M159 item 5 with the seed tool's demo game (all five Northgate members
 * synced it): the picker row reads date, champions and the result in words,
 * never the Riot id; "Review as a team" on the match page and on Team games
 * opens the review with the game chosen. CI seeds the database; locally the
 * test is skipped without the seed.
 *
 * The ±6 h lift: the leader records a match that started two hours ago
 * (Match Day accepts a start up to twelve hours back) and opens the review
 * from it; the seed's second game (ended about three hours before the seed
 * ran, so within six hours of that start in CI) is "Near the scheduled
 * match", the day-old demo game under "Other recent games". The match is
 * removed again at the end, so the seeded Match Day is unchanged for the
 * specs after this one.
 *
 * Frames: post-game-picker-390.png, post-game-picker-1280.png,
 * post-game-picker-1280-flow.png, match-detail-review-1280.png and
 * match-detail-review-390.png when $MISSION_FRAMES_DIR is set.
 */
test("review picker: a team game is chosen from the match page", async ({ page }) => {
  const ok = await signInSeedTop(page);
  if (process.env.CI) expect(ok, "CI must seed the database before starting the server").toBe(true);
  test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
  await page.waitForTimeout(500);
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 1280, height: 800 });

  // The match page offers the team review (the game is the team's).
  await navigateTo(page, `/match/${DEMO_GAME}`);
  const action = page.getByTestId("review-as-team");
  await expect(action).toBeVisible({ timeout: 10000 });
  await expect(action).toHaveText(/^(Review as a team|Open team review)$/);
  await expect(action).toHaveAttribute("href", `/post-game?riot_match_id=${DEMO_GAME}`);
  await settle(page);
  await shot(page, "match-detail-review-1280", false);
  await page.setViewportSize({ width: 390, height: 844 });
  await settle(page);
  await shot(page, "match-detail-review-390", false);
  await page.setViewportSize({ width: 1280, height: 800 });

  // Team games rows carry it too, named with the game.
  await navigateTo(page, "/stats");
  await expect(page.getByTestId("team-game-review").first()).toBeVisible({ timeout: 10000 });

  // Following it chooses the game; the row never shows the Riot id.
  await navigateTo(page, `/post-game?riot_match_id=${DEMO_GAME}`);
  const picker = page.getByTestId("review-game-picker");
  const row = picker.getByTestId("review-game").filter({ has: page.locator(`input[value="${DEMO_GAME}"]`) });
  await expect(row).toHaveCount(1, { timeout: 10000 });
  await expect(row.locator("input[type=radio]")).toBeChecked({ timeout: 10000 });
  await expect(row).toContainText("Win");
  // Our five champions (named for screen readers beside the avatars).
  await expect(row).toContainText("Darius");
  await expect(row).toContainText("Thresh");
  await expect(page.locator("main")).not.toContainText(DEMO_GAME);
  await expect(picker.getByRole("radio", { name: "Not in the list (scrim / custom)" })).toBeVisible();
  await expect(page.getByTestId("review-match-facts")).toBeVisible();

  // A match that started two hours ago, recorded on Match Day.
  await navigateTo(page, "/");
  await page.getByTestId("add-another-match").click();
  const dialog = page.getByRole("dialog", { name: "Add a match" });
  await expect(dialog).toBeVisible();
  const at = await localInputs(page, -2);
  await dialog.locator("#match-date").fill(at.date);
  await dialog.locator("#match-time").fill(at.time);
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });
  try {
    // It started, so it is the next match and its review step links on.
    const reviewLink = page.locator('[data-prep] a[href^="/post-game?"][href*="match_id=scheduled_match:"]').first();
    await expect(reviewLink).toBeVisible({ timeout: 10000 });
    const matchId = new URL((await reviewLink.getAttribute("href")) ?? "", "http://local").searchParams.get("match_id") ?? "";
    expect(matchId).toMatch(/^scheduled_match:/);

    await navigateTo(page, `/post-game?match_id=${matchId}`);
    const picker2 = page.getByTestId("review-game-picker");
    await expect(picker2.locator(`input[value="${DEMO_GAME}"]`)).toHaveCount(1, { timeout: 10000 });
    const near = picker2.getByText("Near the scheduled match", { exact: true });
    const other = picker2.getByText("Other recent games", { exact: true });
    if (process.env.CI) {
      // CI seeds right before the suite: the recent game is within six hours.
      await expect(near).toBeVisible();
      await expect(other).toBeVisible();
      const rows = picker2.getByTestId("review-game");
      await expect(rows.first().locator("input[type=radio]")).toHaveValue(RECENT_GAME);
      await expect(rows.nth(1).locator("input[type=radio]")).toHaveValue(DEMO_GAME);
    }
    // Champions are named in words for screen readers (the avatars are decorative).
    await expect(picker2.getByTestId("review-game").first()).toContainText("Darius");
    await settle(page);
    await shot(page, "post-game-picker-1280", false);
    await shot(page, "post-game-picker-1280-flow", true);
    await page.setViewportSize({ width: 390, height: 844 });
    await settle(page);
    await shot(page, "post-game-picker-390", true);
    await page.setViewportSize({ width: 1280, height: 800 });
  } finally {
    // Remove the match again: later specs read the seeded Match Day.
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/");
    await page.getByTestId("edit-next-match").click();
    await expect(page.getByRole("dialog", { name: "Edit the next match" })).toBeVisible();
    await page.getByRole("dialog", { name: "Edit the next match" }).getByRole("button", { name: "Remove match" }).click();
    const confirm = page.getByRole("dialog", { name: "Remove this match?" });
    await confirm.getByRole("button", { name: "Remove match" }).click();
    await expect(confirm).not.toBeVisible({ timeout: 10000 });
  }
  expect(filterRealErrors(errors)).toHaveLength(0);
});
