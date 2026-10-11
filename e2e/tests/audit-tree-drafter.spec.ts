/**
 * Tree drafter audit tests: create a tree, add the root and a branch, edit
 * a node, switch trees, delete a tree. Trees need a team (teamPage).
 * Requires a running dev server: cargo leptos watch
 *
 * Page facts (src/pages/tree_drafter.rs): the tree list is
 * [data-testid=tree-list]; the new tree form has #tree-name and "Create
 * tree"; a created tree opens at once; branches are role=treeitem rows with
 * "Branch" and "Delete" buttons; the node editor has #node-label, "Save
 * node" and [data-testid=node-save-state].
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

/** Create a tree on /tree-drafter; it opens straight away. */
async function createTree(page: Page, name: string): Promise<void> {
  await navigateTo(page, "/tree-drafter");
  await page.locator("#tree-name").fill(name);
  await page.getByRole("button", { name: "Create tree" }).click();
  await expect(page.getByTestId("tree-list").getByRole("button", { name })).toHaveAttribute(
    "aria-pressed",
    "true",
    { timeout: 5000 }
  );
}

/** Add the open tree's root draft. */
async function addRoot(page: Page): Promise<void> {
  await page.getByRole("button", { name: "Add the root draft" }).click();
  await expect(page.locator("#branch-label")).toHaveValue("Root");
  await page.getByRole("button", { name: "Add branch" }).click();
  await expect(page.getByRole("treeitem").first()).toBeVisible({ timeout: 5000 });
}

test("tree-drafter: create a new tree", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await createTree(page, `AuditTree_${Date.now()}`);
  await expect(page.getByText("This tree has no nodes yet.")).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("tree-drafter: add and edit a node", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await createTree(page, `AuditNodeTree_${Date.now()}`);
  await addRoot(page);

  // Branch off the root.
  const branch = `If they ban Jinx ${Date.now()}`;
  await page.getByRole("treeitem").first().getByRole("button", { name: "Branch", exact: true }).click();
  await expect(page.locator("#branch-label")).toBeFocused();
  await page.locator("#branch-label").fill(branch);
  await page.getByRole("button", { name: "Add branch" }).click();
  const row = page.getByRole("treeitem").filter({ hasText: branch });
  await expect(row).toBeVisible({ timeout: 5000 });

  // Open it and rename it.
  await row.getByRole("button", { name: branch }).click();
  await expect(page.locator("#node-label")).toHaveValue(branch);
  await page.locator("#node-label").fill(`${branch} (edited)`);
  await page.getByRole("button", { name: "Save node" }).click();
  await expect(page.getByTestId("node-save-state")).toHaveText("Saved", { timeout: 5000 });
  await expect(page.getByRole("treeitem").filter({ hasText: `${branch} (edited)` })).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("tree-drafter: switch between trees remains interactive", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const ts = Date.now();
  const treeA = `TreeA_${ts}`;
  const treeB = `TreeB_${ts + 1}`;

  await createTree(page, treeA);
  await page.locator("#tree-name").fill(treeB);
  await page.getByRole("button", { name: "Create tree" }).click();

  const list = page.getByTestId("tree-list");
  await expect(list.getByRole("button", { name: treeA })).toBeVisible({ timeout: 5000 });
  await expect(list.getByRole("button", { name: treeB })).toBeVisible({ timeout: 5000 });

  for (const name of [treeA, treeB, treeA]) {
    await list.getByRole("button", { name }).click();
    await expect(list.getByRole("button", { name })).toHaveAttribute("aria-pressed", "true");
  }

  // Still interactive: the open tree can be deleted and the form takes input.
  await expect(page.getByRole("button", { name: "Delete tree" })).toBeVisible();
  await page.locator("#tree-name").fill("InteractTest");
  await expect(page.locator("#tree-name")).toHaveValue("InteractTest");

  const panics = filterRealErrors(errors).filter(
    (e) => e.includes("[pageerror]") || e.includes("panic") || e.includes("wasm")
  );
  expect(panics).toHaveLength(0);
});

test("tree-drafter: delete a tree", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const treeName = `AuditDeleteTree_${Date.now()}`;
  await createTree(page, treeName);

  // Deleting asks for confirmation first.
  await page.getByRole("button", { name: "Delete tree" }).click();
  await page.locator('[data-confirm="confirm"]').click();

  await expect(page.getByTestId("tree-list").getByRole("button", { name: treeName })).toHaveCount(0, {
    timeout: 5000,
  });
  await expect(page.getByRole("heading", { name: "No tree open" })).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});
