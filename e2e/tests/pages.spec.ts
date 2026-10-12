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
  { path: "/draft", content: /draft|ban|pick/i },
  { path: "/tree-drafter", content: /tree|draft/i },
  { path: "/stats", content: /team games|games/i },
  { path: "/champion-pool", content: /champion|pool|tier/i },
  { path: "/game-plan", content: /game|plan|strategy/i },
  { path: "/post-game", content: /post|game|review/i },
  { path: "/", content: /match day|team/i },
  { path: "/me", content: /my games|ranked|goals/i },
  { path: "/opponents", content: /opponents|dossier/i },
  { path: "/action-items", content: /action items/i },
  { path: "/personal-learnings", content: /learnings|personal|reflections/i },
  { path: "/personal-learnings/new", content: /learning|new|happened|learned/i },
  { path: "/bugs", content: /bug board/i },
  { path: "/what-next", content: /what next/i },
  // /admin/invites is admin-only (404 for others): see admin-invites.spec.ts
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
    await expect(authedPage.locator('nav[aria-label="Primary"]')).toBeVisible();

    // No JS errors (asset 404s and sandbox proxy tunnel failures for
    // third-party hosts are environment noise)
    const realErrors = errors.filter((e) =>
      !e.includes("favicon") &&
      !e.includes("404 (Not Found)") &&
      !e.includes("ERR_TUNNEL_CONNECTION_FAILED")
    );
    expect(realErrors).toHaveLength(0);
  });
}

// Parked pages and the old solo dashboard redirect.
for (const [from, to] of [["/solo", "/me"], ["/analytics", "/stats"], ["/team-builder", "/draft"]]) {
  test(`${from} redirects to ${to}`, async ({ authedPage }) => {
    await authedPage.goto(from);
    await authedPage.waitForURL((u) => new URL(u).pathname === to, { timeout: 15000 });
    // A redirect must not leave its status behind: static files still load
    // with 200 afterwards (a Leptos <Redirect> once made them 302 at random).
    for (let i = 0; i < 8; i++) {
      const res = await authedPage.request.get("/pkg/lol_team_companion.js", { maxRedirects: 0 });
      expect(res.status()).toBe(200);
    }
  });
}
