/**
 * M148 item 3 (atelier card C2): on a phone, a team without a match can add
 * its next match from Match Day. The "Read-only on the phone" caption only
 * shows once there is a match to be read-only about. Runs on a fresh
 * account and team (CI-safe: no seed data needed).
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

function tomorrow(): string {
  const d = new Date(Date.now() + 24 * 3600 * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

async function height(loc: import("@playwright/test").Locator): Promise<number> {
  return loc.evaluate((el) => el.getBoundingClientRect().height);
}

test("match day at 390 px: add the next match from a phone", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await navigateTo(page, "/");
  await expect(page.getByRole("heading", { level: 2, name: "No upcoming match" })).toBeVisible();

  // No match: the action is there and a real touch target; no caption.
  const add = page.getByTestId("add-next-match");
  await expect(add).toBeVisible();
  expect(await height(add)).toBeGreaterThanOrEqual(44);
  await expect(page.getByTestId("phone-read-only")).toHaveCount(0);

  await add.click();
  const dialog = page.getByRole("dialog", { name: "Add the next match" });
  await expect(dialog).toBeVisible();
  // The editor fits the phone: no horizontal scroll, Save and Cancel at 44 px.
  const box = await dialog.boundingBox();
  expect(box).not.toBeNull();
  expect(box!.width).toBeLessThanOrEqual(390);
  const save = page.getByTestId("save-next-match");
  expect(await height(save)).toBeGreaterThanOrEqual(44);
  expect(await height(dialog.getByRole("button", { name: "Cancel" }))).toBeGreaterThanOrEqual(44);
  // A fresh team has no dossiers: the opponent field offers the way there.
  const addOpponent = dialog.getByTestId("add-opponent-link");
  await expect(addOpponent).toBeVisible();
  await expect(addOpponent).toHaveAttribute("href", "/opponents");
  expect(await height(addOpponent)).toBeGreaterThanOrEqual(44);

  await dialog.locator("#match-date").fill(tomorrow());
  await dialog.locator("#match-time").fill("19:00");
  await save.click();
  await expect(dialog).not.toBeVisible({ timeout: 10000 });

  // With a match, the phone is read-only for it: the caption appears and
  // "Edit match" stays a larger-screen action.
  await expect(page.locator("[data-prep]")).toBeVisible({ timeout: 10000 });
  await expect(page.getByTestId("phone-read-only")).toBeVisible();
  await expect(page.getByTestId("edit-next-match")).toBeHidden();
  await expect(page.getByTestId("add-next-match")).toHaveCount(0);
  const noScroll = await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth);
  expect(noScroll).toBe(true);

  // At 1280 the caption hides and Edit match is back.
  await page.setViewportSize({ width: 1280, height: 800 });
  await expect(page.getByTestId("phone-read-only")).toBeHidden();
  await expect(page.getByTestId("edit-next-match")).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});
