/**
 * M150 item 5: suggestion chips on /game-plan.
 *
 * A suggestions section sits between Matchup and Strategy. The server
 * (`plan_suggestions`, over the trait rules only) answers once both sides
 * have five champions it knows; the page shows at most three, collapsed
 * behind one "N suggestions" line. A chip opens its card (inline at 1280, a
 * bottom sheet at 390) with "Add to plan" (writes the claim as a plain line
 * in Win conditions or Notes) and "Not for us" (dismissed on this plan,
 * saved with it).
 *
 * The lineup (champion ids that are also their Data Dragon display names,
 * so the names in the reasons match what was typed, and the trait table in
 * `src/server/graph/champion_traits.csv` resolves them without Riot):
 *
 * - Ours: Malphite (top), Amumu (jungle), Orianna (mid), Jinx (bot),
 *   Leona (support). Malphite, Amumu, Orianna and Leona all carry the
 *   `wombo` style, so `syn.wombo` (weight 3) holds for all six of their
 *   pairs; Amumu's and Leona's hard engage with Malphite's burst also gives
 *   `syn.engage_burst` (weight 3). That is at least six strength-3 synergy
 *   suggestions on six different pairs, and strength-3 claims come only
 *   from synergies (counters weigh at most 2, composition claims 2), so the
 *   three shown are pair claims whose reasons name two of our picks, and
 *   they land in Win conditions (synergies target Win conditions).
 * - Theirs: Gnar (top), Nidalee (jungle), Xerath (mid), Ezreal (bot),
 *   Janna (support): no hard engage and little front line (composition
 *   claims), poke into our engage, Janna's strong disengage undoing our
 *   engage (`ctr.engage_denied`, Notes): plenty more behind the first three,
 *   so dismissing one brings the next.
 *
 * Frames: PNGs prefixed `m150-` go to the top level of $MISSION_FRAMES_DIR
 * (CI commits them as frames/after). Before frames: CI renders only this
 * branch, so there is no `before/` set here; the before strip comes from a
 * run on main (the section does not exist there). When the variable is
 * unset the frames test is a smoke test of the same states.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { captureErrors, filterRealErrors, navigateTo, saveGamePlan } from "./helpers";

const OURS = ["Malphite", "Amumu", "Orianna", "Jinx", "Leona"];
const THEIRS = ["Gnar", "Nidalee", "Xerath", "Ezreal", "Janna"];
const ROLES = ["top", "jungle", "mid", "bot", "support"];
const ALL_PICKS = [...OURS, ...THEIRS];
const WAIT_TEXT = "Suggestions appear when both drafts are in.";

const DIR = process.env.MISSION_FRAMES_DIR;
const PHONE = { width: 390, height: 844 };
const DESK = { width: 1280, height: 800 };

// ---------------------------------------------------------------------------
// Helpers

const section = (page: Page) => page.getByTestId("plan-suggestions");
const toggle = (page: Page) => page.getByTestId("suggestions-toggle");
const chips = (page: Page) => page.locator('button[data-testid="suggestion-chip"][data-key]');
/** The open card (at most one is visible at a time). */
const card = (page: Page) => page.getByTestId("suggestion-card").filter({ visible: true });

/**
 * Type one side's five picks. The autocomplete's value follows the typed
 * text (a champion id the server resolves), so no list choice is needed;
 * Tab closes the list. (Escape would also clear a search input natively.)
 */
async function fillSide(page: Page, side: "ours" | "theirs", names: string[]) {
  for (let i = 0; i < ROLES.length; i++) {
    const input = page.locator(`#plan-${side}-${ROLES[i]}`);
    await input.fill(names[i]);
    await input.press("Tab");
    await expect(input).toHaveValue(names[i]);
  }
}

