/**
 * M7: micro-surveys. A survey cannot go live before its exact wording is
 * approved; a live survey appears once, right after the trigger, at most
 * one per user per week; it can be answered or dismissed and never comes
 * back; the profile switch turns questions off. Each test closes the
 * surveys it opened so other specs never see a band.
 */
import * as fs from "fs";
import * as path from "path";
import { test, expect, type Page } from "@playwright/test";
import { authenticatePage, createTeam, signInAsAdmin } from "./fixtures";

const WASM = path.resolve(__dirname, "../../target/site/pkg/lol_team_companion.wasm");
function fnUrl(name: string): string {
  const bytes = fs.readFileSync(WASM).toString("latin1");
  const m = bytes.match(new RegExp(`/api/${name}[0-9]+`));
  if (!m) throw new Error(`server fn ${name} not found`);
  return m[0];
}

async function threeDrafts(page: Page) {
  for (let i = 0; i < 3; i++) {
    const r = await page.request.post(fnUrl("save_draft"), {
      form: { name: `Survey draft ${i}`, actions_json: "[]", comments_json: "[]", tags_json: "[]" },
    });
    expect(r.status()).toBe(200);
  }
}

async function createLiveSurvey(admin: Page, question: string, trigger: string, kind = "single_choice") {
  await admin.goto("/admin/surveys?id=new");
  await admin.waitForLoadState("networkidle");
  const ed = admin.getByTestId("survey-editor");
  await ed.locator("#survey-question").fill(question);
  await ed.locator("#survey-kind").selectOption(kind);
  if (kind === "single_choice") await ed.locator("#survey-choices").fill("Yes\nNo");
  await ed.locator("#survey-trigger").selectOption(trigger);
  await ed.getByRole("button", { name: "Save", exact: true }).click();
  await expect(admin.getByText("Survey saved.")).toBeVisible();
  await admin.waitForURL(/id=survey/);
  const goLive = admin.getByRole("button", { name: "Go live" });
  await expect(goLive).toBeDisabled();
  await expect(admin.getByText("Not approved. The owner must approve this exact wording")).toBeVisible();
  await admin.getByRole("button", { name: "Approve wording" }).click();
  await expect(admin.getByText("Wording approved.")).toBeVisible();
  await expect(admin.getByRole("button", { name: "Go live" })).toBeEnabled();
  await admin.getByRole("button", { name: "Go live" }).click();
  await expect(admin.getByText("Survey is live.")).toBeVisible();
  return admin.url();
}

async function closeSurvey(admin: Page, url: string) {
  await admin.goto(url);
  await admin.waitForLoadState("networkidle");
  await admin.getByRole("button", { name: "Close survey" }).first().click();
  await admin.getByRole("dialog", { name: "Close this survey?" }).getByRole("button", { name: "Close survey" }).click();
  await expect(admin.getByText("Survey closed.")).toBeVisible();
}

const band = (page: Page) => page.locator("[data-survey-band]");

test("approve, go live, answer once; the weekly cap holds back a second survey", async ({ browser }) => {
  const stamp = Date.now();
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  const q1 = `Draft question ${stamp}?`;
  const q2 = `Board question ${stamp}?`;
  const url1 = await createLiveSurvey(admin, q1, "draft_saved_3");
  const url2 = await createLiveSurvey(admin, q2, "what_next_visit", "short_text");
  try {
    const user = await (await browser.newContext()).newPage();
    await authenticatePage(user);
    await createTeam(user, `SurveyTeam_${stamp}`);
    await threeDrafts(user);
    await user.goto("/draft");
    await expect(band(user)).toContainText(q1, { timeout: 10000 });
    await band(user).getByRole("button", { name: "Yes" }).click();
    await expect(user.getByText("Thanks. That helps.")).toBeVisible();
    await user.goto("/draft");
    await user.waitForTimeout(1500);
    await expect(band(user)).toHaveCount(0);
    // A second survey waits for next week, even on its own trigger page.
    await user.goto("/what-next");
    await user.waitForTimeout(1500);
    await expect(band(user)).toHaveCount(0);

    await admin.goto(url1);
    await admin.getByRole("tab", { name: "Results" }).click();
    await expect(admin.getByTestId("survey-results")).toContainText("Answered");
    await expect(admin.getByTestId("survey-results").locator("table")).toContainText("Yes");
  } finally {
    await closeSurvey(admin, url1);
    await closeSurvey(admin, url2);
  }
});

test("a dismissed question does not return; the profile switch turns questions off", async ({ browser }) => {
  const stamp = Date.now();
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  const url = await createLiveSurvey(admin, `Dismiss question ${stamp}?`, "draft_saved_3");
  try {
    const a = await (await browser.newContext()).newPage();
    await authenticatePage(a);
    await createTeam(a, `DismissTeam_${stamp}`);
    await threeDrafts(a);
    await a.goto("/draft");
    await expect(band(a)).toBeVisible({ timeout: 10000 });
    await band(a).getByRole("button", { name: "Dismiss" }).click();
    await expect(band(a)).toHaveCount(0);
    await a.goto("/");
    await a.waitForTimeout(1500);
    await expect(band(a)).toHaveCount(0);

    const b = await (await browser.newContext()).newPage();
    await authenticatePage(b);
    await createTeam(b, `OptOutTeam_${stamp}`);
    await b.goto("/profile#surveys");
    const sw = b.getByRole("switch", { name: "Ask me short survey questions" });
    await expect(sw).toHaveAttribute("aria-checked", "true");
    await sw.click();
    await expect(b.getByText("Survey questions are off.")).toBeVisible();
    await threeDrafts(b);
    await b.goto("/draft");
    await b.waitForTimeout(1500);
    await expect(band(b)).toHaveCount(0);
  } finally {
    await closeSurvey(admin, url);
  }
});
