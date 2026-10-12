/**
 * M5: Match Day (next match editor, the preparation sequence advancing),
 * share links that open without signing in and show nothing else, the
 * player card, and the phone menu at 400 px.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

function tomorrow(): string {
  const d = new Date(Date.now() + 24 * 3600 * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

async function createOpponent(page: import("@playwright/test").Page, name: string) {
  await navigateTo(page, "/opponents");
  await page.getByTestId("new-opponent").click();
  const dialog = page.getByRole("dialog", { name: "New opponent" });
  await dialog.locator("#opp-new-name").fill(name);
  await dialog.getByRole("button", { name: "Add opponent" }).click();
  await expect(page.getByTestId("dossier").getByRole("heading", { level: 2, name })).toBeVisible({ timeout: 10000 });
}

test("match day: empty, then the next match starts the sequence", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const opp = `Rooks ${Date.now()}`;
  await createOpponent(page, opp);

  await navigateTo(page, "/");
  await expect(page.getByRole("heading", { level: 1, name: "Match Day" })).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "No upcoming match" })).toBeVisible();

  await page.getByTestId("add-next-match").click();
  const dialog = page.getByRole("dialog", { name: "Add the next match" });
  await dialog.locator("#match-date").fill(tomorrow());
  await dialog.locator("#match-time").fill("19:00");
  await dialog.locator("#match-opponent").selectOption({ label: opp });
  await page.getByTestId("save-next-match").click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });

  // The title names the opponent; the brief is the next step.
  await expect(page.getByRole("heading", { level: 1, name: `Scrim vs ${opp}` })).toBeVisible({ timeout: 10000 });
  const prep = page.locator("[data-prep]");
  const steps = prep.locator("li");
  await expect(steps).toHaveCount(5);
  await expect(steps.nth(0)).toHaveAttribute("aria-current", "step");
  await expect(steps.nth(0)).toContainText(`Read the brief on ${opp}`);
  await expect(page.locator("[data-brief]")).toBeVisible();

  // Reading the brief advances the sequence to the draft plans.
  await page.getByTestId("mark-brief-read").click();
  await expect(page.getByTestId("brief-read")).toBeVisible({ timeout: 10000 });
  await expect(steps.nth(0)).not.toHaveAttribute("aria-current", "step");
  await expect(steps.nth(1)).toHaveAttribute("aria-current", "step");
  await expect(steps.nth(1)).toContainText("Draft plans");

  // The rest of the page says how it fills.
  await expect(page.locator("[data-open-items]")).toContainText(/Nothing open/);
  await expect(page.locator("[data-last-review]")).toContainText(/Reviews you write/);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("player card: shared read-only without signing in", async ({ teamPage, browser }) => {
  const page = teamPage;
  if (!page.url().includes("/team/dashboard")) await navigateTo(page, "/team/dashboard");
  // Open your own card from the roster.
  await page.locator("[data-roster-row] a").first().click();
  await page.waitForURL("**/team/player/**");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.locator("[data-player-card]")).toBeVisible();

  await page.getByTestId("share-player-card").click();
  const dialog = page.getByRole("dialog", { name: /Share this player card/ });
  await dialog.getByTestId("create-share-link").click();
  const url = (await dialog.locator("[data-share-url]").first().textContent({ timeout: 10000 }))?.trim() ?? "";
  expect(url).toMatch(/\/s\/[a-z2-9]{32}$/);

  const ctx = await browser.newContext();
  const guest = await ctx.newPage();
  await guest.goto(url);
  await expect(guest.locator("[data-shared-page] [data-player-card]")).toBeVisible({ timeout: 15000 });
  await expect(guest.locator("[data-shared-note]")).toContainText("Read-only");
  // Nothing else of the team: no team navigation, no notes, no roster.
  await expect(guest.locator('nav[aria-label="Primary"]')).toHaveCount(0);
  await expect(guest.locator("[data-team-notes], [data-roster]")).toHaveCount(0);
  await expect(guest.getByRole("link", { name: "Sign in" })).toBeVisible();

  // Revoked, it opens nothing.
  await dialog.getByRole("button", { name: "Revoke" }).first().click();
  await expect(dialog.locator("[data-share-url]")).toHaveCount(0, { timeout: 10000 });
  await guest.reload();
  await expect(guest.getByRole("heading", { level: 1, name: "This link does not open anything" })).toBeVisible({ timeout: 15000 });
  await ctx.close();
});

test("share link: a game plan opens alone, an unknown token opens nothing", async ({ teamPage, browser }) => {
  const page = teamPage;
  const plan = `Shared plan ${Date.now()}`;
  await navigateTo(page, "/game-plan");
  await page.locator("#plan-name").fill(plan);
  await page.getByTestId("plan-save").click();
  await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 10000 });
  await page.getByTestId("plan-share").click();
  const dialog = page.getByRole("dialog", { name: /Share this game plan/ });
  await dialog.getByTestId("create-share-link").click();
  const url = (await dialog.locator("[data-share-url]").first().textContent({ timeout: 10000 }))?.trim() ?? "";

  const ctx = await browser.newContext();
  const guest = await ctx.newPage();
  await guest.goto(url);
  await expect(guest.getByRole("heading", { level: 1, name: plan })).toBeVisible({ timeout: 15000 });
  await expect(guest.locator("[data-shared-note]")).toBeVisible();
  await guest.goto("/s/abcdefghijkmnpqrstuvwxyz23456789");
  await expect(guest.getByRole("heading", { level: 1, name: "This link does not open anything" })).toBeVisible({ timeout: 15000 });
  await ctx.close();
});

test("phone menu at 400 px lists every page by group", async ({ authedPage }) => {
  const page = authedPage;
  await page.setViewportSize({ width: 400, height: 860 });
  await navigateTo(page, "/draft");
  // The desktop hub links and sub-navigation are hidden on phones.
  await expect(page.locator('nav[aria-label="Primary"]')).toBeHidden();
  await expect(page.locator('nav[aria-label="Sub-navigation"]')).toBeHidden();

  await page.getByTestId("phone-menu-trigger").click();
  const menu = page.getByRole("dialog", { name: "Menu" });
  await expect(menu).toBeVisible();
  // It stays a modal sheet once its counts have loaded (covers the screen
  // from the top, page behind it inert).
  await page.waitForTimeout(800);
  expect(await menu.evaluate((d) => d.matches(":modal"))).toBe(true);
  expect(await menu.evaluate((d) => Math.round(d.getBoundingClientRect().top))).toBe(0);
  for (const g of ["Today", "After the game", "Team", "You"]) {
    await expect(menu.getByRole("heading", { level: 3, name: g })).toBeVisible();
  }
  await expect(menu.locator('a[aria-current="page"]')).toHaveText(/Draft/);
  const noScroll = await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth);
  expect(noScroll).toBe(true);

  await menu.getByRole("link", { name: /Team games/ }).click();
  await page.waitForURL("**/stats");
  await expect(menu).not.toBeVisible();
  await expect(page.getByRole("heading", { level: 1, name: "Team games" })).toBeVisible();
});
