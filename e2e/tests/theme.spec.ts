/**
 * Theme spec. Release 1 ships one theme ("demacia"); the user-facing theme
 * toggle was removed in M0. This checks the server renders it for a new user
 * and that no theme toggle is offered.
 */
import { test, expect } from "./fixtures";

test("default theme is demacia for a new user", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  const theme = await authedPage.getAttribute("html", "data-theme");
  expect(theme).toBe("demacia");
});

test("no theme toggle is offered", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  await expect(authedPage.locator('[aria-label="Theme toggle"]')).toHaveCount(0);
  await expect(authedPage.locator('button:has-text("Pandemonium")')).toHaveCount(0);
});
