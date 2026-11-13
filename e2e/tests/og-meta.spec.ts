/**
 * Link previews (M152 item 5): Open Graph tags in the server-rendered HTML,
 * the HTML crawlers and chat apps read (request.get, never the hydrated DOM).
 *
 * The share card is static: a valid, an unknown and a revoked token get
 * byte-identical og/twitter tags, without the token or the shared item's name.
 */
import type { APIRequestContext } from "@playwright/test";
import { test, expect } from "./fixtures";
import { navigateTo } from "./helpers";

const BASE = process.env.BASE_URL ?? "http://127.0.0.1:3020";

/** Every og:* and twitter:* meta tag of the page's head, in document order. */
async function ogBlock(request: APIRequestContext, path: string): Promise<string[]> {
  const res = await request.get(path);
  expect(res.status(), path).toBeLessThan(500);
  const html = await res.text();
  const head = html.split("</head>")[0];
  return head.match(/<meta\b[^>]*(?:property="og:[^"]*"|name="twitter:[^"]*")[^>]*>/g) ?? [];
}

function content(block: string[], key: string): string | undefined {
  const tag = block.find((t) => t.includes(`property="${key}"`) || t.includes(`name="${key}"`));
  return tag?.match(/content="([^"]*)"/)?.[1];
}

/** og:image is absolute on this host and serves a PNG. */
async function checkImage(request: APIRequestContext, block: string[], file: string) {
  const image = content(block, "og:image") ?? "";
  const url = new URL(image);
  expect(url.host).toBe(new URL(BASE).host);
  expect(url.pathname).toBe(`/og/${file}`);
  const res = await request.get(image);
  expect(res.status()).toBe(200);
  expect(res.headers()["content-type"]).toContain("image/png");
}

function checkCommon(block: string[]) {
  expect(content(block, "og:type")).toBe("website");
  expect(content(block, "twitter:card")).toBe("summary_large_image");
  expect(content(block, "og:title")).toBeTruthy();
  expect(content(block, "og:description")).toBeTruthy();
  // One card per page: leptos_meta does not dedupe, so a page adding its own
  // tags would show up as a second og:title.
  expect(block.filter((t) => t.includes('property="og:title"'))).toHaveLength(1);
}

test("landing and closed beta: the landing card, absolute same-host image", async ({ request }) => {
  for (const path of ["/", "/closed-beta"]) {
    const block = await ogBlock(request, path);
    checkCommon(block);
    expect(content(block, "og:url")).toBe(new URL(path, BASE).href);
    await checkImage(request, block, "landing.png");
  }
});

test("other pages: the default card", async ({ request }) => {
  const block = await ogBlock(request, "/auth/login");
  checkCommon(block);
  expect(content(block, "og:url")).toBeUndefined();
  await checkImage(request, block, "default.png");
});

test("shell: theme colour and favicon", async ({ request }) => {
  const html = await (await request.get("/")).text();
  const head = html.split("</head>")[0];
  expect(head).toMatch(/<meta name="theme-color" content="#081415"/);
  expect(head).toMatch(/<link rel="icon" href="\/favicon.svg" type="image\/svg\+xml"/);
  const res = await request.get("/favicon.svg");
  expect(res.status()).toBe(200);
  expect(res.headers()["content-type"]).toContain("image/svg+xml");
});

test("share links: identical card for a valid, an unknown and a revoked token", async ({
  teamPage,
  request,
}) => {
  const page = teamPage;
  const plan = `OG plan ${Date.now()}`;
  await navigateTo(page, "/game-plan");
  await page.locator("#plan-name").fill(plan);
  await page.getByTestId("plan-save").click();
  await expect(page.getByTestId("plan-save-state")).toHaveText("Saved", { timeout: 10000 });
  await page.getByTestId("plan-share").click();
  const dialog = page.getByRole("dialog", { name: /Share this game plan/ });
  await dialog.getByTestId("create-share-link").click();
  const url = (await dialog.locator("[data-share-url]").first().textContent({ timeout: 10000 }))?.trim() ?? "";
  expect(url).toMatch(/\/s\/[a-z2-9]{32}$/);
  const path = new URL(url).pathname;
  const token = path.split("/").pop() ?? "";

  // `request` has no cookies: what a crawler or chat app sees.
  const valid = await ogBlock(request, path);
  // The token really opens the plan for a guest (so this is the valid case).
  expect(await (await request.get(path)).text()).toContain(plan);
  const unknown = await ogBlock(request, "/s/abcdefghijkmnpqrstuvwxyz23456789");

  await dialog.getByRole("button", { name: "Revoke" }).first().click();
  await expect(dialog.locator("[data-share-url]")).toHaveCount(0, { timeout: 10000 });
  const revoked = await ogBlock(request, path);
  expect(await (await request.get(path)).text()).not.toContain(plan);

  checkCommon(valid);
  expect(valid.join("\n")).toBe(unknown.join("\n"));
  expect(valid.join("\n")).toBe(revoked.join("\n"));
  const joined = valid.join("\n");
  expect(joined).not.toContain(token);
  expect(joined).not.toContain(plan);
  expect(joined).not.toContain("/s/");
  expect(content(valid, "og:url")).toBeUndefined();
  await checkImage(request, valid, "share.png");
});
