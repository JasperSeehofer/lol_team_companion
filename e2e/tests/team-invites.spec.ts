/**
 * Team-bound invites (polish pass): a leader creates an invite link on the
 * team page; a new player who registers with it lands in the team (welcome:
 * "You are on TEAM", roster: not placed yet). A signed-in player without a
 * team joins with the link; one already in a team is told to leave first.
 */
import { test, expect, type Page } from "@playwright/test";
import { authenticatePage, createTeam, waitForHome, TEST_PASSWORD } from "./fixtures";

const SHOTS = process.env.TEAM_INVITE_SHOTS_DIR;
const STAMP = `${Date.now()}_${process.env.TEST_WORKER_INDEX ?? "0"}`;

async function shot(page: Page, name: string) {
  if (!SHOTS) return;
  for (const width of [1280, 400]) {
    await page.setViewportSize({ width, height: 900 });
    await page.screenshot({ path: `${SHOTS}/${name}-${width}.png`, fullPage: true });
  }
  await page.setViewportSize({ width: 1280, height: 900 });
}

/** The leader mints a link on the team page and returns it. */
async function mintLink(leader: Page, position: string): Promise<string> {
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const section = leader.locator("[data-team-invites]");
  await expect(section.getByRole("heading", { name: "Invite a player" })).toBeVisible();
  await section.locator("#team-invite-uses").fill("2");
  await section.locator("#team-invite-days").fill("7");
  if (position) await section.locator("#team-invite-position").selectOption(position);
  await section.getByTestId("mint-team-invite").click();
  const link = section.getByTestId("team-invite-link").locator("code");
  await expect(link).toContainText("/auth/register?invite=");
  const text = (await link.textContent()) ?? "";
  await expect(section.locator("[data-team-invite]").first()).toContainText("0 of 2 used");
  return text.trim();
}

test("a new player registers into the team with the leader's link", async ({ browser }) => {
  const leader = await (await browser.newContext()).newPage();
  await authenticatePage(leader);
  const teamName = `Invite_${STAMP}`;
  await createTeam(leader, teamName);
  const link = await mintLink(leader, "mid");
  await shot(leader, "team-invite-leader");

  const player = await (await browser.newContext()).newPage();
  await player.goto(link);
  await expect(player.getByTestId("invite-team")).toContainText(`This invite signs you up for ${teamName}`);
  await shot(player, "team-invite-register");
  await player.fill("input[name=username]", `invited_${STAMP}`);
  await player.fill("input[name=email]", `invited_${STAMP}@test.invalid`);
  await player.fill("input[name=password]", TEST_PASSWORD);
  await player.check("#register-terms");
  await player.click("main button[type=submit]");
  await waitForHome(player);
  await player.waitForLoadState("networkidle");
  await expect(player.locator("[data-welcome]")).toContainText(`You are on ${teamName}`);
  await shot(player, "team-invite-welcome");

  // The leader sees the new member, not placed yet, with the code's position.
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const unplaced = leader.locator('[data-roster-group="unplaced"]');
  await expect(unplaced).toContainText(`invited_${STAMP}`);
  await expect(unplaced).toContainText("MID");
  await expect(leader.locator("[data-team-invite]").first()).toContainText("1 of 2 used");
});

test("a signed-in player joins with the link; one already in a team cannot", async ({ browser }) => {
  const leader = await (await browser.newContext()).newPage();
  await authenticatePage(leader);
  const teamName = `Join_${STAMP}`;
  await createTeam(leader, teamName);
  const link = await mintLink(leader, "");

  // Already in a team (the leader of another one): told to leave first.
  const other = await (await browser.newContext()).newPage();
  await authenticatePage(other);
  await createTeam(other, `Other_${STAMP}`);
  await other.goto(link);
  await expect(other.getByTestId("invite-in-team")).toContainText("already belongs to a team");
  await expect(other.getByRole("heading", { level: 1 })).toHaveText("You are already on a team");
  await expect(other.getByTestId("invite-join-button")).toHaveCount(0);
  // Signed in: no sign-up form (M10 review, orchestrator decision).
  await expect(other.locator("input[name=username]")).toHaveCount(0);
  await shot(other, "team-invite-in-team");

  // Signed in without a team: one click joins.
  const loner = await (await browser.newContext()).newPage();
  await authenticatePage(loner);
  await loner.goto(link);
  const box = loner.getByTestId("invite-join-existing");
  await expect(box).toContainText(`This link is an invite to ${teamName}`);
  await expect(loner.getByRole("heading", { level: 1 })).toHaveText(`Join ${teamName}`);
  await expect(loner.locator("input[name=username]")).toHaveCount(0);
  await shot(loner, "team-invite-signed-in");
  // The button acts only once the page is hydrated (CI run 106 clicked
  // before that and timed out waiting for Home).
  await loner.waitForLoadState("networkidle");
  await box.getByTestId("invite-join-button").click();
  await waitForHome(loner);
  await loner.goto("/team/roster");
  await expect(loner.getByRole("heading", { name: "Roster" })).toBeVisible();
  await expect(loner.locator("main")).toContainText(teamName);
});
