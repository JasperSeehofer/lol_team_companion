/**
 * Tree meta (M160 item 9): the new-tree form's opponent is a select of the
 * team's dossiers ("No opponent" / one per dossier), the open tree's title
 * shows "vs {dossier name}", and the Edit form next to the title renames the
 * tree and keeps the dossier. Requires a running dev server:
 * cargo leptos watch
 *
 * Page facts (src/pages/tree_drafter.rs): the new-tree form has #tree-name,
 * #tree-opponent (a select) and "Create tree"; the open tree's title is
 * [data-testid=tree-title] with [data-testid=tree-opponent-label] beside it;
 * [data-testid=tree-edit-open] opens [data-testid=tree-edit-form] with
 * #tree-edit-name, #tree-edit-opponent and [data-testid=tree-edit-save].
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("tree meta: pick a dossier when creating, rename via Edit, title shows vs {dossier}", async ({
  teamPage,
}) => {
  const page = teamPage;
  const errors = captureErrors(page);
  const ts = Date.now();
  const dossierName = `Meta Dossier ${ts}`;
  const treeName = `MetaTree_${ts}`;
  const renamed = `MetaTreeRenamed_${ts}`;

  // Create a scouting dossier on /opponents.
  await navigateTo(page, "/opponents");
  await page.getByTestId("new-opponent").click();
  const dialog = page.getByRole("dialog", { name: "New opponent" });
  await dialog.locator("#opp-new-name").fill(dossierName);
  await dialog.getByRole("button", { name: "Add opponent" }).click();
  await expect(
    page.getByTestId("dossier-list").getByRole("button", { name: dossierName })
  ).toBeVisible({ timeout: 10000 });

  // Create a tree, picking the dossier in the opponent select.
  await navigateTo(page, "/tree-drafter");
  await page.locator("#tree-name").fill(treeName);
  await page.locator("#tree-opponent").selectOption({ label: dossierName });
  await page.getByRole("button", { name: "Create tree" }).click();
  const list = page.getByTestId("tree-list");
  await expect(list.getByRole("button", { name: treeName })).toHaveAttribute(
    "aria-pressed",
    "true",
    { timeout: 5000 }
  );
  // The open tree's title and the list row show the dossier's name.
  await expect(page.getByTestId("tree-title")).toHaveText(treeName);
  await expect(page.getByTestId("tree-opponent-label")).toHaveText(`vs ${dossierName}`);
  await expect(list.getByRole("button", { name: treeName })).toContainText(
    `vs ${dossierName}`
  );

  // Rename the tree through the Edit form next to the title.
  await page.getByTestId("tree-edit-open").click();
  const form = page.getByTestId("tree-edit-form");
  await expect(form).toBeVisible({ timeout: 5000 });
  await expect(form.locator("#tree-edit-name")).toHaveValue(treeName);
  // The opponent select carries the dossier's id as its value.
  await expect(form.locator("#tree-edit-opponent")).toHaveValue(/^opponent:/);
  await form.locator("#tree-edit-name").fill(renamed);
  await form.getByTestId("tree-edit-save").click();
  await expect(page.getByTestId("tree-title")).toHaveText(renamed, { timeout: 5000 });
  await expect(page.getByTestId("tree-opponent-label")).toHaveText(`vs ${dossierName}`);
  await expect(list.getByRole("button", { name: renamed })).toBeVisible({ timeout: 5000 });
  await expect(
    page.locator('[aria-label="Notifications"]').getByText("Tree saved")
  ).toBeVisible({ timeout: 5000 });

  expect(filterRealErrors(errors)).toHaveLength(0);
});
