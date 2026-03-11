/**
 * Authenticated page smoke tests.
 * Verifies all protected routes load without JS errors.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

const AUTHED_PAGES = [
  { path: "/profile", content: /profile|account|riot/i },
  { path: "/team/dashboard", content: /team|dashboard|roster/i },
  { path: "/team/roster", content: /team|create|join/i },
  { path: "/team-builder", content: /team|builder|composition/i },
  { path: "/draft", content: /draft|ban|pick/i },
  { path: "/tree-drafter", content: /tree|draft/i },
  { path: "/stats", content: /stats|match|history/i },
  { path: "/champion-pool", content: /champion|pool|tier/i },
  { path: "/game-plan", content: /game|plan|strategy/i },
  { path: "/post-game", content: /post|game|review/i },
];

for (const { path, content } of AUTHED_PAGES) {
  test(`${path} loads without JS errors`, async ({ authedPage }) => {
    const errors: string[] = [];
    authedPage.on("pageerror", (e) => errors.push(e.message));
    authedPage.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });

    await authedPage.goto(path);
    await authedPage.waitForLoadState("networkidle");

    // Should not have redirected to login
    expect(authedPage.url()).not.toContain("/auth/login");

    // Page should have rendered something meaningful
    const body = await authedPage.textContent("body");
    expect(body).toMatch(content);

    // Nav should be present
    await expect(authedPage.locator("nav")).toBeVisible();

    // No JS errors (ignore 404s — the Tailwind v4 @import "tailwindcss" causes a
    // harmless 404 in dev mode on every page)
    const realErrors = errors.filter((e) =>
      !e.includes("favicon") &&
      !e.includes("404 (Not Found)")
    );
    expect(realErrors).toHaveLength(0);
  });
}
