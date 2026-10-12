/**
 * Cross-tenant isolation in the browser (M1).
 *
 * Account A (team 1) creates a draft and a game plan. Account B (team 2)
 * then tries to open them by ID, through the pages (?draft_id= / ?plan_id=)
 * and by calling the server functions directly. B must see no data and get
 * "Not found" errors; A's records must be unchanged.
 *
 * Server function URLs carry a build hash (leptos-patterns rule 57), so they
 * are read from the compiled WASM bundle.
 */
import * as fs from "fs";
import * as path from "path";
import { test, expect, type Page, type APIResponse } from "@playwright/test";
import { authenticatePage, createTeam, signInAsAdmin } from "./fixtures";

const WASM = path.resolve(__dirname, "../../target/site/pkg/lol_team_companion.wasm");
const TIMESTAMP = Date.now();

function fnUrl(name: string): string {
  const bytes = fs.readFileSync(WASM).toString("latin1");
  const m = bytes.match(new RegExp(`/api/${name}[0-9]+`));
  if (!m) throw new Error(`server fn ${name} not found in ${WASM}`);
  return m[0];
}

/** Text plus the live values of all inputs and textareas on the page. */
async function visibleData(page: Page): Promise<string> {
  const text = (await page.textContent("body")) ?? "";
  const values = await page
    .locator("input, textarea")
    .evaluateAll((els) => els.map((e) => (e as HTMLInputElement).value));
  return `${text}\n${values.join("\n")}`;
}

async function call(page: Page, name: string, form: Record<string, string>): Promise<APIResponse> {
  return page.request.post(fnUrl(name), { form });
}

