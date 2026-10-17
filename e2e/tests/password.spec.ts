/**
 * M137 item 5: password change on /profile and the admin reset code.
 *
 * - A signed-in user changes the password: this browser stays signed in,
 *   another browser signed in as the same account is signed out, and the
 *   new password signs in.
 * - An admin creates a one-time code on /admin/resets; the player sets a new
 *   password on /auth/reset, lands on sign-in, signs in with it; the code
 *   does not work a second time.
 * - M148: "Copy code" copies the code (success only after the clipboard
 *   resolved) and says plainly when the browser refuses.
 *
 * Needs the e2e server env (CLAUDE.md "Testing"): the bootstrap invite and
 * `ADMIN_EMAILS=e2e-admin@test.invalid`.
 */
import { test, expect, waitForHome, signInAsAdmin, INVITE_CODE, TEST_PASSWORD } from "./fixtures";
import type { Page } from "@playwright/test";

const STAMP = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;
const NEW_PASSWORD = "Fresh-Password-2026";
const RESET_PASSWORD = "Reset-Password-2026";

async function register(page: Page, tag: string): Promise<string> {
  const email = `pw_${tag}_${STAMP}@test.invalid`;
  await page.goto(`/auth/register?invite=${INVITE_CODE}`);
  await page.fill("input[name=username]", `pw_${tag}_${STAMP}`);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");
  await waitForHome(page);
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);
  return email;
}

async function signIn(page: Page, email: string, password: string): Promise<void> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", password);
  await page.click("main button[type=submit]");
  await waitForHome(page);
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);
}

test("the sign-in page says who resets a password and links to the reset page", async ({ page }) => {
  await page.goto("/auth/login");
  await expect(page.getByText("Forgot it? Ask the Team Companion admin for a reset code.")).toBeVisible();
  await page.getByRole("link", { name: "Have a reset code? Set a new password." }).click();
  await expect(page).toHaveURL(/\/auth\/reset$/);
  await expect(page.getByRole("heading", { name: "Set a new password" })).toBeVisible();
});

test("changing the password keeps this session and ends the other one", async ({ browser }) => {
  const main = await (await browser.newContext()).newPage();
  const email = await register(main, "change");
  const other = await (await browser.newContext()).newPage();
  await signIn(other, email, TEST_PASSWORD);

  await main.goto("/profile");
  await main.waitForLoadState("networkidle");
  // A wrong current password is reported on its field.
  await main.getByLabel("Current password", { exact: true }).fill("not-my-password");
  await main.getByLabel("New password", { exact: true }).fill(NEW_PASSWORD);
  await main.getByLabel("Repeat new password", { exact: true }).fill(NEW_PASSWORD);
  await main.getByRole("button", { name: "Change password" }).click();
  await expect(main.getByText("That is not your current password.")).toBeVisible();

  await main.getByLabel("Current password", { exact: true }).fill(TEST_PASSWORD);
  await main.getByRole("button", { name: "Change password" }).click();
  await expect(main.getByTestId("password-status")).toContainText(
    "Password changed. Your other sessions were signed out."
  );

  // This browser is still signed in.
  await main.reload();
  await expect(main.getByTestId("profile-email")).toHaveText(email);

  // The other browser is signed out (protected pages send it to sign in).
  await other.goto("/profile");
  await other.waitForURL("**/auth/login", { timeout: 15000 });

  // The old password no longer works; the new one does.
  await other.fill("input[name=email]", email);
  await other.fill("input[name=password]", TEST_PASSWORD);
  await other.click("main button[type=submit]");
  await expect(other.getByText("Invalid email or password")).toBeVisible();
  await signIn(other, email, NEW_PASSWORD);
});

