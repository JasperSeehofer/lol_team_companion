/**
 * M7: the What next board. Only published proposals appear; one vote per
 * user per idea, persistent and withdrawable; open ideas sorted by votes;
 * shipped ideas link to the changelog and take no votes.
 */
import { test, expect, type Page } from "@playwright/test";
import { authenticatePage, signInAsAdmin } from "./fixtures";

const M7 = "milestone-m7-community-and-transparency-loop";

async function createProposal(
  admin: Page,
  title: string,
  opts: { status?: string; published?: boolean; shippedIn?: string } = {}
) {
  await admin.goto("/admin/proposals?id=new");
  await admin.waitForLoadState("networkidle");
  const editor = admin.getByTestId("proposal-editor");
  await editor.locator("#proposal-title").fill(title);
  await editor.locator("#proposal-summary").fill(`Summary of ${title}.`);
  await editor.locator("#proposal-status").selectOption(opts.status ?? "proposed");
  if (opts.shippedIn) await editor.locator("#proposal-shipped").selectOption(opts.shippedIn);
  if (opts.published ?? true) await editor.getByLabel("Published (shown on What next)").check();
  await editor.getByRole("button", { name: "Save", exact: true }).click();
  await expect(admin.getByText("Proposal saved.")).toBeVisible();
  await admin.waitForURL(/id=proposal/);
}

const idea = (page: Page, title: string) =>
  page.locator("li[data-proposal]").filter({ has: page.getByRole("heading", { name: title }) });

test("votes persist, count once, can be withdrawn and order the board", async ({ browser }) => {
  const stamp = Date.now();
  const older = `Older idea ${stamp}`;
  const newer = `Newer idea ${stamp}`;
  const draft = `Draft idea ${stamp}`;
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  await createProposal(admin, older);
  await createProposal(admin, newer);
  await createProposal(admin, draft, { status: "draft", published: false });

  const user = await (await browser.newContext()).newPage();
  await authenticatePage(user);
  await user.goto("/what-next");
  await expect(idea(user, older)).toBeVisible();
  await expect(user.getByRole("heading", { name: draft })).toHaveCount(0);

  // Equal votes: the older idea first.
  const order = async () =>
    user.locator("li[data-proposal] h2").allTextContents();
  let titles = await order();
  expect(titles.indexOf(older)).toBeLessThan(titles.indexOf(newer));

  // Vote for the newer idea; it moves up on the next load.
  const vote = idea(user, newer).getByRole("button", { name: `Vote for ${newer}` });
  await vote.click();
  await expect(vote).toHaveAttribute("aria-pressed", "true");
  await expect(idea(user, newer).getByTestId("vote-count")).toHaveText("1 vote");
  await user.reload();
  await expect(idea(user, newer).getByRole("button", { name: `Vote for ${newer}` })).toHaveAttribute("aria-pressed", "true");
  await expect(idea(user, newer).getByTestId("vote-count")).toHaveText("1 vote");
  titles = await order();
  expect(titles.indexOf(newer)).toBeLessThan(titles.indexOf(older));

  // A double click never counts twice.
  const voteOlder = idea(user, older).getByRole("button", { name: `Vote for ${older}` });
  await voteOlder.dblclick();
  await user.waitForTimeout(800);
  await user.reload();
  await expect(idea(user, older).getByTestId("vote-count")).not.toHaveText("2 votes");

  // Take the vote back.
  await idea(user, newer).getByRole("button", { name: `Vote for ${newer}` }).click();
  await expect(idea(user, newer).getByTestId("vote-count")).toHaveText("0 votes");
  await user.reload();
  await expect(idea(user, newer).getByRole("button", { name: `Vote for ${newer}` })).toHaveAttribute("aria-pressed", "false");
});

test("a shipped idea links to the changelog and takes no votes", async ({ browser }) => {
  const stamp = Date.now();
  const shipped = `Shipped idea ${stamp}`;
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  await createProposal(admin, shipped, { status: "shipped", shippedIn: M7 });

  const user = await (await browser.newContext()).newPage();
  await authenticatePage(user);
  await user.goto("/what-next");
  await user.getByRole("tab", { name: /Shipped/ }).click();
  const row = user.locator("#what-next-panel-shipped li").filter({ hasText: shipped });
  await expect(row).toBeVisible();
  await expect(row.locator(`a[href="/changelog#${M7}"]`)).toBeVisible();
  await expect(row.getByRole("button", { name: /Vote/ })).toHaveCount(0);
});
