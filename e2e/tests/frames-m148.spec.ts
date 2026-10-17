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
import { test, expect, authenticatePage, createTeam, signInAsAdmin, INVITE_CODE, TEST_PASSWORD } from "./fixtures";
import type { Page } from "@playwright/test";
import { createDraft, saveGamePlan } from "./helpers";

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
    // The 390 viewport-only frame, paired with the before extra of the same name.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
    await page.waitForTimeout(300);
    if (DIR) await page.screenshot({ path: path.join(DIR, "match-day-phone-390-flow.png"), fullPage: false });
  });

  test("admin resets with a minted code", async ({ browser }) => {
    // An account to mint a code for.
    const player = await (await browser.newContext()).newPage();
    const stamp = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;
    const email = `frames_reset_${stamp}@test.invalid`;
    await player.goto(`/auth/register?invite=${INVITE_CODE}`);
    await player.fill("input[name=username]", `frames_reset_${stamp}`);
    await player.fill("input[name=email]", email);
    await player.fill("input[name=password]", TEST_PASSWORD);
    await player.check("#register-terms");
    await player.click("main button[type=submit]");
    await player.waitForURL((u) => new URL(u).pathname === "/", { timeout: 20000 });

    const admin = await (await browser.newContext()).newPage();
    await signInAsAdmin(admin);
    await admin.setViewportSize({ width: 1280, height: 800 });
    await admin.goto("/admin/resets", { waitUntil: "networkidle" });
    await admin.getByLabel("Email of the account").fill(email);
    await admin.getByRole("button", { name: "Create reset code" }).click();
    await expect(admin.getByTestId("reset-code")).toBeVisible({ timeout: 15000 });
    await expect(admin.getByTestId("copy-reset-code")).toBeVisible();
    await settle(admin);
    // The minted code lives in the page's state: resize, never reload.
    await frames(admin, "admin-resets");
  });

  test("sign-in reached from /draft (login-next)", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/draft");
    await page.waitForURL((u) => new URL(u).pathname === "/auth/login", { timeout: 20000 });
    expect(new URL(page.url()).search).toBe("?next=%2Fdraft");
    await settle(page);
    await expect(page).toHaveTitle("Sign in \u00b7 Team Companion");
    await frames(page, "login-next");
  });

  test("register with the 16+ and terms box", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto(`/auth/register?invite=${INVITE_CODE}`);
    await settle(page);
    await expect(page.locator("#register-terms")).toBeVisible();
    await frames(page, "register", async () => {
      // A1: the whole label is a 44px tap target on phones.
      const vw = page.viewportSize()?.width ?? 1280;
      const h = await page.locator("label", { has: page.locator("#register-terms") }).evaluate((el) => el.getBoundingClientRect().height);
      if (vw < 768) expect(h).toBeGreaterThanOrEqual(44);
    });
  });

  test("team page invite links after a reload (dashboard-invites)", async ({ page }) => {
    await authenticatePage(page);
    await createTeam(page, `FramesInv_${Date.now()}`);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/team/dashboard");
    await page.waitForLoadState("networkidle");
    const section = page.locator("[data-team-invites]");
    await section.getByTestId("mint-team-invite").click();
    await expect(section.getByTestId("team-invite-link")).toContainText("/auth/register?invite=");
    await page.reload();
    await page.waitForLoadState("networkidle");
    await expect(section.getByTestId("team-invite-url").first()).toHaveText(/^https?:\/\//, { timeout: 10000 });
    await settle(page);
    await section.scrollIntoViewIfNeeded();
    await frames(page, "dashboard-invites", async () => {
      // A1: invite actions are 44px tall on phones.
      const vw = page.viewportSize()?.width ?? 1280;
      const h = await section.getByTestId("copy-invite-link").first().evaluate((el) => el.getBoundingClientRect().height);
      if (vw < 768) expect(h).toBeGreaterThanOrEqual(44);
    });
  });
  // M148 item 4: the editors' save state, saved and failed (a 503 on the
  // save server function; the route stays on while the frames are taken).
  async function failSaves(page: Page, fn: string) {
    await page.route(`**/api/${fn}*`, (route) => route.fulfill({ status: 503, body: "" }));
  }

  test("draft saved and failed (draft, draft-save-failed)", async ({ page }) => {
    await authenticatePage(page);
    await createTeam(page, `FramesDraft_${Date.now()}`);
    await page.setViewportSize({ width: 1280, height: 800 });
    await createDraft(page, `Frames draft ${Date.now()}`);
    await settle(page);
    await frames(page, "draft");
    await failSaves(page, "update_draft");
    await page.locator("#draft-name").fill(`Frames draft edited ${Date.now()}`);
    await expect(page.getByTestId("draft-save-state")).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
    await frames(page, "draft-save-failed");
  });

  test("game plan saved and failed (game-plan, game-plan-save-failed)", async ({ page }) => {
    await authenticatePage(page);
    await createTeam(page, `FramesPlan_${Date.now()}`);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/game-plan", { waitUntil: "networkidle" });
    await saveGamePlan(page, `Frames plan ${Date.now()}`);
    await settle(page);
    await frames(page, "game-plan");
    await failSaves(page, "update_plan");
    await page.locator("#plan-name").fill(`Frames plan edited ${Date.now()}`);
    await expect(page.getByTestId("plan-save-state")).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
    await frames(page, "game-plan-save-failed");
  });

  test("tree drafter saved and failed (tree-drafter, tree-save-failed)", async ({ page }) => {
    await authenticatePage(page);
    await createTeam(page, `FramesTree_${Date.now()}`);
    await page.setViewportSize({ width: 1280, height: 800 });
    const name = `Frames tree ${Date.now()}`;
    await page.goto("/tree-drafter", { waitUntil: "networkidle" });
    await page.locator("#tree-name").fill(name);
    await page.getByRole("button", { name: "Create tree" }).click();
    await expect(page.getByTestId("tree-list").getByRole("button", { name })).toHaveAttribute("aria-pressed", "true", { timeout: 5000 });
    const label = `Frames branch ${Date.now()}`;
    await page.getByRole("treeitem").first().getByRole("button", { name: "Branch", exact: true }).click();
    await page.locator("#branch-label").fill(label);
    await page.getByRole("button", { name: "Add branch" }).click();
    const row = page.getByRole("treeitem").filter({ hasText: label }).last();
    await expect(row).toBeVisible({ timeout: 5000 });
    await row.getByRole("button", { name: label }).click();
    await expect(page.locator("#node-label")).toHaveValue(label);
    await page.getByRole("button", { name: "Save node" }).click();
    await expect(page.getByTestId("node-save-state")).toHaveText("Saved", { timeout: 5000 });
    await settle(page);
    await frames(page, "tree-drafter");
    await failSaves(page, "save_node");
    await page.locator("#node-label").fill(`${label} edited`);
    await expect(page.getByTestId("node-save-state")).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
    await frames(page, "tree-save-failed");
  });

  // M148 item 8 (page titles): the sign-in and reset pages, paired with the
  // "before" frames of the same names.
  test("sign-in page (login)", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/auth/login");
    await settle(page);
    await expect(page).toHaveTitle("Sign in \u00b7 Team Companion");
    await frames(page, "login");
  });

  test("password reset page (auth-reset)", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/auth/reset");
    await settle(page);
    await expect(page.locator("main")).toBeVisible();
    await frames(page, "auth-reset");
  });

  // M137 auth surfaces with before frames only: the hint under the password
  // field on the sign-in page, and the Password section on the profile.
  test("sign-in hint under the password field (login-hint)", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/auth/login");
    await settle(page);
    const hint = page.getByText("Forgot it?").first();
    await expect(hint).toBeVisible();
    await frames(page, "login-hint", async () => {
      await hint.scrollIntoViewIfNeeded();
    });
  });

  test("profile password section (profile-password)", async ({ page }) => {
    await authenticatePage(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/profile");
    await settle(page);
    const section = page.locator("[data-password]").first();
    await expect(section).toBeVisible();
    await frames(page, "profile-password", async () => {
      await section.scrollIntoViewIfNeeded();
    });
    // The section in view at 1280 (viewport only).
    await page.setViewportSize({ width: 1280, height: 800 });
    await section.scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    if (DIR) await page.screenshot({ path: path.join(DIR, "profile-password-1280-section.png"), fullPage: false });
  });
});
