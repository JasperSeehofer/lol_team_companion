/**
 * M165 Review together (TC-J10, TC-J11, TC-J12): a multi-user spec.
 *
 * Five members registered by the spec (a leader creates the team, four
 * join with the team's invite link), one browser context each, and one
 * scheduled match (an official best of 5 that started an hour ago, so its
 * review step links on and games 2 to 4 have sessions of their own).
 *
 * Game 1 is the full session: the leader opens it from the match thread
 * and facilitates at 1280, one member writes async on a 390 px phone, the
 * others at 1280, one member passes every prompt. Then:
 *   (1) sealed until the reveal: every server-fn response body and the page
 *       HTML (hydration payload included) of each context, the
 *       facilitator's too, holds no other member's words; the facilitator
 *       sees one written count, never per prompt; after the reveal an edit
 *       replayed to the server is refused;
 *   (2) anonymous: the "What didn't go to plan" group (outerHTML, every
 *       data-, aria- and title attribute), its part of the poll JSON and of
 *       the hydration payload, and a proposal made from one of its cards
 *       carry none of the five usernames or user ids, in every context; the
 *       author's export still lists the card;
 *   (3) decide ends with three action items owned by members' usernames;
 *       the generated review keeps "What went well" and "To improve" empty
 *       and saving it again creates no action item;
 *   (4) the generated review shows one voices line, "4 of 5 voices" (one
 *       member passed everything); the quiet nudge is a count, absent with
 *       two closed sessions, shown to the facilitator of game 4 only (after
 *       three), and "5 of 5 voices" when everyone writes in game 4.
 *
 * Frames: when $MISSION_FRAMES_DIR is set, PNGs (and the page HTML beside
 * each, scripts stripped, for rubric-check.mjs) of the thread's review
 * step, the silent round (phone and desktop), the facilitator view, the
 * reveal (plus a paused-transition sequence and reduced motion), decide
 * and the generated review.
 */
import fs from "node:fs";
import path from "node:path";
import { test, expect, type Browser, type BrowserContext, type Page } from "@playwright/test";
import { INVITE_CODE, TEST_PASSWORD, createTeam, waitForHome } from "./fixtures";

const DIR = process.env.MISSION_FRAMES_DIR;
const BASE = process.env.BASE_URL ?? "http://127.0.0.1:3020";
const SHORT = `${Date.now() % 1_000_000_000}${process.env.TEST_WORKER_INDEX ?? "0"}`;
const PROMPTS = ["decided", "to_plan", "off_plan", "next_time"] as const;
type PromptKey = (typeof PROMPTS)[number];
const WHO = ["a", "b", "c", "d", "e"] as const;
type Who = (typeof WHO)[number];

const PLACEHOLDER_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
  "base64"
);

/** A marker no other member's text contains. */
function mark(who: Who, p: PromptKey): string {
  return `mk${who}${SHORT}${p.replace("_", "")}`;
}

/** The words each member writes (b and d write alike for next time: a merge). */
const WORDS: Record<PromptKey, string> = {
  decided: "the call at the objective decided it",
  to_plan: "lane assignments held as drafted",
  off_plan: "rotations came late and nobody followed up",
  next_time: "ward river dragon pit earlier tempo",
};
function text(who: Who, p: PromptKey): string {
  const words: Record<Who, Record<PromptKey, string>> = {
    a: { ...WORDS, decided: "baron throw at twenty eight", to_plan: "level one invade worked", off_plan: "jungle pathing gave away first drake", next_time: "reset before soul point" },
    b: { ...WORDS, decided: "bot dive at six minutes", to_plan: "early wave control held", off_plan: "support roams left the carry alone" },
    c: { ...WORDS, decided: "mid priority lost at fifteen", to_plan: "vision denial in top side", off_plan: "calls overlapped in fights", next_time: "single shot caller in teamfights" },
    d: { ...WORDS, decided: "elder steal swung everything", to_plan: "teleport flank landed", off_plan: "split pushes without cover" },
    e: { ...WORDS },
  };
  return `${words[who][p]} ${mark(who, p)}`;
}

type Member = {
  who: Who;
  ctx: BrowserContext;
  page: Page;
  username: string;
  email: string;
  userKey: string;
  /** Every /api/ response body and every document this context loaded. */
  log: { url: string; body: string }[];
  errors: string[];
};

async function newMember(browser: Browser, who: Who, width: number): Promise<Member> {
  const height = width < 500 ? 844 : 900;
  const ctx = await browser.newContext({ viewport: { width, height } });
  await ctx.route(/\/img\/(champion|item)\//, (route) =>
    route.fulfill({ status: 200, contentType: "image/png", body: PLACEHOLDER_PNG })
  );
  const page = await ctx.newPage();
  const m: Member = {
    who,
    ctx,
    page,
    username: `rtu_${SHORT}_${who}`,
    email: `rtu_${SHORT}_${who}@test.invalid`,
    userKey: "",
    log: [],
    errors: [],
  };
  page.on("pageerror", (e) => m.errors.push(e.message));
  page.on("response", async (resp) => {
    const url = resp.url();
    const doc = resp.request().resourceType() === "document";
    if (!url.includes("/api/") && !doc) return;
    try {
      m.log.push({ url, body: await resp.text() });
    } catch {
      // A redirect or a body that went with a navigation: nothing to read.
    }
  });
  return m;
}

