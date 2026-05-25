/**
 * Shared test fixtures — provides an authenticated page for e2e tests.
 *
 * Usage:
 *   import { test, expect } from "./fixtures";
 *   test("my test", async ({ authedPage }) => { ... });
 *   test("team test", async ({ teamPage }) => { ... });
 */
import { test as base, expect, type Page } from "@playwright/test";

const TIMESTAMP = Date.now();
const TEST_PASSWORD = "Test1234!";
let userCounter = 0;

/** Register a new user — registration auto-logs in and redirects to /solo.
 *
 * Phase 17 plan 17-06 introduced the closed-beta gate (D-16): `/auth/register`
 * without `?invite=...` redirects to `/closed-beta` via a WASM Effect. The
 * fixture must therefore always pass an invite query param. Phase 19.1 will
 * wire real server-side validation against the `invite_code` table; for now
 * any non-empty string is echoed into the hidden input and the registration
 * server fn accepts it.
 *
 * Phase 17 plan 17-06 also introduced D-03: new users default to solo mode and
 * `register_action()` now returns `/solo` (not `/team/dashboard`). The fixture
 * waits for `/solo`; tests that need a team explicitly use the `teamPage`
 * fixture which navigates from `/solo` → `/team/roster` → creates a team.
 */
async function authenticatePage(page: Page): Promise<void> {
  const id = ++userCounter;
  const email = `pages_${TIMESTAMP}_${id}@test.invalid`;
  const username = `pagesuser_${TIMESTAMP}_${id}`;

  await page.goto("/auth/register?invite=E2E-TEST");
  await page.fill("input[name=username]", username);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");

  // Registration auto-logs in and fires window.location.set_href("/solo")
  // via a WASM Effect (D-03). Wait for the URL to reach /solo.
  await page.waitForURL("**/solo", { timeout: 20000 });

  // Critical: The WASM hydration on /solo may fire ANOTHER redirect
  // (the register page's Effect can queue a redirect that fires after we land).
  // Wait for the page to fully stabilize — no more navigations.
  await page.waitForLoadState("load");
  // Give WASM a moment to hydrate and fire any remaining Effects
  await page.waitForTimeout(500);

  // Now verify we're properly authenticated
  if (page.url().includes("/auth/")) {
    throw new Error(`Authentication failed — ended at ${page.url()}`);
  }
}

/**
 * Create a team for the current authenticated user.
 * Navigates to /team/roster, fills the form if visible, and waits for redirect.
 * Safe to call even if the user already has a team — the form won't be visible.
 *
 * Source: derived from existing pipeline.spec.ts pattern (lines 25-40).
 */
export async function createTeam(page: Page, name: string): Promise<void> {
  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  const teamNameInput = page.locator('input[name="name"]');
  if (await teamNameInput.isVisible({ timeout: 3000 }).catch(() => false)) {
    await teamNameInput.fill(name);
    const regionSelect = page.locator('select[name="region"]');
    if (await regionSelect.isVisible({ timeout: 1000 }).catch(() => false)) {
      await regionSelect.selectOption("NA");
    }
    await page.click('button:has-text("Create Team")');
    await page.waitForURL("**/team/dashboard", { timeout: 15000 });
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }
}

type Fixtures = {
  authedPage: Page;
  teamPage: Page;
};

export const test = base.extend<Fixtures>({
  authedPage: async ({ page }, use) => {
    await authenticatePage(page);
    await use(page);
  },

  teamPage: async ({ page }, use) => {
    await authenticatePage(page);
    await createTeam(page, `AuditTeam_${Date.now()}`);
    await use(page);
  },
});

export { expect };

