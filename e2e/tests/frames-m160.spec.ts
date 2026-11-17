/**
 * M160 frame-strip capture: full-page screenshots of the surfaces this
 * mission touches at 390 and 1280 wide, plus a viewport-only "reading flow"
 * frame at 1280 (the page as opened, nothing expanded). Writes
 * `<surface>-390.png`, `<surface>-1280.png` and `<surface>-1280-flow.png` to
 * $MISSION_FRAMES_DIR/<sub>/ where <sub> = process.env.MISSION_FRAMES_SUB || "after".
 * When it is unset the spec is a smoke test.
 * Modelled on frames-m148.spec.ts and frames-m152.spec.ts.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, authenticatePage, createTeam, TEST_PASSWORD } from "./fixtures";
import type { Page } from "@playwright/test";
import { navigateTo } from "./helpers";

const DIR = process.env.MISSION_FRAMES_DIR;
const SUB = process.env.MISSION_FRAMES_SUB || "after";
const SIZES = [
  { w: 390, h: 844 },
  { w: 1280, h: 800 },
];

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(1200);
}

/** Close the toasts left by setup steps, so the flow frame shows the page as opened. */
async function dismissToasts(page: Page) {
  const buttons = page.locator("[data-toast] button[aria-label=Dismiss]");
  for (let i = 0; i < 6 && (await buttons.count()) > 0; i++) {
    await buttons.first().click({ timeout: 2000 }).catch(() => {});
    await page.waitForTimeout(150);
  }
}

async function frames(page: Page, name: string, prep?: () => Promise<void>) {
  for (const s of SIZES) {
    await page.setViewportSize({ width: s.w, height: s.h });
    if (prep) await prep();
    await page.waitForTimeout(400);
    if (s.w === 390) {
      // No sideways scroll on a phone: the page is never wider than the viewport.
      const sw = await page.evaluate(() => document.documentElement.scrollWidth);
      expect.soft(sw, `${name} at 390 is ${sw}px wide`).toBeLessThanOrEqual(s.w);
    }
    if (!DIR) continue;
    const outDir = path.join(DIR, SUB);
    fs.mkdirSync(outDir, { recursive: true });
    // Full-page shots from the top, so the sticky header is painted in place.
    await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
    await page.waitForTimeout(200);
    await page.screenshot({ path: path.join(outDir, `${name}-${s.w}.png`), fullPage: true });
    if (s.w === 1280) {
      await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
      await page.waitForTimeout(200);
      await page.screenshot({ path: path.join(outDir, `${name}-1280-flow.png`), fullPage: false });
    }
  }
}

/** Wait for network idle like the M148 spec does */
async function waitNetworkIdle(page: Page) {
  await page.waitForLoadState("networkidle");
}

