/**
 * Legal pages and the Riot verification file: Impressum, privacy and terms
 * render for visitors with the Riot disclaimer and the operator's facts
 * (owner's launch facts, 12 Oct 2026); no placeholder is left. The privacy
 * page's legal bases and the terms' liability and law clauses are drafts
 * pending the operator's adviser (the privacy callout says so). /riot.txt is
 * served as plain text.
 */
import { test, expect } from "@playwright/test";

const SHOTS = process.env.LEGAL_SHOTS_DIR ?? process.env.M8_SHOTS_DIR;

for (const { path, heading, placeholders } of [
  { path: "/legal/impressum", heading: "Impressum", placeholders: 0 },
  { path: "/legal/datenschutz", heading: "Privacy", placeholders: 0 },
  { path: "/legal/terms", heading: "Terms of use", placeholders: 0 },
]) {
  test(`${path} renders for visitors`, async ({ page }) => {
    const res = await page.goto(path);
    expect(res?.status()).toBe(200);
    await expect(page.locator("h1")).toHaveText(heading);
    // Facts only the operator knows are never invented: what is still open
    // is a visible placeholder, everything else is filled in.
    await expect(page.locator("[data-owner-placeholder]")).toHaveCount(placeholders);
    await expect(page.getByTestId("riot-disclaimer")).toContainText("Team Companion isn't endorsed by Riot Games");
    // The footer links all three pages.
    const legal = page.getByRole("navigation", { name: "Legal" });
    for (const label of ["Impressum", "Datenschutz", "Terms"]) {
      await expect(legal.getByRole("link", { name: label })).toBeVisible();
    }
    if (SHOTS) {
      for (const width of [1280, 400]) {
        await page.setViewportSize({ width, height: 900 });
        await page.screenshot({ path: `${SHOTS}/legal${path.replace(/\//g, "-")}-${width}.png`, fullPage: true });
      }
    }
  });
}

test("the Impressum names the operator", async ({ page }) => {
  await page.goto("/legal/impressum");
  const main = page.locator("main");
  await expect(main).toContainText("Jasper Seehofer");
  await expect(main).toContainText("Augustenstrasse 4");
  await expect(main).toContainText("70178 Stuttgart");
  await expect(main.getByRole("link", { name: "jasper@jseehofer.eu" })).toHaveAttribute("href", "mailto:jasper@jseehofer.eu");
  await expect(main.getByText("Not filled in yet")).toHaveCount(0);
});

test("the terms state the price, liability and law", async ({ page }) => {
  await page.goto("/legal/terms");
  const main = page.locator("main");
  await expect(main).toContainText("Using Team Companion costs nothing; support is voluntary.");
  await expect(main).toContainText("intent and gross negligence");
  await expect(main).toContainText("law of the Federal Republic of Germany");
  await expect(main.getByText("Partly a draft")).toHaveCount(0);
});

test("privacy page names cookies, logs, retention, rights and third parties", async ({ page }) => {
  await page.goto("/legal/datenschutz");
  for (const section of [
    "Who is responsible",
    "What Team Companion stores",
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
  // The operator's facts.
  await expect(page.getByTestId("controller")).toContainText("Jasper Seehofer");
  await expect(main).toContainText("Hetzner Online GmbH, Helsinki (Finland)");
  await expect(main).toContainText("Hetzner Online GmbH, Falkenstein (Germany)");
  await expect(main).toContainText("kept for 14 days");
  await expect(page.getByTestId("authority")).toContainText("Baden-Württemberg");
  // Legal bases: drafted defaults, stated as text, flagged as drafts.
  const bases = page.getByTestId("legal-bases");
  await expect(bases).toContainText("performance of the contract with you (Art. 6(1)(b) GDPR)");
  await expect(bases).toContainText("legitimate interest in the security and operation of the service (Art. 6(1)(f) GDPR)");
  await expect(main.getByText("Partly a draft")).toBeVisible();
  await expect(main).toContainText("drafts pending the operator's adviser");
});

test("/riot.txt is served as plain text", async ({ request }) => {
  const res = await request.get("/riot.txt");
  expect(res.status()).toBe(200);
  expect(res.headers()["content-type"]).toContain("text/plain");
  expect((await res.text()).trim().length).toBeGreaterThan(0);
});
