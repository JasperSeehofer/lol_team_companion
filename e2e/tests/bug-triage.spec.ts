/**
 * M7: a report goes from the widget through triage to the bug board and
 * back to the reporter ("Your reports", the account menu count), and a fix
 * links to its changelog entry. The board never shows the reporter's words.
 */
import { test, expect, type Page } from "@playwright/test";
import { authenticatePage, signInAsAdmin } from "./fixtures";

const M7 = "milestone-m7-community-and-transparency-loop";

async function sendReport(page: Page, text: string) {
  await page.goto("/draft");
  // Wait for hydration: a click on the widget before it would do nothing.
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
  await page.locator('button[aria-label="Report a bug or a wish"]').click();
  await page.locator("[data-feedback-hint]").getByRole("button", { name: "Skip" }).click();
  const d = page.getByRole("dialog", { name: "Report" });
  await d.locator("textarea").fill(text);
  await d.getByRole("button", { name: "Send report" }).click();
  await expect(page.getByText("Thanks. Your report is in.")).toBeVisible({ timeout: 5000 });
}

async function openInTriage(admin: Page, firstLine: string, filter = "New") {
  await admin.goto("/admin/bugs");
  await admin.waitForLoadState("networkidle");
  await admin.locator('[data-segmented="bug-filter"]').getByRole("radio", { name: filter, exact: true }).click();
  await admin.getByRole("link", { name: new RegExp(firstLine) }).click();
  await expect(admin.getByTestId("bug-editor")).toBeVisible();
  await expect(admin.getByTestId("reporter-words")).toHaveText(firstLine);
}

test("triage reaches the reporter and the board; a fix links the changelog", async ({ browser }) => {
  const stamp = Date.now();
  const words = `Private words with team details ${stamp}`;
  const title = `Board title ${stamp}`;

  const user = await (await browser.newContext()).newPage();
  await authenticatePage(user);
  await sendReport(user, words);

  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  await openInTriage(admin, words);
  await admin.locator("#triage-status").selectOption("triaged");
  await admin.locator("#triage-title").fill(title);
  await admin.locator("#triage-note").fill("We can reproduce it.");
  await admin.getByLabel("Show on the bug board").check();
  await admin.getByRole("button", { name: "Save", exact: true }).click();
  await expect(admin.getByText("Saved. The reporter sees the new status.")).toBeVisible();

  // The reporter's account menu counts the update.
  await user.goto("/changelog");
  await user.waitForLoadState("networkidle");
  await user.getByRole("button", { name: /Account menu/ }).click();
  await expect(user.getByRole("menuitem", { name: "Your reports (1 updated)" })).toBeVisible();
  await user.keyboard.press("Escape");

  await user.goto("/bugs?tab=mine");
  const mine = user.locator("#bugs-panel-mine");
  await expect(mine).toContainText("Updated");
  await expect(mine).toContainText("We can reproduce it.");
  await expect(mine).toContainText("Triaged");

  // The board shows the admin's title, never the reporter's words.
  await user.goto("/bugs");
  const open = user.locator("#bugs-panel-open");
  await expect(open).toContainText(title);
  const boardText = (await open.textContent()) ?? "";
  expect(boardText).not.toContain(words);

  // Fixed needs a changelog entry; the board's Fixed tab links to it.
  await openInTriage(admin, words, "Triaged");
  await admin.locator("#triage-status").selectOption("fixed");
  await admin.locator("#triage-fixed").selectOption(M7);
  await admin.getByRole("button", { name: "Save", exact: true }).click();
  await expect(admin.getByText("Saved. The reporter sees the new status.")).toBeVisible();

  await user.goto("/bugs");
  // The tabs switch only once hydrated; a click before that is lost (CI flake).
  await user.waitForLoadState("networkidle");
  await user.waitForTimeout(500);
  await user.getByRole("tab", { name: /Fixed/ }).click();
  const fixed = user.locator("#bugs-panel-fixed");
  await expect(fixed).toContainText(title);
  await expect(fixed.locator("tr").filter({ hasText: title }).locator(`a[href="/changelog#${M7}"]`)).toBeVisible();
});

test("another user sees only their own reports", async ({ browser }) => {
  const stamp = Date.now();
  const a = await (await browser.newContext()).newPage();
  await authenticatePage(a);
  await sendReport(a, `Alice only ${stamp}`);
  const b = await (await browser.newContext()).newPage();
  await authenticatePage(b);
  await b.goto("/bugs?tab=mine");
  await expect(b.locator("#bugs-panel-mine")).toContainText("You have not sent a report yet.");
  await expect(b.locator("main")).not.toContainText(`Alice only ${stamp}`);
});
