/**
 * M148 frame-strip capture: full-page screenshots of the surfaces this
 * mission touches at 390 and 1280 wide, plus a viewport-only "reading flow"
 * frame at 1280 (the page as opened, nothing expanded). Writes
 * `<surface>-390.png`, `<surface>-1280.png` and `<surface>-1280-flow.png` to
 * $MISSION_FRAMES_DIR; when it is unset the spec is a smoke test.
 * Modelled on frames-m141.spec.ts.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, authenticatePage, createTeam } from "./fixtures";
import type { Page } from "@playwright/test";

const DIR = process.env.MISSION_FRAMES_DIR;
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
      await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
      await page.waitForTimeout(200);
      await page.screenshot({ path: path.join(DIR, `${name}-1280-flow.png`), fullPage: false });
    }
  }
}

test.describe("M148 frames", () => {
  test.setTimeout(240_000);

  test("match day on a phone, no match", async ({ page }) => {
    // A fresh team: nothing scheduled (the seed team has a match tomorrow).
    await authenticatePage(page);
    await createTeam(page, `Frames_${Date.now()}`);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");
    await settle(page);
    await expect(page.getByTestId("add-next-match")).toBeVisible();
    await frames(page, "match-day-phone");
  });
});