async function register(m: Member, link: string) {
  const { page } = m;
  await page.goto(link);
  await page.fill("input[name=username]", m.username);
  await page.fill("input[name=email]", m.email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.check("#register-terms");
  await page.click("main button[type=submit]");
  await waitForHome(page, 30000);
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);
}

/** The user's key (the part after `user:`) from their own export. */
async function exportOf(m: Member): Promise<any> {
  const resp = await m.page.request.get("/account/export");
  expect(resp.ok(), `export for ${m.who}`).toBe(true);
  return resp.json();
}
function userKeyOf(exp: any): string {
  const acc = Array.isArray(exp?.data?.account) ? exp.data.account[0] : exp?.data?.account;
  const raw = typeof acc?.id === "string" ? acc.id : JSON.stringify(acc?.id ?? "");
  const key = raw.replace(/^user:/, "").match(/[A-Za-z0-9_]{8,}/)?.[0] ?? "";
  expect(key.length, `a user id in the export: ${raw}`).toBeGreaterThanOrEqual(8);
  return key;
}

async function go(page: Page, route: string) {
  await page.goto(route);
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(500);
}

async function clearToasts(page: Page) {
  const region = page.locator('section[aria-label="Notifications"]');
  const dismiss = region.getByRole("button", { name: "Dismiss" });
  for (let i = 0; i < 8 && (await dismiss.count()) > 0; i++) {
    await dismiss.first().click({ timeout: 2000 }).catch(() => {});
  }
  await expect(region.locator("li")).toHaveCount(0, { timeout: 8000 });
}

/** A frame (and its HTML) when $MISSION_FRAMES_DIR is set; the page never scrolls sideways. */
async function shot(page: Page, name: string, fullPage = false) {
  await clearToasts(page);
  const sw = await page.evaluate(() => document.documentElement.scrollWidth);
  const vw = page.viewportSize()?.width ?? 0;
  expect(sw, `${name}: ${sw}px wide in a ${vw}px viewport`).toBeLessThanOrEqual(vw);
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.evaluate(() => window.scrollTo(0, 0)).catch(() => {});
  await page.waitForTimeout(200);
  await page.screenshot({ path: path.join(DIR, `${name}.png`), fullPage });
  await saveHtml(page, name);
}

async function saveHtml(page: Page, name: string) {
  if (!DIR) return;
  const html = (await page.content())
    .replace(/<script\b[\s\S]*?<\/script>/gi, "")
    .replace(/<head([^>]*)>/i, `<head$1><base href="${BASE.replace(/\/?$/, "/")}">`);
  fs.writeFileSync(path.join(DIR, `${name}.html`), html);
}

/** A frame without the toast and overflow checks (the reveal sequence). */
async function rawShot(page: Page, name: string) {
  if (!DIR) return;
  fs.mkdirSync(DIR, { recursive: true });
  await page.screenshot({ path: path.join(DIR, `${name}.png`) });
}

/** Local date and time `hours` from now, as the match editor takes them. */
async function localInputs(page: Page, hours: number): Promise<{ date: string; time: string }> {
  return page.evaluate((h) => {
    const d = new Date(Date.now() + h * 3600 * 1000);
    const pad = (n: number) => String(n).padStart(2, "0");
    return {
      date: `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`,
      time: `${pad(d.getHours())}:${pad(d.getMinutes())}`,
    };
  }, hours);
}

function section(page: Page, p: PromptKey) {
  return page.locator(`section[data-prompt="${p}"]`);
}

/** Write one note under a prompt (autosave on blur) and wait for the count. */
async function writeNote(m: Member, p: PromptKey, words: string, answered: number) {
  const box = section(m.page, p).locator("[data-entry] textarea").first();
  await expect(box).toBeVisible({ timeout: 15000 });
  await box.fill(words);
  await box.press("Tab");
  await expect(m.page.locator("[data-answered]")).toHaveText(`${answered} of 4 answered`, { timeout: 15000 });
}

async function openSession(m: Member, route: string): Promise<string> {
  await go(m.page, route);
  await expect
    .poll(() => new URL(m.page.url()).searchParams.get("session") ?? "", { timeout: 20000 })
    .toMatch(/^review_session:/);
  await expect(m.page.locator("[data-session-side]")).toBeVisible({ timeout: 15000 });
  return new URL(m.page.url()).searchParams.get("session") ?? "";
}

async function revealNow(page: Page) {
  const tools = page.locator("[data-facilitator-tools]");
  if (!(await page.locator("#round-controls").isVisible().catch(() => false))) {
    await tools.getByRole("button", { name: "Round controls" }).click();
  }
  await page.getByTestId("reveal-now").click();
  const dialog = page.locator("#confirm-reveal");
  await expect(dialog).toBeVisible();
  await dialog.locator('[data-confirm="confirm"]').click();
  await expect(page.locator("[data-reveal]")).toBeVisible({ timeout: 20000 });
}

function noteRow(page: Page, p: PromptKey, marker: string) {
  return page.locator(`section[data-note-group="${p}"] li`).filter({ hasText: marker }).first();
}

