/**
 * Opponents page: dossier list, the new opponent dialog, creating a
 * dossier and adding a player to an empty role.
 *
 * Riot IDs are left out (or only stored) so nothing here depends on the
 * Riot API being reachable.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

const ROLES = ["top", "jungle", "mid", "bot", "support"];

test.describe("Opponents Page", () => {
  test("loads with heading and new opponent button", async ({ teamPage }) => {
    const page = teamPage;
    await navigateTo(page, "/opponents");

    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Opponents");
    await expect(page.getByTestId("new-opponent")).toBeVisible();
    await expect(page.getByRole("heading", { level: 2, name: "No dossier open" })).toBeVisible();
  });

  test("new opponent dialog has a name and five Riot ID fields", async ({ teamPage }) => {
    const page = teamPage;
    await navigateTo(page, "/opponents");

    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await expect(dialog).toBeVisible();
    await expect(dialog.locator("#opp-new-name")).toBeVisible();
    for (const role of ROLES) {
      await expect(dialog.locator(`#opp-new-${role}`)).toBeVisible();
    }

    // Cancel closes it without creating anything.
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).not.toBeVisible();
    await expect(page.getByTestId("dossier-list")).toHaveCount(0);
  });

  test("create an opponent and add a player", async ({ teamPage }) => {
    const page = teamPage;
    const errors = captureErrors(page);
    const name = `Audit Rooks ${Date.now()}`;
    await navigateTo(page, "/opponents");

    await page.getByTestId("new-opponent").click();
    const dialog = page.getByRole("dialog", { name: "New opponent" });
    await dialog.locator("#opp-new-name").fill(name);
    await dialog.getByRole("button", { name: "Add opponent" }).click();

    // The new dossier opens and is listed with its evidence.
    const dossier = page.getByTestId("dossier");
    await expect(dossier.getByRole("heading", { level: 2, name })).toBeVisible({ timeout: 10000 });
    const row = page.getByTestId("dossier-list").getByRole("button", { name: new RegExp(name) });
    await expect(row).toHaveAttribute("aria-pressed", "true");
    await expect(row).toContainText("No players yet");
    await expect(row).toContainText("never surveyed");

    // Fill the empty mid slot by name (no Riot lookup).
    await dossier.locator("#opp-add-mid").fill("Pallas");
    await dossier.locator('li[data-role="mid"]').getByRole("button", { name: "Add player" }).click();
    await expect(dossier.locator('li[data-role="mid"]')).toContainText("Pallas", { timeout: 5000 });
    await expect(dossier.locator("#opp-add-mid")).toHaveCount(0);
    await expect(row).toContainText("1 player");

    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});
