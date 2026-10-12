/**
 * Team page and roster (M5): create a team, the team page's sections, join
 * and place a second player (start, bench, unbench), the roster page for
 * members, and team notes with Undo on edit and delete.
 * Requires a running dev server: cargo leptos watch
 */
import { test, expect, authenticatePage, createTeam } from "./fixtures";
import { captureErrors, filterRealErrors, navigateTo } from "./helpers";

test("team: create a new team and see the team page", async ({ authedPage }) => {
  const page = authedPage;
  const errors = captureErrors(page);
  const teamName = `AuditTeam_${Date.now()}`;
  await createTeam(page, teamName);
  await expect(page.getByRole("heading", { level: 1, name: teamName })).toBeVisible();
  for (const h of ["Roster", "Join requests", "Team notes", "Recent games", "Pool gaps"]) {
    await expect(page.getByRole("heading", { level: 2, name: h })).toBeVisible();
  }
  await expect(page.getByRole("heading", { level: 3, name: "Starting five" })).toBeVisible();
  // The creator is not placed yet; the five positions are open.
  await expect(page.locator('[data-roster-group="starters"] [data-roster-row="open"]')).toHaveCount(5);
  await expect(page.locator('[data-roster-group="unplaced"]')).toBeVisible();
  expect(filterRealErrors(errors)).toHaveLength(0);
});

test("roster: members see their team, never 'not on a team'", async ({ teamPage }) => {
  const page = teamPage;
  await navigateTo(page, "/team/roster");
  await expect(page.getByRole("heading", { level: 1, name: "Roster" })).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "Members" })).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "How players join" })).toBeVisible();
  await expect(page.locator("text=/not on a team/i")).toHaveCount(0);
  await expect(page.getByTestId("create-team")).toHaveCount(0);
});

test("bench: the leader starts, benches and unbenches a player", async ({ authedPage, browser }) => {
  const leader = authedPage;
  const teamName = `BenchTeam_${Date.now()}`;
  await createTeam(leader, teamName);

  // A second player asks to join.
  const ctx = await browser.newContext();
  const player = await ctx.newPage();
  await authenticatePage(player);
  await navigateTo(player, "/team/roster");
  await player.locator("#team-search").fill(teamName);
  const row = player.locator("[data-join-team] li", { hasText: teamName });
  await row.getByRole("button", { name: "Ask to join" }).click();
  await expect(row.getByText("Request sent", { exact: true })).toBeVisible({ timeout: 10000 });

  // The leader accepts on the team page.
  await navigateTo(leader, "/team/dashboard");
  const requests = leader.locator("[data-join-requests]");
  await requests.getByRole("button", { name: "Accept" }).first().click();
  await expect(leader.locator('[data-roster-group="unplaced"] [data-roster-row]')).toHaveCount(2, { timeout: 10000 });

  // Start them as Mid, then bench, then start them as Top.
  const playerRow = () =>
    leader.locator("[data-roster-row]", { hasText: /pagesuser/ }).filter({ hasNotText: "you" }).first();
  const act = async (item: string) => {
    await playerRow().getByRole("button", { name: /Roster actions for/ }).click();
    await leader.getByRole("menuitem", { name: item }).click();
  };
  await act("Start as Mid");
  await expect(leader.locator('[data-roster-group="starters"] [data-place="starter"]')).toHaveCount(1, { timeout: 10000 });
  await act("Move to the bench");
  await expect(leader.locator('[data-roster-group="substitutes"] [data-place="substitute"]')).toHaveCount(1, { timeout: 10000 });
  await expect(leader.locator('[data-roster-group="starters"] [data-place="starter"]')).toHaveCount(0);
  await act("Start as Top");
  await expect(leader.locator('[data-roster-group="starters"] [data-place="starter"]')).toHaveCount(1, { timeout: 10000 });
  await expect(leader.locator('[data-roster-group="substitutes"] [data-place="substitute"]')).toHaveCount(0);

  // The player sees the roster with their team; no leader actions.
  await navigateTo(player, "/team/roster");
  await expect(player.getByRole("heading", { level: 2, name: "Members" })).toBeVisible();
  await navigateTo(player, "/team/dashboard");
  await expect(player.getByRole("button", { name: /Roster actions for/ })).toHaveCount(0);
  await ctx.close();
});

test("team notes: edit and delete can be undone", async ({ teamPage }) => {
  const page = teamPage;
  const errors = captureErrors(page);
  if (!page.url().includes("/team/dashboard")) await navigateTo(page, "/team/dashboard");
  const notes = page.locator("[data-team-notes]");
  await notes.locator("#new-team-note").fill("Ward the enemy raptors at 2:45");
  await page.getByTestId("add-team-note").click();
  const note = notes.locator("[data-team-note]").first();
  await expect(note.locator("[data-note-text]")).toHaveText("Ward the enemy raptors at 2:45", { timeout: 10000 });

  // Edit, then Undo from the toast.
  await note.getByRole("button", { name: "Edit" }).click();
  await note.getByRole("textbox", { name: "Edit note" }).fill("Ward their raptors at 3:00");
  await note.getByRole("button", { name: "Save note" }).click();
  await expect(note.locator("[data-note-text]")).toHaveText("Ward their raptors at 3:00", { timeout: 10000 });
  await expect(note).toContainText("edited by");
  const region = page.getByRole("region", { name: "Notifications" });
  await region.getByRole("button", { name: "Undo" }).last().click();
  await expect(note.locator("[data-note-text]")).toHaveText("Ward the enemy raptors at 2:45", { timeout: 10000 });

  // Delete, then Undo.
  await note.getByRole("button", { name: "Delete" }).click();
  await expect(notes.locator("[data-team-note]")).toHaveCount(0, { timeout: 10000 });
  await region.getByRole("button", { name: "Undo" }).last().click();
  await expect(notes.locator("[data-team-note]")).toHaveCount(1, { timeout: 10000 });
  await expect(notes.locator("[data-note-text]").first()).toHaveText("Ward the enemy raptors at 2:45");
  expect(filterRealErrors(errors)).toHaveLength(0);
});