/** A fresh plan with both drafts in, saved (the URL then has ?plan_id=). */
async function fullPlan(page: Page, name: string) {
  await navigateTo(page, "/game-plan");
  await fillSide(page, "ours", OURS);
  await fillSide(page, "theirs", THEIRS);
  await saveGamePlan(page, name);
  await expect(page).toHaveURL(/plan_id=/);
}

async function reloadSettled(page: Page) {
  await page.reload();
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
}

/** Open the collapsed suggestions line; returns the shown chips' keys. */
async function openSuggestions(page: Page): Promise<string[]> {
  const t = toggle(page);
  await expect(t).toBeVisible({ timeout: 15000 });
  if ((await t.getAttribute("aria-expanded")) !== "true") await t.click();
  await expect(t).toHaveAttribute("aria-expanded", "true");
  await expect(chips(page).first()).toBeVisible();
  return keys(page);
}

async function keys(page: Page): Promise<string[]> {
  return chips(page).evaluateAll((els) => els.map((e) => e.getAttribute("data-key") ?? ""));
}

const chipByKey = (page: Page, key: string) =>
  page.locator(`button[data-testid="suggestion-chip"][data-key="${key}"]`);

/** Open a chip's card. */
async function openCard(page: Page, chip: Locator): Promise<Locator> {
  await chip.click();
  const c = card(page);
  await expect(c).toBeVisible();
  return c;
}

/**
 * Run `action` and wait for the autosave it causes (2 s debounce) to reach
 * the server and the header to read "Saved" again.
 */
async function autosaved(page: Page, action: () => Promise<void>) {
  const saved = page.waitForResponse((r) => r.url().includes("/api/update_plan") && r.ok(), {
    timeout: 15000,
  });
  await action();
  await saved;
  await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 10000 });
}

/** The lines of a textarea's value that were not there before. */
function newLines(before: string, after: string): string[] {
  const old = new Set(before.split("\n").map((l) => l.trim().toLowerCase()));
  return after
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !old.has(l.toLowerCase()));
}

/** Every 44 px tap target rule: at least 44 px tall (half a pixel of rounding). */
async function expectTall(target: Locator, what: string) {
  const box = await target.boundingBox();
  expect(box, `${what} has a box`).not.toBeNull();
  expect(box!.height, `${what} is ${box!.height}px tall`).toBeGreaterThanOrEqual(43.5);
}

async function shot(page: Page, name: string, fullPage = false) {
  const sw = await page.evaluate(() => document.documentElement.scrollWidth);
  const vw = page.viewportSize()?.width ?? 0;
  expect(sw, `${name}: ${sw}px wide in a ${vw}px viewport`).toBeLessThanOrEqual(vw);
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.screenshot({ path: path.join(DIR, `m150-${name}.png`), fullPage });
}

/** Scroll the suggestions section to the top of the viewport. */
async function frameSection(page: Page) {
  await section(page).evaluate((el) => el.scrollIntoView({ block: "start" }));
  await page.evaluate(() => window.scrollBy(0, -80));
  await page.waitForTimeout(200);
}

// ---------------------------------------------------------------------------
// Behaviour

