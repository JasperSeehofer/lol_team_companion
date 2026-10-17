/**
 * Admin invite management (M1).
 *
 * Needs the server started with `ADMIN_EMAILS` containing ADMIN_EMAIL below
 * (CI and CLAUDE.md set `ADMIN_EMAILS=e2e-admin@test.invalid`). The admin
 * account is registered on the first run and signed into afterwards.
 */
import { test, expect } from "@playwright/test";
import { authenticatePage, INVITE_CODE, TEST_PASSWORD, signInAsAdmin, ADMIN_EMAIL } from "./fixtures";

const TIMESTAMP = Date.now();

test("non-admin users get a 404 and no admin link", async ({ page }) => {
  await authenticatePage(page);
  const resp = await page.goto("/admin/invites");
  expect(resp?.status()).toBe(404);
  await expect(page.locator("text=Page not found")).toBeVisible();
  await expect(page.locator('a[href="/admin/invites"]')).toHaveCount(0);
});

test("admin can mint a code that registers a new account, then disable it", async ({
  page,
  browser,
}) => {
  await signInAsAdmin(page);
  const resp = await page.goto("/admin/invites", { waitUntil: "networkidle" });
  test.skip(
    resp?.status() === 404,
    `server not started with ADMIN_EMAILS=${ADMIN_EMAIL}; admin smoke skipped`
  );
  await expect(page.locator("h1", { hasText: "Beta invitations" })).toBeVisible();

  // Mint a single-use code.
  await page.fill("#invite-max-uses", "1");
  await page.fill("#invite-note", `e2e ${TIMESTAMP}`);
  await page.click('button:has-text("Mint code")');
  const minted = page.locator('[data-testid="minted-code"] code').first();
  await expect(minted).toBeVisible({ timeout: 10000 });
  const code = (await minted.textContent())?.trim() ?? "";
  expect(code).toMatch(/^[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}$/);
  await expect(page.locator("td code", { hasText: code })).toBeVisible();

  // The code registers exactly one new account (fresh browser context).
  const ctx = await browser.newContext();
  const guest = await ctx.newPage();
  await guest.goto(`/auth/register?invite=${code}`);
  await guest.fill("input[name=username]", `invited_${TIMESTAMP}`);
  await guest.fill("input[name=email]", `invited_${TIMESTAMP}@test.invalid`);
  await guest.fill("input[name=password]", TEST_PASSWORD);
  await guest.check("#register-terms");
  await guest.click("button[type=submit]");
  await guest.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });
  await ctx.close();

  const ctx2 = await browser.newContext();
  const second = await ctx2.newPage();
  await second.goto(`/auth/register?invite=${code}`);
  await second.fill("input[name=username]", `invited2_${TIMESTAMP}`);
  await second.fill("input[name=email]", `invited2_${TIMESTAMP}@test.invalid`);
  await second.fill("input[name=password]", TEST_PASSWORD);
  await second.check("#register-terms");
  await second.click("button[type=submit]");
  await expect(second.locator("text=/Registration failed/i")).toBeVisible({ timeout: 10000 });
  await ctx2.close();

  // Mint another and disable it.
  await page.reload({ waitUntil: "networkidle" });
  await page.fill("#invite-max-uses", "3");
  await page.click('button:has-text("Mint code")');
  const second_code_el = page.locator('[data-testid="minted-code"] code').first();
  await expect(second_code_el).toBeVisible({ timeout: 10000 });
  const code2 = (await second_code_el.textContent())?.trim() ?? "";
  const row = page.locator("tr", { has: page.locator("td code", { hasText: code2 }) });
  await expect(row).toBeVisible();
  await row.locator('button:has-text("Disable")').click();
  await expect(row.locator("td", { hasText: "disabled" })).toBeVisible({ timeout: 10000 });
});

test("mint row: the four controls share one line, hints sit below (M141, report 1jjv5ji2)", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await signInAsAdmin(page);
  const resp = await page.goto("/admin/invites", { waitUntil: "networkidle" });
  test.skip(resp?.status() === 404, `server not started with ADMIN_EMAILS=${ADMIN_EMAIL}`);
  const tops = await Promise.all(
    ["#invite-max-uses", "#invite-expiry", "#invite-team", "#invite-note"].map(async (sel) => {
      const box = await page.locator(sel).boundingBox();
      return Math.round(box?.y ?? -1);
    })
  );
  for (const t of tops) expect(Math.abs(t - tops[0])).toBeLessThanOrEqual(1);
  const control = await page.locator("#invite-expiry").boundingBox();
  const hint = await page.getByText("Empty: never").boundingBox();
  expect((hint?.y ?? 0)).toBeGreaterThan((control?.y ?? 0) + (control?.height ?? 0) - 1);
});
