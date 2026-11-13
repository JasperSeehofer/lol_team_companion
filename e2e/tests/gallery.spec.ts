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
  "sort",
  "tabs",
  "toggles",
  "keycap",
  "callout",
  "status",
  "toast",
  "empty",
  "skeleton",
  "tooltip",
  "dialog",
  "menu",
  "command",
  "avatar",
  "markers",
  "marks",
  "readout",
  "bar",
  "draft",
  "prep",
  "legend",
  "canvas",
  "timeline",
  "rift",
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
      // Exactly one visible section heading (closed dialogs hold their own h2).
      await expect(section.locator("h2:visible")).toHaveCount(1);
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

test("the drawn Rift zooms from sketch to survey from the keyboard", async ({ adminPage }) => {
  await openGallery(adminPage);
  const zoom = adminPage.getByTestId("rift-zoom");
  await zoom.scrollIntoViewIfNeeded();
  await expect(zoom.locator('[data-rift="sketch"]')).toBeVisible();
  // Decorative: the drawing is hidden from assistive technology.
  await expect(zoom.locator('[data-rift="sketch"]')).toHaveAttribute("aria-hidden", "true");
  const stops = zoom.locator('[role="radio"]');
  await expect(stops).toHaveCount(3);
  await stops.nth(0).focus();
  await adminPage.keyboard.press("ArrowRight");
  await adminPage.keyboard.press("Enter");
  await expect(zoom.locator('[data-rift="survey"][data-rift-zoom="2"]')).toBeVisible();
  await adminPage.keyboard.press("ArrowRight");
  await adminPage.keyboard.press("Enter");
  const survey = zoom.locator('[data-rift="survey"][data-rift-zoom="4"]');
  await expect(survey).toBeVisible();
  await expect(survey.locator('[data-rift-layer="walls"]')).toHaveCount(1);
  // The frame scrolls inside itself; the page never scrolls sideways.
  const frame = adminPage.getByTestId("rift-frame");
  await frame.focus();
  await adminPage.keyboard.press("ArrowRight");
  const overflow = await adminPage.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth
  );
  expect(overflow).toBeLessThanOrEqual(0);
});

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

test("the draft example is the page's board, in tournament order", async ({ adminPage }) => {
  await openGallery(adminPage);
  const board = adminPage.locator('[data-draft-board="gallery-draft"]');
  const cells = board.locator('[role="gridcell"]');
  await expect(cells).toHaveCount(20);
  // Every step once; five bans and five picks per side.
  const steps = await cells.evaluateAll((els) =>
    els.map((e) => Number(e.getAttribute("data-step"))).sort((a, b) => a - b)
  );
  expect(steps).toEqual(Array.from({ length: 20 }, (_, i) => i + 1));
  for (const side of ["blue", "red"]) {
    for (const action of ["ban", "pick"]) {
      await expect(board.locator(`[data-side="${side}"][data-action="${action}"]`)).toHaveCount(5);
    }
  }
  // Only the step being decided is current, and the counter names it.
  const current = board.locator('[aria-current="step"]');
  await expect(current).toHaveCount(1);
  await expect(current).toHaveAttribute("aria-label", /^Blue pick 5, .*Support, selecting now/);
  await expect(board.locator("[data-draft-status]")).toHaveText("Pick 9 of 10 \u00b7 Blue to pick");
  // Keyboard: one Tab stop, arrows move, Enter chooses.
  await adminPage.locator("#gallery-draft-slot-0").focus();
  await adminPage.keyboard.press("ArrowRight");
  await expect(adminPage.locator("#gallery-draft-slot-2")).toBeFocused();
  await adminPage.keyboard.press("Enter");
  await expect(adminPage.locator("[data-gallery-draft-chosen]")).toHaveText("Chosen: Blue ban 2");
  // The complete example has no current step.
  const done = adminPage.locator('[data-draft-board="gallery-draft-done"]');
  await expect(done.locator('[aria-current="step"]')).toHaveCount(0);
  await expect(done.locator("[data-draft-status]")).toHaveText("Draft complete");
});

