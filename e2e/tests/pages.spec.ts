/**
 * Authenticated page smoke tests.
 * Verifies all protected routes load without JS errors.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

const AUTHED_PAGES = [
  // /profile and /team/dashboard redirect to /throne after Plan 29-05 IA healing.
  // They are retained here without standalone-content assertions so the redirect
  // chain stays exercised; the destination /throne entry below covers content.
  { path: "/profile", content: /profile|account|riot|identity|throne/i },
  { path: "/team/dashboard", content: /team|dashboard|roster|throne/i },
  { path: "/team/roster", content: /team|create|join/i },
  { path: "/team-builder", content: /team|builder|composition/i },
  { path: "/draft", content: /draft|ban|pick/i },
  { path: "/tree-drafter", content: /tree|draft/i },
  { path: "/champion-pool", content: /champion|pool|tier/i },
  { path: "/game-plan", content: /game|plan|strategy/i },
  { path: "/solo", content: /dashboard|matches|ranked|solo/i },
  { path: "/personal-learnings/new", content: /learning|new|happened|learned/i },
  // Phase 17 plan 17-01 stubs (auth — filled by plan 06)
  { path: "/admin/invites", content: /admin|invites|placeholder/i },
  // Phase 27 War Room (SC1 smoke — route ships in Plan 27-07)
  { path: "/war-room", content: /draft|war|room|plan/i },
  // Phase 28 Library room (SC1 smoke — route ships in Plan 28-05; /stats, /analytics,
  // /post-game, /personal-learnings redirect here after IA healing)
  { path: "/library", content: /stats|match|analytics|post.game|learnings/i },
  // Phase 29 Throne room (SC1 smoke — route ships in Plan 29-05)
  { path: "/throne", content: /identity|dashboard|roster|development|solo/i },
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

    // Nav should be present. Use first() because the War Room page renders a
    // second <nav role="tablist"> (RoomSurface) alongside the primary nav —
    // strict mode would fail with an "resolved to 2 elements" error on /war-room.
    await expect(authedPage.locator("nav").first()).toBeVisible();

    // No JS errors (ignore 404s — the Tailwind v4 @import "tailwindcss" causes a
    // harmless 404 in dev mode on every page)
    const realErrors = errors.filter((e) =>
      !e.includes("favicon") &&
      !e.includes("404 (Not Found)")
    );
    expect(realErrors).toHaveLength(0);
  });
}
