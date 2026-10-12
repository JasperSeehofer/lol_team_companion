/**
 * M9 guidance: the welcome on first sign-in (Skip mutes it, the checklist
 * stays), the getting-started checklist ticking itself from real records,
 * a first-use hint that shows once and not after "Got it", the profile
 * switch hiding everything, the command bar's Guide me, and, for the
 * seeded returning user (`cargo run --features ssr --bin seed` before the
 * server starts), the returning-user line and a "New" mark that clears.
 * Requires a running dev server started with the e2e env (CLAUDE.md).
 */
import { test, expect } from "./fixtures";
import { captureErrors, createDraft, filterRealErrors, navigateTo } from "./helpers";
import type { Page } from "@playwright/test";

const SHOTS = process.env.M9_SHOTS_DIR;

async function shot(page: Page, name: string) {
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/${name}.png`, fullPage: true });
}

test("first sign-in shows the welcome once; Skip keeps the checklist", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);
  await navigateTo(page, "/");
  const welcome = page.locator("[data-welcome]");
  await expect(welcome).toBeVisible();
  await expect(welcome.getByRole("heading", { level: 2 })).toContainText("Welcome to Companion");
  await expect(welcome.getByRole("list", { name: "The loop" })).toContainText("Review");
  await expect(page.getByTestId("welcome-start")).toHaveAttribute("href", "/team/roster");
  const checklist = page.locator("[data-checklist]");
  await expect(checklist).toBeVisible();
  await expect(checklist).toContainText("0 of 7 done");
  await shot(page, "welcome-1280");

  await page.getByTestId("welcome-skip").click();
  await expect(welcome).toHaveCount(0, { timeout: 10000 });
  await expect(checklist).toBeVisible();
  // Muted for good: not back after a reload.
  await navigateTo(page, "/");
  await expect(page.locator("[data-welcome]")).toHaveCount(0);
  await expect(page.locator("[data-checklist]")).toBeVisible();
  expect(filterRealErrors(errors)).toEqual([]);
});

test("the checklist ticks after creating a team and a draft, and hides on request", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/");
  const checklist = page.locator("[data-checklist]");
  await expect(checklist).toContainText("1 of 7 done");
  // The next open step is the starting five, with a link to the team page.
  const current = checklist.locator('li[aria-current="step"]');
  await expect(current).toContainText("Fill the starting five");

  await createDraft(page, `Guide draft ${Date.now()}`);
  await navigateTo(page, "/");
  await expect(checklist).toContainText("2 of 7 done");
  await expect(checklist.locator("li", { hasText: "Make a draft" })).toContainText("1 draft");
  await shot(page, "checklist-ticked-1280");

  await page.getByTestId("hide-checklist").click();
  await expect(page.locator("[data-checklist]")).toHaveCount(0, { timeout: 10000 });
  await expect(page.getByText("Checklist hidden.")).toBeVisible();
  // Guide me (command bar) brings it back for this visit.
  await page.getByTestId("command-trigger").click();
  await page.getByRole("combobox").fill("guide me");
  await page.keyboard.press("Enter");
  await expect(page.locator("[data-checklist]")).toBeVisible({ timeout: 10000 });
  await expect(page.locator("[data-checklist]")).toContainText("Shown because you asked for it");
});

test("a hint shows once on the draft board and not after Got it", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/draft");
  const hint = page.locator('[data-guidance-hint="hint.draft_keys"]');
  await expect(hint).toBeVisible({ timeout: 10000 });
  await expect(hint).toContainText("The board works from the keyboard");
  // Never more than one hint on a page.
  await expect(page.locator("[data-guidance-hint]")).toHaveCount(1);
  await shot(page, "hint-draft-keys-1280");
  await hint.getByTestId("hint-got-it").click();
  await expect(hint).toHaveCount(0);
  await navigateTo(page, "/draft");
  await page.waitForTimeout(1000);
  await expect(page.locator('[data-guidance-hint="hint.draft_keys"]')).toHaveCount(0);
});

test("the profile switch hides every piece of guidance", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/profile");
  const section = page.locator("[data-guidance-settings]");
  await expect(section).toBeVisible();
  await shot(page, "profile-guidance-1280");
  const sw = section.getByRole("switch", { name: "Show guidance" });
  await expect(sw).toHaveAttribute("aria-checked", "true");
  await sw.click();
  await expect(sw).toHaveAttribute("aria-checked", "false");
  await expect(page.getByText("Guidance is off:")).toBeVisible();

  await navigateTo(page, "/");
  await expect(page.locator("[data-welcome]")).toHaveCount(0);
  await expect(page.locator("[data-checklist]")).toHaveCount(0);
  await expect(page.locator("[data-nudge]")).toHaveCount(0);
  await navigateTo(page, "/draft");
  await page.waitForTimeout(1000);
  await expect(page.locator("[data-guidance-hint]")).toHaveCount(0);
  await navigateTo(page, "/stats");
  await page.waitForTimeout(1000);
  await expect(page.locator('[data-guidance-hint="hint.command_bar"]')).toHaveCount(0);

  // On again: the hint is back.
  await navigateTo(page, "/profile");
  await page.locator("[data-guidance-settings]").getByRole("switch", { name: "Show guidance" }).click();
  await expect(page.getByText("Guidance is on.")).toBeVisible();
  await navigateTo(page, "/draft");
  await expect(page.locator('[data-guidance-hint="hint.draft_keys"]')).toBeVisible({ timeout: 10000 });
});

test("admin pages get no guidance", async ({ adminPage }) => {
  const page = adminPage;
  await navigateTo(page, "/admin");
  await page.waitForTimeout(1000);
  await expect(page.locator("[data-guidance-hint], [data-guidance-intro], [data-welcome]")).toHaveCount(0);
});

const RETURNING = "seed-beta-sup@example.com";

async function signInSeeded(page: Page): Promise<boolean> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", RETURNING);
  await page.fill("input[name=password]", "seedpass123");
  await page.click("main button[type=submit]");
  return page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 15000 })
    .then(() => true)
    .catch(() => false);
}

test.describe("seeded returning user", () => {
  test.describe.configure({ mode: "serial" });

  test("the returning-user line shows; a New mark appears and clears", async ({ page }) => {
    const ok = await signInSeeded(page);
    test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
    await page.waitForTimeout(500);
    await navigateTo(page, "/");
    // An account older than the welcome window gets no welcome.
    await expect(page.locator("[data-welcome]")).toHaveCount(0);
    const nudge = page.locator('[data-nudge="nudge.returning"]');
    await expect(nudge).toContainText("Picking up again?");
    await expect(nudge.getByRole("link", { name: "Open the checklist" })).toHaveAttribute("href", "/?guide=1#getting-started");
    await shot(page, "nudge-returning-1280");

    // The account predates the M9 entry: Profile carries a New mark until
    // the page is visited (desktop: the You hub and its sub-navigation).
    const mark = page.locator('[data-new-mark="/profile"], [data-new-hub="you"]');
    const intro = page.locator('[data-guidance-intro="/profile"]');
    if ((await mark.count()) > 0) {
      await shot(page, "new-mark-1280");
      await navigateTo(page, "/profile");
      await expect(intro).toBeVisible({ timeout: 10000 });
      await expect(intro).toContainText("New: Guidance");
      // Nothing else guides beside it (the command-bar line waits).
      await expect(page.locator("[data-guidance-hint]")).toHaveCount(0);
      await shot(page, "intro-profile-1280");
      await navigateTo(page, "/me");
      await page.waitForTimeout(1000);
      await expect(page.locator('[data-new-mark="/profile"]')).toHaveCount(0);
      await navigateTo(page, "/profile");
      await page.waitForTimeout(1000);
      await expect(intro).toHaveCount(0);
    } else {
      // Seen in an earlier run: nothing marks it any more.
      await navigateTo(page, "/profile");
      await page.waitForTimeout(1000);
      await expect(intro).toHaveCount(0);
    }

  });
});
