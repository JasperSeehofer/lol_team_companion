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
import { authenticatePage, createTeam } from "./fixtures";

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
