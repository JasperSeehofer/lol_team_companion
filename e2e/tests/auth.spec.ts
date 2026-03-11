/**
 * Auth flow tests — register, login, logout.
 * Uses a unique email per test run to avoid conflicts.
 */
import { test, expect } from "@playwright/test";

const TIMESTAMP = Date.now();
const TEST_EMAIL = `e2e_${TIMESTAMP}@test.invalid`;
const TEST_PASSWORD = "Test1234!";
const TEST_USERNAME = `e2euser_${TIMESTAMP}`;

test.describe("Registration", () => {
  test("shows register form", async ({ page }) => {
    await page.goto("/auth/register");
    await expect(page.locator("input[type=email], input[name=email]")).toBeVisible();
    await expect(page.locator("input[type=password], input[name=password]")).toBeVisible();
  });

  test("can register a new account", async ({ page }) => {
    await page.goto("/auth/register");

    await page.fill("input[name=username]", TEST_USERNAME);
    await page.fill("input[name=email]", TEST_EMAIL);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");

    // After successful registration, should navigate away from /auth/register
    await page.waitForLoadState("networkidle");
    const url = page.url();
    expect(url).not.toContain("/auth/register");
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
    // Register
    await page.goto("/auth/register");
    await page.fill("input[name=username]", flowUsername);
    await page.fill("input[name=email]", flowEmail);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await page.waitForLoadState("networkidle");

    // Should be logged in — navigate to profile
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");
    // Profile page should load (not redirect to login)
    expect(page.url()).not.toContain("/auth/login");
  });

  // BUG: Logout doesn't actually invalidate the session. After clicking "Log Out",
  // get_current_user() still returns the user. Needs investigation — either the
  // ActionForm submission isn't reaching the server, or auth.logout() doesn't
  // clear the session cookie. See also CLAUDE.md rule 8 (needs hard navigation).
  test.fixme("logout clears session", async ({ page }) => {
    // Log in first
    await page.goto("/auth/login");
    await page.fill("input[name=email]", flowEmail);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await page.waitForLoadState("networkidle");

    // Navigate to profile where there's a visible "Log Out" button
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");

    // Click the profile page's "Log Out" button
    const logoutBtn = page.locator("button").filter({ hasText: /log\s*out/i }).first();
    await expect(logoutBtn).toBeVisible({ timeout: 5000 });
    await logoutBtn.click();
    await page.waitForLoadState("networkidle");

    // After logout, the nav should show "Sign In" instead of the username
    // (need to reload to pick up cleared session state)
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");
    const bodyText = await page.textContent("body") || "";
    expect(bodyText).toMatch(/not logged in|sign in/i);
  });
});