async function propose(page: Page, p: PromptKey, marker: string, words: string) {
  const row = noteRow(page, p, marker);
  await row.getByTestId("propose").click();
  const form = row.locator("[data-propose-form]");
  await expect(form).toBeVisible();
  await form.locator("textarea").fill(words);
  await form.getByTestId("add-proposal").click();
  await expect(page.locator("li[data-proposal-id]").filter({ hasText: words })).toHaveCount(1, { timeout: 15000 });
}

function proposal(page: Page, words: string) {
  return page.locator("li[data-proposal-id]").filter({ hasText: words });
}

async function closeSession(page: Page) {
  const close = page.getByTestId("close-session");
  await expect(close).toBeEnabled({ timeout: 15000 });
  await close.click();
  const dialog = page.locator("#confirm-close");
  await expect(dialog).toBeVisible();
  await dialog.locator('[data-confirm="confirm"]').click();
  await page.waitForURL(/\/post-game\?review_id=post_game_learning/, { timeout: 20000 });
  await page.waitForLoadState("networkidle").catch(() => {});
}

/** One quick session of a game, the facilitator alone: one signed note, reveal, one owned item, close. */
async function quickSession(lead: Member, matchId: string, game: number, owner: string): Promise<string> {
  const route = `/review/together?match_id=${encodeURIComponent(matchId)}&game=${game}`;
  const sid = await openSession(lead, route);
  await writeNote(lead, "decided", `game ${game} call ${mark("a", "decided")}g${game}`, 1);
  await revealNow(lead.page);
  const words = `Drill game ${game} setups g${game}${SHORT}`;
  await propose(lead.page, "decided", `${mark("a", "decided")}g${game}`, words);
  const row = proposal(lead.page, words);
  await row.getByRole("checkbox", { name: "Accept" }).check();
  await row.locator("[data-owner-chips]").getByRole("button", { name: owner, exact: true }).click();
  await closeSession(lead.page);
  return sid;
}

/** Every value under a key named `key`, anywhere in a JSON value. */
function collect(v: any, key: string, out: any[] = []): any[] {
  if (Array.isArray(v)) v.forEach((x) => collect(x, key, out));
  else if (v && typeof v === "object") {
    for (const [k, x] of Object.entries(v)) {
      if (k === key) out.push(x);
      collect(x, key, out);
    }
  }
  return out;
}
function parseJson(s: string): any {
  try {
    return JSON.parse(s);
  } catch {
    return undefined;
  }
}

/** The page's serialized resources (the hydration payload), parsed. */
async function hydration(page: Page): Promise<any[]> {
  const raw: string[] = await page.evaluate(() => {
    const r = (window as any).__RESOLVED_RESOURCES;
    return Array.isArray(r) ? r.filter((x: unknown) => typeof x === "string") : [];
  });
  return raw.map(parseJson).filter((x) => x !== undefined);
}

/** The poll bodies (`get_session_state`) logged since `from`, parsed. */
function polls(m: Member, from = 0): any[] {
  return m.log
    .slice(from)
    .filter((l) => l.url.includes("/api/get_session_state"))
    .map((l) => parseJson(l.body))
    .filter((x) => x !== undefined);
}

