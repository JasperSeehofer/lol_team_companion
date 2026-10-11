/**
 * Draft Graph, stage A (M4): the ranked list beside the board, the Why
 * panel, keyboard picks with a guessed role, a team note that changes the
 * ranking, and the admin trait editor.
 *
 * Needs a running server with the e2e env (CLAUDE.md "Testing").
 */
import { test, expect } from "./fixtures";
import { test as plain } from "@playwright/test";
import type { Page } from "@playwright/test";
import {
  captureErrors,
  draftSlot,
  filterRealErrors,
  selectChampion,
} from "./helpers";
import { authenticatePage, signInAsAdmin, ADMIN_EMAIL } from "./fixtures";

const BANS = ["Azir", "Zed", "Kalista", "Yone", "Varus", "Viego"];

/** Fill the board up to Blue pick 2 (slot 9): six bans, B1, R1, R2. */
async function midDraft(page: Page) {
  await page.goto("/draft", { waitUntil: "networkidle" });
  for (const c of [...BANS, "Jinx", "Ahri", "Sejuani"]) {
    await selectChampion(page, c);
  }
  await expect(page.locator("[data-draft-status]")).toContainText("Blue to pick");
}

async function openGraph(page: Page) {
  await page.getByRole("radio", { name: "Draft Graph" }).click();
  await expect(page.locator("[data-graph-list] [role=option]").first()).toBeVisible({
    timeout: 20000,
  });
}

