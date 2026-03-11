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

    // Should auto-login and redirect to /team/dashboard
    await page.waitForURL("**/team/dashboard", { timeout: 10000 });

    // Navigate to profile — should be accessible (not redirected to login)
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");
    expect(page.url()).not.toContain("/auth/login");

    // Verify username appears in the page
    const bodyText = await page.textContent("body") || "";
    expect(bodyText).toContain(flowUsername);
  });

  test("logout clears session", async ({ page }) => {
    // Log in first
    await page.goto("/auth/login");
    await page.fill("input[name=email]", flowEmail);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await page.waitForURL("**/team/dashboard", { timeout: 10000 });

    // Navigate to profile where there's a visible "Log Out" button
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");

    // Click the profile page's "Log Out" button
    const logoutBtn = page.locator("button").filter({ hasText: /log\s*out/i }).first();
    await expect(logoutBtn).toBeVisible({ timeout: 5000 });
    await logoutBtn.click();

    // Hard navigation to "/" should happen after logout
    await page.waitForURL("**/", { timeout: 10000 });

    // After logout, visiting profile should redirect to login
    await page.goto("/profile");
    await page.waitForLoadState("networkidle");
    // Should either redirect to /auth/login or show "not logged in" text
    const url = page.url();
    const bodyText = await page.textContent("body") || "";
    const loggedOut = url.includes("/auth/login") || bodyText.match(/not logged in|sign in/i);
    expect(loggedOut).toBeTruthy();
  });
});
