/**
 * M160 item 3: a coach places players and answers join requests. The leader
 * creates a team and makes a second account a coach (the roster menu's
 * "Make coach", as in team-invites.spec.ts); a third account asks to join.
 * The coach sees the request under "Join requests", accepts it, and places
 * the new member on the bench. On the leader's row and on a coach's row the
 * coach gets no roster menu, and the menu has no "Make coach" (coaches do
 * not make or unmake coaches; that stays with the leader).
 */
import { test, expect } from "@playwright/test";
import { authenticatePage, createTeam, waitForHome } from "./fixtures";

const STAMP = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;

test("a coach answers a join request and places the new member", async ({ browser }) => {
  const leader = await (await browser.newContext()).newPage();
  const leaderWho = await authenticatePage(leader);
  const teamName = `CoachRights_${STAMP}`;
  await createTeam(leader, teamName);

  // The leader mints an invite link; the coach account joins with it.
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const section = leader.locator("[data-team-invites]");
  await section.locator("#team-invite-uses").fill("2");
  await section.locator("#team-invite-days").fill("7");
  await section.getByTestId("mint-team-invite").click();
  const link = ((await section.getByTestId("team-invite-link").locator("code").textContent()) ?? "").trim();
  expect(link).toContain("/auth/register?invite=");
  const coach = await (await browser.newContext()).newPage();
  const coachWho = await authenticatePage(coach);
  await coach.goto(link);
  await coach.waitForLoadState("networkidle");
  await coach.getByTestId("invite-join-existing").getByTestId("invite-join-button").click();
  await waitForHome(coach);

  // The leader makes the coach a coach.
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const coachRow = leader.locator("[data-roster-row]", { hasText: coachWho.username }).first();
  await coachRow.getByRole("button", { name: /Roster actions for/ }).click();
  await leader.getByRole("menuitem", { name: "Make coach" }).click();
  await expect(leader.locator('[data-roster-group="coaches"]')).toContainText(coachWho.username, { timeout: 10000 });

  // A third account asks to join.
  const joiner = await (await browser.newContext()).newPage();
  const joinerWho = await authenticatePage(joiner);
  await joiner.goto("/team/dashboard");
  await joiner.waitForLoadState("networkidle");
  await joiner.locator("#team-search").fill(teamName);
  const joinRow = joiner.locator("[data-join-team] li", { hasText: teamName });
  await joinRow.getByRole("button", { name: "Ask to join" }).click();
  await expect(joinRow.getByText("Request sent", { exact: true })).toBeVisible({ timeout: 10000 });

  // The coach sees the request on the team page (M160 item 3) and accepts it.
  await coach.goto("/team/dashboard");
  await coach.waitForLoadState("networkidle");
  const requests = coach.locator("[data-join-requests]");
  await expect(requests).toHaveCount(1);
  await expect(requests).toContainText(joinerWho.username);
  await requests.getByRole("button", { name: "Accept" }).click();
  await expect(coach.locator('[data-roster-group="unplaced"]')).toContainText(joinerWho.username, { timeout: 10000 });

  // The coach places the new member on the bench.
  const joinerRow = coach.locator("[data-roster-row]", { hasText: joinerWho.username }).first();
  await joinerRow.getByRole("button", { name: /Roster actions for/ }).click();
  await coach.getByRole("menuitem", { name: "Move to the bench" }).click();
  await expect(coach.locator('[data-roster-group="substitutes"]')).toContainText(joinerWho.username, { timeout: 10000 });

  // No roster menu on the leader's row or on a coach's row for the coach,
  // and no "Make coach" / "Make team leader" / "Remove from team" options.
  await coach.reload();
  await coach.waitForLoadState("networkidle");
  const leaderRow = coach.locator("[data-roster-row]", { hasText: leaderWho.username }).first();
  await expect(leaderRow.getByRole("button", { name: /Roster actions for/ })).toHaveCount(0);
  const ownRow = coach.locator("[data-roster-row]", { hasText: coachWho.username }).first();
  await expect(ownRow.getByRole("button", { name: /Roster actions for/ })).toHaveCount(0);
  const newcomerRow = coach.locator("[data-roster-row]", { hasText: joinerWho.username }).first();
  await newcomerRow.getByRole("button", { name: /Roster actions for/ }).click();
  await expect(coach.getByRole("menuitem", { name: "Move to the bench" })).toHaveCount(0, { timeout: 5000 });
  await expect(coach.getByRole("menuitem", { name: "Make coach" })).toHaveCount(0);
  await expect(coach.getByRole("menuitem", { name: "Make team leader" })).toHaveCount(0);
  await expect(coach.getByRole("menuitem", { name: "Remove from team" })).toHaveCount(0);
  await expect(coach.getByRole("menuitem", { name: "Start as Mid" })).toHaveCount(1);
  await coach.keyboard.press("Escape");

  // M160 item 3 fix 1: the leader starts as Mid from their own menu, then
  // the coach's menu on the member row has no "Start as Mid" — the server
  // refuses to bench the leader (`assign_to_slot`'s `protect_leader`).
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const leaderOwnRow = leader.locator("[data-roster-row]", { hasText: leaderWho.username }).first();
  await leaderOwnRow.getByRole("button", { name: /Roster actions for/ }).click();
  await leader.getByRole("menuitem", { name: "Start as Mid" }).click();
  await expect(leader.locator('[data-roster-group="starters"]')).toContainText(leaderWho.username, { timeout: 10000 });

  await coach.goto("/team/dashboard");
  await coach.waitForLoadState("networkidle");
  const memberRow = coach.locator("[data-roster-row]", { hasText: joinerWho.username }).first();
  await memberRow.getByRole("button", { name: /Roster actions for/ }).click();
  await expect(coach.getByRole("menuitem", { name: "Start as Mid" })).toHaveCount(0);
  await expect(coach.getByRole("menuitem", { name: "Start as Top" })).toHaveCount(1);
  await coach.keyboard.press("Escape");
});