test("a second team cannot open the first team's draft or game plan by ID", async ({
  browser,
}) => {
  // --- Account A creates a draft and a plan ---
  const ctxA = await browser.newContext();
  const a = await ctxA.newPage();
  await authenticatePage(a);
  await createTeam(a, `TenantA_${TIMESTAMP}`);

  const draftName = `A-secret-draft-${TIMESTAMP}`;
  const draftResp = await call(a, "save_draft", {
    name: draftName,
    actions_json: "[]",
    comments_json: "[]",
    tags_json: "[]",
  });
  expect(draftResp.status()).toBe(200);
  const draftId: string = await draftResp.json();
  expect(draftId).toMatch(/^draft:/);

  const planName = `A-secret-plan-${TIMESTAMP}`;
  const plan = {
    id: null,
    team_id: "",
    draft_id: draftId,
    name: planName,
    our_champions: [],
    enemy_champions: [],
    win_conditions: ["scale"],
    objective_priority: [],
    teamfight_strategy: "front to back",
    early_game: null,
    top_strategy: null,
    jungle_strategy: null,
    mid_strategy: null,
    bot_strategy: null,
    support_strategy: null,
    notes: "A's notes",
    win_condition_tag: null,
  };
  const planResp = await call(a, "create_plan", { plan_json: JSON.stringify(plan) });
  expect(planResp.status()).toBe(200);
  const planId: string = await planResp.json();
  expect(planId).toMatch(/^game_plan:/);

  // A can open both.
  await a.goto(`/game-plan?plan_id=${encodeURIComponent(planId)}`, { waitUntil: "networkidle" });
  await expect(a.locator("body")).toContainText(planName, { timeout: 10000 });
  const prefillA = await call(a, "get_draft_for_prefill", { draft_id: draftId });
  expect((await prefillA.json())?.name).toBe(draftName);

  // --- Account B (its own team) tries the same IDs ---
  const ctxB = await browser.newContext();
  const b = await ctxB.newPage();
  await authenticatePage(b);
  await createTeam(b, `TenantB_${TIMESTAMP}`);

  // Pages: nothing of A's shows up.
  await b.goto(`/draft?draft_id=${encodeURIComponent(draftId)}`, { waitUntil: "networkidle" });
  await b.waitForTimeout(1000);
  expect(await visibleData(b)).not.toContain(draftName);

  await b.goto(`/game-plan?plan_id=${encodeURIComponent(planId)}`, { waitUntil: "networkidle" });
  await b.waitForTimeout(1000);
  expect(await visibleData(b)).not.toContain(planName);
  if (process.env.SHOTS_DIR) {
    await b.screenshot({ path: `${process.env.SHOTS_DIR}/cross-tenant-b-game-plan.png` });
    await a.goto(`/game-plan?plan_id=${encodeURIComponent(planId)}`, { waitUntil: "networkidle" });
    await a.screenshot({ path: `${process.env.SHOTS_DIR}/cross-tenant-a-game-plan.png` });
  }

  // Server functions: reads come back empty, writes fail with "Not found".
  const prefillB = await call(b, "get_draft_for_prefill", { draft_id: draftId });
  expect(prefillB.status()).toBe(200);
  expect(await prefillB.json()).toBeNull();

  const forged = { ...plan, id: planId, name: "B was here" };
  const updateB = await call(b, "update_plan", { plan_json: JSON.stringify(forged) });
  expect(updateB.status()).toBeGreaterThanOrEqual(400);
  expect(await updateB.text()).toContain("Not found");

  const deleteB = await call(b, "delete_plan", { plan_id: planId });
  expect(deleteB.status()).toBeGreaterThanOrEqual(400);
  expect(await deleteB.text()).toContain("Not found");

  const draftUpdateB = await call(b, "update_draft", {
    draft_id: draftId,
    name: "B was here",
    actions_json: "[]",
    comments_json: "[]",
    tags_json: "[]",
  });
  expect(draftUpdateB.status()).toBeGreaterThanOrEqual(400);
  expect(await draftUpdateB.text()).toContain("Not found");

  // B cannot plant a draft in A's team by sending A's team id.
  const teamA = (await prefillA.json())?.team_id as string;
  const plantB = await call(b, "save_draft", {
    name: "planted",
    team_id: teamA,
    actions_json: "[]",
    comments_json: "[]",
    tags_json: "[]",
  });
  expect(plantB.status()).toBeGreaterThanOrEqual(400);

  // --- A's records are unchanged ---
  const listA = await call(a, "list_plans", {});
  const plans = (await listA.json()) as Array<{ id: string; name: string }>;
  expect(plans.find((p) => p.id === planId)?.name).toBe(planName);
  const prefillAgain = await call(a, "get_draft_for_prefill", { draft_id: draftId });
  expect((await prefillAgain.json())?.name).toBe(draftName);

  await ctxA.close();
  await ctxB.close();
});

test("a second team cannot share, list, revoke or edit the first team's M5 records", async ({
  browser,
}) => {
  const ctxA = await browser.newContext();
  const a = await ctxA.newPage();
  await authenticatePage(a);
  await createTeam(a, `ShareA_${TIMESTAMP}`);
  const planName = `A-shared-plan-${TIMESTAMP}`;
  const plan = {
    id: null, team_id: "", draft_id: null, name: planName, our_champions: [], enemy_champions: [],
    win_conditions: [], objective_priority: [], teamfight_strategy: "", early_game: null,
    top_strategy: null, jungle_strategy: null, mid_strategy: null, bot_strategy: null,
    support_strategy: null, notes: null, win_condition_tag: null,
  };
  const planId: string = await (await call(a, "create_plan", { plan_json: JSON.stringify(plan) })).json();
  const link = await (await call(a, "create_share", { kind: "game_plan", target_id: planId, days: "7" })).json();
  expect(link.token).toMatch(/^[a-z2-9]{32}$/);
  const matchId: string = await (
    await call(a, "save_next_match", {
      input_json: JSON.stringify({
        id: null,
        starts_at: new Date(Date.now() + 86400000).toISOString(),
        kind: "scrim",
        format: "bo1",
        opponent_id: null,
        notes: "A's secret note",
      }),
    })
  ).json();

  const ctxB = await browser.newContext();
  const b = await ctxB.newPage();
  await authenticatePage(b);
  await createTeam(b, `ShareB_${TIMESTAMP}`);

  for (const [name, form] of [
    ["create_share", { kind: "game_plan", target_id: planId, days: "7" }],
    ["revoke_share", { link_id: link.id }],
    ["remove_next_match", { match_id: matchId }],
    ["mark_brief_read", { match_id: matchId, read: "true" }],
  ] as const) {
    const r = await call(b, name, form as Record<string, string>);
    expect(r.status(), name).toBeGreaterThanOrEqual(400);
    expect(await r.text(), name).toContain("Not found");
  }
  const listed = await (await call(b, "list_shares", { kind: "game_plan", target_id: planId })).json();
  expect(listed).toEqual([]);
  // B's Match Day shows nothing of A's match.
  await b.goto("/");
  await expect(b.getByRole("heading", { level: 1, name: "Match Day" })).toBeVisible({ timeout: 15000 });
  expect(await visibleData(b)).not.toContain("A's secret note");

  // The link still works for anyone (B could not revoke it).
  const guest = await (await browser.newContext()).newPage();
  await guest.goto(`/s/${link.token}`);
  await expect(guest.getByRole("heading", { level: 1, name: planName })).toBeVisible({ timeout: 15000 });

  await ctxA.close();
  await ctxB.close();
});

