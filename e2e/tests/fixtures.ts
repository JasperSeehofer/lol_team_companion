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
/** At least 10 characters (the server's minimum since M1). */
export const TEST_PASSWORD = "Test1234!long";
/**
 * Invite code the server accepts. Start the server with
 * `BOOTSTRAP_INVITE_CODE=<code>` and `BOOTSTRAP_INVITE_MAX_USES=100000` (see
 * CLAUDE.md); the same value goes in `E2E_INVITE_CODE` here.
 */
export const INVITE_CODE = process.env.E2E_INVITE_CODE ?? "E2E-INVITE-CODE";
let userCounter = 0;

/** Register a new user — registration auto-logs in and redirects to /solo.
 *
 * Registration is gated by invite codes, checked on the server since M1:
 * `/auth/register` without `?invite=...` redirects to `/closed-beta`, and an
 * unknown or used-up code is rejected. The fixture uses `INVITE_CODE`.
 *
 * Phase 17 plan 17-06 also introduced D-03: new users default to solo mode and
 * `register_action()` now returns `/solo` (not `/team/dashboard`). The fixture
 * waits for `/solo`; tests that need a team explicitly use the `teamPage`
 * fixture which navigates from `/solo` → `/team/roster` → creates a team.
 */
export async function authenticatePage(page: Page): Promise<void> {
  const id = ++userCounter;
  const email = `pages_${TIMESTAMP}_${id}@test.invalid`;
  const username = `pagesuser_${TIMESTAMP}_${id}`;

  await page.goto(`/auth/register?invite=${INVITE_CODE}`);
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

  // New users start in solo mode, where /team/roster shows a "Switch to Team
  // Mode" gate instead of the create form. Switch first (the page reloads).
  const switchBtn = page.locator('button:has-text("Switch to Team Mode")');
  if (await switchBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
    await switchBtn.click();
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }

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
  adminPage: Page;
};

/**
 * The admin account. The server must list it in `ADMIN_EMAILS` (CI and
 * CLAUDE.md use `ADMIN_EMAILS=e2e-admin@test.invalid`).
 */
export const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? "e2e-admin@test.invalid";

/**
 * Sign in as the admin, registering the account on the first run against a
 * fresh database. Lands on /solo or /team/dashboard.
 */
export async function signInAsAdmin(page: Page): Promise<void> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", ADMIN_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");
  const landed = await page
    .waitForURL(/\/(solo|team\/dashboard)/, { timeout: 8000 })
    .then(() => true)
    .catch(() => false);
  if (!landed) {
    await page.goto(`/auth/register?invite=${INVITE_CODE}`);
    await page.fill("input[name=username]", `e2eadmin_${Date.now()}`);
    await page.fill("input[name=email]", ADMIN_EMAIL);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("button[type=submit]");
    await page.waitForURL("**/solo", { timeout: 20000 });
  }
  await page.waitForLoadState("load");
  // wasm-patterns rule 56: let the post-login hard navigation settle.
  await page.waitForTimeout(500);
}

/** A 1x1 transparent PNG served in place of champion art. */
const PLACEHOLDER_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
  "base64"
);

/**
 * Champion and item art comes from the app's own image endpoint
 * (`/img/champion/...`, which fetches from Data Dragon once per patch); the
 * browser loads nothing from a third party (role icons are typographic). The
 * page "load" event waits for every image, so one stalled upstream fetch
 * (common through the cloud-session proxy) used to time out `page.goto` at
 * random. Tests serve a placeholder instead: they check our pages, not the
 * image source. (`smoke.spec.ts` checks the endpoint itself.)
 */
async function stubExternalImages(page: Page): Promise<void> {
  await page.route(/\/img\/(champion|item)\//, (route) =>
    route.fulfill({ status: 200, contentType: "image/png", body: PLACEHOLDER_PNG })
  );
}

export const test = base.extend<Fixtures>({
  page: async ({ page }, use) => {
    await stubExternalImages(page);
    await use(page);
  },

  authedPage: async ({ page }, use) => {
    await authenticatePage(page);
    await use(page);
  },

  teamPage: async ({ page }, use) => {
    await authenticatePage(page);
    await createTeam(page, `AuditTeam_${Date.now()}`);
    await use(page);
  },

  adminPage: async ({ page }, use) => {
    await signInAsAdmin(page);
    await use(page);
  },
});

export { expect };

/**
 * Set the mode for the current page by clicking a ModeToggle segment
 * (a SegmentedControl: `role="radio"` buttons with `aria-checked`).
 *
 * Labels (components/region/controls.rs ModeToggle callers):
 *
 * Draft:   carousel→"Carousel", war-table→"Table", ledger→"Ledger"
 * Solo:    constellation→"Overview", forge→"Rank", journal→"Journal"
 *
 * @param page - Playwright Page
 * @param mode - canonical mode key, e.g. "carousel", "war-table", "journal"
 */
export async function setMode(
  page: import("@playwright/test").Page,
  mode: string
): Promise<void> {
  // Mapping from canonical mode key to its button label.
  const LABEL_MAP: Record<string, string[]> = {
    "carousel": ["Carousel"],
    "war-table": ["Table"],
    "ledger": ["Ledger"],
    "constellation": ["Overview"],
    "forge": ["Rank"],
    "journal": ["Journal"],
  };

  const candidates = LABEL_MAP[mode] ?? [
    // Fallback: try title-case and uppercase+underscore
    mode.charAt(0).toUpperCase() + mode.slice(1),
    mode.toUpperCase().replace(/-/g, "_"),
  ];

  // Build a CSS selector that matches any candidate label (exact text match)
  const selector = candidates
    .map((label) => `[data-segmented="page-mode"] button[role="radio"]:text-is("${label}")`)
    .join(", ");
  await page.click(selector, { timeout: 10000 });
  await page.waitForTimeout(500);
}
