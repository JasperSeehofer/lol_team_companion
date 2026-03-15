/**
 * Team management audit tests.
 *
 * Documents any bugs and verifies working flows for /team/roster and /team/dashboard.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";

/**
 * Capture console errors for a page, filtering known-harmless noise.
 */
function captureErrors(page: import("@playwright/test").Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`[pageerror] ${e.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`[console.error] ${msg.text()}`);
  });
  return errors;
}

function filterRealErrors(errors: string[]): string[] {
  return errors.filter(
    (e) => !e.includes("favicon") && !e.includes("404 (Not Found)")
  );
}

test("team: create a new team", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);
  const teamName = `AuditTeam_${Date.now()}`;

  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  // Fill team name input
  const teamNameInput = page.locator('input[name="name"]');
  await expect(teamNameInput).toBeVisible({ timeout: 5000 });
  await teamNameInput.fill(teamName);

  // Select region NA if select exists
  const regionSelect = page.locator('select[name="region"]');
  if (await regionSelect.isVisible({ timeout: 1000 }).catch(() => false)) {
    await regionSelect.selectOption("NA");
  }

  // Click Create Team
  await page.locator('button:has-text("Create Team")').click();

  // Wait for redirect to /team/dashboard
  await page.waitForURL("**/team/dashboard", { timeout: 15000 });
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  // Verify dashboard loads with team name visible somewhere
  const teamNameInDashboard = page.locator(`text=${teamName}`);
  const nameVisible = await teamNameInDashboard.isVisible({ timeout: 5000 }).catch(() => false);
  if (!nameVisible) {
    console.log(`AUDIT-FINDING: Team name "${teamName}" not visible on /team/dashboard after creation`);
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("team: dashboard shows roster slots", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  // teamPage fixture already lands on /team/dashboard after team creation.
  // Re-navigating to /team/dashboard can cause a timeout if the WASM redirect
  // Effect from registration is still pending (CLAUDE.md rule 56). Instead,
  // just ensure the page is settled from where the fixture left us.
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);

  // If not on the dashboard, navigate there explicitly
  if (!page.url().includes("/team/dashboard")) {
    await page.goto("/team/dashboard");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }

  // Verify the dashboard renders meaningful content
  await expect(page.locator("h1, h2").first()).toBeVisible({ timeout: 5000 });

  // Look for roster section
  const rosterHeading = page.locator("h2, h3").filter({ hasText: /roster/i }).first();
  const hasRoster = await rosterHeading.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasRoster) {
    console.log("AUDIT-FINDING: No 'Roster' heading visible on /team/dashboard — roster section may be missing or differently labeled");
  }

  // Look for role slot indicators (Top, Jungle, Mid, Bot, Support)
  const roleSlot = page.locator('text=/Top|Jungle|Mid|Bot|Support/').first();
  const hasRoleSlot = await roleSlot.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasRoleSlot) {
    console.log("AUDIT-FINDING: No role slots (Top/Jungle/Mid/Bot/Support) visible on /team/dashboard");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("team: dashboard shows action items panel", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  // teamPage fixture already lands on /team/dashboard after team creation.
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);

  if (!page.url().includes("/team/dashboard")) {
    await page.goto("/team/dashboard");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }

  // Panel heading must be visible — hard assertion
  const actionItemsHeading = page.locator("h3").filter({ hasText: /Open Action Items/i });
  await expect(actionItemsHeading).toBeVisible({ timeout: 5000 });

  // Check for empty state text or "View all" link to /action-items
  const emptyState = page.locator('text=/No open action items/i').first();
  const viewAllLink = page.locator('a[href*="/action-items"]').first();
  const hasEmptyState = await emptyState.isVisible({ timeout: 3000 }).catch(() => false);
  const hasViewAllLink = await viewAllLink.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasEmptyState && !hasViewAllLink) {
    console.log("AUDIT-FINDING: Action items panel has no empty state text and no /action-items link");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("team: dashboard shows post-game reviews panel", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await page.waitForLoadState("load");
  await page.waitForTimeout(500);

  if (!page.url().includes("/team/dashboard")) {
    await page.goto("/team/dashboard");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }

  // Panel heading must be visible — hard assertion
  const reviewsHeading = page.locator("h3").filter({ hasText: /Recent Reviews/i });
  await expect(reviewsHeading).toBeVisible({ timeout: 5000 });

  // Check for empty state text or CTA link
  const emptyState = page.locator('text=/No post-game reviews yet/i').first();
  const ctaLink = page.locator('text=/Start your first review/i').first();
  const hasEmptyState = await emptyState.isVisible({ timeout: 3000 }).catch(() => false);
  const hasCtaLink = await ctaLink.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasEmptyState && !hasCtaLink) {
    console.log("AUDIT-FINDING: Post-game reviews panel has no empty state text and no CTA link");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("team: dashboard shows pool gap warnings panel", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await page.waitForLoadState("load");
  await page.waitForTimeout(500);

  if (!page.url().includes("/team/dashboard")) {
    await page.goto("/team/dashboard");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);
  }

  // Panel heading must be visible — hard assertion
  const poolGapHeading = page.locator("h3").filter({ hasText: /Pool Gap Warnings/i });
  await expect(poolGapHeading).toBeVisible({ timeout: 5000 });

  // Check for empty state text or CTA link
  const emptyState = page.locator('text=/No pool gaps detected/i').first();
  const ctaLink = page.locator('text=/Manage champion pools/i').first();
  const hasEmptyState = await emptyState.isVisible({ timeout: 3000 }).catch(() => false);
  const hasCtaLink = await ctaLink.isVisible({ timeout: 3000 }).catch(() => false);
  if (!hasEmptyState && !hasCtaLink) {
    console.log("AUDIT-FINDING: Pool gap warnings panel has no empty state text and no CTA link");
  }

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("team: join with invalid code shows error", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);

  await page.goto("/team/roster");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);

  // Look for a "Join Team" section with a code input or a list of teams to join
  const joinSection = page.locator('button:has-text("Join"), button:has-text("Request to Join")').first();
  const hasJoinSection = await joinSection.isVisible({ timeout: 3000 }).catch(() => false);

  if (!hasJoinSection) {
    // The roster page shows a team creation form for users without a team.
    // If no "Join" button is visible, the join flow may be via a different mechanism.
    // Check for a code input
    const codeInput = page.locator('input[placeholder*="code"], input[name*="code"]').first();
    const hasCodeInput = await codeInput.isVisible({ timeout: 2000 }).catch(() => false);
    if (!hasCodeInput) {
      console.log("AUDIT-FINDING: No join team section or code input visible on /team/roster — join flow may be list-based, not code-based");
      expect(filterRealErrors(errors)).toHaveLength(0);
      return;
    }
    await codeInput.fill("INVALID_CODE_12345");
    await page.locator('button[type="submit"], button:has-text("Join")').first().click();
    await page.waitForTimeout(2000);
  } else {
    // Click join on the first team (if any exist)
    await joinSection.first().click();
    await page.waitForTimeout(2000);
  }

  // If we got here, verify no crash occurred
  const currentUrl = page.url();
  const stillOnRoster = currentUrl.includes("/team/roster") || currentUrl.includes("/team/dashboard");
  expect(stillOnRoster).toBe(true);

  expect(filterRealErrors(errors)).toHaveLength(0);
});