test("users see only their own bug reports and cannot answer another user's survey", async ({
  browser,
}) => {
  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  const question = `Cross-user question ${TIMESTAMP}?`;
  const saved = await call(admin, "admin_save_survey", {
    survey_json: JSON.stringify({
      id: null,
      question,
      kind: "short_text",
      choices_text: "",
      scale_low_label: "",
      scale_high_label: "",
      trigger: "what_next_visit",
      note: "",
    }),
  });
  expect(saved.status()).toBe(200);
  const surveyId: string = await saved.json();
  const list = (await (await call(admin, "admin_list_surveys", {})).json()) as Array<{
    id: string;
    wording_hash: string;
  }>;
  const hash = list.find((s) => s.id === surveyId)!.wording_hash;
  expect((await call(admin, "admin_approve_survey", { id: surveyId, wording_hash: hash })).status()).toBe(200);
  expect((await call(admin, "admin_set_survey_status", { id: surveyId, status: "live" })).status()).toBe(200);
  try {
    const a = await (await browser.newContext()).newPage();
    await authenticatePage(a);
    const sent = await call(a, "submit_bug_report", {
      page_url: "/draft",
      element_label: "Draft",
      description: `A's private report ${TIMESTAMP}`,
      category: "bug",
    });
    expect(sent.status()).toBe(200);
    const prompt = await (await call(a, "get_pending_survey", { path: "/what-next" })).json();
    expect(prompt?.question).toBe(question);

    const b = await (await browser.newContext()).newPage();
    await authenticatePage(b);
    const mineB = (await (await call(b, "list_my_bug_reports", {})).json()) as unknown[];
    expect(JSON.stringify(mineB)).not.toContain(`A's private report ${TIMESTAMP}`);
    const board = await (await call(b, "list_bug_board", {})).text();
    expect(board).not.toContain(`A's private report ${TIMESTAMP}`);

    const forged = await call(b, "answer_survey", {
      prompt_id: prompt.prompt_id,
      answer_json: JSON.stringify({ Text: "B was here" }),
    });
    expect(forged.status()).toBeGreaterThanOrEqual(400);
    expect(await forged.text()).toContain("Not found");
    const dismiss = await call(b, "dismiss_survey", { prompt_id: prompt.prompt_id });
    expect(dismiss.status()).toBeGreaterThanOrEqual(400);

    // A can still answer.
    const own = await call(a, "answer_survey", {
      prompt_id: prompt.prompt_id,
      answer_json: JSON.stringify({ Text: "mine" }),
    });
    expect(own.status()).toBe(200);
  } finally {
    await call(admin, "admin_set_survey_status", { id: surveyId, status: "closed" });
  }
});
