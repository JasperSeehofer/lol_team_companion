/**
 * M141 (report rucyy6k1): data tables of comparable rows sort by column.
 * The gallery spec covers the SortTh primitive itself (click, keyboard,
 * stable order, the status line); this spec checks the pages use it. It
 * signs in as the seed player whose synced demo game fills My games; CI
 * seeds the database before the server starts, locally it is skipped
 * without the seed.
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";

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

test.describe("sortable tables", () => {
  test.beforeEach(async ({ page }) => {
    const ok = await signInSeedTop(page);
    if (process.env.CI) expect(ok, "CI must seed the database before starting the server").toBe(true);
    test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
    await page.waitForTimeout(500);
  });

  test("champion trends sort by any column, by click and keyboard", async ({ page }) => {
    await page.goto("/me");
    const trends = page.locator("[data-champion-trends]");
    // The trends default to 30 days; the demo game is a day old.
    const games = trends.locator("th", { has: page.getByRole("button", { name: "Sort by Games" }) });
    await expect(games).toHaveAttribute("aria-sort", "descending");
    const rate = trends.getByRole("button", { name: "Sort by Win rate" });
    await rate.click();
    const rateTh = trends.locator("th", { has: rate });
    await expect(rateTh).toHaveAttribute("aria-sort", "descending");
    await expect(games).not.toHaveAttribute("aria-sort", /.+/);
    await expect(rate).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(rateTh).toHaveAttribute("aria-sort", "ascending");
    await expect(trends.getByRole("status")).toHaveText("Sorted by Win rate, ascending");
  });

  test("recent games sort and return to newest first", async ({ page }) => {
    await page.goto("/me");
    const recent = page.locator("[data-my-recent]");
    await expect(recent.locator("tbody tr").first()).toBeVisible();
    await expect(recent.getByRole("button", { name: "Newest first" })).toHaveCount(0);
    const cs = recent.getByRole("button", { name: "Sort by CS" });
    await cs.click();
    await expect(recent.locator("th", { has: cs })).toHaveAttribute("aria-sort", "descending");
    const reset = recent.getByRole("button", { name: "Newest first" });
    await expect(reset).toBeVisible();
    await reset.click();
    await expect(recent.locator("th", { has: cs })).not.toHaveAttribute("aria-sort", /.+/);
  });

  test("team games by member sort by column", async ({ page }) => {
    await page.goto("/stats");
    const members = page.locator("[data-member-summary]");
    if ((await members.count()) === 0) {
      // No team games stored for the seed team on this server: nothing to sort.
      test.skip(true, "no team games to sort");
    }
    const kda = members.getByRole("button", { name: "Sort by KDA" });
    await kda.click();
    await expect(members.locator("th", { has: kda })).toHaveAttribute("aria-sort", "descending");
  });
});