test("manual segmented control moves focus first, checkbox toggles with Space", async ({ adminPage }) => {
  await openGallery(adminPage);
  const solo = adminPage.locator("#gal-manual-seg-0");
  const team = adminPage.locator("#gal-manual-seg-1");
  await expect(solo).toHaveAttribute("aria-checked", "true");
  await solo.focus();
  // Arrow keys move focus only; Enter or Space switches.
  await adminPage.keyboard.press("ArrowRight");
  await expect(team).toBeFocused();
  await expect(solo).toHaveAttribute("aria-checked", "true");
  await expect(adminPage.getByTestId("segmented-manual-value")).toHaveText("Workspace: solo");
  await adminPage.keyboard.press("Enter");
  await expect(team).toHaveAttribute("aria-checked", "true");
  await expect(adminPage.getByTestId("segmented-manual-value")).toHaveText("Workspace: team");

  const box = adminPage.getByTestId("checkbox-demo").getByRole("checkbox", { name: "Agree on the level 1 plan" });
  const before = await box.isChecked();
  await box.focus();
  await adminPage.keyboard.press("Space");
  await expect(box).toBeChecked({ checked: !before });
  await expect(
    adminPage.getByTestId("checkbox-demo").getByRole("checkbox", { name: "Disabled item" })
  ).toBeDisabled();
});

test("dialog: focus moves in, Escape closes and focus returns", async ({ adminPage }) => {
  await openGallery(adminPage);
  const open = adminPage.getByTestId("open-dialog");
  const dialog = adminPage.locator('dialog[data-dialog="gallery-dialog"]');
  await open.focus();
  await adminPage.keyboard.press("Enter");
  await expect(dialog).toHaveAttribute("open", "");
  await expect(dialog).toHaveAttribute("aria-modal", "true");
  await expect(dialog).toHaveAttribute("aria-labelledby", "gallery-dialog-title");
  expect(await dialog.evaluate((d) => d.contains(document.activeElement))).toBe(true);
  // Tab stays inside the modal.
  for (let i = 0; i < 6; i++) await adminPage.keyboard.press("Tab");
  expect(await dialog.evaluate((d) => d.contains(document.activeElement))).toBe(true);
  await adminPage.keyboard.press("Escape");
  await expect(dialog).not.toHaveAttribute("open", "");
  await expect(open).toBeFocused();
});

test("confirm dialog: Cancel has focus first for destructive actions", async ({ adminPage }) => {
  await openGallery(adminPage);
  await adminPage.getByTestId("open-confirm").click();
  const dialog = adminPage.locator('dialog[data-dialog="gallery-confirm"]');
  await expect(dialog).toHaveAttribute("open", "");
  await expect(dialog.locator('[data-confirm="cancel"]')).toBeFocused();
  await adminPage.keyboard.press("Tab");
  await expect(dialog.locator('[data-confirm="confirm"]')).toBeFocused();
  await adminPage.keyboard.press("Enter");
  await expect(dialog).not.toHaveAttribute("open", "");
  await expect(adminPage.getByTestId("dialog-result")).toHaveText(/Plan deleted/);
});

test("menu: arrows, typeahead, Escape and selection", async ({ adminPage }) => {
  await openGallery(adminPage);
  const button = adminPage.locator("#gallery-menu-button");
  await expect(button).toHaveAttribute("aria-haspopup", "menu");
  await button.focus();
  await adminPage.keyboard.press("ArrowDown");
  await expect(button).toHaveAttribute("aria-expanded", "true");
  await expect(adminPage.locator("#gallery-menu-item-0")).toBeFocused();
  await adminPage.keyboard.press("ArrowDown");
  await expect(adminPage.locator("#gallery-menu-item-1")).toBeFocused();
  // Item 2 is disabled and skipped.
  await adminPage.keyboard.press("ArrowDown");
  await expect(adminPage.locator("#gallery-menu-item-3")).toBeFocused();
  await adminPage.keyboard.press("d");
  await expect(adminPage.locator("#gallery-menu-item-4")).toBeFocused();
  await adminPage.keyboard.press("Home");
  await expect(adminPage.locator("#gallery-menu-item-0")).toBeFocused();
  await adminPage.keyboard.press("Escape");
  await expect(button).toHaveAttribute("aria-expanded", "false");
  await expect(button).toBeFocused();
  await expect(adminPage.locator("#gallery-menu-menu")).toBeHidden();

  await adminPage.keyboard.press("Enter");
  await expect(adminPage.locator("#gallery-menu-item-0")).toBeFocused();
  await adminPage.keyboard.press("Enter");
  await expect(adminPage.getByTestId("menu-result")).toHaveText("Chose: duplicate");
  await expect(button).toBeFocused();
});

