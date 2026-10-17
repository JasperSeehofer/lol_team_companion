/**
 * M148 "before" frames: the surfaces M148 changes, captured on main
 * (9b7da51) on a helper branch that is never merged. Full-page screenshots at
 * 390 and 1280 wide, plus a viewport-only "reading flow" frame at 1280 as the
 * page first opens (nothing expanded). Writes to $MISSION_FRAMES_DIR/before
 * (skips writing when unset, so the spec is still a smoke test). Every surface
 * is its own test and tolerant on purpose: a missing optional element is
 * logged and the frame is taken anyway.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, authenticatePage, createTeam, signInAsAdmin, INVITE_CODE, TEST_PASSWORD } from "./fixtures";
import type { Page } from "@playwright/test";

const ROOT = process.env.MISSION_FRAMES_DIR;
const DIR = ROOT ? path.join(ROOT, "before") : undefined;
const SIZES = [
  { w: 390, h: 844 },
  { w: 1280, h: 800 },
];

function note(msg: string) {
  console.log(`[frames-m148-before] ${msg}`);
}

async function settle(page: Page) {
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForLoadState("networkidle", { timeout: 10000 }).catch(() => {});
  await page.waitForTimeout(1200);
}

/** One screenshot at the current viewport. */
async function single(page: Page, file: string, fullPage: boolean) {
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.screenshot({ path: path.join(DIR, file), fullPage }).catch((e) => note(`screenshot ${file} failed: ${e}`));
}

/** The flow frame first (1280 viewport, top of the page as opened), then full pages at 390 and 1280. */
async function frames(page: Page, name: string, prep?: () => Promise<void>) {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
  await page.waitForTimeout(300);
  await single(page, `${name}-1280-flow.png`, false);
  for (const s of SIZES) {
    await page.setViewportSize({ width: s.w, height: s.h });
    if (prep) await prep().catch((e) => note(`${name}: prep failed: ${e}`));
    await page.waitForTimeout(500);
    await single(page, `${name}-${s.w}.png`, true);
  }
  await page.setViewportSize({ width: 1280, height: 800 });
}

async function shoot(page: Page, name: string, url: string, prep?: () => Promise<void>) {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(url).catch((e) => note(`${name}: goto ${url} failed: ${e}`));
  await settle(page);
  note(`${name}: at ${page.url()}`);
  await frames(page, name, prep);
  await expect(page.locator("body")).toBeVisible();
}

async function signInSeed(page: Page) {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", "seed-alpha-top@example.com");
  await page.fill("input[name=password]", "seedpass123");
  await page.click("main button[type=submit]");
  const ok = await page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 30000 })
    .then(() => true)
    .catch(() => false);
  if (!ok) {
    note("seed sign-in failed; falling back to a fresh account with a team");
    await authenticatePage(page);
    await createTeam(page, `Frames_${Date.now()}`);
  }
  await settle(page);
}

/** Open the first saved item in a list (if any) and take the frames again as `<name>-open`. */
async function openFirst(page: Page, name: string, selector: string) {
  const item = page.locator(selector).first();
  if (!(await item.count())) {
    note(`${name}: no saved item (${selector}) to open`);
    return;
  }
  await item.click({ timeout: 10000 }).catch((e) => note(`${name}: open failed: ${e}`));
  await settle(page);
  note(`${name}-open: at ${page.url()}`);
  await frames(page, `${name}-open`);
}

