/**
 * M152 frame strips (items 3 and 4): the hero surfaces and Match Day's field
 * card at 390 and 1280, the landing's reading-flow frame (390, full page,
 * nothing expanded), a RoleLabel in context at 390 and the /design Drawn
 * Rift section at 1280. Writes PNGs to the top level of $MISSION_FRAMES_DIR
 * (CI commits them as frames/after); when it is unset the spec is a smoke
 * test of the same surfaces.
 *
 * Before frames: CI renders only this branch, so there is no `before/` set
 * here; the before strip comes from M148's committed frames of main.
 *
 * Seeded surfaces (share, Match Day, roster) sign in as the seeded leader
 * `seed-beta-top@example.com`; CI seeds before the server starts, so there a
 * missing seed is a failure, locally a skip.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, signInAsAdmin } from "./fixtures";
import type { Page } from "@playwright/test";

const DIR = process.env.MISSION_FRAMES_DIR;
const PHONE = { width: 390, height: 844 };
const DESK = { width: 1280, height: 800 };

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(1000);
}

async function shot(page: Page, name: string, fullPage: boolean) {
  const sw = await page.evaluate(() => document.documentElement.scrollWidth);
  const vw = page.viewportSize()?.width ?? 0;
  expect(sw, `${name}: ${sw}px wide in a ${vw}px viewport`).toBeLessThanOrEqual(vw);
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
  await page.waitForTimeout(200);
  await page.screenshot({ path: path.join(DIR, `${name}.png`), fullPage });
}

/** `<name>-390.png` and `<name>-1280.png`: the first viewport at each width. */
async function pair(page: Page, url: string, name: string, ready: (p: Page) => Promise<void>) {
  for (const size of [PHONE, DESK]) {
    await page.setViewportSize(size);
    await gotoSettled(page, url);
    await settle(page);
    await ready(page);
    await shot(page, `${name}-${size.width}`, false);
  }
}

/**
 * Sign-in navigates twice (the server's redirect, then the hydrated page's
 * hard navigation to the same address), so a goto right after it can be
 * "interrupted by another navigation". Settle first, and retry the goto
 * when the late navigation still lands on it.
 */
async function gotoSettled(page: Page, url: string) {
  for (let attempt = 0; ; attempt++) {
    try {
      return await page.goto(url);
    } catch (e) {
      if (attempt >= 2 || !/interrupted by another navigation/.test(String(e))) throw e;
      await page.waitForLoadState("load").catch(() => {});
    }
  }
}

async function signInSeeded(page: Page): Promise<boolean> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", "seed-beta-top@example.com");
  await page.fill("input[name=password]", "seedpass123");
  await page.click("main button[type=submit]");
  const ok = await page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 15000 })
    .then(async () => {
      await page.waitForLoadState("networkidle").catch(() => {});
      return true;
    })
    .catch(() => false);
  if (process.env.CI) expect(ok, "CI must seed the database (seed tool) before starting the server").toBe(true);
  test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
  await page.waitForTimeout(500);
  return ok;
}

test.describe("M152 frames", () => {
  test.setTimeout(180_000);

  test("landing (hero) and its reading flow", async ({ page }) => {
    await pair(page, "/", "landing", async (p) => {
      await expect(p.locator("[data-hero] [data-rift]")).toBeVisible();
    });
    // Desk full page too, and the phone reading flow: the whole page as
    // opened, nothing expanded (Law 11).
    await page.setViewportSize(DESK);
    await page.goto("/");
    await settle(page);
    await shot(page, "landing-1280-full", true);
    await page.setViewportSize(PHONE);
    await page.goto("/");
    await settle(page);
    await shot(page, "landing-390-flow", true);
  });

  test("closed-beta page", async ({ page }) => {
    await pair(page, "/closed-beta", "closed-beta", async (p) => {
      await expect(p.locator("[data-hero] [data-rift]")).toBeVisible();
    });
  });

  test("share page (seeded game plan) and Match Day field card", async ({ page, browser }) => {
    await signInSeeded(page);

    // Match Day: the seeded team has a scrim tomorrow.
    await pair(page, "/", "match-day", async (p) => {
      await expect(p.locator("[data-field-card]")).toBeVisible({ timeout: 15000 });
    });
    await page.setViewportSize(PHONE);
    await gotoSettled(page, "/");
    await settle(page);
    await shot(page, "match-day-390-full", true);

    // RoleLabel in context: the roster at 390.
    await page.setViewportSize(PHONE);
    await gotoSettled(page, "/team/dashboard");
    await settle(page);
    await expect(page.locator("[data-roster-row]").first()).toBeVisible({ timeout: 15000 });
    await shot(page, "rolelabel-roster-390", false);

    // Share the seeded Game 1 plan and open it as a guest.
    await page.setViewportSize(DESK);
    await gotoSettled(page, "/game-plan");
    const item = page.getByTestId("plan-list").getByRole("button", { name: /Game 1 plan/ });
    await expect(item).toBeVisible({ timeout: 15000 });
    await item.click();
    await expect(page.locator("#plan-name")).toHaveValue(/Game 1 plan/, { timeout: 10000 });
    await page.getByTestId("plan-share").click();
    const dialog = page.getByRole("dialog", { name: /Share this game plan/ });
    await dialog.getByTestId("create-share-link").click();
    const url = (await dialog.locator("[data-share-url]").first().textContent({ timeout: 10000 }))?.trim() ?? "";
    expect(url).toMatch(/\/s\/[a-z2-9]{32}$/);

    const ctx = await browser.newContext();
    await ctx.route(/\/img\/(champion|item)\//, (route) => route.fulfill({ status: 404, body: "" }));
    const guest = await ctx.newPage();
    await pair(guest, url, "share", async (p) => {
      await expect(p.locator("[data-share-strip]")).toBeVisible({ timeout: 15000 });
    });
    await guest.setViewportSize(PHONE);
    await guest.goto(url);
    await settle(guest);
    await shot(guest, "share-390-full", true);
    await ctx.close();
  });

  test("/design Drawn Rift section", async ({ page }) => {
    await signInAsAdmin(page);
    await page.setViewportSize(DESK);
    await gotoSettled(page, "/design#rift");
    await settle(page);
    const section = page.locator('section[data-gallery="rift"]');
    await expect(section).toBeVisible({ timeout: 15000 });
    await section.scrollIntoViewIfNeeded();
    if (DIR) {
      fs.mkdirSync(DIR, { recursive: true });
      await section.screenshot({ path: path.join(DIR, "design-rift-1280.png") });
    }
  });
});
