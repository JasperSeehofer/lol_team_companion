/**
 * Action items: create one with an owner and a due day, complete it, edit
 * it in place, and delete it with Undo, mostly from the keyboard.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("action items: create, complete, edit and delete with undo", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const text = `Ward the enemy jungle ${Date.now()}`;

  await navigateTo(page, "/action-items");
  await expect(page.getByRole("heading", { level: 1, name: "Action items" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "No action items yet" })).toBeVisible();

  // Create: text, the first team member as owner, a due day; Enter adds it.
  await page.locator("#ai-new-text").fill(text);
  await page.locator("#ai-new-owner").selectOption({ index: 1 });
  const owner = await page.locator("#ai-new-owner option:checked").textContent();
  await page.locator("#ai-new-due").fill("2030-01-15");
  await page.locator("#ai-new-text").press("Enter");

  const row = () => page.locator("[data-action-item]", { hasText: text });
  await expect(row()).toHaveAttribute("data-status", "open", { timeout: 10000 });
  await expect(row().locator("[data-owner]")).toHaveText(`Owner ${owner}`);
  await expect(row().locator("[data-due]")).toHaveText("Due 15 Jan 2030");
  await expect(page.getByTestId("action-counts")).toHaveText(/1 open/);

  // Complete it: focus lands on the new row's checkbox; Space ticks it.
  const box = page.getByRole("checkbox", { name: text });
  await expect(box).toBeFocused();
  await page.keyboard.press("Space");
  await expect(row()).toHaveAttribute("data-status", "done", { timeout: 10000 });
  await expect(page.locator('[data-group="done"]')).toContainText(text);

  // Edit in place: E opens the editor, Enter saves.
  await page.getByRole("checkbox", { name: text }).focus();
  await page.keyboard.press("e");
  const editor = page.locator("[data-item-editor]");
  await expect(editor).toBeVisible();
  await page.keyboard.press("End");
  await page.keyboard.type(" (both sides)");
  await page.keyboard.press("Enter");
  const edited = `${text} (both sides)`;
  await expect(page.locator("[data-action-item]", { hasText: edited })).toHaveAttribute("data-status", "done", {
    timeout: 10000,
  });

  // Delete with the Delete key, then Undo brings it back as it was.
  await page.getByRole("checkbox", { name: edited }).focus();
  await page.keyboard.press("Delete");
  await expect(page.locator("[data-action-item]", { hasText: edited })).toHaveCount(0, { timeout: 10000 });
  await page.getByRole("button", { name: "Undo" }).click();
  const restored = page.locator("[data-action-item]", { hasText: edited });
  await expect(restored).toHaveAttribute("data-status", "done", { timeout: 10000 });
  await expect(restored.locator("[data-due]")).toHaveText("Due 15 Jan 2030");

  expect(filterRealErrors(errors)).toHaveLength(0);
});
