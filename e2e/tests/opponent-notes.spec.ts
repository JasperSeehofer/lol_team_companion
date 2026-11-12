/**
 * Opponent player notes: collapsed-by-default notes control on PlayerRow,
 * save on blur and Ctrl+Enter, preview when collapsed, server-side length validation.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

// Helper to open a dossier by name
async function openDossier(page: any, opponentName: string) {
  const dossierList = page.getByTestId("dossier-list");
  await dossierList
    .getByRole("button", { name: opponentName })
    .click();
  const dossier = page.getByTestId("dossier");
  await expect(
    dossier.getByRole("heading", { level: 2, name: opponentName })
  ).toBeVisible({ timeout: 10000 });
  return dossier;
}

test.describe("Opponent Player Notes", () => {
  test("add a note to a player, blur saves it, preview shows on reload", async ({
    teamPage,
  }) => {
    const page = teamPage;
    const errors = captureErrors(page);

    // Navigate to opponents page
    await navigateTo(page, "/opponents");

    // Create an opponent
    const opponentName = `Test Opponent ${Date.now()}`;
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(opponentName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();

    // Wait for the dossier to open
    let dossier = await openDossier(page, opponentName);

    // Add a player to the mid role
    await dossier.locator("#opp-add-mid").fill("TestPlayer");
    await dossier
      .locator('li[data-role="mid"]')
      .getByRole("button", { name: "Add player" })
      .click();

    // Wait for the player row to appear
    const playerRow = dossier.locator('li[data-role="mid"]');
    await expect(playerRow).toContainText("TestPlayer", { timeout: 5000 });

    // Find the "Add note" button
    const addNoteButton = playerRow.getByRole("button", { name: "Add note" });
    await expect(addNoteButton).toBeVisible();

    // Click to expand the notes field
    await addNoteButton.click();

    // Wait for the textarea to appear
    const textarea = playerRow.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 2000 });

    // Type a note
    const noteText = "This player is very aggressive in lane";
    await textarea.fill(noteText);

    // Blur to save (click outside the field)
    await playerRow.locator("#opp-riot-mid").click();

    // Wait for save to complete (check for "Saved" status or the button to change)
    await expect(
      playerRow.getByRole("button", { name: "Edit note" })
    ).toBeVisible({ timeout: 3000 });

    // The preview should be visible
    const preview = playerRow.getByTestId("opp-note-preview");
    await expect(preview).toHaveText(noteText);

    // Reload the page
    await page.reload();
    await page.waitForLoadState("networkidle");

    // Re-open the dossier after reload
    dossier = await openDossier(page, opponentName);

    // The note should still be there with preview visible and field collapsed
    const playerRowAfterReload = dossier.locator('li[data-role="mid"]');
    await expect(playerRowAfterReload).toContainText("TestPlayer");

    // The button should say "Edit note" (not "Add note")
    await expect(
      playerRowAfterReload.getByRole("button", { name: "Edit note" })
    ).toBeVisible();

    // The preview should still show the note
    const previewAfterReload = playerRowAfterReload.getByTestId("opp-note-preview");
    await expect(previewAfterReload).toHaveText(noteText);

    // The textarea should not be visible (collapsed)
    await expect(playerRowAfterReload.locator("textarea")).not.toBeVisible();

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("type a note and press Control+Enter saves it, preview shows after reload", async ({
    teamPage,
  }) => {
    const page = teamPage;
    const errors = captureErrors(page);

    await navigateTo(page, "/opponents");

    // Create an opponent
    const opponentName = `Ctrl Enter Test ${Date.now()}`;
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(opponentName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();

    // Open the dossier
    let dossier = await openDossier(page, opponentName);

    // Add a player
    await dossier.locator("#opp-add-mid").fill("CtrlPlayer");
    await dossier
      .locator('li[data-role="mid"]')
      .getByRole("button", { name: "Add player" })
      .click();

    const playerRow = dossier.locator('li[data-role="mid"]');
    await expect(playerRow).toContainText("CtrlPlayer", { timeout: 5000 });

    // Open notes field
    await playerRow.getByRole("button", { name: "Add note" }).click();
    const textarea = playerRow.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 2000 });

    // Type a note
    const noteText = "Ctrl+Enter save test";
    await textarea.fill(noteText);

    // Press Control+Enter to save (no blur)
    await textarea.press("Control+Enter");

    // Wait for save to complete
    await expect(
      playerRow.getByRole("button", { name: "Edit note" })
    ).toBeVisible({ timeout: 3000 });

    // Preview should show the text
    const preview = playerRow.getByTestId("opp-note-preview");
    await expect(preview).toHaveText(noteText);

    // Reload and verify
    await page.reload();
    await page.waitForLoadState("networkidle");

    // Re-open the dossier
    dossier = await openDossier(page, opponentName);
    const playerRowAfterReload = dossier.locator('li[data-role="mid"]');

    // Preview should still show the note
    const previewAfterReload = playerRowAfterReload.getByTestId("opp-note-preview");
    await expect(previewAfterReload).toHaveText(noteText);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("long note gets truncated in preview", async ({ teamPage }) => {
    const page = teamPage;
    const errors = captureErrors(page);

    await navigateTo(page, "/opponents");

    // Create an opponent and player
    const opponentName = `Truncate Test ${Date.now()}`;
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(opponentName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();

    let dossier = await openDossier(page, opponentName);

    await dossier.locator("#opp-add-mid").fill("LongNotePlayer");
    await dossier
      .locator('li[data-role="mid"]')
      .getByRole("button", { name: "Add player" })
      .click();

    const playerRow = dossier.locator('li[data-role="mid"]');
    await expect(playerRow).toContainText("LongNotePlayer", { timeout: 5000 });

    // Add a long note
    await playerRow.getByRole("button", { name: "Add note" }).click();
    const textarea = playerRow.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 2000 });
    const longNote =
      "This is a very long note that should be truncated in the preview because it exceeds the maximum length for a single line display in the user interface.";
    await textarea.fill(longNote);
    await playerRow.locator("#opp-riot-mid").click();
    await expect(
      playerRow.getByRole("button", { name: "Edit note" })
    ).toBeVisible({ timeout: 3000 });

    // Preview should be truncated
    const preview = playerRow.getByTestId("opp-note-preview");
    const previewText = await preview.textContent();
    expect(previewText).toContain("…");
    expect(previewText?.length).toBeLessThan(longNote.length);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("notes field has maxlength of 1000", async ({ teamPage }) => {
    const page = teamPage;
    const errors = captureErrors(page);

    await navigateTo(page, "/opponents");

    // Create an opponent and player
    const opponentName = `Maxlength Test ${Date.now()}`;
    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(opponentName);
    await dialog.getByRole("button", { name: "Add opponent" }).click();

    let dossier = await openDossier(page, opponentName);

    await dossier.locator("#opp-add-mid").fill("MaxPlayer");
    await dossier
      .locator('li[data-role="mid"]')
      .getByRole("button", { name: "Add player" })
      .click();

    const playerRow = dossier.locator('li[data-role="mid"]');
    await expect(playerRow).toContainText("MaxPlayer", { timeout: 5000 });

    // Open notes field
    await playerRow.getByRole("button", { name: "Add note" }).click();
    const textarea = playerRow.locator("textarea");
    await expect(textarea).toBeVisible({ timeout: 2000 });

    // Check maxlength attribute
    const maxlength = await textarea.getAttribute("maxlength");
    expect(maxlength).toBe("1000");

    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});
