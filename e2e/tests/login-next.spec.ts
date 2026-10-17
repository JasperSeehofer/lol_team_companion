/**
 * M148 item 8: every page has its own tab title, and sign-in returns to
 * the page that sent a signed-out visitor to it (`/auth/login?next=...`),
 * but only to a path on this site.
 */
import { test, expect, authenticatePage, TEST_PASSWORD } from "./fixtures";
import type { Browser, Page } from "@playwright/test";

/** A fresh account, then a fresh signed-out page (a new context). */
async function accountAndSignedOutPage(browser: Browser): Promise<{ email: string; page: Page }> {
  const ctx = await browser.newContext();
  const reg = await ctx.newPage();
  const { email } = await authenticatePage(reg);
  await ctx.close();
  const page = await (await browser.newContext()).newPage();
  return { email, page };
}

async function signIn(page: Page, email: string) {
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");
}

test.describe("Return to the page after sign-in", () => {
  test("a signed-out visit to /draft signs in and returns to /draft", async ({ browser }) => {
    const { email, page } = await accountAndSignedOutPage(browser);
    await page.goto("/draft?draft_id=nothing");
    await page.waitForURL((u) => new URL(u).pathname === "/auth/login", { timeout: 20000 });
    const url = new URL(page.url());
    expect(url.search).toBe("?next=%2Fdraft%3Fdraft_id%3Dnothing");
    // The page to return to rides in the form as a hidden field.
    await expect(page.locator("input[type=hidden][name=next]")).toHaveValue("/draft?draft_id=nothing");

    await signIn(page, email);
    await page.waitForURL((u) => new URL(u).pathname === "/draft", { timeout: 20000 });
    expect(new URL(page.url()).search).toBe("?draft_id=nothing");
  });

  test("plain /draft carries next=%2Fdraft", async ({ page }) => {
    await page.goto("/draft");
    await page.waitForURL((u) => new URL(u).pathname === "/auth/login", { timeout: 20000 });
    expect(new URL(page.url()).search).toBe("?next=%2Fdraft");
  });

  test("a forged next to another site lands on Home", async ({ browser, baseURL }) => {
    const { email, page } = await accountAndSignedOutPage(browser);
    const origin = new URL(baseURL ?? "http://127.0.0.1:3020").origin;
    await page.goto("/auth/login?next=//evil.example");
    await signIn(page, email);
    await page.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });
    expect(new URL(page.url()).origin).toBe(origin);
  });

  test("next pointing back at an auth page lands on Home", async ({ browser }) => {
    const { email, page } = await accountAndSignedOutPage(browser);
    await page.goto("/auth/login?next=%2Fauth%2Flogin");
    await signIn(page, email);
    await page.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });
  });
});

test.describe("Tab titles", () => {
  test("public pages each name themselves", async ({ page }) => {
    const cases: [string, string][] = [
      ["/auth/login", "Sign in · Team Companion"],
      ["/closed-beta", "Closed beta · Team Companion"],
      ["/legal/terms", "Terms of use · Team Companion"],
      ["/legal/impressum", "Impressum · Team Companion"],
      ["/no-such-page-m148", "Page not found · Team Companion"],
    ];
    for (const [path, title] of cases) {
      await page.goto(path);
      await expect(page).toHaveTitle(title, { timeout: 10000 });
    }
  });

  test("signed-in pages each name themselves", async ({ authedPage: page }) => {
    const cases: [string, string][] = [
      ["/stats", "Team games · Team Companion"],
      ["/opponents", "Opponents · Team Companion"],
      ["/profile", "Profile · Team Companion"],
    ];
    for (const [path, title] of cases) {
      await page.goto(path);
      await expect(page).toHaveTitle(title, { timeout: 15000 });
    }
  });
});