test.describe("M150 suggestion chips", () => {
  test.setTimeout(120_000);

  test("full plan: collapsed line, at most three chips naming the picks, same order after reload", async ({
    teamPage: page,
  }) => {
    const errors = captureErrors(page);
    await fullPlan(page, `Chips_${Date.now()}`);

    await expect(section(page)).toBeVisible();
    const t = toggle(page);
    await expect(t).toBeVisible({ timeout: 15000 });
    await expect(t).toHaveAttribute("aria-expanded", "false");
    await expect(t).toContainText(/\b[1-3] suggestions?\b/);
    // Collapsed by default: no chip is shown until the line is opened.
    await expect(chips(page).first()).toBeHidden();
    await expect(page.getByTestId("suggestions-wait")).toHaveCount(0);

    const first = await openSuggestions(page);
    expect(first.length).toBeGreaterThanOrEqual(1);
    expect(first.length).toBeLessThanOrEqual(3);
    await expect(t).toContainText(`${first.length} suggestion`);

    // Each card's reason names a champion from the picks.
    for (const key of first) {
      const c = await openCard(page, chipByKey(page, key));
      const text = (await c.textContent()) ?? "";
      expect(
        ALL_PICKS.some((n) => text.includes(n)),
        `card for ${key} names a pick: ${text}`
      ).toBe(true);
      await expect(c.getByRole("button", { name: "Add to plan" })).toBeVisible();
      await expect(c.getByRole("button", { name: "Not for us" })).toBeVisible();
    }

    // Determinism: the same picks give the same chips in the same order.
    await reloadSettled(page);
    await expect(toggle(page)).toHaveAttribute("aria-expanded", "false", { timeout: 15000 });
    expect(await openSuggestions(page)).toEqual(first);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("one side empty: the wait line, no chips and no toggle", async ({ teamPage: page }) => {
    const errors = captureErrors(page);
    await navigateTo(page, "/game-plan");
    await fillSide(page, "ours", OURS);

    const wait = page.getByTestId("suggestions-wait");
    await expect(wait).toBeVisible({ timeout: 15000 });
    await expect(wait).toHaveText(WAIT_TEXT);
    // Give a late answer the chance to appear before asserting it does not.
    await page.waitForTimeout(1500);
    await expect(toggle(page)).toHaveCount(0);
    await expect(chips(page)).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("accept writes the claim as an editable line and retires the chip", async ({ teamPage: page }) => {
    const errors = captureErrors(page);
    await fullPlan(page, `Accept_${Date.now()}`);
    const [key] = await openSuggestions(page);
    const chip = chipByKey(page, key);
    const chipText = ((await chip.textContent()) ?? "").toLowerCase();

    const win = page.locator("#plan-win");
    const notes = page.locator("#plan-notes");
    const winBefore = await win.inputValue();
    const notesBefore = await notes.inputValue();

    const c = await openCard(page, chip);
    await autosaved(page, () => c.getByRole("button", { name: "Add to plan" }).click());

    // Exactly one new line, in one of the two fields, and it is the claim
    // the chip showed (the chip text may carry more, such as its strength).
    const added = [
      ...newLines(winBefore, await win.inputValue()).map((l) => ({ field: win, line: l })),
      ...newLines(notesBefore, await notes.inputValue()).map((l) => ({ field: notes, line: l })),
    ];
    expect(added, "one line added").toHaveLength(1);
    const { field, line } = added[0];
    expect(chipText).toContain(line.toLowerCase());
    await expect(chipByKey(page, key)).toHaveCount(0);

    // The line is plain text the coach owns: append to it.
    await field.focus();
    await field.press("End");
    await page.keyboard.press("Control+End");
    await page.keyboard.type("\nOur own line");
    await expect(field).toHaveValue(new RegExp(`${escapeRe(line)}[\\s\\S]*Our own line`));
    // Edited, not marked: the claim still stands as its own line.
    const lines = (await field.inputValue()).split("\n").map((l) => l.trim());
    expect(lines).toContain(line);

    // Stored: after the autosave and a reload, the line is there and the
    // chip stays retired.
    await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 15000 });
    await reloadSettled(page);
    await expect(field).toHaveValue(new RegExp(escapeRe(line)), { timeout: 15000 });
    await openSuggestions(page);
    await expect(chipByKey(page, key)).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("dismiss persists across reload; dismissing all leaves the none line", async ({ teamPage: page }) => {
    const errors = captureErrors(page);
    await fullPlan(page, `Dismiss_${Date.now()}`);
    const [key] = await openSuggestions(page);

    let c = await openCard(page, chipByKey(page, key));
    await autosaved(page, () => c.getByRole("button", { name: "Not for us" }).click());
    await expect(chipByKey(page, key)).toHaveCount(0);

    await reloadSettled(page);
    await openSuggestions(page);
    await expect(chipByKey(page, key)).toHaveCount(0);

    // Dismiss the rest (the server sends at most twelve).
    const gone = [key];
    for (let i = 0; i < 16; i++) {
      if ((await chips(page).count()) === 0) break;
      const next = (await keys(page))[0];
      c = await openCard(page, chipByKey(page, next));
      await c.getByRole("button", { name: "Not for us" }).click();
      await expect(chipByKey(page, next)).toHaveCount(0);
      gone.push(next);
    }
    await expect(chips(page)).toHaveCount(0);
    expect(gone.length).toBeGreaterThanOrEqual(3);
    // Let the last dismissal save.
    await page.waitForTimeout(2500);
    await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 15000 });

    await reloadSettled(page);
    const none = page.getByTestId("suggestions-none");
    // The none line may stand alone or sit behind the line: open it if so.
    await expect(none.or(toggle(page))).toBeVisible({ timeout: 15000 });
    if (!(await none.isVisible()) && (await toggle(page).isVisible())) await toggle(page).click();
    await expect(none).toBeVisible();
    await expect(chips(page)).toHaveCount(0);
    await expect(page.getByTestId("suggestions-wait")).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("no Riot or Data Dragon request from the browser in the path", async ({ teamPage: page }) => {
    const urls: string[] = [];
    page.on("request", (r) => urls.push(r.url()));
    const base = new URL(page.url());

    await fullPlan(page, `NoRiot_${Date.now()}`);
    const [key] = await openSuggestions(page);
    const c = await openCard(page, chipByKey(page, key));
    await autosaved(page, () => c.getByRole("button", { name: "Add to plan" }).click());
    await reloadSettled(page);
    await openSuggestions(page);

    const web = urls.filter((u) => /^https?:/.test(u));
    expect(web.length).toBeGreaterThan(0);
    expect(web.filter((u) => /riotgames|ddragon/i.test(u))).toEqual([]);
    expect(web.filter((u) => new URL(u).hostname !== base.hostname)).toEqual([]);
  });

  test("phone 390: the card opens as a bottom sheet with 44 px targets", async ({ teamPage: page }) => {
    const errors = captureErrors(page);
    await page.setViewportSize(PHONE);
    await fullPlan(page, `Phone_${Date.now()}`);

    await expectTall(toggle(page), "the suggestions line");
    const [key] = await openSuggestions(page);
    const chip = chipByKey(page, key);
    await expectTall(chip, "a chip");

    const c = await openCard(page, chip);
    const inSheet = await c.evaluate((el) => el.closest("dialog[open]") !== null);
    expect(inSheet, "the card is inside an open dialog").toBe(true);
    await expectTall(c.getByRole("button", { name: "Add to plan" }), "Add to plan");
    await expectTall(c.getByRole("button", { name: "Not for us" }), "Not for us");

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("reduced motion: the accept echo is gone within 400 ms", async ({ teamPage: page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await fullPlan(page, `Reduced_${Date.now()}`);
    const [key] = await openSuggestions(page);
    const c = await openCard(page, chipByKey(page, key));
    await c.getByRole("button", { name: "Add to plan" }).click();
    await expect(page.getByTestId("suggestion-echo")).toHaveCount(0, { timeout: 400 });
    await expect(chipByKey(page, key)).toHaveCount(0);
  });

  // -------------------------------------------------------------------------
  // Frames (390 and 1280)

  test("frames", async ({ teamPage: page }) => {
    test.setTimeout(240_000);
    const errors = captureErrors(page);

    // The wait line: one side in, on a plan not saved yet.
    await page.setViewportSize(DESK);
    await navigateTo(page, "/game-plan");
    await fillSide(page, "ours", OURS);
    await expect(page.getByTestId("suggestions-wait")).toBeVisible({ timeout: 15000 });
    await frameSection(page);
    await shot(page, "wait-line-1280");
    await page.setViewportSize(PHONE);
    await page.waitForTimeout(300);
    await frameSection(page);
    await shot(page, "wait-line-390");

    // Both drafts in, saved.
    await page.setViewportSize(DESK);
    await fillSide(page, "theirs", THEIRS);
    await saveGamePlan(page, `Frames_${Date.now()}`);
    await expect(page).toHaveURL(/plan_id=/);

    // 390: reading flow (opened, nothing expanded), chips, the sheet.
    await page.setViewportSize(PHONE);
    await reloadSettled(page);
    await expect(toggle(page)).toHaveAttribute("aria-expanded", "false", { timeout: 15000 });
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(300);
    await shot(page, "reading-flow-390", true);
    let [key] = await openSuggestions(page);
    await frameSection(page);
    await shot(page, "chips-open-390");
    let c = await openCard(page, chipByKey(page, key));
    await page.waitForTimeout(400);
    await shot(page, "sheet-390");
    await page.keyboard.press("Escape");
    await expect(c).toBeHidden();

    // 1280: the same, the card inline.
    await page.setViewportSize(DESK);
    await reloadSettled(page);
    await expect(toggle(page)).toHaveAttribute("aria-expanded", "false", { timeout: 15000 });
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(300);
    await shot(page, "reading-flow-1280", true);
    [key] = await openSuggestions(page);
    await frameSection(page);
    await shot(page, "chips-open-1280");
    c = await openCard(page, chipByKey(page, key));
    await page.waitForTimeout(300);
    await frameSection(page);
    await shot(page, "card-1280");

    // The accept strip at 1280: chip, mid-slide (~120 ms), settled (~400 ms).
    // The first chip is a strength-3 synergy, so it lands in Win conditions.
    await shot(page, "accept-strip-1-chip-1280");
    const win = page.locator("#plan-win");
    const winBefore = await win.inputValue();
    const t0 = Date.now();
    await c.getByRole("button", { name: "Add to plan" }).click();
    await page.waitForTimeout(Math.max(0, 120 - (Date.now() - t0)));
    await shot(page, "accept-strip-2-mid-1280");
    await page.waitForTimeout(Math.max(0, 400 - (Date.now() - t0)));
    await shot(page, "accept-strip-3-settled-1280");
    await expect(page.getByTestId("suggestion-echo")).toHaveCount(0, { timeout: 2000 });
    const added = newLines(winBefore, await win.inputValue());
    expect(added, "the accepted line is in Win conditions").toHaveLength(1);
    await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 15000 });

    // The accepted line in Win conditions, both widths.
    await win.evaluate((el) => el.scrollIntoView({ block: "center" }));
    await page.waitForTimeout(200);
    await shot(page, "accepted-win-1280");
    await page.setViewportSize(PHONE);
    await reloadSettled(page);
    await expect(win).toHaveValue(new RegExp(escapeRe(added[0])), { timeout: 15000 });
    await win.evaluate((el) => el.scrollIntoView({ block: "center" }));
    await page.waitForTimeout(200);
    await shot(page, "accepted-win-390");

    // The same strip with reduced motion: no slide, the line just appears.
    await page.setViewportSize(DESK);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await reloadSettled(page);
    [key] = await openSuggestions(page);
    c = await openCard(page, chipByKey(page, key));
    await frameSection(page);
    await shot(page, "accept-strip-reduced-1-chip-1280");
    const t1 = Date.now();
    await c.getByRole("button", { name: "Add to plan" }).click();
    await page.waitForTimeout(Math.max(0, 120 - (Date.now() - t1)));
    await shot(page, "accept-strip-reduced-2-mid-1280");
    await page.waitForTimeout(Math.max(0, 400 - (Date.now() - t1)));
    await shot(page, "accept-strip-reduced-3-settled-1280");
    await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 15000 });

    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});

function escapeRe(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