test("review together: sealed, anonymous, decided and counted by five members", async ({ browser }) => {
  test.setTimeout(25 * 60_000);
  const members: Record<Who, Member> = {
    a: await newMember(browser, "a", 1280),
    b: await newMember(browser, "b", 390),
    c: await newMember(browser, "c", 1280),
    d: await newMember(browser, "d", 1280),
    e: await newMember(browser, "e", 1280),
  };
  const all = WHO.map((w) => members[w]);
  const { a: lead, b: phone, c: author } = members;

  // --- Five members: the leader's team, four join with its invite link.
  await register(lead, `/auth/register?invite=${INVITE_CODE}`);
  await createTeam(lead.page, `Together_${SHORT}`);
  await go(lead.page, "/team/dashboard");
  const invites = lead.page.locator("[data-team-invites]");
  await expect(invites.getByRole("heading", { name: "Invite a player" })).toBeVisible({ timeout: 15000 });
  await invites.locator("#team-invite-uses").fill("4");
  await invites.locator("#team-invite-days").fill("7");
  await invites.getByTestId("mint-team-invite").click();
  const code = invites.getByTestId("team-invite-link").locator("code");
  await expect(code).toContainText("/auth/register?invite=", { timeout: 15000 });
  const link = ((await code.textContent()) ?? "").trim();
  for (const m of all.slice(1)) await register(m, link);
  for (const m of all) m.userKey = userKeyOf(await exportOf(m));
  const names = all.map((m) => m.username);
  const ids = all.map((m) => m.userKey);
  const identities = [...names, ...ids, `rtu_${SHORT}`];
  const leaks = (s: string) => identities.filter((x) => s.includes(x));

  // --- The match: an official best of 5 that started an hour ago.
  await go(lead.page, "/");
  await lead.page.getByTestId("add-next-match").click();
  const editor = lead.page.getByRole("dialog", { name: "Add the next match" });
  await expect(editor).toBeVisible();
  await editor.getByRole("radiogroup", { name: "Kind of match" }).getByRole("radio", { name: "Official match" }).click();
  await editor.getByRole("radiogroup", { name: "Format" }).getByRole("radio", { name: "Best of 5" }).click();
  const at = await localInputs(lead.page, -1);
  await editor.locator("#match-date").fill(at.date);
  await editor.locator("#match-time").fill(at.time);
  await lead.page.getByTestId("save-next-match").click();
  await expect(editor).not.toBeVisible({ timeout: 10000 });

  // The thread's review step: "Review together" is the action, writing alone the quiet link.
  const together = lead.page.locator('[data-prep] a[href^="/review/together?"]');
  await expect(together).toBeVisible({ timeout: 15000 });
  await expect(together).toContainText("Review together");
  await expect(lead.page.locator("[data-prep] [data-prep-alt]")).toHaveText("Write it alone");
  const href = (await together.getAttribute("href")) ?? "";
  expect(href).toMatch(/^\/review\/together\?match_id=scheduled_match:[^&]+.*game=1/);
  const matchId = new URL(href, "http://local").searchParams.get("match_id") ?? "";
  await lead.page.waitForTimeout(800);
  await shot(lead.page, "match-day-review-together-1280");
  await lead.page.setViewportSize({ width: 390, height: 844 });
  await lead.page.waitForTimeout(800);
  await shot(lead.page, "match-day-review-together-390");
  await lead.page.setViewportSize({ width: 1280, height: 900 });

  // --- Game 1: the leader opens it from the thread and facilitates.
  await together.click();
  await expect
    .poll(() => new URL(lead.page.url()).searchParams.get("session") ?? "", { timeout: 20000 })
    .toMatch(/^review_session:/);
  const sid = new URL(lead.page.url()).searchParams.get("session") ?? "";
  const sessionUrl = `/review/together?session=${encodeURIComponent(sid)}`;
  await expect(lead.page.locator("[data-written-count]")).toContainText("0 of 5", { timeout: 15000 });

  // The phone member writes alone, before any round (async).
  await openSession(phone, sessionUrl);
  await expect(phone.page.locator("[data-deadline]")).toContainText("Open until");
  for (const [i, p] of PROMPTS.entries()) await writeNote(phone, p, text("b", p), i + 1);
  await expect(lead.page.locator("[data-written-count]")).toContainText("1 of 5", { timeout: 15000 });
  await shot(phone.page, "review-together-writing-390");

  // The others open the session; presence shows them to the facilitator.
  for (const m of [members.c, members.d, members.e]) await openSession(m, sessionUrl);
  const here = lead.page.locator('[data-presence] ul[aria-label="Here now"] li');
  await expect.poll(() => here.count(), { timeout: 20000 }).toBeGreaterThanOrEqual(4);

  // The facilitator, collapsed (reading flow), then the round controls.
  await shot(lead.page, "review-together-facilitator-1280");
  await lead.page.locator("[data-facilitator-tools]").getByRole("button", { name: "Round controls" }).click();
  await expect(lead.page.locator("#round-controls")).toBeVisible();
  await lead.page.getByTestId("start-round").click();
  await expect(lead.page.locator("[data-deadline]")).toContainText("min left", { timeout: 15000 });
  await expect(lead.page.getByTestId("add-minute")).toBeVisible({ timeout: 15000 });
  await lead.page.getByTestId("add-minute").click();
  await expect(phone.page.locator("[data-deadline]")).toContainText("Closes", { timeout: 15000 });
  await shot(lead.page, "review-together-facilitator-1280-expanded");

  // Everyone writes at 1280 in the timed round; e passes every prompt.
  const capture: { req?: { url: string; headers: Record<string, string>; body: string } } = {};
  author.page.on("request", (r) => {
    if (r.url().includes("/api/upsert_entry") && r.method() === "POST") {
      capture.req = { url: r.url(), headers: r.headers(), body: r.postData() ?? "" };
    }
  });
  for (const m of [lead, members.c, members.d]) {
    for (const [i, p] of PROMPTS.entries()) await writeNote(m, p, text(m.who, p), i + 1);
  }
  await shot(members.c.page, "review-together-writing-1280");
  await shot(members.c.page, "review-together-writing-1280-flow", true);
  await expect(lead.page.locator("[data-written-count]")).toContainText("4 of 5", { timeout: 15000 });
  const quietOne = members.e;
  for (const [i, p] of PROMPTS.entries()) {
    await section(quietOne.page, p).getByRole("checkbox", { name: "Nothing to add here" }).check();
    await expect(quietOne.page.locator("[data-answered]")).toHaveText(`${i + 1} of 4 answered`, { timeout: 15000 });
  }
  await expect(lead.page.locator("[data-written-count]")).toContainText("5 of 5", { timeout: 15000 });
  expect(capture.req, "an upsert_entry request was seen").toBeTruthy();

  // --- (1) Sealed: reload every context (a fresh hydration payload), let
  // the poll run, then search everything each context received.
  for (const m of all) {
    await go(m.page, sessionUrl);
    await expect(m.page.locator("[data-session-side]")).toBeVisible({ timeout: 15000 });
  }
  await lead.page.waitForTimeout(4500);
  for (const m of all) {
    const corpus = m.log.map((l) => l.body).join("\n") + "\n" + (await m.page.content());
    for (const other of all) {
      for (const p of PROMPTS) {
        const marker = mark(other.who, p);
        if (other.who === m.who) {
          if (m.who !== "e") expect(corpus.includes(marker), `${m.who} receives its own ${p}`).toBe(true);
        } else {
          expect(corpus.includes(marker), `${m.who} must not receive ${other.who}'s ${p} before the reveal`).toBe(false);
        }
      }
    }
  }
  // The facilitator: one count over all four prompts, never per prompt.
  await expect(lead.page.locator("[data-written-count]")).toContainText("5 of 5");
  const leadMain = await lead.page.locator("main").innerText();
  expect(leadMain.match(/\b\d+ of 5\b/g) ?? []).toHaveLength(1);
  const leadStates = [...(await hydration(lead.page)), ...polls(lead)].flatMap((x) => collect(x, "written_count").length ? [x] : []);
  expect(leadStates.length).toBeGreaterThan(0);
  for (const s of leadStates) {
    const keys = JSON.stringify(s).match(/"[a-z_]*count[a-z_]*":/g) ?? [];
    expect([...new Set(keys)].sort()).toEqual(['"member_count":', '"written_count":']);
  }
  await shot(lead.page, "review-together-facilitator-sealed-1280");

  // --- The reveal: a paused-transition sequence at 1280 (d) and 390 (b),
  // reduced motion in e's context.
  const catcher = () => {
    (window as any).__rt = { caught: false };
    document.addEventListener(
      "transitionrun",
      (e) => {
        const el = e.target as Element;
        if (!(el instanceof Element) || !el.matches("[data-reveal]") || (window as any).__rt.caught) return;
        (window as any).__rt.caught = true;
        el.getAnimations().forEach((a) => a.pause());
      },
      true
    );
  };
  for (const m of [members.d, phone]) await m.page.evaluate(catcher);
  await quietOne.page.emulateMedia({ reducedMotion: "reduce" });
  await go(quietOne.page, sessionUrl);
  await rawShot(members.d.page, "review-together-reveal-seq-1280-before");
  await rawShot(phone.page, "review-together-reveal-seq-390-before");
  const marks = all.map((m) => m.log.length);
  await revealNow(lead.page);

  async function sequence(m: Member, width: number) {
    const caught = await m.page
      .waitForFunction(() => (window as any).__rt?.caught === true, null, { timeout: 15000 })
      .then(() => true)
      .catch(() => false);
    if (!caught) {
      test.info().annotations.push({ type: "reveal-sequence", description: `${m.who}: no transition caught` });
      return;
    }
    for (const t of [0, 50, 110, 170]) {
      const probe = await m.page.evaluate((ms) => {
        const w = document.querySelector("[data-reveal]") as HTMLElement;
        w.getAnimations().forEach((a) => (a.currentTime = ms));
        // Animations inside the wrapper (a stagger would be one per note).
        const anims = document.getAnimations().filter((a) => {
          const t = (a.effect as KeyframeEffect | null)?.target;
          return t instanceof Element && w.contains(t);
        });
        const notes = Array.from(document.querySelectorAll("[data-reveal] [data-note-id]")).map(
          (n) => getComputedStyle(n).opacity
        );
        return {
          wrapper: Number(getComputedStyle(w).opacity),
          others: anims.filter((a) => (a.effect as KeyframeEffect).target !== w).length,
          notes,
        };
      }, t);
      // One wrapper fades: no note has an opacity or animation of its own.
      expect(probe.others, "no animation besides the reveal wrapper").toBe(0);
      expect(probe.notes.every((o) => o === "1")).toBe(true);
      if (t === 110) {
        expect(probe.wrapper).toBeGreaterThan(0.05);
        expect(probe.wrapper).toBeLessThan(0.95);
      }
      await rawShot(m.page, `review-together-reveal-seq-${width}-${t}ms`);
    }
    await m.page.evaluate(() =>
      document.querySelector("[data-reveal]")?.getAnimations().forEach((a) => a.finish())
    );
    await m.page.waitForTimeout(100);
    await rawShot(m.page, `review-together-reveal-seq-${width}-after`);
  }
  await Promise.all([sequence(members.d, 1280), sequence(phone, 390)]);

  // Reduced motion: final on the first frame, no transition.
  await expect(quietOne.page.locator("[data-reveal]")).toBeVisible({ timeout: 15000 });
  const reduced = await quietOne.page.evaluate(() => {
    const w = document.querySelector("[data-reveal]") as HTMLElement;
    return { opacity: getComputedStyle(w).opacity, anims: w.getAnimations().length };
  });
  expect(reduced).toEqual({ opacity: "1", anims: 0 });
  await rawShot(quietOne.page, "review-together-reveal-reduced-1280");
  for (const m of all) await expect(m.page.locator("[data-reveal]")).toBeVisible({ timeout: 15000 });
  await expect(quietOne.page.locator("[data-reveal-status]")).toContainText("Revealed");

  // After the reveal an edit is refused by the server.
  const replay = await author.page.evaluate(async (req) => {
    const ct = req.headers["content-type"] ?? "application/x-www-form-urlencoded";
    let body = req.body;
    if (ct.includes("x-www-form-urlencoded")) {
      const form = new URLSearchParams(body);
      form.set("text", `EDITED after reveal ${Date.now()}`);
      body = form.toString();
    }
    const resp = await fetch(req.url, {
      method: "POST",
      headers: { "content-type": ct, accept: req.headers["accept"] ?? "application/json" },
      body,
    });
    return { status: resp.status, ok: resp.ok, text: await resp.text() };
  }, capture.req!);
  expect(replay.ok, `a late edit must be refused: ${replay.status} ${replay.text}`).toBe(false);
  expect(replay.text).toContain("revealed");
  await expect(author.page.locator("[data-entry] textarea")).toHaveCount(0);

  // --- (2) Anonymous: in every context, the facilitator's and the authors'.
  // The poll JSON first (each context got a full state at the reveal).
  for (const m of all) {
    await expect
      .poll(() => polls(m, marks[all.indexOf(m)]).flatMap((x) => collect(x, "off_plan")).length, {
        timeout: 20000,
        message: `${m.who} received the revealed state`,
      })
      .toBeGreaterThan(0);
  }
  for (const m of all) await go(m.page, sessionUrl);
  for (const m of all) {
    await expect(m.page.locator('section[data-note-group="off_plan"]')).toBeVisible({ timeout: 15000 });
    const group = await m.page.evaluate(() => {
      const sec = document.querySelector('section[data-note-group="off_plan"]') as HTMLElement;
      const attrs: string[] = [];
      for (const el of [sec, ...Array.from(sec.querySelectorAll("*"))]) {
        for (const at of Array.from(el.attributes)) {
          if (/^(data-|aria-)/.test(at.name) || at.name === "title") attrs.push(`${at.name}=${at.value}`);
        }
      }
      const shapes = Array.from(sec.querySelectorAll(":scope > ul > li")).map((li) =>
        Array.from(li.querySelectorAll("*"))
          .map((e) => `${e.tagName}.${e.getAttribute("class") ?? ""}`)
          .join(",")
      );
      return { html: sec.outerHTML, attrs: attrs.join("\n"), shapes, avatars: sec.querySelectorAll("img").length };
    });
    for (const w of ["a", "b", "c", "d"] as Who[]) expect(group.html).toContain(mark(w, "off_plan"));
    expect(leaks(group.html), `${m.who}: off_plan HTML`).toEqual([]);
    expect(leaks(group.attrs), `${m.who}: off_plan attributes`).toEqual([]);
    expect(group.avatars).toBe(0);
    expect(new Set(group.shapes).size, `${m.who}: one markup for every anonymous row`).toBe(1);
    const hyd = (await hydration(m.page)).flatMap((x) => collect(x, "off_plan"));
    expect(hyd.length, `${m.who}: off_plan in the hydration payload`).toBeGreaterThan(0);
    const fromPolls = polls(m, marks[all.indexOf(m)]).flatMap((x) => collect(x, "off_plan"));
    for (const [where, part] of [["hydration", hyd], ["poll", fromPolls]] as const) {
      const json = JSON.stringify(part);
      expect(json).toContain(mark("c", "off_plan"));
      expect(leaks(json), `${m.who}: off_plan in the ${where} payload`).toEqual([]);
      for (const card of part.flat()) {
        expect(Object.keys(card).sort(), `${m.who}: anonymous card fields (${where})`).toEqual(
          ["cluster", "i_plus_oned", "id", "plus_ones", "text"]
        );
      }
    }
  }
  // The author's export still lists the anonymous cards.
  for (const m of [lead, phone, author, members.d]) {
    const cards: any[] = (await exportOf(m)).data.review_cards_written ?? [];
    const card = cards.find((c) => String(c.text ?? "").includes(mark(m.who, "off_plan")));
    expect(card, `${m.who}'s export lists its anonymous card`).toBeTruthy();
    expect(card.anonymous).toBe(true);
  }

  // +1 (a minority card keeps 0), the facilitator merges b's and d's alike notes.
  await noteRow(phone.page, "decided", mark("c", "decided")).locator("button[data-plus-one]").click();
  await noteRow(members.d.page, "off_plan", mark("a", "off_plan")).locator("button[data-plus-one]").click();
  await noteRow(author.page, "off_plan", mark("a", "off_plan")).locator("button[data-plus-one]").click();
  await expect(noteRow(lead.page, "off_plan", mark("a", "off_plan")).locator("button[data-plus-one]")).toContainText(
    "Agree, 2 so far",
    { timeout: 15000 }
  );
  const alike = lead.page.locator('section[data-note-group="next_time"] [data-suggestions]');
  await expect(alike).toBeVisible({ timeout: 15000 });
  await alike.getByRole("button").first().click();
  await alike.getByRole("button", { name: "Merge" }).first().click();
  const cluster = lead.page.locator('section[data-note-group="next_time"] li').filter({ hasText: "2 said much the same" });
  await expect(cluster).toHaveCount(1, { timeout: 15000 });
  await expect(cluster).toContainText(mark("b", "next_time"));
  await expect(cluster).toContainText(mark("d", "next_time"));
  await expect(
    phone.page.locator('section[data-note-group="next_time"] li').filter({ hasText: "2 said much the same" })
  ).toHaveCount(1, { timeout: 15000 });

  // Proposals: one from an anonymous card (by its author), three more.
  const P = (n: number) => `Action ${n} p${n}x${SHORT}`;
  const beforeProposal = all.map((m) => m.log.length);
  await propose(author.page, "off_plan", mark("c", "off_plan"), P(1));
  await propose(phone.page, "decided", mark("a", "decided"), P(2));
  await propose(members.d.page, "next_time", mark("b", "next_time"), P(3));
  await propose(lead.page, "to_plan", mark("d", "to_plan"), P(4));
  // The proposal from the anonymous card names nobody, in every context.
  for (const m of all) {
    const row = proposal(m.page, P(1));
    await expect(row).toHaveCount(1, { timeout: 20000 });
    const html = await row.evaluate((el) => el.outerHTML);
    expect(leaks(html), `${m.who}: proposal row`).toEqual([]);
    await expect
      .poll(() => polls(m, beforeProposal[all.indexOf(m)]).filter((x) => JSON.stringify(x).includes(P(1))).length, {
        timeout: 20000,
      })
      .toBeGreaterThan(0);
    const views = polls(m, beforeProposal[all.indexOf(m)])
      .flatMap((x) => collect(x, "proposals"))
      .flat()
      .filter((p: any) => p?.text === P(1));
    expect(views.length).toBeGreaterThan(0);
    for (const v of views) {
      expect(leaks(JSON.stringify(v)), `${m.who}: proposal payload`).toEqual([]);
      expect(Object.keys(v).sort()).toEqual(["accepted", "i_voted", "id", "owner", "source", "suggested", "text", "votes"]);
    }
  }
  await shot(lead.page, "review-together-reveal-1280");
  await shot(lead.page, "review-together-reveal-1280-flow", true);
  await shot(phone.page, "review-together-reveal-390", true);

  // --- (3) Decide: three votes each, the cap, then three owned items.
  const vote = async (m: Member, n: number) => {
    const b = proposal(m.page, P(n)).getByTestId("decide-vote");
    await b.click();
    await expect(b).toHaveAttribute("aria-pressed", "true", { timeout: 15000 });
  };
  for (const n of [1, 2, 3]) await vote(lead, n);
  await expect(lead.page.locator("[data-votes-left]")).toHaveText("No votes left", { timeout: 15000 });
  await expect(proposal(lead.page, P(4)).getByTestId("decide-vote")).toBeDisabled();
  for (const n of [1, 2]) await vote(phone, n);
  for (const n of [1, 3]) await vote(author, n);
  await vote(members.d, 2);
  await vote(quietOne, 4);
  await lead.page.getByTestId("start-deciding").click();
  for (const n of [1, 2, 3]) {
    await expect(proposal(lead.page, P(n)).getByRole("checkbox", { name: "Accept" })).toBeChecked({ timeout: 15000 });
  }
  await expect(proposal(lead.page, P(4)).getByRole("checkbox", { name: "Accept" })).toBeDisabled();
  const owners: Record<number, Member> = { 1: phone, 2: author, 3: members.d };
  for (const n of [1, 2, 3]) {
    const chip = proposal(lead.page, P(n)).locator("[data-owner-chips]").getByRole("button", { name: owners[n].username, exact: true });
    await chip.click();
    await expect(chip).toHaveAttribute("aria-pressed", "true", { timeout: 15000 });
  }
  await expect(proposal(phone.page, P(1))).toContainText(`owner ${phone.username}`, { timeout: 15000 });
  await shot(lead.page, "review-together-decide-1280");
  await shot(lead.page, "review-together-decide-1280-flow", true);
  await shot(phone.page, "review-together-decide-390", true);

  // Action items before the close (e's view of the team's list).
  await go(quietOne.page, "/action-items");
  await quietOne.page.waitForTimeout(1000);
  const itemsBefore = await quietOne.page.locator("[data-action-item]").count();
  await closeSession(lead.page);
  const reviewUrl = lead.page.url();

  // --- (4) The generated review: one voices line, empty points, three items.
  const reviewPage = lead.page;
  await expect(reviewPage.locator("[data-reviewed-together]")).toBeVisible({ timeout: 15000 });
  await expect(reviewPage.locator("[data-voices]")).toHaveCount(1);
  await expect(reviewPage.locator("[data-voices]")).toHaveText("4 of 5 voices");
  const reviewText = await reviewPage.locator("main").innerText();
  expect(reviewText.match(/of 5 voices/g) ?? []).toHaveLength(1);
  expect(reviewText.match(/\b\d+ of \d+ voices\b/g) ?? []).toHaveLength(1);
  await expect(reviewPage.locator("ul[data-decided] > li")).toHaveCount(3);
  for (const n of [1, 2, 3]) {
    await expect(reviewPage.locator("ul[data-decided]")).toContainText(`${P(n)}`);
    await expect(reviewPage.locator("ul[data-decided]")).toContainText(`Owner: ${owners[n].username}`);
  }
  await expect(reviewPage.locator("#review-well")).toHaveValue("");
  await expect(reviewPage.locator("#review-improve")).toHaveValue("");
  await shot(reviewPage, "review-together-review-1280");
  await shot(reviewPage, "review-together-review-1280-flow", true);
  await reviewPage.setViewportSize({ width: 390, height: 844 });
  await reviewPage.waitForTimeout(800);
  await shot(reviewPage, "review-together-review-390");
  await reviewPage.setViewportSize({ width: 1280, height: 900 });

  await go(quietOne.page, "/action-items");
  await expect.poll(() => quietOne.page.locator("[data-action-item]").count(), { timeout: 15000 }).toBe(itemsBefore + 3);
  for (const n of [1, 2, 3]) {
    const item = quietOne.page.locator("[data-action-item]").filter({ hasText: P(n) });
    await expect(item).toHaveCount(1);
    await expect(item).toContainText(owners[n].username);
  }
  // Saving the review again in the editor creates no action item.
  await go(reviewPage, reviewUrl);
  await reviewPage.getByTestId("review-save").click();
  await expect(reviewPage.getByTestId("review-save-state")).toHaveText("Saved", { timeout: 15000 });
  await go(quietOne.page, "/action-items");
  await quietOne.page.waitForTimeout(1000);
  expect(await quietOne.page.locator("[data-action-item]").count()).toBe(itemsBefore + 3);
  await go(reviewPage, reviewUrl);
  await expect(reviewPage.locator("#review-well")).toHaveValue("");
  await expect(reviewPage.locator("#review-improve")).toHaveValue("");
  // The review is the thread's game 1.
  await go(reviewPage, "/");
  await expect(reviewPage.locator("[data-prep] li").nth(4)).toContainText("G1 done", { timeout: 15000 });

  // --- The quiet nudge: not with two closed sessions, then for the
  // facilitator only. e passed in game 1 and is absent in games 2 and 3.
  await quickSession(lead, matchId, 2, lead.username);
  const g3 = await openSession(lead, `/review/together?match_id=${encodeURIComponent(matchId)}&game=3`);
  expect(g3).not.toBe(sid);
  await lead.page.waitForTimeout(2500);
  await expect(lead.page.locator("[data-nudge]")).toHaveCount(0);
  for (const q of (await hydration(lead.page)).flatMap((x) => collect(x, "quiet"))) expect(q).toBeNull();
  await go(lead.page, "/");
  await quickSession(lead, matchId, 3, lead.username);

  const nudgeFrom = lead.log.length;
  const g4 = await openSession(lead, `/review/together?match_id=${encodeURIComponent(matchId)}&game=4`);
  const nudge = lead.page.locator('[data-nudge="nudge.quiet_player"]');
  await expect(nudge).toBeVisible({ timeout: 15000 });
  await expect(nudge).toContainText("One player has been quiet in the last 3 reviews.");
  expect(leaks(await nudge.evaluate((el) => el.outerHTML))).toEqual([]);
  const quietVals = [...(await hydration(lead.page)), ...polls(lead, nudgeFrom)].flatMap((x) => collect(x, "quiet"));
  expect(quietVals.length).toBeGreaterThan(0);
  for (const q of quietVals) expect(q).toBe(1);
  await shot(lead.page, "review-together-nudge-1280");
  // A member at 1280 sees no nudge, and its payload has none.
  await phone.page.setViewportSize({ width: 1280, height: 900 });
  const g4Url = `/review/together?session=${encodeURIComponent(g4)}`;
  const memberFrom = phone.log.length;
  await openSession(phone, g4Url);
  await phone.page.waitForTimeout(4500);
  await expect(phone.page.locator("[data-nudge]")).toHaveCount(0);
  const memberQuiet = [...(await hydration(phone.page)), ...polls(phone, memberFrom)].flatMap((x) => collect(x, "quiet"));
  expect(memberQuiet.length).toBeGreaterThan(0);
  for (const q of memberQuiet) expect(q).toBeNull();
  // Muting hides it, also after a reload.
  await lead.page.getByTestId("mute-nudge").click();
  await expect(nudge).toHaveCount(0);
  await lead.page.waitForTimeout(1000);
  await go(lead.page, g4Url);
  await expect(lead.page.locator("[data-session-side]")).toBeVisible({ timeout: 15000 });
  await lead.page.waitForTimeout(2500);
  await expect(nudge).toHaveCount(0);

  // Game 4 with everyone writing: "5 of 5 voices".
  await writeNote(lead, "decided", `all five ${mark("a", "decided")}g4`, 1);
  for (const m of [phone, author, members.d, quietOne]) {
    if (m !== phone) await openSession(m, g4Url);
    await writeNote(m, "decided", `all five ${mark(m.who, "decided")}g4`, 1);
  }
  await expect(lead.page.locator("[data-written-count]")).toContainText("5 of 5", { timeout: 15000 });
  await revealNow(lead.page);
  const words = `Keep all five talking g4x${SHORT}`;
  await propose(lead.page, "decided", `${mark("e", "decided")}g4`, words);
  const row = proposal(lead.page, words);
  await row.getByRole("checkbox", { name: "Accept" }).check();
  await row.locator("[data-owner-chips]").getByRole("button", { name: quietOne.username, exact: true }).click();
  await closeSession(lead.page);
  await expect(lead.page.locator("[data-voices]")).toHaveCount(1);
  await expect(lead.page.locator("[data-voices]")).toHaveText("5 of 5 voices");

  for (const m of all) expect(m.errors, `${m.who}: no page errors`).toEqual([]);
  for (const m of all) await m.ctx.close();
});
