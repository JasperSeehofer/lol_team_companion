/**
 * Auth flow tests — register, login, logout.
 * Uses a unique email per test run to avoid conflicts.
 */
import { test, expect } from "@playwright/test";
import { INVITE_CODE, TEST_PASSWORD } from "./fixtures";

const TIMESTAMP = Date.now();
const TEST_EMAIL = `e2e_${TIMESTAMP}@test.invalid`;
const TEST_USERNAME = `e2euser_${TIMESTAMP}`;
const REGISTER_URL = `/auth/register?invite=${INVITE_CODE}`;

test.describe("Registration", () => {
  test("shows register form", async ({ page }) => {
    await page.goto(REGISTER_URL);
    await expect(page.locator("input[type=email], input[name=email]")).toBeVisible();
    await expect(page.locator("input[type=password], input[name=password]")).toBeVisible();
  });

  test("can register a new account", async ({ page }) => {
    await page.goto(REGISTER_URL);

    await page.fill("input[name=username]", TEST_USERNAME);
    await page.fill("input[name=email]", TEST_EMAIL);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");

    // After successful registration, should navigate away from /auth/register
    // Registration auto-logs in and redirects to /solo (new users start in solo mode)
    await page.waitForLoadState("networkidle");
    const url = page.url();
    expect(url).not.toContain("/auth/register");
  });

  test("register without an invite redirects to the closed-beta page", async ({ page }) => {
    await page.goto("/auth/register");
    await page.waitForURL("**/closed-beta", { timeout: 15000 });
  });

  test("an unknown invite code is rejected on the server", async ({ page }) => {
    await page.goto("/auth/register?invite=NOT-A-REAL-CODE");
    await page.fill("input[name=username]", `badinvite_${TIMESTAMP}`);
    await page.fill("input[name=email]", `badinvite_${TIMESTAMP}@test.invalid`);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");

    await expect(page.locator("text=/Registration failed/i")).toBeVisible({ timeout: 10000 });
    expect(page.url()).toContain("/auth/register");

    // The account was not created: signing in fails.
    await page.goto("/auth/login");
    await page.fill("input[name=email]", `badinvite_${TIMESTAMP}@test.invalid`);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("main button[type=submit]");
    await expect(page.locator("text=/Invalid email or password/i")).toBeVisible({
      timeout: 10000,
    });
  });

  test("a duplicate email gets the same generic message", async ({ page }) => {
    await page.goto(REGISTER_URL);
    await page.fill("input[name=username]", `dupe_${TIMESTAMP}`);
    await page.fill("input[name=email]", TEST_EMAIL.toUpperCase());
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await expect(page.locator("text=/Registration failed/i")).toBeVisible({ timeout: 10000 });
  });

  test("a short password is rejected with a clear message", async ({ page }) => {
    await page.goto(REGISTER_URL);
    await page.fill("input[name=username]", `shortpw_${TIMESTAMP}`);
    await page.fill("input[name=email]", `shortpw_${TIMESTAMP}@test.invalid`);
    // Bypass the browser's minlength check to exercise the server rule.
    await page.locator("input[name=password]").evaluate((el) => el.removeAttribute("minlength"));
    await page.fill("input[name=password]", "short1!");
    await page.click("button[type=submit]");
    await expect(page.locator("text=/at least 10 characters/i")).toBeVisible({ timeout: 10000 });
  });
});

test.describe("Login", () => {
  test("shows login form", async ({ page }) => {
    await page.goto("/auth/login");
    await expect(page.locator("input[type=email], input[name=email]")).toBeVisible();
    await expect(page.locator("input[type=password], input[name=password]")).toBeVisible();
  });

  test("shows error for invalid credentials", async ({ page }) => {
    await page.goto("/auth/login");

    await page.fill("input[name=email]", "nobody@nowhere.invalid");
    await page.fill("input[name=password]", "wrongpassword");
    await page.click("button[type=submit]");

    await page.waitForLoadState("networkidle");
    // Should stay on login page or show error
    const hasError =
      (await page.locator("text=/invalid|incorrect|failed|wrong/i").count()) > 0 ||
      page.url().includes("/auth/login");
    expect(hasError).toBeTruthy();
  });
});

test.describe("Full auth flow", () => {
  const flowEmail = `flow_${TIMESTAMP}@test.invalid`;
  const flowUsername = `flowuser_${TIMESTAMP}`;

  test("register → auto-login → profile accessible", async ({ page }) => {
    await page.goto(REGISTER_URL);
    await page.fill("input[name=username]", flowUsername);
    await page.fill("input[name=email]", flowEmail);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");

    // Should auto-login and redirect to /solo (new users start in solo mode)
    await page.waitForURL("**/solo", { timeout: 15000 });
    await page.waitForLoadState("networkidle");

    // Navigate to profile — should be accessible (not redirected to login)
    await page.goto("/profile", { waitUntil: "networkidle" });
    expect(page.url()).not.toContain("/auth/login");

    // Verify username appears in the page
    const bodyText = await page.textContent("body") || "";
    expect(bodyText).toContain(flowUsername);
  });

  test("logout clears session", async ({ page }) => {
    // Log in first — use a more specific selector to avoid matching hidden nav buttons
    await page.goto("/auth/login");
    await page.fill("input[name=email]", flowEmail);
    await page.fill("input[name=password]", TEST_PASSWORD);
    // Use the form's submit button specifically (not nav logout button)
    await page.locator("main button[type=submit], main input[type=submit]").first().click();
    // Login lands on /solo for a solo-mode user
    await page.waitForURL("**/solo", { timeout: 15000 });

    // Let WASM Effects settle before navigating
    await page.waitForLoadState("load");
    await page.waitForTimeout(500);

    // Navigate to profile where there's a visible "Log Out" button
    await page.goto("/profile", { waitUntil: "networkidle" });

    // Click the profile page's "Log Out" button
    const logoutBtn = page.locator("button").filter({ hasText: /log\s*out/i }).first();
    await expect(logoutBtn).toBeVisible({ timeout: 5000 });
    await logoutBtn.click();

    // Hard navigation to "/" should happen after logout
    await page.waitForURL("**/", { timeout: 10000 });
    // wasm-patterns rule 56: let the hard-nav Effect settle (it can fire a
    // second set_href("/")) before navigating again.
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);

    // After logout, visiting profile should redirect to login
    await page.goto("/profile", { waitUntil: "networkidle" });
    // Should either redirect to /auth/login or show "not logged in" text
    const url = page.url();
    const bodyText = await page.textContent("body") || "";
    const loggedOut = url.includes("/auth/login") || bodyText.match(/not logged in|sign in/i);
    expect(loggedOut).toBeTruthy();
  });
});
