/**
 * M141 (report 3z6n0emq): choosing a match on the post-game review fills
 * the fields the match can answer (the result), says so with a note and an
 * Undo, marks the filled field, and shows the facts that have no field in a
 * read-only "From the match" block. Signs in as the seed player whose demo
 * game is a synced team match; CI seeds the database before the server
 * starts, locally the spec is skipped without the seed.
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";

const DEMO = "SEED1_1000000001";

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

test.describe("post-game: filled from match", () => {
  test.beforeEach(async ({ page }) => {
    const ok = await signInSeedTop(page);
    if (process.env.CI) expect(ok, "CI must seed the database before starting the server").toBe(true);
    test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
    await page.waitForTimeout(500);
  });

  test("choosing a match fills the result, marks it and can be undone", async ({ page }) => {
    // The demo game's scoreboard is stored by the seed, so side and picks are known.
    await page.goto("/post-game");
    const select = page.locator("#review-match");
    await expect(select.locator(`option[value="${DEMO}"]`)).toHaveCount(1);
    const outcome = page.locator("#review-outcome");
    await expect(outcome.getByRole("radio", { name: "Not set" })).toHaveAttribute("aria-checked", "true");

    await select.selectOption(DEMO);
    await expect(outcome.getByRole("radio", { name: "Win" })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByTestId("review-filled-note")).toContainText("Filled from match: Result");
    await expect(page.getByTestId("review-outcome-from-match")).toBeVisible();

    const facts = page.getByTestId("review-match-facts");
    await expect(facts).toContainText("From the match");
    await expect(facts).toContainText("Blue side");
    await expect(facts).toContainText("29:14");
    await expect(facts).toContainText("Darius");
    // Read only: no inputs inside the facts block.
    await expect(facts.locator("input, select, textarea")).toHaveCount(0);

    // Editing the filled field removes its mark; Undo restores what was there.
    await outcome.getByRole("radio", { name: "Loss" }).click();
    await expect(page.getByTestId("review-outcome-from-match")).toHaveCount(0);
    await outcome.getByRole("radio", { name: "Win" }).click();
    await page.getByTestId("review-filled-note").getByRole("button", { name: "Undo" }).click();
    await expect(outcome.getByRole("radio", { name: "Not set" })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByTestId("review-filled-note")).toHaveText("");
  });

  test("a result someone set is never overwritten", async ({ page }) => {
    await page.goto("/post-game");
    const outcome = page.locator("#review-outcome");
    await outcome.getByRole("radio", { name: "Loss" }).click();
    await page.locator("#review-match").selectOption(DEMO);
    await expect(page.getByTestId("review-match-facts")).toBeVisible();
    await expect(outcome.getByRole("radio", { name: "Loss" })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByTestId("review-outcome-from-match")).toHaveCount(0);
  });
});
