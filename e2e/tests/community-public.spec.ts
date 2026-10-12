/**
 * M7: the public community pages, signed out. The changelog lists the
 * build's CHANGELOG.md with stable anchors; the transparency page shows
 * nothing about money until the owner approves the texts (a fresh e2e
 * database has none approved), and no support link without SUPPORT_URL.
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors } from "./helpers";

test("changelog loads signed out with the milestone entries and anchors", async ({ page }) => {
  const errors = captureErrors(page);
  const missing: string[] = [];
  page.on("response", (r) => {
    if (r.url().includes("/pkg/") && r.status() === 404) missing.push(r.url());
  });
  const resp = await page.goto("/changelog");
  expect(resp?.status()).toBe(200);
  await expect(page.getByRole("heading", { level: 1, name: "Changelog" })).toBeVisible();
  const m7 = page.locator("section#milestone-m7-community-and-transparency-loop");
  await expect(m7).toBeVisible();
  await page.goto("/changelog#milestone-m5-remaining-pages-navigation-mobile");
  await expect(page.locator("section#milestone-m5-remaining-pages-navigation-mobile h2")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Game patches" })).toBeVisible();
  // Signed-out visitors see no board extras.
  await expect(page.getByText("Fixed from the bug board")).toHaveCount(0);
  await page.waitForTimeout(500);
  expect(missing).toEqual([]);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("transparency shows the neutral state and nothing about money before approval", async ({ page }) => {
  const errors = captureErrors(page);
  const resp = await page.goto("/transparency");
  expect(resp?.status()).toBe(200);
  await expect(page.getByRole("heading", { level: 1, name: "Transparency" })).toBeVisible();
  const main = page.locator("main");
  const text = (await main.textContent()) ?? "";
  // Either the gate is closed (fresh database: nothing approved) or the
  // owner approved the texts on this database.
  if (!text.includes("How support is used")) {
    await expect(main.getByText("No months published yet.")).toBeVisible();
    expect(text.toLowerCase()).not.toContain("support");
    expect(text.toLowerCase()).not.toContain("maintainer");
  }
  // No support link without SUPPORT_URL (e2e runs without it).
  await expect(page.getByTestId("support-link")).toHaveCount(0);
  const lower = text.toLowerCase();
  expect(lower).not.toMatch(/\bdonate\b|\bspende\b|\bperks?\b/);
  expect(lower.replace(/no donation receipt is issued/g, "")).not.toContain("donation");
  await page.waitForTimeout(500);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("footer lists the public community links only to signed-out visitors", async ({ page }) => {
  await page.goto("/legal/impressum");
  const community = page.getByRole("navigation", { name: "Community" });
  await expect(community.getByRole("link", { name: "Changelog" })).toBeVisible();
  await expect(community.getByRole("link", { name: "Transparency" })).toBeVisible();
  await expect(community.getByRole("link", { name: "What next" })).toHaveCount(0);
  await expect(community.getByRole("link", { name: "Bug board" })).toHaveCount(0);
});
