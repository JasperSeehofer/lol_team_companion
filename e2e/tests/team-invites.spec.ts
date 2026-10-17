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

// ---------------------------------------------------------------------------
// M148 item 2 (atelier card C1): full links, Copy, coach invites
// ---------------------------------------------------------------------------

/** The newest invite row on the team page after a reload. */
async function reloadedRow(leader: Page) {
  await leader.reload();
  await leader.waitForLoadState("networkidle");
  return leader.locator("[data-team-invites] [data-team-invite]").first();
}

test("after a reload an open link shows in full and Copy copies it", async ({ browser }) => {
  const ctx = await browser.newContext();
  await ctx.grantPermissions(["clipboard-read", "clipboard-write"]);
  const leader = await ctx.newPage();
  await authenticatePage(leader);
  await createTeam(leader, `Copy_${STAMP}`);
  const link = await mintLink(leader, "");
  expect(link).toMatch(/^https?:\/\/[^/]+\/auth\/register\?invite=/);

  const row = await reloadedRow(leader);
  const url = row.getByTestId("team-invite-url");
  // The absolute link (the server renders the path; the page adds its origin).
  await expect(url).toHaveText(link, { timeout: 10000 });
  const copy = row.getByTestId("copy-invite-link");
  await expect(copy).toHaveAccessibleName("Copy link");
  await expect(row.getByTestId("invite-copy-status")).toHaveText("");
  await leader.setViewportSize({ width: 390, height: 844 });
  expect(await copy.evaluate((el) => el.getBoundingClientRect().height)).toBeGreaterThanOrEqual(44);
  await copy.click();
  await expect(copy).toHaveAccessibleName("Copied", { timeout: 5000 });
  await expect(row.getByTestId("invite-copy-status")).toHaveText("Link copied");
  await expect(row.getByTestId("invite-copy-error")).toHaveCount(0);
  expect(await leader.evaluate(() => navigator.clipboard.readText())).toBe(link);
  await expect(copy).toHaveAccessibleName("Copy link", { timeout: 5000 });
  // Disable stays beside it.
  await expect(row.getByRole("button", { name: "Disable" })).toBeVisible();
  await ctx.close();
});

test("a refused copy says so and never claims success", async ({ browser }) => {
  const ctx = await browser.newContext();
  // The browser refuses every clipboard write (plain HTTP, a denied permission).
  await ctx.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: () => Promise.reject(new DOMException("Denied", "NotAllowedError")) },
    });
  });
  const leader = await ctx.newPage();
  await authenticatePage(leader);
  await createTeam(leader, `NoCopy_${STAMP}`);
  await mintLink(leader, "");
  const row = await reloadedRow(leader);
  await expect(row.getByTestId("team-invite-url")).toContainText("/auth/register?invite=");
  const copy = row.getByTestId("copy-invite-link");
  await copy.click();
  await expect(row.getByTestId("invite-copy-error")).toHaveText(
    "Could not copy. Select the link and copy it by hand.",
    { timeout: 5000 },
  );
  await expect(copy).toHaveAccessibleName("Copy link");
  await expect(row.getByTestId("invite-copy-status")).toHaveText("");
  // The link is selected for copying by hand.
  const selected = await leader.evaluate(() => window.getSelection()?.toString() ?? "");
  expect(selected).toContain("/auth/register?invite=");
  await ctx.close();
});

test("a coach invites players; a plain member sees no invite panel", async ({ browser }) => {
  const leader = await (await browser.newContext()).newPage();
  await authenticatePage(leader);
  const teamName = `Coach_${STAMP}`;
  await createTeam(leader, teamName);
  const link = await mintLink(leader, "");

  // Two players join with the leader's link (2 uses).
  const join = async () => {
    const page = await (await browser.newContext()).newPage();
    const who = await authenticatePage(page);
    await page.goto(link);
    await page.waitForLoadState("networkidle");
    await page.getByTestId("invite-join-existing").getByTestId("invite-join-button").click();
    await waitForHome(page);
    return { page, username: who.username };
  };
  const coach = await join();
  const member = await join();

  // The leader makes the first one a coach.
  await leader.goto("/team/dashboard");
  await leader.waitForLoadState("networkidle");
  const coachRow = leader.locator("[data-roster-row]", { hasText: coach.username }).first();
  await coachRow.getByRole("button", { name: /Roster actions for/ }).click();
  await leader.getByRole("menuitem", { name: "Make coach" }).click();
  await expect(leader.locator('[data-roster-group="coaches"]')).toContainText(coach.username, { timeout: 10000 });

  // The coach sees "Invite a player" and mints a link of their own.
  const coachLink = await mintLink(coach.page, "");
  expect(coachLink).not.toBe(link);
  await expect(coach.page.locator("[data-team-invites]")).toContainText("until the leader places them");
  // Both links of the team are listed for the coach.
  await expect(coach.page.locator("[data-team-invites] [data-team-invite]")).toHaveCount(2);
  // No join requests panel for the coach: answering them stays with the leader.
  await expect(coach.page.locator("[data-join-requests]")).toHaveCount(0);

  // A plain member gets no invite panel.
  await member.page.goto("/team/dashboard");
  await member.page.waitForLoadState("networkidle");
  await expect(member.page.getByRole("heading", { level: 2, name: "Roster" })).toBeVisible();
  await expect(member.page.locator("[data-team-invites]")).toHaveCount(0);
});
