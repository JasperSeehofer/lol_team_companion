/**
 * Draft page audit tests: save, list, reopen, delete and the board's
 * keyboard flow. Saving needs a team, so these use the teamPage fixture.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import {
  captureErrors,
  createDraft,
  DRAFT_SEARCH,
  draftSlot,
  filterRealErrors,
  navigateTo,
  selectChampion,
} from "./helpers";

test("draft: save a new draft", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  await createDraft(page, `AuditDraft_${Date.now()}`);
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("draft: saved draft appears in saved drafts list", async ({ teamPage }) => {
  const page = teamPage;
  const draftName = `AuditList_${Date.now()}`;
  await createDraft(page, draftName);

  await expect(page.getByRole("heading", { level: 2, name: "Saved drafts" })).toBeVisible();
  await expect(page.getByRole("link", { name: draftName })).toBeVisible({ timeout: 5000 });
});

test("draft: load draft from saved list", async ({ teamPage }) => {
  const page = teamPage;
  const draftName = `AuditLoad_${Date.now()}`;
  await createDraft(page, draftName);

  // Start a fresh draft, then reopen the saved one from the list.
  await page.getByRole("button", { name: "New draft" }).click();
  await expect(page.locator("#draft-name")).toHaveValue("");
  await expect(page.getByTestId("draft-save-state")).toHaveText("Not saved yet");

  await page.getByRole("link", { name: draftName }).click();
  await expect(page.locator("#draft-name")).toHaveValue(draftName, { timeout: 5000 });
  await expect(page.getByTestId("draft-save-state")).toHaveText("Saved");
});

test("draft: delete a draft", async ({ teamPage }) => {
  const page = teamPage;
  const draftName = `AuditDelete_${Date.now()}`;
  await createDraft(page, draftName);

  await page.getByRole("button", { name: "Delete", exact: true }).click();
  await page.locator('dialog[data-dialog="delete-draft"] [data-confirm="confirm"]').click();

  // The editor is back to a new draft and the list no longer has it.
  await expect(page.getByTestId("draft-save-state")).toHaveText("Not saved yet", { timeout: 5000 });
  await expect(page.getByRole("link", { name: draftName })).toHaveCount(0);
});

test("draft: no console errors during board interactions", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);

  await navigateTo(page, "/draft");
  await page.locator("#draft-name").fill(`AuditInteract_${Date.now()}`);

  // Keyboard: focus the first ban, open the picker, fill three steps.
  await page.locator(draftSlot(0)).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(DRAFT_SEARCH)).toBeFocused();
  for (const name of ["Aatrox", "Ahri", "Akali"]) {
    await selectChampion(page, name);
  }
  for (let i = 0; i < 3; i++) {
    await expect(page.locator(draftSlot(i))).toHaveAttribute("data-filled", "true");
  }

  // Escape returns to the board; arrows move between slots.
  await page.keyboard.press("Escape");
  await expect(page.locator(draftSlot(3))).toBeFocused();
  await page.keyboard.press("ArrowUp");

  // Switch our side.
  await page.locator('[data-segmented="our-side"] [role="radio"]:text-is("Red")').click();

  expect(filterRealErrors(errors)).toHaveLength(0);
});