test("command bar: Ctrl+K opens it, typing filters, Enter goes", async ({ adminPage }) => {
  await openGallery(adminPage);
  const bar = adminPage.locator('dialog[data-dialog="command-bar"]');
  await adminPage.keyboard.press("Control+k");
  await expect(bar).toHaveAttribute("open", "");
  const input = bar.locator('[role="combobox"]');
  await expect(input).toBeFocused();
  await adminPage.keyboard.type("roster");
  const options = bar.locator('[role="option"]');
  await expect(options.first()).toHaveText(/Roster/);
  await expect(options.first()).toHaveAttribute("aria-selected", "true");
  await expect(input).toHaveAttribute("aria-activedescendant", /command-bar-opt-/);
  await adminPage.keyboard.press("Escape");
  await expect(bar).not.toHaveAttribute("open", "");

  // The gallery instance runs a command without a link.
  await adminPage.getByTestId("open-command").click();
  const gal = adminPage.locator('dialog[data-dialog="gallery-command"]');
  await expect(gal).toHaveAttribute("open", "");
  await adminPage.keyboard.type("toast");
  await adminPage.keyboard.press("Enter");
  await expect(gal).not.toHaveAttribute("open", "");
  await expect(adminPage.getByTestId("command-result")).toHaveText("Ran: toast");

  // Arrow keys move the active option, and Enter follows a link.
  await adminPage.keyboard.press("Control+k");
  await expect(bar).toHaveAttribute("open", "");
  await adminPage.keyboard.type("roster");
  await adminPage.keyboard.press("Enter");
  await adminPage.waitForURL("**/team/roster");
});

test("segmented control and switch work from the keyboard", async ({ adminPage }) => {
  await openGallery(adminPage);
  const group = adminPage.locator('[data-segmented="gal-mode"]');
  await expect(group).toHaveAttribute("role", "radiogroup");
  const team = adminPage.locator("#gal-mode-seg-1");
  const solo = adminPage.locator("#gal-mode-seg-0");
  await expect(team).toHaveAttribute("aria-checked", "true");
  await expect(solo).toHaveAttribute("tabindex", "-1");
  await team.focus();
  await adminPage.keyboard.press("ArrowLeft");
  await expect(solo).toHaveAttribute("aria-checked", "true");
  await expect(solo).toBeFocused();
  await expect(adminPage.getByTestId("segmented-value")).toHaveText("Workspace: solo");
  await adminPage.keyboard.press("ArrowLeft");
  await expect(team).toHaveAttribute("aria-checked", "true");
  await adminPage.keyboard.press("Home");
  await expect(solo).toHaveAttribute("aria-checked", "true");

  const sw = adminPage.locator('[data-testid="switch-demo"] [role="switch"]');
  await expect(sw).toHaveAttribute("aria-checked", "true");
  await sw.focus();
  await adminPage.keyboard.press("Space");
  await expect(sw).toHaveAttribute("aria-checked", "false");
  await adminPage.keyboard.press("Enter");
  await expect(sw).toHaveAttribute("aria-checked", "true");
});

test("tooltip shows on focus, describes its control and hides on Escape", async ({ adminPage }) => {
  await openGallery(adminPage);
  const trigger = adminPage.getByTestId("tooltip-trigger");
  await expect(trigger).toHaveAttribute("aria-describedby", "tip-info");
  const tip = adminPage.locator("#tip-info");
  await expect(tip).toHaveAttribute("role", "tooltip");
  await trigger.focus();
  await expect(tip).toBeVisible();
  await adminPage.keyboard.press("Escape");
  await expect(tip).toBeHidden();
});

test("toast with Undo runs the action and the region is polite", async ({ adminPage }) => {
  await openGallery(adminPage);
  const region = adminPage.locator('section[aria-label="Notifications"] ol');
  await expect(region).toHaveAttribute("aria-live", "polite");
  await adminPage.getByTestId("toast-undo").click();
  await expect(adminPage.getByTestId("toast-result")).toHaveText("Note deleted");
  const toast = region.locator("li", { hasText: "Team note deleted." });
  await expect(toast).toBeVisible();
  await toast.getByRole("button", { name: "Undo" }).click();
  await expect(adminPage.getByTestId("toast-result")).toHaveText("Note restored by Undo");
  await expect(toast).toHaveCount(0);
});

test("timeline scrub: slider keys and moments", async ({ adminPage }) => {
  await openGallery(adminPage);
  const slider = adminPage.locator('[data-testid="timeline-demo"] [role="slider"]');
  await expect(slider).toHaveAttribute("aria-valuenow", "1025");
  await expect(slider).toHaveAttribute("aria-valuetext", "17:05, moment: Pick-off");
  await slider.focus();
  await adminPage.keyboard.press("l");
  await expect(slider).toHaveAttribute("aria-valuenow", "1450");
  await adminPage.keyboard.press("Shift+ArrowLeft");
  await expect(slider).toHaveAttribute("aria-valuenow", "1440");
  await adminPage.keyboard.press("j");
  await expect(slider).toHaveAttribute("aria-valuenow", "1020");
  await adminPage.keyboard.press("Home");
  await expect(slider).toHaveAttribute("aria-valuenow", "0");
});

