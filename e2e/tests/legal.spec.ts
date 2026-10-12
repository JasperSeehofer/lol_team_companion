/**
 * M8 legal pages and the Riot verification file: Impressum, privacy and
 * terms render for visitors with the Riot disclaimer and clearly marked
 * owner placeholders; /riot.txt is served as plain text.
 */
import { test, expect } from "@playwright/test";

const SHOTS = process.env.M8_SHOTS_DIR;

for (const { path, heading } of [
  { path: "/legal/impressum", heading: "Impressum" },
  { path: "/legal/datenschutz", heading: "Privacy" },
  { path: "/legal/terms", heading: "Terms of use" },
]) {
  test(`${path} renders for visitors`, async ({ page }) => {
    const res = await page.goto(path);
    expect(res?.status()).toBe(200);
    await expect(page.locator("h1")).toHaveText(heading);
    // Facts only the operator knows are placeholders, never invented.
    await expect(page.locator("[data-owner-placeholder]").first()).toBeVisible();
    await expect(page.getByTestId("riot-disclaimer")).toContainText("isn't endorsed by Riot Games");
    // The footer links all three pages.
    const legal = page.getByRole("navigation", { name: "Legal" });
    for (const label of ["Impressum", "Datenschutz", "Terms"]) {
      await expect(legal.getByRole("link", { name: label })).toBeVisible();
    }
    if (SHOTS) await page.screenshot({ path: `${SHOTS}/legal${path.replace(/\//g, "-")}.png`, fullPage: true });
  });
}

test("privacy page names cookies, logs, retention, rights and third parties", async ({ page }) => {
  await page.goto("/legal/datenschutz");
  for (const section of [
    "Who is responsible",
    "What Companion stores",
    "Why, and on what legal basis",
    "Who else is involved",
    "Cookies",
    "Server logs",
    "How long data is kept",
    "Your rights",
  ]) {
    await expect(page.getByRole("heading", { name: section, exact: true })).toBeVisible();
  }
  const main = page.locator("main");
  await expect(main).toContainText("Riot Games");
  await expect(main).toContainText("Backups: 30 days");
  await expect(main).toContainText("No analytics");
  await expect(main.getByRole("link", { name: "Download my data" })).toHaveAttribute("href", "/profile#your-data");
});

test("/riot.txt is served as plain text", async ({ request }) => {
  const res = await request.get("/riot.txt");
  expect(res.status()).toBe(200);
  expect(res.headers()["content-type"]).toContain("text/plain");
  expect((await res.text()).trim().length).toBeGreaterThan(0);
});
