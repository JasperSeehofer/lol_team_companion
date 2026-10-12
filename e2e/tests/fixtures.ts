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

/** Wait until the page is Home (`/`), where sign-in and registration land. */
export async function waitForHome(page: Page, timeout = 20000): Promise<void> {
  await page.waitForURL((u) => new URL(u).pathname === "/", { timeout });
}

/** Register a new user. Registration auto-logs in and lands on Home (`/`):
 * Match Day for team members, the start page (create or join a team) for a
 * new user.
 *
 * Registration is gated by invite codes, checked on the server since M1:
 * `/auth/register` without `?invite=...` redirects to `/closed-beta`, and an
 * unknown or used-up code is rejected. The fixture uses `INVITE_CODE`.
 * Tests that need a team use the `teamPage` fixture, which creates one on
 * `/team/roster`.
 */
export async function authenticatePage(page: Page): Promise<void> {
  // The worker index keeps parallel workers (same TIMESTAMP) apart.
  const id = `${process.env.TEST_WORKER_INDEX ?? "0"}x${++userCounter}`;
  const email = `pages_${TIMESTAMP}_${id}@test.invalid`;
  const username = `pagesuser_${TIMESTAMP}_${id}`;

  await page.goto(`/auth/register?invite=${INVITE_CODE}`);
  await page.fill("input[name=username]", username);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");

  // Registration fires window.location.set_href("/") from a WASM Effect.
  await waitForHome(page);
  await page.waitForLoadState("load");
  // Give WASM a moment to hydrate and fire any remaining Effects (rule 56).
  await page.waitForTimeout(500);

  if (page.url().includes("/auth/")) {
    throw new Error(`Authentication failed — ended at ${page.url()}`);
  }
}

/**
 * Create a team for the current authenticated user on /team/roster and
 * wait for the team page. Safe to call when the user already has a team
 * (the create form is not shown then).
 */
export async function createTeam(page: Page, name: string): Promise<void> {
  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  const teamNameInput = page.locator('input[name="name"]');
  if (await teamNameInput.isVisible({ timeout: 5000 }).catch(() => false)) {
    await teamNameInput.fill(name);
    const regionSelect = page.locator('select[name="region"]');
    if (await regionSelect.isVisible({ timeout: 1000 }).catch(() => false)) {
      await regionSelect.selectOption("NA");
    }
    await page.getByTestId("create-team").click();
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
 * fresh database. Lands on Home (`/`).
 */
export async function signInAsAdmin(page: Page): Promise<void> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", ADMIN_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");
  const landed = await page
    // Generous: argon2 in a debug build under a long e2e run can take a while.
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 15000 })
    .then(() => true)
    .catch(() => false);
  if (!landed) {
    await page.goto(`/auth/register?invite=${INVITE_CODE}`);
    await page.fill("input[name=username]", `e2eadmin_${Date.now()}`);
    await page.fill("input[name=email]", ADMIN_EMAIL);
    await page.fill("input[name=password]", TEST_PASSWORD);
    await page.click("main button[type=submit]");
    await waitForHome(page);
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
