/**
 * M141 group D (bug_report zgoormwy): a single-line field with an attached
 * primary button runs it on Enter; in a textarea Enter is a newline and
 * Ctrl+Enter submits (DESIGN 6.6 "Enter submits").
 * Requires a running dev server (CI runs it).
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("enter submits: tree drafter adds a branch on Enter", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const name = `EnterTree_${Date.now()}`;
  await navigateTo(page, "/tree-drafter");
  // The new tree field submits on Enter too (it already was a form).
  await page.locator("#tree-name").fill(name);
  await page.getByRole("button", { name: "Create tree" }).click();
  await expect(page.getByTestId("tree-list").getByRole("button", { name })).toHaveAttribute(
    "aria-pressed",
    "true",
    { timeout: 5000 }
  );

  const branch = `If they ban Jinx ${Date.now()}`;
  await page.getByRole("treeitem").first().getByRole("button", { name: "Branch", exact: true }).click();
  const field = page.locator("#branch-label");
  await expect(field).toBeFocused();
  await field.fill(branch);
  const before = page.url();
  await field.press("Enter");
  await expect(page.getByRole("treeitem").filter({ hasText: branch }).last()).toBeVisible({ timeout: 5000 });
  // No navigation: the form only runs the action.
  expect(page.url()).toBe(before);

  // An empty label is refused and nothing is added. The root's treeitem
  // now holds the new child's Branch button too; its own comes first.
  await page.getByRole("treeitem").first().getByRole("button", { name: "Branch", exact: true }).first().click();
  await page.locator("#branch-label").press("Enter");
  await expect(page.getByText("Give the branch a label.")).toBeVisible();

  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("enter submits: team note textarea keeps Enter as a newline, Ctrl+Enter adds", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await navigateTo(page, "/team/dashboard");
  const note = page.locator("#new-team-note");
  await expect(note).toBeVisible();
  const text = `Ward the river at 2:45 ${Date.now()}`;
  await note.fill("First line");
  await note.press("Enter");
  await note.pressSequentially(text);
  await expect(note).toHaveValue(`First line\n${text}`);
  await note.press("Control+Enter");
  await expect(page.locator("[data-team-notes] ul").getByText(text)).toBeVisible({ timeout: 5000 });
  await expect(note).toHaveValue("");

  expect(filterRealErrors(errors)).toHaveLength(0);
});
