/**
 * Solo Journey room — immersive room system spike verification.
 *
 * Proves the four de-risking unknowns from the plan as observable e2e
 * behaviour:
 *   1. Surfaces render over the atmosphere (the room hosts the real,
 *      accessible Solo UI — never forces tiny-illustration clicks).
 *   2. Parallax bridge works: pointer movement mutates `--px`/`--py` on
 *      the room root (and nothing else needs to re-render).
 *   3. `prefers-reduced-motion` freezes parallax: the listener never
 *      attaches, so `--px` stays 0 after a mouse move.
 *   4. Growth-state attributes are present and numeric (0..=3); the
 *      pure tier logic itself is unit-tested in `models/growth.rs`.
 * Plus: QuickNav command palette opens (Ctrl+K) and navigates, and the
 * room's art stays within the 400 KB asset budget.
 *
 * The hydration-no-panic gate adds `/journey` separately (see
 * hydration-no-panic.spec.ts) — this file is behaviour, that one is the
 * SSR/hydrate symmetry canary.
 */

import { test, expect } from "./fixtures";

const ROOM_BG_RE = /\/img\/rooms\//;

test.describe("Solo Journey room", () => {
  test("renders the Solo surfaces over the parallax atmosphere", async ({
    authedPage,
  }) => {
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");

    // The atmosphere root and at least the sky plane exist.
    await expect(authedPage.locator(".room-atmosphere")).toBeVisible();
    expect(await authedPage.locator(".room-plane").count()).toBeGreaterThanOrEqual(
      3
    );

    // The real surface is present: the Solo mode toggle (Constellation/Forge/
    // Journal) renders, and there is no error banner.
    await expect(
      authedPage.locator('[data-feedback-label="Solo Journey room"]')
    ).toBeVisible();
    expect(await authedPage.locator("text=/Failed to load/i").count()).toBe(0);
  });

  test("pointer movement drives the parallax CSS variables", async ({
    authedPage,
  }) => {
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500); // hydrate settle (listener attaches)

    const root = authedPage.locator(".room-atmosphere");
    const before = await root.evaluate((el) =>
      getComputedStyle(el).getPropertyValue("--px").trim()
    );
    expect(before).toBe("0");

    // Move to a corner so the normalised pointer is well away from centre.
    await authedPage.mouse.move(40, 40);
    await authedPage.mouse.move(60, 60); // second move ensures a rAF tick
    await authedPage.waitForTimeout(120);

    const after = await root.evaluate((el) =>
      getComputedStyle(el).getPropertyValue("--px").trim()
    );
    expect(after).not.toBe("0");
    expect(Number.isNaN(parseFloat(after))).toBe(false);
  });

  test("prefers-reduced-motion freezes the parallax", async ({ authedPage }) => {
    await authedPage.emulateMedia({ reducedMotion: "reduce" });
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    await authedPage.mouse.move(40, 40);
    await authedPage.mouse.move(800, 600);
    await authedPage.waitForTimeout(150);

    const root = authedPage.locator(".room-atmosphere");
    const px = await root.evaluate((el) =>
      getComputedStyle(el).getPropertyValue("--px").trim()
    );
    // Listener never attached → variable stays at its inline default.
    expect(px).toBe("0");

    // And the CSS gate zeroes the translate (transform is pure scale).
    const plane = authedPage.locator(".room-plane").first();
    const transform = await plane.evaluate(
      (el) => getComputedStyle(el).transform
    );
    // matrix(a, b, c, d, tx, ty) — tx/ty must be 0 (scale only, no translate).
    const m = transform.match(/matrix\(([^)]+)\)/);
    if (m) {
      const parts = m[1].split(",").map((s) => parseFloat(s.trim()));
      expect(Math.abs(parts[4])).toBeLessThan(0.5); // tx ≈ 0
      expect(Math.abs(parts[5])).toBeLessThan(0.5); // ty ≈ 0
    }
  });

  test("growth-state tier attributes are present and in range", async ({
    authedPage,
  }) => {
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(800); // growth resource resolve

    const root = authedPage.locator(".room-atmosphere");
    for (const axis of ["lib", "armory", "banner"]) {
      const val = await root.getAttribute(`data-${axis}-tier`);
      expect(val, `data-${axis}-tier present`).not.toBeNull();
      const n = parseInt(val ?? "", 10);
      expect(Number.isNaN(n)).toBe(false);
      expect(n).toBeGreaterThanOrEqual(0);
      expect(n).toBeLessThanOrEqual(3);
    }
  });

  test("QuickNav command palette opens and navigates", async ({
    authedPage,
  }) => {
    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    // Ctrl+K opens the palette (Cmd on mac, Control elsewhere — Linux CI).
    await authedPage.keyboard.press("Control+k");
    const dialog = authedPage.locator('[role="dialog"][aria-label="Quick navigation"]');
    await expect(dialog).toBeVisible();

    // Escape closes it.
    await authedPage.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    // The floating launcher also opens it (pointer/touch path).
    await authedPage.locator('button[aria-label*="quick navigation" i]').click();
    await expect(dialog).toBeVisible();

    // Navigating via a palette link closes it and changes route.
    await dialog.locator('a[href="/champion-pool"]').first().click();
    await authedPage.waitForURL("**/champion-pool");
    await expect(dialog).toBeHidden();
  });

  test("room art stays within the 400 KB asset budget", async ({
    authedPage,
  }) => {
    let roomBytes = 0;
    authedPage.on("response", async (resp) => {
      if (ROOM_BG_RE.test(resp.url())) {
        const len = resp.headers()["content-length"];
        if (len) roomBytes += parseInt(len, 10);
      }
    });

    await authedPage.goto("/journey");
    await authedPage.waitForLoadState("networkidle");
    await authedPage.waitForTimeout(500);

    // Only the active region's planes/overlays load; well under budget.
    expect(roomBytes).toBeLessThan(400 * 1024);
  });
});