test("champion avatars use the app's image endpoint only", async ({ adminPage }) => {
  const external: string[] = [];
  adminPage.on("request", (r) => {
    const u = new URL(r.url());
    if (!["127.0.0.1", "localhost"].includes(u.hostname)) external.push(r.url());
  });
  await openGallery(adminPage);
  const img = adminPage.locator('[data-gallery="avatar"] img').first();
  await expect(img).toHaveAttribute("src", /^\/img\/champion\/.+\/icon/);
  await expect(adminPage.locator('[data-gallery="markers"] [data-role="Jungle"]').first()).toContainText("JGL");
  expect(external, external.join("\n")).toEqual([]);
});

test("marks are hidden from screen readers and stand beside their word", async ({ adminPage }) => {
  await openGallery(adminPage);
  const section = adminPage.locator('[data-gallery="marks"]');
  // Nine inline marks, nine at 24 and nine 64 px specimens, all aria-hidden.
  await expect(section.locator("svg")).toHaveCount(27);
  await expect(section.locator('svg:not([aria-hidden="true"])')).toHaveCount(0);
  for (const word of ["Top", "Jungle", "Mid", "Bot", "Support", "Dragon", "Baron", "Herald", "Tower"]) {
    await expect(section.locator('[data-testid="marks-inline"]')).toContainText(word);
  }
});

// Outside dev builds the gallery is admin-only: everyone else gets the 404
// page with HTTP 404. CI runs the server in production mode and sets
// E2E_EXPECT_PRODUCTION; a local `cargo leptos watch` is a dev build, where
// the gallery is open to everyone, so the check is skipped there.
test.describe("gallery access", () => {
  test.skip(process.env.E2E_EXPECT_PRODUCTION !== "1", "dev server: the gallery is open to everyone");

  test("the gallery is a 404 for visitors and non-admins outside dev", async ({ browser, authedPage }) => {
    const anonymous = await browser.newContext();
    const visitor = await anonymous.newPage();
    const resp = await visitor.goto("/design");
    expect(resp?.status()).toBe(404);
    await expect(visitor.locator("h1")).toHaveText(/page not found/i);
    await anonymous.close();

    const member = await authedPage.goto("/design");
    expect(member?.status()).toBe(404);
    await expect(authedPage.locator("h1")).toHaveText(/page not found/i);
  });
});

// M141 (reports l31jk71m, rucyy6k1): a button keeps its size in every state,
// and a sortable header sorts by click and keyboard with aria-sort.
test("a button keeps its size while loading and with a changed label", async ({ adminPage }) => {
  await openGallery(adminPage);
  const idle = await adminPage.getByTestId("button-size-idle").boundingBox();
  const loading = await adminPage.getByTestId("button-size-loading").boundingBox();
  const done = await adminPage.getByTestId("button-size-done").boundingBox();
  expect(idle && loading && done).toBeTruthy();
  expect(Math.abs(idle!.width - loading!.width)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(idle!.height - loading!.height)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(idle!.width - done!.width)).toBeLessThanOrEqual(0.5);
  // The label stays in the accessible name while loading.
  await expect(adminPage.getByTestId("button-size-loading")).toHaveAccessibleName(/Sync my games/);
});

test("sortable headers sort by click and keyboard, with aria-sort", async ({ adminPage }) => {
  await openGallery(adminPage);
  const demo = adminPage.getByTestId("sort-demo");
  const firstCell = demo.locator("tbody tr").first().locator("th, td").first();
  const games = demo.locator("th", { has: adminPage.getByRole("button", { name: "Sort by Games" }) });
  await expect(games).toHaveAttribute("aria-sort", "descending");
  // Ties keep their order: Orianna (14) before Syndra (14).
  await expect(firstCell).toHaveText("Orianna");
  const byName = demo.getByRole("button", { name: "Sort by Champion" });
  await byName.click();
  await expect(firstCell).toHaveText("Azir");
  await expect(games).not.toHaveAttribute("aria-sort", /.+/);
  // Keyboard: focus stays on the header, Enter flips the order.
  await expect(byName).toBeFocused();
  await adminPage.keyboard.press("Enter");
  await expect(firstCell).toHaveText("Taliyah");
  await expect(demo.locator("th").first()).toHaveAttribute("aria-sort", "descending");
  await expect(demo.getByRole("status")).toHaveText("Sorted by Champion, descending");
});