test.describe("M160 frames", () => {
  test.setTimeout(240_000);

  // 1. solo-nav: a fresh authedPage (no team) on `/` — desktop nav at 1280; at 390 open the phone menu
  test("solo-nav: no team on home, phone menu open at 390", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/");
    await waitNetworkIdle(page);
    await settle(page);
    await frames(page, "solo-nav");
    
    // At 390 open the phone menu
    await page.setViewportSize({ width: 390, height: 844 });
    await navigateTo(page, "/");
    await waitNetworkIdle(page);
    try {
      await page.getByTestId("phone-menu-trigger").click({ timeout: 3000 });
      await page.waitForTimeout(500);
    } catch {}
    if (DIR) {
      const outDir = path.join(DIR, SUB);
      fs.mkdirSync(outDir, { recursive: true });
      await page.screenshot({ path: path.join(outDir, "solo-nav-menu-390.png"), fullPage: true });
    }
  });

  // 2. no-team-draft, no-team-game-plan, no-team-post-game: authedPage on /draft, /game-plan, /post-game
  test("no-team-draft: authedPage on /draft", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/draft");
    await waitNetworkIdle(page);
    await settle(page);
    // Wrap in try/catch for before run where selector might not exist
    try {
      await expect(page.getByTestId("no-team-page")).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "no-team-draft");
  });

  test("no-team-game-plan: authedPage on /game-plan", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/game-plan");
    await waitNetworkIdle(page);
    await settle(page);
    try {
      await expect(page.getByTestId("no-team-page")).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "no-team-game-plan");
  });

  test("no-team-post-game: authedPage on /post-game", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/post-game");
    await waitNetworkIdle(page);
    await settle(page);
    try {
      await expect(page.getByTestId("no-team-page")).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "no-team-post-game");
  });

  // 3. checklist: teamPage on `/` (the getting-started checklist visible)
  test("checklist: teamPage on / with getting-started checklist", async ({ teamPage: page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/");
    await waitNetworkIdle(page);
    await settle(page);
    try {
      const checklist = page.locator("[data-checklist]");
      await expect(checklist).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "checklist");
  });

  // 4. team-page-none: authedPage on /team/dashboard (create/join)
  test("team-page-none: authedPage on /team/dashboard (create/join)", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/team/dashboard");
    await waitNetworkIdle(page);
    await settle(page);
    try {
      await expect(page.getByTestId("create-team")).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "team-page-none");
  });

  // 4. team-page-leader: teamPage on /team/dashboard
  test("team-page-leader: teamPage on /team/dashboard", async ({ teamPage: page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await navigateTo(page, "/team/dashboard");
    await waitNetworkIdle(page);
    await settle(page);
    try {
      await expect(page.getByRole("heading", { name: "Roster" })).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "team-page-leader");
  });

  // 4. team-page-coach: reuse coach-rights.spec.ts's setup to make a coach and capture the coach's /team/dashboard
  test("team-page-coach: coach's /team/dashboard", async ({ browser }) => {
    const STAMP = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;
    
    // Create leader and team
    const leader = await (await browser.newContext()).newPage();
    await authenticatePage(leader);
    const teamName = `CoachFrames_${STAMP}`;
    await createTeam(leader, teamName);
    
    // Leader mints an invite link
    await leader.goto("/team/dashboard");
    await leader.waitForLoadState("networkidle");
    const section = leader.locator("[data-team-invites]");
    await section.locator("#team-invite-uses").fill("2");
    await section.locator("#team-invite-days").fill("7");
    await section.getByTestId("mint-team-invite").click();
    const link = (await section.getByTestId("team-invite-link").locator("code").textContent()) ?? "";
    
    // Coach account joins with the link
    const coach = await (await browser.newContext()).newPage();
    const { username: coachName } = await authenticatePage(coach);
    await coach.goto(link);
    await coach.waitForLoadState("networkidle");
    await coach.getByTestId("invite-join-existing").getByTestId("invite-join-button").click();
    await coach.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });
    await coach.waitForLoadState("load");
    await coach.waitForTimeout(500);
    
    // Leader makes the coach a coach
    await leader.goto("/team/dashboard");
    await leader.waitForLoadState("networkidle");
    const coachRow = leader.locator("[data-roster-row]", { hasText: coachName }).first();
    await coachRow.getByRole("button", { name: /Roster actions for/ }).click();
    await leader.getByRole("menuitem", { name: "Make coach" }).click();
    await expect(leader.locator('[data-roster-group="coaches"]')).toContainText(coachName, { timeout: 10000 });
    
    // Capture the coach's /team/dashboard
    await coach.goto("/team/dashboard");
    await coach.waitForLoadState("networkidle");
    await coach.setViewportSize({ width: 1280, height: 800 });
    await settle(coach);
    try {
      await expect(coach.getByRole("heading", { name: "Roster" })).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(coach, "team-page-coach");
    
    await leader.close();
    await coach.close();
  });

  // 5. auto-place: reuse team-invites.spec.ts's flow (leader mints an invite with position Mid, a new account registers with it) and capture the leader's /team/dashboard with the [data-auto-placed] notice.
  test("auto-place: leader's /team/dashboard with auto-placed notice", async ({ browser }) => {
    const STAMP = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;
    
    // Create leader and team
    const leader = await (await browser.newContext()).newPage();
    await authenticatePage(leader);
    const teamName = `AutoPlace_${STAMP}`;
    await createTeam(leader, teamName);
    
    // Leader mints an invite link with position Mid
    await leader.goto("/team/dashboard");
    await leader.waitForLoadState("networkidle");
    const section = leader.locator("[data-team-invites]");
    await section.locator("#team-invite-uses").fill("2");
    await section.locator("#team-invite-days").fill("7");
    try {
      await section.locator("#team-invite-position").selectOption("Mid", { timeout: 3000 });
    } catch {}
    await section.getByTestId("mint-team-invite").click();
    const link = (await section.getByTestId("team-invite-link").locator("code").textContent()) ?? "";
    
    // New account registers with the invite
    const player = await (await browser.newContext()).newPage();
    await player.goto(link);
    await player.fill("input[name=username]", `auto_${STAMP}`);
    await player.fill("input[name=email]", `auto_${STAMP}@test.invalid`);
    await player.fill("input[name=password]", TEST_PASSWORD);
    await player.check("#register-terms");
    await player.click("main button[type=submit]");
    await player.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });
    await player.waitForLoadState("load");
    await player.waitForTimeout(500);
    
    // Capture the leader's /team/dashboard with the [data-auto-placed] notice
    await leader.goto("/team/dashboard");
    await leader.waitForLoadState("networkidle");
    await leader.setViewportSize({ width: 1280, height: 800 });
    await settle(leader);
    try {
      await expect(leader.locator("[data-auto-placed]")).toBeVisible({ timeout: 5000 });
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(leader, "auto-place");
    
    await leader.close();
    await player.close();
  });

  // 6. player-notes: reuse opponent-notes.spec.ts's setup (dossier + player); capture the row collapsed with a saved note preview (`-flow`), and once expanded.
  test("player-notes: dossier player with note collapsed and expanded", async ({ teamPage: page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    
    // Create opponent and player with note (from opponent-notes.spec.ts pattern)
    await navigateTo(page, "/opponents");
    const opponentName = `NotesOpp_${Date.now()}`;
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(opponentName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();
    await expect(page.getByTestId("dossier").getByRole("heading", { level: 2, name: opponentName })).toBeVisible({ timeout: 10000 });
    
    // Add a player to the mid role
    await page.locator("#opp-add-mid").fill("NotesPlayer");
    await page.locator('li[data-role="mid"]').getByRole("button", { name: "Add player" }).click();
    
    // Wait for the player row to appear
    const playerRow = page.locator('li[data-role="mid"]');
    await expect(playerRow).toContainText("NotesPlayer", { timeout: 5000 });
    
    // Add a note
    try {
      await playerRow.getByRole("button", { name: "Add note" }).click({ timeout: 3000 });
      const textarea = playerRow.locator("textarea");
      await expect(textarea).toBeVisible({ timeout: 2000 });
      const noteText = "This player is very aggressive in lane";
      await textarea.fill(noteText);
      // Blur to save
      await playerRow.locator("#opp-riot-mid").click({ timeout: 3000 });
      await expect(playerRow.getByRole("button", { name: "Edit note" })).toBeVisible({ timeout: 3000 });
    } catch {}
    
    // Capture collapsed with preview (this is the -flow frame)
    await page.setViewportSize({ width: 1280, height: 800 });
    await settle(page);
    try {
      const preview = playerRow.getByTestId("opp-note-preview");
      await expect(preview).toHaveText(noteText);
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "player-notes");
    
    // Also capture expanded
    try {
      await playerRow.getByRole("button", { name: "Edit note" }).click({ timeout: 3000 });
      await expect(playerRow.locator("textarea")).toBeVisible({ timeout: 2000 });
    } catch {}
    await page.waitForTimeout(300);
    if (DIR) {
      const outDir = path.join(DIR, SUB);
      fs.mkdirSync(outDir, { recursive: true });
      await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
      await page.waitForTimeout(200);
      await page.screenshot({ path: path.join(outDir, "player-notes-expanded-1280.png"), fullPage: false });
    }
  });

  // 7. tree-meta: reuse tree-meta.spec.ts's setup; capture /tree-drafter with a tree open (title "vs {dossier}") and with the Edit form open.
  test("tree-meta: /tree-drafter with tree open and Edit form open", async ({ teamPage: page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    const ts = Date.now();
    const dossierName = `MetaDossier_${ts}`;
    const treeName = `MetaTree_${ts}`;
    
    // Create a scouting dossier on /opponents (from tree-meta.spec.ts)
    await navigateTo(page, "/opponents");
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(dossierName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();
    await expect(page.getByTestId("dossier-list").getByRole("button", { name: dossierName })).toBeVisible({ timeout: 10000 });
    
    // Create a tree, picking the dossier in the opponent select
    await navigateTo(page, "/tree-drafter");
    await page.locator("#tree-name").fill(treeName);
    await page.locator("#tree-opponent").selectOption({ label: dossierName });
    await page.getByRole("button", { name: "Create tree" }).click();
    const list = page.getByTestId("tree-list");
    await expect(list.getByRole("button", { name: treeName })).toHaveAttribute("aria-pressed", "true", { timeout: 5000 });
    
    // Capture with tree open (title "vs {dossier}")
    await settle(page);
    try {
      await expect(page.getByTestId("tree-title")).toHaveText(treeName);
      await expect(page.getByTestId("tree-opponent-label")).toHaveText(`vs ${dossierName}`);
    } catch (e) {
      // Before run: just capture the page as it is
    }
    await frames(page, "tree-meta");
    
    // Capture with the Edit form open
    try {
      await page.getByTestId("tree-edit-open").click({ timeout: 3000 });
      const form = page.getByTestId("tree-edit-form");
      await expect(form).toBeVisible({ timeout: 3000 });
    } catch {}
    await page.waitForTimeout(300);
    if (DIR) {
      const outDir = path.join(DIR, SUB);
      fs.mkdirSync(outDir, { recursive: true });
      await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
      await page.waitForTimeout(200);
      await page.screenshot({ path: path.join(outDir, "tree-meta-edit-1280.png"), fullPage: false });
    }
  });
});
