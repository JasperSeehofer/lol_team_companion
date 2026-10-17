/**
 * M148 item 4 (UI F03/F04, atelier cards C4/C5): a save that fails is never
 * silent. On the draft, the game plan and the tree drafter the header says
 * "Could not save: <reason>" with Retry, no success toast appears and one
 * error toast says the changes are not saved; Retry after the server answers
 * again says "Saved". Leaving the page with work at risk asks first
 * (beforeunload), and a list that did not load says so with "Try again"
 * instead of looking empty.
 *
 * Server functions POST to /api/<name><hash>; the specs fail them with
 * page.route (a 503 or 500 with an empty body, which the page reads as "the
 * server did not answer").
 * Requires a running dev server (CI runs it).
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { createDraft, navigateTo, saveGamePlan } from "./helpers";

const NO_ANSWER = "Could not save: the server did not answer";
const FAILED_TOAST = "Your changes are not saved. Keep this tab open and use Retry.";

async function failRoute(page: Page, fn: string, status = 503) {
  await page.route(`**/api/${fn}*`, (route) => route.fulfill({ status, body: "" }));
}

function toasts(page: Page, text: string) {
  return page.locator("[data-toast]").filter({ hasText: text });
}

/** Create a tree; it opens straight away (as in audit-tree-drafter). */
async function createTree(page: Page, name: string) {
  await navigateTo(page, "/tree-drafter");
  await page.locator("#tree-name").fill(name);
  await page.getByRole("button", { name: "Create tree" }).click();
  await expect(page.getByTestId("tree-list").getByRole("button", { name })).toHaveAttribute(
    "aria-pressed",
    "true",
    { timeout: 5000 }
  );
}

/** Branch off the root and open the branch in the node editor. */
async function openBranch(page: Page, label: string) {
  await page.getByRole("treeitem").first().getByRole("button", { name: "Branch", exact: true }).click();
  await page.locator("#branch-label").fill(label);
  await page.getByRole("button", { name: "Add branch" }).click();
  const row = page.getByRole("treeitem").filter({ hasText: label }).last();
  await expect(row).toBeVisible({ timeout: 5000 });
  await row.getByRole("button", { name: label }).click();
  await expect(page.locator("#node-label")).toHaveValue(label);
}

test("draft: a failed autosave says so, Retry saves once the server answers", async ({ teamPage }) => {
  const page = teamPage;
  const name = `SaveFail_${Date.now()}`;
  await createDraft(page, name);
  // The first save's toast has gone, so any later one would be new.
  await expect(toasts(page, "Draft saved")).toHaveCount(0, { timeout: 15000 });

  await failRoute(page, "update_draft");
  await page.locator("#draft-name").fill(`${name} edited`);
  const state = page.getByTestId("draft-save-state");
  await expect(state).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
  await expect(state).toContainText(NO_ANSWER);
  const retry = state.getByRole("button", { name: "Retry" });
  await expect(retry).toBeVisible();
  await expect(toasts(page, FAILED_TOAST)).toHaveCount(1);
  await expect(toasts(page, "Draft saved")).toHaveCount(0);

  // Retry while the server still fails: the same failure, no second toast.
  await retry.click();
  await expect(state).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
  await expect(toasts(page, FAILED_TOAST)).toHaveCount(1);
  await expect(toasts(page, "Draft saved")).toHaveCount(0);

  await page.unroute("**/api/update_draft*");
  await state.getByRole("button", { name: "Retry" }).click();
  await expect(state).toHaveText("Saved", { timeout: 8000 });
  await expect(toasts(page, "Draft saved")).toHaveCount(1);
  // Stored: a reload shows the new name.
  await navigateTo(page, page.url());
  await expect(page.locator("#draft-name")).toHaveValue(`${name} edited`);
});

test("game plan: a failed autosave says so, Retry saves once the server answers", async ({ teamPage }) => {
  const page = teamPage;
  const name = `PlanFail_${Date.now()}`;
  await navigateTo(page, "/game-plan");
  await saveGamePlan(page, name);
  await expect(toasts(page, "Game plan saved")).toHaveCount(0, { timeout: 15000 });

  await failRoute(page, "update_plan");
  await page.locator("#plan-name").fill(`${name} edited`);
  const state = page.getByTestId("plan-save-state");
  await expect(state).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
  await expect(state).toContainText(NO_ANSWER);
  await expect(toasts(page, FAILED_TOAST)).toHaveCount(1);

  // "Save plan" while it still fails: no success toast, no second error toast.
  await page.getByTestId("plan-save").click();
  await expect(state).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
  await expect(toasts(page, "Game plan saved")).toHaveCount(0);
  await expect(toasts(page, FAILED_TOAST)).toHaveCount(1);

  await page.unroute("**/api/update_plan*");
  await state.getByRole("button", { name: "Retry" }).click();
  await expect(state).toHaveText("Saved", { timeout: 8000 });
  await expect(toasts(page, "Game plan saved")).toHaveCount(1);
});

test("tree drafter: a failed node save and a failed branch say so", async ({ teamPage }) => {
  const page = teamPage;
  const stamp = Date.now();
  await createTree(page, `TreeFail_${stamp}`);
  const label = `Branch ${stamp}`;
  await openBranch(page, label);

  await failRoute(page, "save_node");
  await page.locator("#node-label").fill(`${label} edited`);
  const state = page.getByTestId("node-save-state");
  await expect(state).toHaveAttribute("data-save-state", "failed", { timeout: 8000 });
  await expect(state).toContainText(NO_ANSWER);
  await expect(toasts(page, FAILED_TOAST)).toHaveCount(1);

  // Branch while saving fails: the error, never "Branch created".
  await page.getByRole("button", { name: "Branch from the lit slot" }).click();
  await expect(toasts(page, "Could not save the branch: the server did not answer")).toHaveCount(1, {
    timeout: 8000,
  });
  await expect(toasts(page, "Branch created from this point")).toHaveCount(0);
  // The editor stays on the node that was open.
  await expect(page.locator("#node-label")).toHaveValue(`${label} edited`);

  await page.unroute("**/api/save_node*");
  await state.getByRole("button", { name: "Retry" }).click();
  await expect(state).toHaveText("Saved", { timeout: 8000 });
  await expect(toasts(page, "Node saved")).toHaveCount(1);
});

test("leaving a new draft with picks or a name asks first; a blank one does not", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/draft");
  // A click is a user gesture, so the browser would ask if the page wanted it.
  await page.locator("#draft-name").click();
  let asked = false;
  page.on("dialog", async (d) => {
    asked = true;
    await d.accept();
  });
  await page.close({ runBeforeUnload: true });
  await expect.poll(() => page.isClosed()).toBe(true);
  expect(asked).toBe(false);
});

test("leaving a draft with unsaved work shows the browser's leave dialog", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/draft");
  await page.locator("#draft-name").fill(`Unsaved_${Date.now()}`);
  await expect(page.getByTestId("draft-save-state")).toHaveText("Not saved yet");
  const dialog = page.waitForEvent("dialog");
  await page.close({ runBeforeUnload: true });
  const d = await dialog;
  expect(d.type()).toBe("beforeunload");
  await d.accept();
});

test("leaving a game plan whose save failed shows the leave dialog", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/game-plan");
  await saveGamePlan(page, `LeavePlan_${Date.now()}`);
  await failRoute(page, "update_plan");
  await page.locator("#plan-name").fill(`LeavePlan edited ${Date.now()}`);
  await expect(page.getByTestId("plan-save-state")).toHaveAttribute("data-save-state", "failed", {
    timeout: 8000,
  });
  const dialog = page.waitForEvent("dialog");
  await page.close({ runBeforeUnload: true });
  const d = await dialog;
  expect(d.type()).toBe("beforeunload");
  await d.accept();
});

test("lists that did not load say so and load again with Try again", async ({ teamPage }) => {
  const page = teamPage;
  const stamp = Date.now();

  // Drafts: saving refetches the list, which fails.
  await navigateTo(page, "/draft");
  await failRoute(page, "list_drafts", 500);
  await page.locator("#draft-name").fill(`ListFail_${stamp}`);
  await page.getByTestId("draft-save").click();
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved", { timeout: 10000 });
  const drafts = page.getByRole("alert").filter({ hasText: "The drafts did not load" });
  await expect(drafts).toBeVisible({ timeout: 8000 });
  await expect(page.getByText("Drafts you save appear here")).toHaveCount(0);
  await page.unroute("**/api/list_drafts*");
  await drafts.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByRole("link", { name: new RegExp(`ListFail_${stamp}`) }).first()).toBeVisible({
    timeout: 8000,
  });

  // Plans: the same after a save.
  await navigateTo(page, "/game-plan");
  await failRoute(page, "list_plans", 500);
  await saveGamePlan(page, `ListPlan_${stamp}`);
  const plans = page.getByRole("alert").filter({ hasText: "The plans did not load" });
  await expect(plans).toBeVisible({ timeout: 8000 });
  await page.unroute("**/api/list_plans*");
  await plans.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByTestId("plan-list").getByText(`ListPlan_${stamp}`)).toBeVisible({ timeout: 8000 });

  // Trees: creating one refetches the list.
  await navigateTo(page, "/tree-drafter");
  await failRoute(page, "list_trees", 500);
  await page.locator("#tree-name").fill(`ListTree_${stamp}`);
  await page.getByRole("button", { name: "Create tree" }).click();
  const trees = page.getByRole("alert").filter({ hasText: "The trees did not load" });
  await expect(trees).toBeVisible({ timeout: 8000 });
  await page.unroute("**/api/list_trees*");
  await trees.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByTestId("tree-list").getByRole("button", { name: `ListTree_${stamp}` })).toBeVisible({
    timeout: 8000,
  });
});
