/**
 * Opponents page smoke tests.
 *
 * Verifies: page loads with heading, new opponent form shows 5 role rows,
 * Save & Fetch / Discard Form buttons are present and functional.
 *
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

// Filter out the Tailwind v4 @import 404 — harmless (CLAUDE.md rule 47)
function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) =>
      !e.includes("404") &&
      !e.includes("tailwindcss") &&
      !e.includes("net::ERR")
  );
}

test.describe("Opponents Page", () => {
  test("loads with heading and new opponent button", async ({ teamPage }) => {
    const page = teamPage;
    await page.goto("/opponents");
    await page.waitForLoadState("networkidle");

    await expect(page.locator("h1")).toContainText("Opponents");
    await expect(page.getByText("+ New Opponent")).toBeVisible();
  });

  test("new opponent form shows 5 role rows", async ({ teamPage }) => {
    const page = teamPage;
    await page.goto("/opponents");
    await page.waitForLoadState("networkidle");

    // Open creation form — wait for WASM hydration first
    await page.waitForTimeout(1000);
    await page.getByText("+ New Opponent").click();
    await page.waitForTimeout(500);

    // Verify team name input and Save & Fetch button
    await expect(
      page.getByPlaceholder("Opponent team name...")
    ).toBeVisible({ timeout: 10000 });
    await expect(page.getByText("Save & Fetch")).toBeVisible();
    await expect(page.getByText("Discard Form")).toBeVisible();

    // Verify all 5 role labels visible
    await expect(page.getByText("Top", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("Jungle", { exact: true })).toBeVisible();
    await expect(page.getByText("Mid", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("Bot", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("Support", { exact: true }).first()).toBeVisible();
  });

  test("discard form returns to placeholder", async ({ teamPage }) => {
    const page = teamPage;
    await page.goto("/opponents");
    await page.waitForLoadState("networkidle");

    // Open creation form — wait for WASM hydration first
    await page.waitForTimeout(1000);
    await page.getByText("+ New Opponent").click();
    await page.waitForTimeout(500);
    await expect(page.getByText("Save & Fetch")).toBeVisible({ timeout: 10000 });

    // Discard it
    await page.getByText("Discard Form").click();
    await page.waitForTimeout(300);
    await expect(page.getByText("Save & Fetch")).not.toBeVisible();
  });
});
