/**
 * Draft Graph (M4): stage A, the ranked list beside the board, the Why
 * panel, keyboard picks with a guessed role, a team note that changes the
 * ranking, and the admin trait editor; stage B, the force-directed canvas
 * (agreement with the list, keyboard and pointer picks, the performance
 * budget, reduced motion).
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
  // On the canvas (when the candidate is among the drawn ones) the note is
  // a lit edge to Jinx.
  const node = page.locator(`[data-graph-canvas] [data-node=candidate][data-champion="${id}"]`);
  if ((await node.count()) > 0) {
    await expect(node).toHaveAttribute("data-light", "focused");
    await expect(page.locator(`[data-graph-canvas] [data-edge="${id}-Jinx"]`)).toBeVisible();
  } else {
    await expect(page.locator("[data-graph-canvas-status]")).toContainText(`${name} (rank`);
  }
});

/** Wait for the canvas layout to settle and return its measurements. */
async function settledCanvas(page: Page) {
  const canvas = page.locator("[data-graph-canvas]");
  await expect(canvas).toHaveAttribute("data-sim-state", "settled", { timeout: 15000 });
  return canvas.evaluate((el) => ({
    mode: el.getAttribute("data-sim-mode"),
    ticks: Number(el.getAttribute("data-sim-ticks")),
    settleMs: Number(el.getAttribute("data-sim-settle-ms")),
    tickMaxMs: Number(el.getAttribute("data-sim-tick-max-ms")),
    nodes: Number(el.getAttribute("data-sim-nodes")),
  }));
}

test("canvas: agrees with the ranked list, keyboard pick, inside the budget, no third-party loads", async ({
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
  const sim = await settledCanvas(page);
  console.log(`draft graph canvas: ${JSON.stringify(sim)}`);
  expect(sim.mode).toBe("animated");
  // DESIGN.md 6.8 budget: settles in under 1.5 s, every tick under 8 ms.
  expect(sim.settleMs).toBeLessThan(1500);
  expect(sim.tickMaxMs).toBeLessThan(8);
  expect(sim.nodes).toBeLessThanOrEqual(60);

  const canvas = page.locator("[data-graph-canvas]");
  const options = page.locator("[data-graph-list] [role=option]");
  const nodes = canvas.locator("[data-node=candidate]");
  const drawn = await nodes.count();
  expect(drawn).toBeGreaterThanOrEqual(5);
  expect(drawn).toBeLessThanOrEqual(24);
  // The canvas and the list agree on the top candidate and its fit.
  const topId = await options.first().getAttribute("data-candidate");
  const topFit = Number(await options.first().getAttribute("data-fit"));
  expect(await nodes.first().getAttribute("data-champion")).toBe(topId);
  const fits = await nodes.evaluateAll((els) => els.map((e) => Number(e.getAttribute("data-fit"))));
  expect(Math.max(...fits)).toBe(topFit);
  await expect(canvas.locator(`[data-node=candidate][data-champion="${topId}"]`)).toHaveAttribute(
    "data-light",
    "focused"
  );
  // Locked picks are pinned: Jinx on our side, Ahri and Sejuani on theirs.
  await expect(canvas.locator("[data-node=pin]")).toHaveCount(3);
  await expect(page.locator("[data-graph-canvas-status]")).toContainText(/fit \d+, at [A-H][1-5] \(1 of \d+\)/);

  // One tab stop: arrows move the shared focus in rank order.
  await canvas.focus();
  await page.keyboard.press("ArrowRight");
  const second = options.nth(1);
  await expect(second).toHaveAttribute("aria-selected", "true");
  await expect(canvas).toBeFocused();
  const name = ((await second.locator("span.truncate").first().textContent()) ?? "").trim();
  await expect(page.locator("#graph-why-heading")).toHaveText(`Why ${name}`);
  await expect(page.locator("[data-graph-canvas-status]")).toContainText(`${name}, fit`);
  await page.keyboard.press("End");
  await page.keyboard.press("Home");
  await expect(options.first()).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("ArrowDown");
  // Enter picks the focused candidate into the current slot.
  await page.keyboard.press("Enter");
  await expect(page.locator(draftSlot(9))).toHaveAttribute("aria-label", new RegExp(`${name}, \\w+`));
  await expect(page.locator("[data-graph-status]")).toContainText("Blue pick 3", { timeout: 10000 });

  expect(foreign).toEqual([]);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("canvas: click focuses, double-click picks, a pin lights its candidates, Escape goes back", async ({
  teamPage,
}) => {
  const page = teamPage;
  await midDraft(page);
  await openGraph(page);
  await settledCanvas(page);
  const canvas = page.locator("[data-graph-canvas]");
  const third = canvas.locator("[data-node=candidate]").nth(2);
  const id = (await third.getAttribute("data-champion")) ?? "";
  const name = (await third.getAttribute("data-name")) ?? "";
  await third.click();
  await expect(third).toHaveAttribute("data-light", "focused");
  await expect(page.locator(`[data-graph-list] [role=option][data-candidate="${id}"]`)).toHaveAttribute(
    "aria-selected",
    "true"
  );
  await expect(page.locator("#graph-why-heading")).toHaveText(`Why ${name}`);

  // A locked pick: the candidates related to it light up, the rest fade.
  const pin = canvas.locator("[data-node=pin][data-champion=Jinx]");
  await pin.click();
  await expect(pin).toHaveAttribute("data-light", "focused");
  await expect(page.locator("[data-graph-canvas-status]")).toContainText("Jinx (your pick");
  expect(await canvas.locator("[data-node=candidate][data-light=lit]").count()).toBeGreaterThan(0);

  // Escape returns to the board on the current step.
  await canvas.focus();
  await page.keyboard.press("Escape");
  await expect(page.locator(draftSlot(9))).toBeFocused();

  // Double-click picks.
  await third.dblclick();
  await expect(page.locator(draftSlot(9))).toHaveAttribute("aria-label", new RegExp(`${name}, \\w+`));
});

test.describe("reduced motion", () => {
  test("canvas: the settled layout is drawn directly", async ({ teamPage }) => {
    const page = teamPage;
    // The fixture opens its own context, so emulate the preference here.
    await page.emulateMedia({ reducedMotion: "reduce" });
    await midDraft(page);
    await openGraph(page);
    const sim = await settledCanvas(page);
    expect(sim.mode).toBe("instant");
    expect(sim.ticks).toBeGreaterThan(0);
    const nodes = page.locator("[data-graph-canvas] [data-node=candidate]");
    expect(await nodes.count()).toBeGreaterThanOrEqual(5);
    // Every node has a place inside the canvas.
    const box = await page.locator("[data-graph-canvas]").boundingBox();
    const first = await nodes.first().boundingBox();
    expect(box && first).toBeTruthy();
    expect(first!.x).toBeGreaterThan(box!.x);
    expect(first!.x + first!.width).toBeLessThan(box!.x + box!.width);
  });
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
