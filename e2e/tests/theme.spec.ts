/**
 * One theme. Release 1 is dark only and M2 removed the theme plumbing: no
 * `data-theme` attribute, no theme toggle, `color-scheme: dark` on :root,
 * and the Survey tokens and faces are what the page uses.
 */
import { test, expect } from "./fixtures";

test("there is no theme attribute and no theme toggle", async ({ authedPage }) => {
  await authedPage.goto("/team/dashboard");
  await authedPage.waitForLoadState("networkidle");
  expect(await authedPage.getAttribute("html", "data-theme")).toBeNull();
  await expect(authedPage.locator('[aria-label="Theme toggle"]')).toHaveCount(0);
  await expect(authedPage.locator('button:has-text("Pandemonium")')).toHaveCount(0);
});

test("dark color scheme, Survey ground and faces", async ({ page }) => {
  await page.goto("/auth/login");
  await page.waitForLoadState("networkidle");
  const styles = await page.evaluate(() => {
    const root = getComputedStyle(document.documentElement);
    const body = getComputedStyle(document.body);
    return {
      scheme: root.colorScheme,
      page: root.getPropertyValue("--color-page").trim(),
      font: body.fontFamily,
      h1: getComputedStyle(document.querySelector("h1")!).fontFamily,
    };
  });
  expect(styles.scheme).toContain("dark");
  expect(styles.page).not.toBe("");
  expect(styles.font).toContain("Public Sans");
  expect(styles.h1).toContain("Familjen Grotesk");
});