test("graph: ranked list and Why panel for the current pick, no third-party loads", async ({
  teamPage,
}) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const base = new URL(process.env.BASE_URL ?? "http://127.0.0.1:3020").origin;
  const foreign: string[] = [];
  page.on("request", (r) => {
    const u = r.url();
    if (!u.startsWith(base) && !u.startsWith("data:")) foreign.push(u);
  });
  await midDraft(page);
  await openGraph(page);

  await expect(page.locator("[data-graph-status]")).toContainText("candidates for Blue pick 2");
  const options = page.locator("[data-graph-list] [role=option]");
  expect(await options.count()).toBeGreaterThanOrEqual(5);
  // Best first, fit 0 to 100, every option shows a role.
  const fits = await options.evaluateAll((els) => els.map((e) => Number(e.getAttribute("data-fit"))));
  for (let i = 1; i < fits.length; i++) expect(fits[i - 1]).toBeGreaterThanOrEqual(fits[i]);
  expect(Math.max(...fits)).toBeLessThanOrEqual(100);
  // The banned and picked champions are not candidates.
  const ids = await options.evaluateAll((els) => els.map((e) => e.getAttribute("data-candidate")));
  for (const gone of [...BANS, "Jinx", "Ahri", "Sejuani"]) expect(ids).not.toContain(gone);

  // The Why panel explains the focused (first) candidate by part and source.
  const why = page.locator("[data-graph-why]");
  await expect(why).toBeVisible();
  await expect(why.locator("[data-graph-parts] dt")).toHaveCount(5);
  await expect(why.getByText("Trait rules", { exact: true })).toBeVisible();
  await expect(page.locator("[data-graph-profile]")).toContainText("Engage");
  await expect(page.locator("[data-graph-notes]")).toContainText("Fit is explained");

  expect(foreign).toEqual([]);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("graph: keyboard focus moves the Why panel, Enter picks with a guessed role", async ({
  teamPage,
}) => {
  const page = teamPage;
  await midDraft(page);
  await openGraph(page);
  const options = page.locator("[data-graph-list] [role=option]");
  await options.first().focus();
  await page.keyboard.press("ArrowDown");
  const second = options.nth(1);
  await expect(second).toHaveAttribute("aria-selected", "true");
  await expect(second).toBeFocused();
  const id = (await second.getAttribute("data-candidate")) ?? "";
  const name = ((await second.locator("span.truncate").first().textContent()) ?? "").trim();
  await expect(page.locator("#graph-why-heading")).toHaveText(`Why ${name}`);

  await page.keyboard.press("Enter");
  const slot = page.locator(draftSlot(9));
  await expect(slot).toHaveAttribute("aria-label", new RegExp(`${name}, \\w+ \\(role guessed\\)`));
  await expect(slot.locator("[data-role-guessed=true]")).toHaveCount(1);
  // The board moves on to Blue pick 3 and the graph follows.
  await expect(page.locator("[data-graph-status]")).toContainText("Blue pick 3", { timeout: 10000 });
  expect(id).not.toBe("");

  // Escape returns to the board, on the current step.
  await options.first().focus();
  await page.keyboard.press("Escape");
  await expect(page.locator(draftSlot(10))).toBeFocused();
});

test("graph: a team note raises the pair and shows in the explanation", async ({ teamPage }) => {
  const page = teamPage;
  await midDraft(page);
  await openGraph(page);
  const options = page.locator("[data-graph-list] [role=option]");
  // The last of all candidates: the least likely to have synergy already.
  await page.getByRole("button", { name: /Show all/ }).click();
  const row = options.last();
  const id = (await row.getAttribute("data-candidate")) ?? "";
  const name = ((await row.locator("span.truncate").first().textContent()) ?? "").trim();
  const before = Number(await row.getAttribute("data-fit"));

  await page.getByTestId("graph-add-note").click();
  const a = page.locator("#graph-note-a");
  await a.fill(name);
  await a.press("Enter");
  const b = page.locator("#graph-note-b");
  await b.fill("Jinx");
  await b.press("Enter");
  await page.getByRole("radio", { name: "Strong" }).click();
  await page.locator("#graph-note-reason").fill(`e2e: ${name} keeps Jinx alive`);
  await page.getByTestId("graph-note-save").click();

  const moved = page.locator(`[data-graph-list] [role=option][data-candidate="${id}"]`);
  await expect
    .poll(async () => Number(await moved.getAttribute("data-fit")), { timeout: 15000 })
    .toBeGreaterThan(before);
  await moved.click();
  const why = page.locator("[data-graph-why]");
  await expect(why.getByText("Team notes", { exact: true })).toBeVisible();
  await expect(why).toContainText(`e2e: ${name} keeps Jinx alive`);
  await expect(why).toContainText("team note by");
  await expect(page.locator("[data-graph-team-notes]")).toContainText(`${name} works with Jinx`);
});

test("graph: ban steps and an empty board say what the graph ranks", async ({ teamPage }) => {
  const page = teamPage;
  await page.goto("/draft", { waitUntil: "networkidle" });
  await page.getByRole("radio", { name: "Draft Graph" }).click();
  await expect(page.locator("[data-draft-graph]")).toContainText("This step is a ban", {
    timeout: 15000,
  });
  // Back to the picker: it is there again (the graph view unmounts cleanly).
  await page.getByRole("radio", { name: "Picker" }).click();
  await expect(page.locator("#draft-picker-search")).toBeVisible();
});

plain("admin traits: 404 for non-admins and signed-out visitors", async ({ page }) => {
  const anon = await page.goto("/admin/traits");
  expect(anon?.status()).toBe(404);
  await authenticatePage(page);
  const resp = await page.goto("/admin/traits");
  expect(resp?.status()).toBe(404);
  await expect(page.locator("text=Page not found")).toBeVisible();
});

plain("admin traits: search, edit a row, reset it", async ({ page }) => {
  await signInAsAdmin(page);
  const resp = await page.goto("/admin/traits", { waitUntil: "networkidle" });
  plain.skip(resp?.status() === 404, `server not started with ADMIN_EMAILS=${ADMIN_EMAIL}`);
  await expect(page.locator("h1", { hasText: "Champion traits" })).toBeVisible();
  await page.locator("#traits-search").fill("Ahri");
  const rows = page.locator("tbody tr");
  await expect(rows).toHaveCount(1);

  await page.getByRole("button", { name: "Edit Ahri" }).click();
  const dialog = page.locator("dialog[open]");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("radiogroup", { name: "Engage" }).getByRole("radio", { name: "Hard" }).click();
  await page.getByTestId("trait-save").click();
  await expect(dialog).toHaveCount(0);
  await expect(rows.first()).toContainText("Edited");
  await expect(rows.first()).toContainText("Hard");

  // Put it back.
  await page.getByRole("button", { name: "Edit Ahri" }).click();
  await page.getByRole("button", { name: "Reset to default" }).click();
  await expect(page.locator("dialog[open]")).toHaveCount(0);
  await expect(rows.first()).toContainText("Default");
  await expect(rows.first()).toContainText("Conditional");
});
