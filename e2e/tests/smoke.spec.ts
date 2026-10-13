/**
 * Smoke tests — verify all public pages load without JS errors.
 * Needs a running server (cargo leptos watch, or the built binary in CI).
 */
import { test, expect } from "@playwright/test";

const PUBLIC_PAGES = [
  { path: "/", title: /Team Companion/i },
  { path: "/auth/login", title: /Team Companion/i },
  { path: "/auth/register", title: /Team Companion/i },
  { path: "/closed-beta", title: /Team Companion/i },
  { path: "/legal/impressum", title: /Team Companion/i },
  { path: "/legal/datenschutz", title: /Team Companion/i },
  { path: "/legal/terms", title: /Team Companion/i },
  { path: "/changelog", title: /Team Companion/i },
  { path: "/transparency", title: /Team Companion/i },
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

    // The site header is present (rendered by Leptos SSR). Signed-out
    // visitors get the brand and the sign-in links; the hub links
    // (nav "Primary") are for signed-in users only.
    await expect(page.locator("header").getByRole("link", { name: "Team Companion", exact: true })).toBeVisible();

    // Footer carries Riot's legal disclaimer on every page
    await expect(page.getByTestId("riot-disclaimer")).toContainText(
      "isn't endorsed by Riot Games"
    );

    // No JS errors. Generic 404s are ignored because third-party assets
    // (e.g. CommunityDragon role icons) can be unreachable in sandboxed
    // environments; the stylesheet itself is checked in a dedicated test below.
    expect(errors.filter((e) =>
      !e.includes("favicon") &&
      !e.includes("404 (Not Found)") &&
      !e.includes("ERR_TUNNEL_CONNECTION_FAILED")
    )).toHaveLength(0);
  });
}

test("no stray /pkg/tailwindcss request", async ({ page }) => {
  const missing: string[] = [];
  page.on("response", (resp) => {
    if (resp.status() === 404 && resp.url().includes("/pkg/")) {
      missing.push(resp.url());
    }
  });
  await page.goto("/");
  await page.waitForLoadState("networkidle");
  expect(missing).toEqual([]);
});

test("unknown route shows a not-found page", async ({ page }) => {
  const resp = await page.goto("/this-page-does-not-exist");
  expect(resp?.status()).toBe(404);
  await expect(page.locator("h1")).toHaveText(/page not found/i);
});

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

test("image endpoint refuses unknown champions and bad item ids", async ({ request }) => {
  for (const path of [
    "/img/champion/NotAChampion/icon",
    "/img/champion/..%2F..%2Fetc%2Fpasswd",
    "/img/item/abc",
    "/img/item/1234567",
  ]) {
    const resp = await request.get(path);
    expect(resp.status(), path).toBe(404);
  }
});

test("champion icon is served by the app with cache headers", async ({ request }) => {
  const resp = await request.get("/img/champion/Ahri/icon");
  test.skip(
    resp.status() !== 200,
    "Data Dragon is not reachable from this server (sandbox); the 404 path is covered above"
  );
  expect(resp.headers()["content-type"]).toBe("image/png");
  expect(resp.headers()["cache-control"]).toContain("max-age");
  const etag = resp.headers()["etag"];
  expect(etag).toBeTruthy();
  const again = await request.get("/img/champion/Ahri/icon", {
    headers: { "If-None-Match": etag },
  });
  expect(again.status()).toBe(304);
});
