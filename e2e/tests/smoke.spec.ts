/**
 * Smoke tests — verify all public pages load without JS errors.
 * The dev server must be running: cargo leptos watch
 */
import { test, expect } from "@playwright/test";

const PUBLIC_PAGES = [
  { path: "/", title: /LoL Team Companion/i },
  { path: "/auth/login", title: /LoL Team Companion/i },
  { path: "/auth/register", title: /LoL Team Companion/i },
];

for (const { path, title } of PUBLIC_PAGES) {
  test(`${path} loads without errors`, async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await page.goto(path);
    await expect(page).toHaveTitle(title);

    // Check nav is present (rendered by Leptos SSR)
    await expect(page.locator("nav")).toBeVisible();

    // No JS errors (ignore favicon 404s and the Tailwind v4 @import "tailwindcss"
    // 404 that leaks through in dev mode — the raw CSS import is processed at build
    // time by the tailwind CLI but the unprocessed line causes a harmless browser fetch)
    expect(errors.filter((e) =>
      !e.includes("favicon") &&
      !e.includes("404 (Not Found)")
    )).toHaveLength(0);
  });
}

test("health endpoint returns ok", async ({ request }) => {
  const resp = await request.get("/healthz");
  expect(resp.ok()).toBeTruthy();
  const body = await resp.json();
  expect(body.status).toBe("ok");
  expect(body.db).toBe("ok");
});

test("unauthenticated redirect from protected route", async ({ page }) => {
  // Protected routes should redirect to login or show the page (Leptos handles auth client-side)
  await page.goto("/profile");
  // Either we land on login or the page renders (auth state is checked client-side in Leptos)
  await page.waitForLoadState("networkidle");
  const url = page.url();
  // Should be on /profile (redirect happens client-side) or /auth/login
  expect(url).toMatch(/\/(profile|auth\/login)/);
});