test("an admin reset code sets a new password once", async ({ browser }) => {
  const player = await (await browser.newContext()).newPage();
  const email = await register(player, "reset");

  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  await admin.goto("/admin/resets", { waitUntil: "networkidle" });
  await expect(admin.getByRole("heading", { name: "Password resets" })).toBeVisible();
  await admin.getByLabel("Email of the account").fill(email);
  await admin.getByRole("button", { name: "Create reset code" }).click();
  const codeEl = admin.getByTestId("reset-code");
  await expect(codeEl).toBeVisible({ timeout: 15000 });
  const code = (await codeEl.textContent())?.trim() ?? "";
  expect(code).toMatch(/^[A-Z2-9]{4}-[A-Z2-9]{4}-[A-Z2-9]{4}-[A-Z2-9]{4}$/);
  // The list shows the code as open, never the code itself.
  await expect(admin.locator("table")).toContainText("open");
  await expect(admin.locator("table")).not.toContainText(code);

  const visitor = await (await browser.newContext()).newPage();
  await visitor.goto("/auth/reset");
  await visitor.waitForLoadState("networkidle");
  await visitor.fill("input[name=email]", email);
  await visitor.fill("input[name=code]", code.toLowerCase());
  await visitor.fill("input[name=new_password]", RESET_PASSWORD);
  await visitor.fill("input[name=confirm_password]", RESET_PASSWORD);
  await visitor.click("main button[type=submit]");
  await visitor.waitForURL("**/auth/login?reset=1", { timeout: 20000 });
  await expect(visitor.getByText("Password changed. Sign in with the new one.")).toBeVisible();
  await signIn(visitor, email, RESET_PASSWORD);

  // The player's old session ended with the reset.
  await player.goto("/profile");
  await player.waitForURL("**/auth/login", { timeout: 15000 });

  // The code works once: a second try gets the one generic message.
  const again = await (await browser.newContext()).newPage();
  await again.goto("/auth/reset");
  await again.waitForLoadState("networkidle");
  await again.fill("input[name=email]", email);
  await again.fill("input[name=code]", code);
  await again.fill("input[name=new_password]", "Another-Password-1");
  await again.fill("input[name=confirm_password]", "Another-Password-1");
  await again.click("main button[type=submit]");
  await expect(
    again.getByText("That code did not work. It may have expired or been used. Ask the admin for a new one.")
  ).toBeVisible({ timeout: 15000 });
});

// M148 item 2b (atelier card C9): "Copy code" says Copied only after the
// clipboard promise resolved, and says so plainly when the copy is refused.
async function mintFor(admin: Page, email: string): Promise<string> {
  await admin.goto("/admin/resets", { waitUntil: "networkidle" });
  await admin.getByLabel("Email of the account").fill(email);
  await admin.getByRole("button", { name: "Create reset code" }).click();
  const codeEl = admin.getByTestId("reset-code");
  await expect(codeEl).toBeVisible({ timeout: 15000 });
  return (await codeEl.textContent())?.trim() ?? "";
}

test("an admin copies the reset code to the clipboard", async ({ browser }) => {
  const player = await (await browser.newContext()).newPage();
  const email = await register(player, "copy");

  const ctx = await browser.newContext();
  await ctx.grantPermissions(["clipboard-read", "clipboard-write"]);
  const admin = await ctx.newPage();
  await signInAsAdmin(admin);
  await admin.setViewportSize({ width: 390, height: 844 });
  const code = await mintFor(admin, email);

  const copy = admin.getByTestId("copy-reset-code");
  await expect(copy).toHaveText("Copy code");
  expect(await copy.evaluate((el) => el.getBoundingClientRect().height)).toBeGreaterThanOrEqual(44);
  await expect(admin.getByTestId("reset-copy-status")).toHaveCount(0);
  await copy.click();
  await expect(copy).toHaveText("Copied", { timeout: 5000 });
  await expect(admin.getByTestId("reset-copy-status")).toHaveText("Code copied");
  await expect(admin.getByTestId("reset-copy-error")).toHaveCount(0);
  const clip = await admin.evaluate(() => navigator.clipboard.readText());
  expect(clip).toBe(code);
  // The label goes back after a moment; the code stays on the page.
  await expect(copy).toHaveText("Copy code", { timeout: 5000 });
  await expect(admin.getByTestId("reset-code")).toHaveText(code);
  await ctx.close();
});

test("a refused copy says so and never claims success", async ({ browser }) => {
  const player = await (await browser.newContext()).newPage();
  const email = await register(player, "nocopy");

  const ctx = await browser.newContext();
  // The browser refuses every clipboard write (as on plain HTTP or with a
  // denied permission).
  await ctx.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: () => Promise.reject(new DOMException("Denied", "NotAllowedError")) },
    });
  });
  const admin = await ctx.newPage();
  await signInAsAdmin(admin);
  await mintFor(admin, email);
  const copy = admin.getByTestId("copy-reset-code");
  await copy.click();
  await expect(admin.getByTestId("reset-copy-error")).toHaveText(
    "Could not copy. Select the code and copy it by hand."
  );
  await expect(copy).toHaveText("Copy code");
  await expect(admin.getByTestId("reset-copy-status")).toHaveCount(0);
  // The code is selected, ready to copy by hand.
  const selected = await admin.evaluate(() => window.getSelection()?.toString().trim() ?? "");
  expect(selected).toBe((await admin.getByTestId("reset-code").textContent())?.trim());
  await ctx.close();
});

test("/admin/resets is a 404 for a signed-in non-admin", async ({ authedPage }) => {
  const resp = await authedPage.goto("/admin/resets");
  expect(resp?.status()).toBe(404);
});
