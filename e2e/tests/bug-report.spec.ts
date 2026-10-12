/**
 * The bug-report widget (M7 rebuild): a docked trigger, a select-mode hint
 * bar with Skip and Escape, the report dialog, the toast with "Your
 * reports", and the dark-pattern gate (no emoji, no exclamation marks).
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors } from "./helpers";

const REPORT_BUTTON = 'button[aria-label="Report a bug or a wish"]';
const HINT = "[data-feedback-hint]";
const TAGGED_DRAFT_ELEMENT = '[data-feedback-label*="Draft"]';

const dialog = (page: import("@playwright/test").Page) =>
  page.getByRole("dialog", { name: "Report" });

test.describe("bug-report widget", () => {
  test("docked on signed-in pages, absent on public ones", async ({ authedPage, browser }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    const trigger = authedPage.locator(REPORT_BUTTON);
    await expect(trigger).toBeVisible({ timeout: 5000 });
    // Docked in the page flow: not fixed over the content.
    const position = await trigger.evaluate((el) => {
      let n: HTMLElement | null = el as HTMLElement;
      while (n) {
        if (getComputedStyle(n).position === "fixed") return "fixed";
        n = n.parentElement;
      }
      return "flow";
    });
    expect(position).toBe("flow");
    expect(filterRealErrors(errors)).toHaveLength(0);

    const guest = await (await browser.newContext()).newPage();
    await guest.goto("/auth/login");
    await guest.waitForTimeout(500);
    await expect(guest.locator(REPORT_BUTTON)).toHaveCount(0);
    await guest.goto("/changelog");
    await guest.waitForTimeout(500);
    await expect(guest.locator(REPORT_BUTTON)).toHaveCount(0);
  });

  test("select mode shows the hint, captures a labelled element", async ({ authedPage }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    await authedPage.locator(REPORT_BUTTON).click();
    await expect(authedPage.locator(HINT)).toContainText("Click the part of the page your report is about.");
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await expect(dialog(authedPage)).toBeVisible({ timeout: 3000 });
    await expect(authedPage.getByTestId("report-about")).toContainText("Draft");
    await expect(authedPage.locator(HINT)).toHaveCount(0);
    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("Skip opens the dialog for the whole page; Escape cancels select mode", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.locator(HINT).getByRole("button", { name: "Skip" }).click();
    await expect(dialog(authedPage)).toBeVisible();
    await expect(authedPage.getByTestId("report-about")).toHaveText("The whole page");
    await dialog(authedPage).getByRole("button", { name: "Cancel" }).click();
    await expect(dialog(authedPage)).toBeHidden();

    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.keyboard.press("Escape");
    await expect(authedPage.locator(HINT)).toHaveCount(0);
    // The capture listener is gone: a click on a tagged element opens nothing.
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await authedPage.waitForTimeout(300);
    await expect(dialog(authedPage)).toBeHidden();
  });

  test("sending shows the toast and the report appears under Your reports", async ({ authedPage }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.locator(HINT).getByRole("button", { name: "Skip" }).click();
    const d = dialog(authedPage);
    await d.getByRole("radio", { name: "Wishlist" }).click();
    const text = `e2e wish ${Date.now()}`;
    await d.locator("textarea").fill(text);
    await d.getByRole("button", { name: "Send report" }).click();
    await expect(authedPage.getByText("Thanks. Your report is in.")).toBeVisible({ timeout: 5000 });
    await expect(d).toBeHidden();
    await authedPage.getByRole("button", { name: "Your reports" }).click();
    await authedPage.waitForURL("**/bugs?tab=mine");
    const panel = authedPage.locator("#bugs-panel-mine");
    await expect(panel).toContainText(text);
    await expect(panel).toContainText("New");
    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("an empty report is refused in the dialog", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.locator(HINT).getByRole("button", { name: "Skip" }).click();
    await dialog(authedPage).getByRole("button", { name: "Send report" }).click();
    await expect(dialog(authedPage)).toContainText("Please describe the problem or the wish.");
  });

  test("widget DOM has no forbidden characters", async ({ authedPage }) => {
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.locator(HINT).getByRole("button", { name: "Skip" }).click();
    const text = (await dialog(authedPage).textContent()) ?? "";
    expect(text).not.toMatch(/[\u{1F300}-\u{1FAFF}]/u);
    expect(text).not.toContain("!");
  });
});
