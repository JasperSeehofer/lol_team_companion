/**
 * Interface icons (M152): role, objective and structure icons come from the
 * app's own `/img/ui/{kind}/{name}` route, a fixed allow-list with pinned,
 * committed copies. The browser never requests raw.communitydragon.org (or
 * any other origin), the route refuses anything off the list, and SVGs are
 * sent with nosniff and a sandboxing CSP.
 */
import { test, expect } from "./fixtures";

const ROLE_ICONS = ["top", "jungle", "middle", "bottom", "utility"];
const PNG_ICONS = [
  "objective/baron",
  "objective/riftherald",
  "objective/grub",
  "objective/dragon",
  "objective/dragon_elder",
  "objective/atakhan_v",
  "structure/tower",
  "structure/inhibitor",
  "structure/nexus",
];

test("every allowed icon is served same-origin with its headers", async ({ request }) => {
  for (const name of ROLE_ICONS) {
    const r = await request.get(`/img/ui/position/${name}`);
    expect(r.status(), name).toBe(200);
    expect(r.headers()["content-type"]).toBe("image/svg+xml");
    expect(r.headers()["x-content-type-options"]).toBe("nosniff");
    expect(r.headers()["content-security-policy"]).toBe(
      "default-src 'none'; style-src 'unsafe-inline'; sandbox"
    );
    expect(r.headers()["cache-control"]).toBe("public, max-age=604800");
    const body = (await r.text()).toLowerCase();
    expect(body.startsWith("<svg")).toBe(true);
    expect(body).not.toContain("<script");
    expect(body).not.toContain("href");
    expect(body).toContain("currentcolor");
  }
  for (const path of PNG_ICONS) {
    const r = await request.get(`/img/ui/${path}`);
    expect(r.status(), path).toBe(200);
    expect(r.headers()["content-type"]).toBe("image/png");
    expect(r.headers()["x-content-type-options"]).toBe("nosniff");
    const body = await r.body();
    expect(body.subarray(0, 4).toString("latin1")).toBe("\u0089PNG");
  }
});

test("the ETag answers 304", async ({ request }) => {
  const first = await request.get("/img/ui/structure/nexus");
  const etag = first.headers()["etag"];
  expect(etag).toBeTruthy();
  const again = await request.get("/img/ui/structure/nexus", {
    headers: { "If-None-Match": etag },
  });
  expect(again.status()).toBe(304);
});

test("anything off the allow-list is 404", async ({ request }) => {
  for (const path of [
    "/img/ui/position/support",
    "/img/ui/position/top.svg",
    "/img/ui/position/..%2ftop",
    "/img/ui/position/..%2F..%2FCargo.toml",
    "/img/ui/position/%2e%2e%2fCargo.toml",
    "/img/ui/champion/baron",
    "/img/ui/objective/map11",
  ]) {
    const r = await request.get(path, { maxRedirects: 0 });
    expect(r.status(), path).toBe(404);
  }
});

test("role icons on the gallery load from /img/ui/ and nothing reaches a third party", async ({
  adminPage,
}) => {
  const page = adminPage;
  const base = new URL(process.env.BASE_URL ?? "http://127.0.0.1:3020").origin;
  const foreign: string[] = [];
  const iconStatus = new Map<string, number>();
  page.on("request", (r) => {
    const u = r.url();
    if (!u.startsWith(base) && !u.startsWith("data:")) foreign.push(u);
  });
  page.on("response", (r) => {
    const u = new URL(r.url());
    if (u.pathname.startsWith("/img/ui/position/")) iconStatus.set(u.pathname, r.status());
  });
  const resp = await page.goto("/design#markers", { waitUntil: "networkidle" });
  test.skip(resp?.status() === 404, "server not started with ADMIN_EMAILS for the gallery");

  const section = page.locator('[data-gallery="markers"]');
  await section.scrollIntoViewIfNeeded();
  const icons = section.locator("[data-role-icon]");
  await expect(icons.first()).toBeVisible();
  // Every RoleLabel carries an aria-hidden icon and keeps its spoken word.
  const count = await icons.count();
  expect(count).toBeGreaterThanOrEqual(5);
  for (let i = 0; i < count; i++) {
    await expect(icons.nth(i)).toHaveAttribute("aria-hidden", "true");
  }
  await expect(section.locator('[data-role="Jungle"] .sr-only').first()).toHaveText("Jungle");
  // The guessed state keeps its dashed underline and "?".
  const guessed = section.locator('[data-role-guessed="true"]').first();
  await expect(guessed).toContainText("?");
  await expect(guessed.locator(".sr-only")).toHaveText("Jungle, role guessed");

  // The mask points at the app's own route and the browser loaded each icon.
  const masks = await icons.evaluateAll((els) =>
    els.map((el) => getComputedStyle(el).maskImage || getComputedStyle(el).webkitMaskImage)
  );
  for (const m of masks) {
    expect(m).toMatch(/url\("?[^"]*\/img\/ui\/position\/(top|jungle|middle|bottom|utility)"?\)/);
    expect(m).not.toContain("communitydragon");
  }
  await expect
    .poll(() => ROLE_ICONS.filter((n) => iconStatus.get(`/img/ui/position/${n}`) === 200).length, {
      timeout: 10000,
    })
    .toBe(ROLE_ICONS.length);

  // The icon is never the accent: on an Active label the letters are chalk
  // and the icon keeps the secondary ink (the same as a Boxed label's icon).
  const ink = await section.locator('[data-role="Support"]').evaluateAll((els) =>
    els.map((el) => {
      const icon = el.querySelector("[data-role-icon]") as HTMLElement;
      return {
        label: getComputedStyle(el).color,
        icon: getComputedStyle(icon).backgroundColor,
        active: el.className.includes("text-accent"),
        boxed: el.className.includes("border-outline"),
      };
    })
  );
  const active = ink.find((i) => i.active);
  const boxed = ink.find((i) => i.boxed);
  expect(active).toBeTruthy();
  expect(boxed).toBeTruthy();
  expect(active!.icon).not.toBe(active!.label);
  expect(active!.icon).toBe(boxed!.icon);

  expect(foreign.filter((u) => u.includes("communitydragon.org"))).toEqual([]);
  expect(foreign).toEqual([]);
});
