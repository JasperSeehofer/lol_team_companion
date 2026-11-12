/**
 * M149 frame strips: match detail (card C's 44 px phone event row and card
 * A1's rail captions), the feedback dialog with the desktop area picker
 * (card B) and its phone floor, /me with the disabled "Sync my games"
 * (card D) and the new /admin/reimport (card A2), at 390 and 1280. Writes
 * PNGs to the top level of $MISSION_FRAMES_DIR (CI commits them as
 * frames/after); when it is unset the spec is a smoke test of the same
 * surfaces.
 *
 * Match detail signs in as the seeded `seed-alpha-top@example.com` and opens
 * the seed's demo game; CI seeds before the server starts, so there a
 * missing seed is a failure, locally a skip.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, authenticatePage, signInAsAdmin } from "./fixtures";
import type { Page } from "@playwright/test";

const DIR = process.env.MISSION_FRAMES_DIR;
const PHONE = { width: 390, height: 844 };
const DESK = { width: 1280, height: 800 };
const DEMO = "/match/SEED1_1000000001";
const AREA_BUTTON = "Select an area of the page";

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(1000);
}

async function shot(page: Page, name: string, fullPage: boolean, keepScroll = false) {
  const sw = await page.evaluate(() => document.documentElement.scrollWidth);
  const vw = page.viewportSize()?.width ?? 0;
  expect(sw, `${name}: ${sw}px wide in a ${vw}px viewport`).toBeLessThanOrEqual(vw);
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  if (!keepScroll) {
    await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
  }
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
  await page.fill("input[name=email]", "seed-alpha-top@example.com");
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

/** getDisplayMedia stubbed with a canvas stream, as in feedback-attachment.spec.ts. */
async function stubDisplayMedia(page: Page) {
  await page.addInitScript(() => {
    const getDisplayMedia = async () => {
      const c = document.createElement("canvas");
      c.width = 1280;
      c.height = 720;
      const ctx = c.getContext("2d")!;
      const paint = () => {
        ctx.fillStyle = "rgb(40, 120, 90)";
        ctx.fillRect(0, 0, c.width, c.height);
        ctx.fillStyle = "rgb(220, 225, 210)";
        ctx.fillRect(80, 80, 480, 60);
        ctx.fillRect(80, 180, 720, 24);
        ctx.fillRect(80, 220, 640, 24);
      };
      paint();
      setInterval(paint, 50);
      return c.captureStream(30);
    };
    if (!navigator.mediaDevices) {
      Object.defineProperty(navigator, "mediaDevices", { value: {}, configurable: true });
    }
    Object.defineProperty(navigator.mediaDevices, "getDisplayMedia", {
      value: getDisplayMedia,
      configurable: true,
      writable: true,
    });
  });
}

async function openDialog(page: Page) {
  await gotoSettled(page, "/draft");
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
  await page.locator('button[aria-label="Give Feedback"]').click();
  await page.locator("[data-feedback-hint]").getByRole("button", { name: "Skip" }).click();
  const d = page.getByRole("dialog", { name: "Give Feedback" });
  await expect(d.getByRole("button", { name: "Attach an image" })).toBeVisible();
  await page.waitForTimeout(500);
  return d;
}

test.describe("M149 frames", () => {
  test.setTimeout(120_000);

  test("match detail: phone event row and rail captions", async ({ page }) => {
    await signInSeeded(page);
    await pair(page, DEMO, "match-detail", async (p) => {
      await expect(p.locator("[data-scoreboard=blue] table")).toBeVisible({ timeout: 15000 });
    });
    // The phone reading flow: the whole page as opened, nothing expanded (Law 11).
    await page.setViewportSize(PHONE);
    await gotoSettled(page, DEMO);
    await settle(page);
    await expect(page.locator("[data-rail]")).toHaveCount(3, { timeout: 15000 });
    await shot(page, "match-detail-390-flow", true);
    // The timeline in view: the 44 px event row and the captions (the seed
    // game has no purchases, so the vision rail says to re-import it).
    const row = page.getByRole("group", { name: "Events near the playhead" });
    await expect(row).toBeVisible();
    await expect(page.getByText("Re-import this game to see recalls and positions.").first()).toBeVisible();
    await page.locator("[data-rail]").first().scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await shot(page, "match-detail-timeline-390", false, true);
  });

  test("feedback dialog at 1280 with the area button", async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: DESK });
    const page = await ctx.newPage();
    await stubDisplayMedia(page);
    await authenticatePage(page);
    const d = await openDialog(page);
    await expect(d.getByRole("button", { name: AREA_BUTTON })).toBeVisible();
    await shot(page, "feedback-dialog-1280", false, true);
    await ctx.close();
  });

  test("feedback area picker at 1280 with a selection", async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: DESK });
    const page = await ctx.newPage();
    await stubDisplayMedia(page);
    await authenticatePage(page);
    const d = await openDialog(page);
    await d.getByRole("button", { name: AREA_BUTTON }).click();
    const picker = page.getByRole("dialog", { name: AREA_BUTTON });
    await expect(picker).toBeVisible();
    await expect(picker.getByTestId("area-size")).toBeVisible();
    await page.keyboard.press("Alt+ArrowRight");
    await page.waitForTimeout(300);
    await shot(page, "feedback-picker-1280", false, true);
    await page.keyboard.press("Escape");
    await ctx.close();
  });

  test("feedback dialog at 390: the floor, no area button", async ({ browser }) => {
    const ctx = await browser.newContext({ viewport: PHONE });
    const page = await ctx.newPage();
    await stubDisplayMedia(page);
    await authenticatePage(page);
    const d = await openDialog(page);
    await expect(d.getByRole("button", { name: AREA_BUTTON })).toHaveCount(0);
    await shot(page, "feedback-dialog-390", false, true);
    await ctx.close();
  });

  test("/me with the disabled Sync my games", async ({ authedPage }) => {
    // A fresh account has no Riot link, so the button is disabled.
    await pair(authedPage, "/me", "me-sync", async (p) => {
      await expect(p.getByTestId("sync-my-games")).toBeDisabled({ timeout: 15000 });
    });
  });

  test("/admin/reimport", async ({ page }) => {
    await signInAsAdmin(page);
    await pair(page, "/admin/reimport", "admin-reimport", async (p) => {
      await expect(p.getByRole("heading", { name: "Re-import games" })).toBeVisible({ timeout: 15000 });
    });
  });
});
