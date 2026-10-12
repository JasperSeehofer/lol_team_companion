/**
 * M7: the transparency ledger. A draft month stays private; publishing
 * shows it with the computed split; costs above support show as not
 * covered; editing a published month marks it revised. The public page
 * needs the approved money texts, so the test approves them on the test
 * database and withdraws the approvals afterwards.
 */
import { test, expect, type Page } from "@playwright/test";
import { signInAsAdmin } from "./fixtures";

const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

function testMonths(): [string, string, string, string] {
  // Months long past, spread by time so reruns rarely meet.
  const n = Math.floor(Date.now() / 1000) % 3000;
  const y = 1990 + Math.floor(n / 100);
  const m = (n % 6) * 2 + 1;
  const a = `${y}-${String(m).padStart(2, "0")}`;
  const b = `${y}-${String(m + 1).padStart(2, "0")}`;
  return [a, b, `${MONTHS[m - 1]} ${y}`, `${MONTHS[m]} ${y}`];
}

async function setApproval(admin: Page, slug: string, approve: boolean) {
  await admin.goto("/admin/texts");
  await admin.waitForLoadState("networkidle");
  const section = admin.locator(`section[data-site-text="${slug}"]`);
  const approveBtn = section.getByRole("button", { name: "Approve this wording" });
  const withdrawBtn = section.getByRole("button", { name: "Withdraw approval" });
  if (approve && (await approveBtn.count()) > 0) {
    await approveBtn.click();
    await expect(section.getByText("Approved", { exact: true })).toBeVisible();
  }
  if (!approve && (await withdrawBtn.count()) > 0) {
    await withdrawBtn.click();
    await expect(section.getByText("Needs approval")).toBeVisible();
  }
}

async function saveMonth(admin: Page, month: string, support: string, hosting: string, expectComputed: RegExp) {
  await admin.goto("/admin/ledger");
  await admin.waitForLoadState("networkidle");
  await admin.getByRole("button", { name: "Add month" }).click();
  const d = admin.getByRole("dialog", { name: "Month" });
  await d.locator("#ledger-month").fill(month);
  await d.locator("#ledger-support").fill(support);
  await d.locator("#ledger-hosting").fill(hosting);
  await expect(admin.getByTestId("ledger-computed")).toHaveText(expectComputed);
  await d.getByRole("button", { name: "Save as draft" }).click();
  await expect(admin.getByText("Draft saved.")).toBeVisible();
}

async function publish(admin: Page, label: string) {
  const row = admin.locator("tr").filter({ hasText: label });
  await row.getByRole("button", { name: "Publish" }).click();
  await admin.getByRole("dialog", { name: "Publish this month?" }).getByRole("button", { name: "Publish" }).click();
  await expect(admin.getByText("Published.")).toBeVisible();
}

test("draft, publish, not covered and revised", async ({ browser }) => {
  const [m1, m2, label1, label2] = testMonths();
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  const guest = await (await browser.newContext()).newPage();
  try {
    await setApproval(admin, "money_policy", true);
    await setApproval(admin, "ledger_labels", true);

    await saveMonth(admin, m1, "20", "5", /Running costs: €5\.00 · To the maintainer: €15\.00/);
    await expect(admin.locator("tr").filter({ hasText: label1 })).toContainText("Draft");
    await guest.goto("/transparency");
    await expect(guest.getByText("How support is used")).toBeVisible();
    await expect(guest.locator("main")).not.toContainText(label1);

    await publish(admin, label1);
    await guest.reload();
    const row1 = guest.locator("tr").filter({ hasText: label1 });
    await expect(row1).toContainText("€20.00");
    await expect(row1).toContainText("€15.00");

    await saveMonth(admin, m2, "1", "4", /Not covered by support: €3\.00/);
    await publish(admin, label2);
    await guest.reload();
    await expect(guest.locator("tr").filter({ hasText: label2 })).toContainText("Not covered by support: €3.00");

    // Edit the published first month: a revision.
    await admin.locator("tr").filter({ hasText: label1 }).getByRole("button", { name: "Edit" }).click();
    const d = admin.getByRole("dialog", { name: "Month" });
    await d.locator("#ledger-hosting").fill("6");
    await d.getByRole("button", { name: "Save revision" }).click();
    await expect(admin.getByText("Revision saved.")).toBeVisible();
    await guest.reload();
    await expect(guest.locator("main")).toContainText("Revised on");
  } finally {
    await setApproval(admin, "money_policy", false);
    await setApproval(admin, "ledger_labels", false);
  }
  await guest.goto("/transparency");
  await expect(guest.getByText("No months published yet.")).toBeVisible();
});