/**
 * Set the region (theme) for the current page.
 *
 * Sets the `lol_companion_theme` cookie at the browser-context level and reloads so
 * SSR honours the new region. Previously clicked the toggle and waited for the WASM
 * Effect to flip `data-theme` post-hydration; that approach only flipped tokens, not
 * structural region branches. Post-Phase-18.1, structural branches activate when SSR
 * sees the cookie (see `src/server/theme_layer.rs::theme_injection_middleware`).
 *
 * The helper is backwards-compatible — every existing caller continues to work because
 * the reload happens transparently here. Tests that call `setRegion(page, "pandemonium")`
 * BEFORE `page.goto(...)` work because the cookie is set in the browser context first,
 * so the next navigation already carries it. Tests that call `setRegion` AFTER
 * `page.goto(...)` work because the helper reloads the current page.
 *
 * Phase 18.1 spec_lock D-01: cookie name=`lol_companion_theme`, path=`/`, sameSite=`Lax`.
 * D-05: only `demacia` / `pandemonium` are valid values (TypeScript enforced).
 *
 * Edge case: when called BEFORE any `page.goto()`, `page.url()` is `about:blank`. The
 * `url` field in `addCookies` is set unconditionally to the dev server origin so the
 * cookie is scoped correctly even in that case; the subsequent `page.reload()` on
 * `about:blank` is a no-op and the next real navigation picks up the cookie.
 */
export async function setRegion(
  page: import("@playwright/test").Page,
  region: "demacia" | "pandemonium"
): Promise<void> {
  // 1. Inject the cookie at the browser-context layer (sent on next request).
  await page.context().addCookies([
    {
      name: "lol_companion_theme",
      value: region,
      url: "http://127.0.0.1:3020",
      path: "/",
      sameSite: "Lax",
    },
  ]);

  // 2. Reload so SSR re-renders the page with the new cookie. If the page hasn't
  //    navigated yet (about:blank), reload is a no-op and the next goto() will
  //    pick up the cookie. Skip the reload in that case to avoid spurious waits.
  const currentUrl = page.url();
  if (currentUrl && currentUrl !== "about:blank") {
    await page.reload({ waitUntil: "networkidle" });
    // wasm-patterns rule 56: 500ms hydrate settle.
    await page.waitForTimeout(500);

    // 3. Assert SSR honoured the cookie.
    const themeAttr = await page.getAttribute("html", "data-theme");
    if (themeAttr !== region) {
      throw new Error(`setRegion failed: expected ${region}, got ${themeAttr}`);
    }
  }
}

/**
 * Set the mode for the current page by clicking a ModeToggle button.
 *
 * Mode toggle buttons render labels per region per UI-SPEC.
 * Actual label text from controls.rs (18-08):
 *
 * Draft:   carousel→"Carousel"/"CAROUSEL",  war-table→"War Table"/"WAR_TABLE",  ledger→"Ledger"/"LEDGER"
 * Solo:    constellation→"Constellation"/"CONSTELLATION",  forge→"Forge"/"FORGE",  journal→"Journal"/"JOURNAL"
 * Team:    dashboard→"Dashboard"/"DASHBOARD",  brief→"Game Day Brief"/"GAME_DAY"
 *
 * The helper tries title-case, uppercase+underscore, and a label map for
 * multi-word labels ("war-table" → "War Table") and demacia-only labels
 * ("brief" → "Game Day Brief").
 *
 * @param page - Playwright Page
 * @param mode - canonical mode key, e.g. "carousel", "war-table", "brief"
 */
export async function setMode(
  page: import("@playwright/test").Page,
  mode: string
): Promise<void> {
  // Mapping from canonical mode key → known label variants (demacia + pandemonium)
  const LABEL_MAP: Record<string, string[]> = {
    "carousel": ["Carousel", "CAROUSEL"],
    "war-table": ["War Table", "WAR_TABLE"],
    "ledger": ["Ledger", "LEDGER"],
    "constellation": ["Constellation", "CONSTELLATION"],
    "forge": ["Forge", "FORGE"],
    "journal": ["Journal", "JOURNAL"],
    "dashboard": ["Dashboard", "DASHBOARD"],
    // "brief" has different labels per region: "Game Day Brief" (dem) vs "GAME_DAY" (pan)
    "brief": ["Game Day Brief", "GAME_DAY"],
  };

  const candidates = LABEL_MAP[mode] ?? [
    // Fallback: try title-case and uppercase+underscore
    mode.charAt(0).toUpperCase() + mode.slice(1),
    mode.toUpperCase().replace(/-/g, "_"),
  ];

  // Build a CSS selector that matches any candidate label (exact text match)
  const selector = candidates.map(label => `button:has-text("${label}")`).join(", ");
  await page.click(selector, { timeout: 10000 });
  await page.waitForTimeout(500);
}
