/**
 * M7: the admin hub lists every admin area with a live status, and the
 * account menu has one "Admin" entry.
 */
import { test, expect } from "./fixtures";

test("hub lists the admin areas with their status", async ({ adminPage }) => {
  const resp = await adminPage.goto("/admin");
  expect(resp?.status()).toBe(200);
  const table = adminPage.getByRole("table", { name: "Admin areas" });
  for (const area of ["Bug reports", "Surveys", "What next", "Transparency ledger", "Public texts", "Invites", "Champion traits", "Design system"]) {
    await expect(table.getByRole("link", { name: area, exact: true })).toBeVisible();
  }
  await expect(table).toContainText(/waiting for approval/);
  await expect(table).toContainText(/not published/);
});

test("the account menu has one Admin entry and Your reports", async ({ adminPage }) => {
  await adminPage.goto("/changelog");
  await adminPage.waitForLoadState("networkidle");
  await adminPage.getByRole("button", { name: /Account menu/ }).click();
  await expect(adminPage.getByRole("menuitem", { name: "Admin", exact: true })).toBeVisible();
  await expect(adminPage.getByRole("menuitem", { name: /^Your reports/ })).toBeVisible();
  await expect(adminPage.getByRole("menuitem", { name: "Invites" })).toHaveCount(0);
});