test.describe("M148 before frames", () => {
  test.setTimeout(300_000);

  test("dashboard-invites", async ({ page }) => {
    await signInSeed(page);
    await page.goto("/team/dashboard");
    await settle(page);
    const section = page.locator("[data-team-invites]");
    if (await section.count()) {
      await section.locator("#team-invite-uses").fill("2").catch((e) => note(`uses: ${e}`));
      await section.locator("#team-invite-days").fill("7").catch((e) => note(`days: ${e}`));
      await section.getByTestId("mint-team-invite").click({ timeout: 10000 }).catch((e) => note(`mint: ${e}`));
      await section
        .locator("[data-team-invite]")
        .first()
        .waitFor({ timeout: 15000 })
        .catch(() => note("no invite row after minting"));
      await page.waitForTimeout(800);
      // Right after minting (link shown, row visible).
      await frames(page, "dashboard-invites-minted");
    } else {
      note("no [data-team-invites] section on the dashboard");
    }
    // After a reload: what a row looks like then.
    await page.reload().catch(() => {});
    await settle(page);
    await page
      .locator("[data-team-invites]")
      .first()
      .scrollIntoViewIfNeeded()
      .catch(() => {});
    await frames(page, "dashboard-invites");
    await expect(page.locator("body")).toBeVisible();
  });

  test("admin-resets", async ({ page }) => {
    await signInAsAdmin(page);
    await page.goto("/admin/resets");
    await settle(page);
    const email = page.getByLabel("Email of the account");
    if (await email.count()) {
      await email.fill("seed-alpha-mid@example.com").catch((e) => note(`email: ${e}`));
      await page.getByRole("button", { name: "Create reset code" }).click({ timeout: 10000 }).catch((e) => note(`create: ${e}`));
      await page
        .getByTestId("reset-code")
        .waitFor({ timeout: 15000 })
        .catch(() => note("no reset code shown"));
      await page.waitForTimeout(800);
    } else {
      note("no email field on /admin/resets");
    }
    await frames(page, "admin-resets");
    await expect(page.locator("body")).toBeVisible();
  });

  test("match-day-phone", async ({ page }) => {
    await authenticatePage(page);
    await createTeam(page, `Before_${Date.now()}`);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await settle(page);
    await frames(page, "match-day-phone");
    // The 390 viewport-only frame as well: the key frame for this surface.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
    await page.waitForTimeout(300);
    await single(page, "match-day-phone-390-flow.png", false);
    await expect(page.locator("body")).toBeVisible();
  });

  test("draft", async ({ page }) => {
    await signInSeed(page);
    await shoot(page, "draft", "/draft");
    await openFirst(page, "draft", 'a[href^="/draft?draft_id="]');
  });

  test("game-plan", async ({ page }) => {
    await signInSeed(page);
    await shoot(page, "game-plan", "/game-plan");
    await openFirst(page, "game-plan", '[data-testid="plan-list"] li a, [data-testid="plan-list"] li button');
  });

  test("tree-drafter", async ({ page }) => {
    await signInSeed(page);
    await shoot(page, "tree-drafter", "/tree-drafter");
    await openFirst(page, "tree-drafter", '[data-testid="tree-list"] li a, [data-testid="tree-list"] li button');
  });

  test("register", async ({ page }) => {
    await shoot(page, "register", `/auth/register?invite=${INVITE_CODE}`);
  });

  test("login", async ({ page }) => {
    await shoot(page, "login", "/auth/login");
  });

  test("login-next", async ({ page }) => {
    await page.goto("/draft").catch((e) => note(`goto /draft: ${e}`));
    await page.waitForURL(/\/auth\/login/, { timeout: 20000 }).catch(() => note(`no redirect to login; at ${page.url()}`));
    await settle(page);
    note(`login-next: at ${page.url()}`);
    await frames(page, "login-next");
    await expect(page.locator("body")).toBeVisible();
  });

  test("login-hint", async ({ page }) => {
    // On main the hint is plain text under the password field (no disclosure).
    await page.goto("/auth/login");
    await settle(page);
    const hint = page.getByText("Forgot it?").first();
    const found = await hint.count();
    if (!found) note("no forgotten-password hint text found");
    const disclosure = page.locator("main details").first();
    await frames(page, "login-hint", async () => {
      await hint.scrollIntoViewIfNeeded().catch(() => {});
    });
    if (await disclosure.count()) {
      await disclosure.locator("summary").click().catch(() => {});
      await page.waitForTimeout(400);
      await single(page, "login-hint-open-1280.png", true);
    } else {
      note("the hint is not a disclosure; no login-hint-open frame");
    }
    await expect(page.locator("body")).toBeVisible();
  });

  test("profile-password", async ({ page }) => {
    void TEST_PASSWORD;
    await authenticatePage(page);
    await page.goto("/profile");
    await settle(page);
    const section = page.locator("[data-password]").first();
    if (!(await section.count())) note("no [data-password] section on /profile");
    await frames(page, "profile-password", async () => {
      await section.scrollIntoViewIfNeeded().catch(() => {});
    });
    // The section in view at 1280 (viewport only).
    await section.scrollIntoViewIfNeeded().catch(() => {});
    await page.waitForTimeout(300);
    await single(page, "profile-password-1280-section.png", false);
    await expect(page.locator("body")).toBeVisible();
  });

  test("auth-reset", async ({ page }) => {
    await shoot(page, "auth-reset", "/auth/reset");
  });
});
