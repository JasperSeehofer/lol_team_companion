/**
 * M152 item 3: the hero surfaces.
 *
 * - The landing opens on the drawn Rift with exactly one primary button and
 *   at most two chalk (accent) elements in the first viewport: the surveyed
 *   label and that button (recipe card D).
 * - Jasper's match-up rule on a shared game plan: each team's five champion
 *   icons never wrap. At 320, 390 and 1280 all five share one offsetTop
 *   (a row) or one offsetLeft (a column).
 *
 * The match-up check shares a seeded plan (five picks a side, linked to a
 * draft on the blue side). CI seeds its database before the server starts,
 * so there a missing seed is a failure; locally the test skips without it.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";

const SEEDED_LEADER = "seed-beta-top@example.com";
const SEED_PASSWORD = "seedpass123";

/**
 * Chalk in the first viewport, read from computed styles (never from class
 * names): HTML elements whose fill is the accent, whose own text is in the
 * accent, or whose drawn border is the accent. The leader inside the
 * drawing belongs to its label (card D), so SVG is not counted.
 */
async function chalkInViewport(page: Page, wholePage = false): Promise<string[]> {
  return page.evaluate((wholePage) => {
    const probe = document.createElement("div");
    probe.style.color = "var(--color-accent)";
    document.body.appendChild(probe);
    const accent = getComputedStyle(probe).color;
    probe.remove();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const hits: Element[] = [];
    for (const el of Array.from(document.body.querySelectorAll("*"))) {
      if (el instanceof SVGElement) continue;
      const r = el.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      if (!wholePage && (r.bottom <= 0 || r.top >= vh || r.right <= 0 || r.left >= vw)) continue;
      const cs = getComputedStyle(el);
      if (cs.visibility === "hidden" || cs.display === "none" || cs.opacity === "0") continue;
      const ownText = Array.from(el.childNodes).some(
        (n) => n.nodeType === Node.TEXT_NODE && (n.textContent ?? "").trim() !== ""
      );
      const border = (["Top", "Right", "Bottom", "Left"] as const).some(
        (s) =>
          parseFloat(cs.getPropertyValue(`border-${s.toLowerCase()}-width`)) > 0 &&
          cs.getPropertyValue(`border-${s.toLowerCase()}-color`) === accent
      );
      if (cs.backgroundColor === accent || border || (ownText && cs.color === accent)) {
        if (!hits.some((h) => h.contains(el))) hits.push(el);
      }
    }
    return hits.map((h) => `${h.tagName.toLowerCase()}${h.getAttribute("data-chalk") ? `[data-chalk=${h.getAttribute("data-chalk")}]` : ""}${h.getAttribute("data-rift-chalk") ? `[data-rift-chalk=${h.getAttribute("data-rift-chalk")}]` : ""}: ${(h.textContent ?? "").trim().slice(0, 40)}`);
  }, wholePage);
}

for (const size of [
  { w: 390, h: 844 },
  { w: 1280, h: 800 },
]) {
  test(`landing hero at ${size.w}: the Rift, one primary action, chalk at most twice`, async ({ page }) => {
    await page.setViewportSize({ width: size.w, height: size.h });
    await page.goto("/");
    const hero = page.locator("[data-hero]");
    await expect(hero).toBeVisible({ timeout: 15000 });
    await page.waitForLoadState("load").catch(() => {});
    // The drawing is in the server's HTML (no fetch) and inside the first viewport.
    const sketch = hero.locator('[data-rift="sketch"] svg');
    await expect(sketch).toHaveCount(1);
    await expect(sketch).toBeInViewport();
    // One primary action, and it is the surveyed label's only companion.
    await expect(page.locator("[data-chalk]")).toHaveCount(1);
    await expect(page.locator('[data-chalk="action"]')).toHaveText("Sign in");
    await expect(hero.locator('[data-rift-chalk="label"]')).toHaveText(/your team/i);
    const chalk = await chalkInViewport(page);
    expect(chalk.length, `chalk in the first viewport: ${chalk.join(" | ")}`).toBeLessThanOrEqual(2);
    // Exactly one filled (primary) button on the page.
    const filled = await page.evaluate(() => {
      const probe = document.createElement("div");
      probe.style.color = "var(--color-accent)";
      document.body.appendChild(probe);
      const accent = getComputedStyle(probe).color;
      probe.remove();
      return Array.from(document.querySelectorAll("a, button")).filter((el) => {
        const r = el.getBoundingClientRect();
        return r.width > 0 && r.height > 0 && getComputedStyle(el).backgroundColor === accent;
      }).length;
    });
    expect(filled).toBe(1);
    const sw = await page.evaluate(() => document.documentElement.scrollWidth);
    expect(sw).toBeLessThanOrEqual(size.w);
  });
}

async function signInSeeded(page: Page): Promise<boolean> {
  await page.goto("/auth/login");
  await page.fill("input[name=email]", SEEDED_LEADER);
  await page.fill("input[name=password]", SEED_PASSWORD);
  await page.click("main button[type=submit]");
  return page
    .waitForURL((u) => new URL(u).pathname === "/", { timeout: 15000 })
    .then(() => true)
    .catch(() => false);
}

/** Share the seeded "Game 1" plan; returns the link. */
async function shareSeededPlan(page: Page): Promise<string> {
  await page.goto("/game-plan");
  const item = page.getByTestId("plan-list").getByRole("button", { name: /Game 1 plan/ });
  await expect(item).toBeVisible({ timeout: 15000 });
  await item.click();
  await expect(page.locator("#plan-name")).toHaveValue(/Game 1 plan/, { timeout: 10000 });
  await page.getByTestId("plan-share").click();
  const dialog = page.getByRole("dialog", { name: /Share this game plan/ });
  await dialog.getByTestId("create-share-link").click();
  const url = (await dialog.locator("[data-share-url]").first().textContent({ timeout: 10000 }))?.trim() ?? "";
  expect(url).toMatch(/\/s\/[a-z2-9]{32}$/);
  return url;
}

