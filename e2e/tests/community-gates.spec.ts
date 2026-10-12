/**
 * M7: who may open what. The bug board and What next need a signed-in
 * beta account; every admin page is the 404 page for everyone else.
 */
import { test, expect } from "./fixtures";

for (const path of ["/bugs", "/what-next"]) {
  test(`${path} sends signed-out visitors to sign in`, async ({ page }) => {
    await page.goto(path);
    await page.waitForURL("**/auth/login", { timeout: 10000 });
  });
}

const ADMIN_PAGES = [
  "/admin",
  "/admin/bugs",
  "/admin/surveys",
  "/admin/proposals",
  "/admin/ledger",
  "/admin/texts",
];

test("admin pages are 404 for a signed-in non-admin", async ({ authedPage }) => {
  for (const path of ADMIN_PAGES) {
    const resp = await authedPage.goto(path);
    expect(resp?.status(), path).toBe(404);
    await expect(authedPage.getByRole("heading", { name: /not found/i })).toBeVisible();
  }
});

test("admin pages are 404 signed out", async ({ page }) => {
  for (const path of ADMIN_PAGES) {
    const resp = await page.goto(path);
    expect(resp?.status(), path).toBe(404);
  }
});

test("signed-in users see the member community links", async ({ authedPage }) => {
  await authedPage.goto("/changelog");
  const community = authedPage.getByRole("navigation", { name: "Community" });
  await expect(community.getByRole("link", { name: "What next" })).toBeVisible();
  await expect(community.getByRole("link", { name: "Bug board" })).toBeVisible();
});
