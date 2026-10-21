/**
 * M160 item 1, solo mode: an account without a team sees only Match Day and
 * You in the navigation (desktop bar, phone menu, command bar) plus a quiet
 * "Find or create a team" link, and the draft, game plan and review pages
 * show the no-team state instead of an editor. A team member still sees
 * every hub.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("desktop: no team hubs, the find-a-team link instead", async ({ authedPage: page }) => {
  await page.setViewportSize({ width: 1280, height: 860 });
  await navigateTo(page, "/me");
  const bar = page.locator('nav[aria-label="Primary"]');
  await expect(bar.getByRole("link", { name: /^You/ })).toBeVisible({ timeout: 15000 });
  await expect(bar.getByRole("link", { name: /^Match Day/ })).toBeVisible();
  for (const hub of ["Prep", "Review", "Team"]) {
    await expect(bar.getByRole("link", { name: new RegExp(`^${hub}`) })).toHaveCount(0);
  }
  const find = page.getByTestId("find-team-link");
  await expect(find).toBeVisible();
  await expect(find).toHaveText("Find or create a team");
  await expect(find).toHaveAttribute("href", "/team/dashboard");

  // A team page reached by address draws no team sub-navigation.
  await navigateTo(page, "/draft");
  await expect(page.locator('nav[aria-label="Sub-navigation"]')).toHaveCount(0);
});

test("phone menu at 400 px: Today and You, then the find-a-team row", async ({ authedPage: page }) => {
  await page.setViewportSize({ width: 400, height: 860 });
  await navigateTo(page, "/me");
  await page.getByTestId("phone-menu-trigger").click();
  const menu = page.getByRole("dialog", { name: "Menu" });
  await expect(menu).toBeVisible();
  await expect(menu.getByRole("heading", { level: 3, name: "Today" })).toBeVisible();
  await expect(menu.getByRole("heading", { level: 3, name: "You" })).toBeVisible();
  for (const g of ["After the game", "Team"]) {
    await expect(menu.getByRole("heading", { level: 3, name: g, exact: true })).toHaveCount(0);
  }
  for (const label of ["Draft", "Game plan", "Reviews", "Opponents", "Team games"]) {
    await expect(menu.getByRole("link", { name: label, exact: true })).toHaveCount(0);
  }
  await expect(menu.getByRole("link", { name: /Champion pool/ })).toBeVisible();
  await expect(menu.getByTestId("phone-find-team")).toHaveText("Find or create a team");
  const noScroll = await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth);
  expect(noScroll).toBe(true);
});

for (const [path, title] of [
  ["/draft", "Drafts belong to a team"],
  ["/game-plan", "Game plans belong to a team"],
  ["/post-game", "Reviews belong to a team"],
] as const) {
  test(`${path} shows the no-team state, not an editor`, async ({ authedPage: page }) => {
    const errors = captureErrors(page);
    await navigateTo(page, path);
    const state = page.getByTestId("no-team-page");
    await expect(state).toBeVisible({ timeout: 15000 });
    await expect(state.locator('[data-no-team="true"]')).toBeVisible();
    await expect(state.getByRole("heading", { name: title })).toBeVisible();
    const cta = state.getByRole("link", { name: "Find or create a team" });
    await expect(cta).toHaveAttribute("href", "/team/dashboard");
    // The editor's save controls are not on screen.
    await expect(page.getByTestId("plan-save")).toBeHidden();
    await expect(page.locator("#draft-board-slot-0")).toBeHidden();
    expect(filterRealErrors(errors)).toHaveLength(0);
  });
}

test("a team member still sees every hub and the editors", async ({ teamPage: page }) => {
  await page.setViewportSize({ width: 1280, height: 860 });
  await navigateTo(page, "/game-plan");
  const bar = page.locator('nav[aria-label="Primary"]');
  for (const hub of ["Match Day", "Prep", "Review", "Team", "You"]) {
    await expect(bar.getByRole("link", { name: new RegExp(`^${hub}`) })).toBeVisible({ timeout: 15000 });
  }
  await expect(page.getByTestId("find-team-link")).toHaveCount(0);
  await expect(page.getByTestId("no-team-page")).toHaveCount(0);
  await expect(page.getByTestId("plan-save")).toBeVisible();
});
