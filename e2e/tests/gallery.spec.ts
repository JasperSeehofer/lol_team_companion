/**
 * Design-system gallery (/design): every section renders at desktop and phone
 * width without sideways scrolling, interactive primitives work from the
 * keyboard, and hydration does not panic.
 *
 * Screenshots of each section are attached to the report and, when
 * GALLERY_SHOTS_DIR is set, written there as `<section>-<width>.png`.
 * The gallery is visible to admins (and to everyone in dev builds); the
 * `adminPage` fixture signs in as the `ADMIN_EMAILS` account.
 */
import * as fs from "fs";
import * as path from "path";
import { test, expect } from "./fixtures";

const SECTIONS = [
  "tokens",
  "contrast",
  "type",
  "space",
  "layout",
  "text",
  "button",
  "fields",
  "badge",
  "table",
  "tabs",
  "draft",
  "prep",
  "feedback",
];

const PANIC = /panicked at|Option::unwrap\(\) on None|unreachable/;

async function openGallery(page: import("@playwright/test").Page) {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error" && PANIC.test(m.text())) errors.push(m.text());
  });
  const resp = await page.goto("/design", { waitUntil: "networkidle" });
  test.skip(resp?.status() === 404, "server not started with ADMIN_EMAILS for the gallery");
  await expect(page.locator("h1", { hasText: "Design system" })).toBeVisible();
  // Let hydration attach handlers.
  await page.waitForTimeout(800);
  return errors;
}

test("signed-out visitors get the 404 page outside dev builds", async ({ page }) => {
  const resp = await page.goto("/design");
  const status = resp?.status() ?? 0;
  expect([200, 404]).toContain(status);
  if (status === 404) {
    await expect(page.locator("text=Page not found")).toBeVisible();
  }
});

for (const width of [1280, 400]) {
  test(`every section renders at ${width}px without sideways scroll`, async ({ adminPage }) => {
    await adminPage.setViewportSize({ width, height: 900 });
    const errors = await openGallery(adminPage);
    const dir = process.env.GALLERY_SHOTS_DIR;
    if (dir) fs.mkdirSync(dir, { recursive: true });

    for (const id of SECTIONS) {
      const section = adminPage.locator(`[data-gallery="${id}"]`);
      await expect(section, `section ${id}`).toBeVisible();
      await expect(section.locator("h2")).toHaveCount(1);
      await section.scrollIntoViewIfNeeded();
      const shot = await section.screenshot({ animations: "disabled" });
      await test.info().attach(`${id}-${width}.png`, { body: shot, contentType: "image/png" });
      if (dir) fs.writeFileSync(path.join(dir, `design-${id}-${width}.png`), shot);
    }

    const overflow = await adminPage.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth
    );
    expect(overflow, "page scrolls sideways").toBeLessThanOrEqual(0);
    expect(errors, errors.join("\n")).toEqual([]);
  });
}

test("tabs follow the arrow keys, Home and End", async ({ adminPage }) => {
  await openGallery(adminPage);
  const tabs = adminPage.locator('[data-testid="tabs-demo"] [role="tab"]');
  await expect(tabs).toHaveCount(3);
  await tabs.nth(0).click();
  await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
  await expect(tabs.nth(1)).toHaveAttribute("tabindex", "-1");

  await adminPage.keyboard.press("ArrowRight");
  await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");
  await expect(tabs.nth(1)).toBeFocused();
  await expect(adminPage.locator("#gallery-tabs-panel-draft")).toBeVisible();
  await expect(adminPage.locator("#gallery-tabs-panel-match-day")).toBeHidden();

  await adminPage.keyboard.press("End");
  await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");
  await adminPage.keyboard.press("ArrowRight");
  await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
  await adminPage.keyboard.press("ArrowLeft");
  await expect(tabs.nth(2)).toBeFocused();
  await adminPage.keyboard.press("Home");
  await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
});

test("buttons work from the keyboard and show the focus outline", async ({ adminPage }) => {
  await openGallery(adminPage);
  const button = adminPage.locator('[data-testid="button-demo"] button', { hasText: "Count a click" });
  const output = adminPage.getByTestId("button-count");
  await button.focus();
  await adminPage.keyboard.press("Enter");
  await expect(output).toHaveText("Clicked 1 times");
  await adminPage.keyboard.press("Space");
  await expect(output).toHaveText("Clicked 2 times");

  // Keyboard focus draws the solid 2px chalk outline.
  await adminPage.keyboard.press("Shift+Tab");
  await adminPage.keyboard.press("Tab");
  const outline = await button.evaluate((el) => {
    const s = getComputedStyle(el);
    return { style: s.outlineStyle, width: s.outlineWidth };
  });
  expect(outline.style).toBe("solid");
  expect(outline.width).toBe("2px");

  // Disabled and loading buttons cannot be pressed.
  await expect(adminPage.locator('[data-gallery="button"] button[aria-busy="true"]').first()).toBeDisabled();
});

test("text field and select update from the keyboard", async ({ adminPage }) => {
  await openGallery(adminPage);
  const name = adminPage.locator("#g-live-name");
  await name.focus();
  await adminPage.keyboard.type("Nordwind");
  await expect(adminPage.getByTestId("field-echo")).toHaveText("You typed: Nordwind");
  // The label is bound to the field.
  await expect(adminPage.locator('label[for="g-live-name"]')).toHaveText(/Player name/);

  const select = adminPage.locator("#g-live-role");
  await expect(adminPage.getByTestId("select-echo")).toHaveText("Selected: mid");
  await select.focus();
  await select.selectOption("bot");
  await expect(adminPage.getByTestId("select-echo")).toHaveText("Selected: bot");

  // The error example is announced and linked to its field.
  const errorField = adminPage.locator("#g-error");
  await expect(errorField).toHaveAttribute("aria-invalid", "true");
  await expect(errorField).toHaveAttribute("aria-describedby", "g-error-error");
  await expect(adminPage.locator("#g-error-error")).toHaveAttribute("role", "alert");
});

test("the draft example follows the tournament order", async ({ adminPage }) => {
  await openGallery(adminPage);
  const steps = adminPage.locator('[data-gallery="draft"] [data-step]');
  // 20 in the sequence plus the state and interactive examples.
  const orders = await adminPage
    .locator('[data-gallery="draft"] section [data-step]')
    .evaluateAll((els) => els.map((e) => Number(e.getAttribute("data-step"))));
  expect(orders).toEqual(Array.from({ length: 20 }, (_, i) => i + 1));
  expect(await steps.count()).toBeGreaterThan(20);
  const current = adminPage.locator('[data-gallery="draft"] section [aria-current="step"]');
  await expect(current).toHaveCount(1);
  await expect(current).toHaveAttribute("aria-label", /Step 19, Blue pick 5, Support: selecting now/);
});
