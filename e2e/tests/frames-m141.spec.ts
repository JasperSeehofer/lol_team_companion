/**
 * M141 frame-strip capture: full-page screenshots of the surfaces the feedback
 * batch touches, at 390 and 1280 wide, plus a viewport-only "reading flow"
 * frame at 1280. Writes to $M141_SHOTS_DIR (skips writing when unset, so the
 * spec is still a smoke test). Tolerant on purpose: a surface with no data is
 * captured as its empty state.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, authenticatePage, createTeam, signInAsAdmin, TEST_PASSWORD } from "./fixtures";
import type { Page } from "@playwright/test";

const DIR = process.env.M141_SHOTS_DIR;
const SIZES = [
  { w: 390, h: 844 },
  { w: 1280, h: 800 },
];

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(1200);
}

async function frames(page: Page, name: string, prep?: () => Promise<void>) {
  for (const s of SIZES) {
    await page.setViewportSize({ width: s.w, height: s.h });
    if (prep) await prep();
    await page.waitForTimeout(400);
    if (!DIR) continue;
    fs.mkdirSync(DIR, { recursive: true });
    await page.screenshot({ path: path.join(DIR, `${name}-${s.w}.png`), fullPage: true });
    if (s.w === 1280) {
      const main = page.locator("main").first();
      await main.evaluate((el) => el.scrollIntoView()).catch(() => {});
      await page.evaluate(() => window.scrollTo(0, 120)).catch(() => {});
      await page.waitForTimeout(200);
      await page.screenshot({ path: path.join(DIR, `${name}-1280-flow.png`), fullPage: false });
      await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
    }
  }
}

async function shoot(page: Page, name: string, url: string, prep?: () => Promise<void>) {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(url);
  await settle(page);
  await frames(page, name, prep);
  await expect(page.locator("body")).toBeVisible();
}

async function signInSeed(page: Page) {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", "seed-alpha-top@example.com");
  await page.fill("input[name=password]", "seedpass123");
  await page.click("main button[type=submit]");
  const ok = await page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 })
    .then(() => true)
    .catch(() => false);
  if (!ok) {
    // No seed data (local run): fall back to a fresh account with a team.
    await authenticatePage(page);
    await createTeam(page, `Frames_${Date.now()}`);
  }
  await settle(page);
}

test.describe("M141 frames", () => {
  test.setTimeout(240_000);

  test("team surfaces", async ({ page }) => {
    void TEST_PASSWORD;
    await signInSeed(page);

    // Match detail: first synced game linked from /stats, else the stats page itself.
    await page.goto("/stats");
    await settle(page);
    const link = page.locator('a[href^="/match/"]').first();
    if (await link.count()) {
      const href = await link.getAttribute("href");
      await shoot(page, "match-detail", href!);
    } else {
      await shoot(page, "match-detail-no-seeded-match-stats", "/stats");
    }

    await shoot(page, "game-plan", "/game-plan");
    await shoot(page, "champion-pool", "/champion-pool");
    await shoot(page, "tree-drafter", "/tree-drafter");
    await shoot(page, "post-game", "/post-game");
    await shoot(page, "me", "/me");
    await shoot(page, "bugs-mine", "/bugs?tab=mine");

    // Feedback widget opened: trigger -> select mode -> Skip -> report dialog.
    await shoot(page, "feedback-widget", "/", async () => {
      if (await page.getByRole("dialog").count()) return;
      await page.getByRole("button", { name: "Report a bug or a wish" }).first().click().catch(() => {});
      await page.waitForTimeout(300);
      const skip = page.getByRole("button", { name: /skip/i }).first();
      if (await skip.isVisible().catch(() => false)) await skip.click().catch(() => {});
      await page.waitForTimeout(500);
    });
  });

  test("admin surfaces", async ({ page }) => {
    await signInAsAdmin(page);
    await shoot(page, "admin-invites", "/admin/invites");
    await shoot(page, "admin-bugs", "/admin/bugs");
    await shoot(page, "admin-proposals", "/admin/proposals");
  });
});