test("shared game plan: five champions per team never wrap at 320, 390 and 1280", async ({ page, browser }) => {
  test.setTimeout(120_000);
  const ok = await signInSeeded(page);
  if (process.env.CI) expect(ok, "CI must seed the database (seed tool) before starting the server").toBe(true);
  test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
  await page.waitForTimeout(500);
  const url = await shareSeededPlan(page);

  const ctx = await browser.newContext();
  await ctx.route(/\/img\/(champion|item)\//, (route) => route.fulfill({ status: 404, body: "" }));
  const guest = await ctx.newPage();
  await guest.goto(url, { waitUntil: "domcontentloaded" });
  await expect(guest.locator("[data-matchup]")).toBeVisible({ timeout: 15000 });

  // The strip: no hatch. Centred on the base of the side the plan's draft
  // was drafted on, it carries the chalk "Ours" (the word the page prints
  // over our champions) with its leader; without a side, the bridge and no
  // label (critique r1 surfaces fix 5).
  const strip = guest.locator("[data-share-strip]");
  await expect(strip.locator('[data-rift="share"]')).toHaveCount(1);
  await expect(strip.locator('[data-rift-layer="hatch"]')).toHaveCount(0);
  const at = await strip.locator('[data-rift="share"]').getAttribute("data-rift-at");
  if (at) {
    await expect(strip.locator('[data-rift-chalk="label"]:visible')).toHaveCount(1);
    await expect(strip.locator('[data-rift-chalk="label"]:visible')).toHaveText(/^ours$/i);
    await expect(strip.locator('[data-rift-chalk="leader"]:visible')).toHaveCount(1);
  } else {
    await expect(strip.locator("[data-rift-chalk]")).toHaveCount(0);
  }

  for (const w of [320, 390, 1280]) {
    await guest.setViewportSize({ width: w, height: 900 });
    await guest.waitForTimeout(300);
    for (const team of ["ours", "theirs"]) {
      const row = guest.locator(`[data-matchup-team="${team}"] [data-matchup-row]:visible`);
      await expect(row, `${team} at ${w}: one visible icon row`).toHaveCount(1);
      const boxes = await row.evaluate((el) =>
        Array.from(el.children).map((c) => ({
          top: (c as HTMLElement).offsetTop,
          left: (c as HTMLElement).offsetLeft,
        }))
      );
      expect(boxes.length, `${team} at ${w}: five champions`).toBe(5);
      const oneRow = boxes.every((b) => b.top === boxes[0].top);
      const oneColumn = boxes.every((b) => b.left === boxes[0].left);
      expect(oneRow || oneColumn, `${team} at ${w}: ${JSON.stringify(boxes)}`).toBe(true);
      // The row stays inside the page: nothing is pushed off the edge.
      const right = await row.evaluate((el) => el.getBoundingClientRect().right);
      expect(right).toBeLessThanOrEqual(w);
    }
    const sw = await guest.evaluate(() => document.documentElement.scrollWidth);
    expect(sw, `share page at ${w} is ${sw}px wide`).toBeLessThanOrEqual(w);
  }
  await ctx.close();
});

test("share strip: an unknown link gets the bridge and no chalk label", async ({ page }) => {
  await page.goto("/s/" + "a".repeat(32));
  const strip = page.locator("[data-share-strip]");
  await expect(strip).toBeVisible({ timeout: 15000 });
  await expect(strip.locator('[data-rift="share"]')).not.toHaveAttribute("data-rift-at", /.+/);
  await expect(strip.locator("[data-rift-chalk]")).toHaveCount(0);
});

test("Match Day at 390: the field card's button is the page's chalk (at most 2 accent items)", async ({ page }) => {
  test.setTimeout(90_000);
  await page.setViewportSize({ width: 390, height: 844 });
  const ok = await signInSeeded(page);
  if (process.env.CI) expect(ok, "CI must seed the database (seed tool) before starting the server").toBe(true);
  test.skip(!ok, "needs the seed data: stop the server, run the seed tool, start it again");
  await page.goto("/");
  const card = page.locator("[data-field-card]");
  await expect(card).toBeVisible({ timeout: 15000 });
  await page.waitForLoadState("load").catch(() => {});
  await page.waitForTimeout(500);
  // No team names beside the inset (card E): the title already faces them.
  await expect(card.locator("[data-field-inset]")).toHaveText("");
  // The quiet prep sequence carries no chalk; the whole page at most two
  // accent items (the field card's button, plus at most one more).
  const chalk = await chalkInViewport(page, true);
  expect(chalk.length, `chalk on Match Day at 390: ${chalk.join(" | ")}`).toBeLessThanOrEqual(2);
  const prepChalk = await page.locator("[data-prep] ol").evaluate((ol) => {
    const probe = document.createElement("div");
    probe.style.color = "var(--color-accent)";
    document.body.appendChild(probe);
    const accent = getComputedStyle(probe).color;
    probe.remove();
    return Array.from(ol.querySelectorAll("*")).filter((el) => {
      const cs = getComputedStyle(el);
      return cs.color === accent || cs.backgroundColor === accent || cs.borderTopColor === accent;
    }).length;
  });
  expect(prepChalk, "the quiet prep sequence uses no accent").toBe(0);
});
