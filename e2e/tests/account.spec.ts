/**
 * M8 account data: the "Your data" section on the profile, the JSON export,
 * the player-card switch and the account deletion flow.
 */
import { test, expect, TEST_PASSWORD } from "./fixtures";
import { navigateTo } from "./helpers";

const SHOTS = process.env.M8_SHOTS_DIR;

test("signed out, the export answers 401", async ({ request }) => {
  const res = await request.get("/account/export");
  expect(res.status()).toBe(401);
});

test("download my data returns the account as a JSON file", async ({ authedPage: page }) => {
  await navigateTo(page, "/profile");
  const section = page.locator("[data-your-data]");
  await expect(section).toBeVisible();
  await expect(page.getByTestId("export-data")).toHaveAttribute("href", "/account/export");
  if (SHOTS) await section.screenshot({ path: `${SHOTS}/profile-your-data.png` });

  const email = (await page.getByTestId("profile-email").textContent())?.trim() ?? "";
  const res = await page.request.get("/account/export");
  expect(res.status()).toBe(200);
  expect(res.headers()["content-disposition"]).toMatch(/attachment; filename="companion-data-\d{4}-\d{2}-\d{2}\.json"/);
  expect(res.headers()["cache-control"]).toBe("no-store");
  const doc = await res.json();
  expect(doc.format).toBe("companion-export-1");
  expect(doc.data.account.email).toBe(email);
  expect(doc.data.account.password_hash).toBeUndefined();
  for (const key of ["team_membership", "champion_pool", "personal_learnings", "bug_reports", "votes", "survey_questions"]) {
    expect(Array.isArray(doc.data[key]), key).toBe(true);
  }

  // The browser saves it as a file.
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByTestId("export-data").click()]);
  expect(download.suggestedFilename()).toMatch(/^companion-data-.*\.json$/);
});

test("the player-card switch persists", async ({ authedPage: page }) => {
  // Wait for hydration: a click on the server-rendered switch does nothing.
  await navigateTo(page, "/profile");
  const toggle = page.getByRole("switch", { name: "Keep my player card out of share links" });
  await expect(toggle).toHaveAttribute("aria-checked", "false");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-checked", "true");
  await page.waitForTimeout(500);
  await navigateTo(page, "/profile");
  await expect(page.getByRole("switch", { name: "Keep my player card out of share links" })).toHaveAttribute(
    "aria-checked",
    "true"
  );
});

test("deleting the account needs the username and password, then signs out", async ({ authedPage: page }) => {
  await navigateTo(page, "/profile");
  const username = (await page.locator("h1").textContent())?.trim() ?? "";
  const email = (await page.getByTestId("profile-email").textContent())?.trim() ?? "";
  expect(username.length).toBeGreaterThan(0);

  await page.getByTestId("delete-account").click();
  const dialog = page.locator('dialog[data-dialog="delete-account"]');
  await expect(dialog).toBeVisible();
  const confirm = dialog.locator('[data-confirm="confirm"]');
  await expect(confirm).toBeDisabled();
  // Cancel has the initial focus, so Enter by habit deletes nothing.
  await expect(dialog.locator('[data-confirm="cancel"]')).toBeFocused();

  await dialog.locator("input[name=confirm_username]").fill(username);
  await dialog.locator("input[name=password]").fill("wrong-password-123");
  await expect(confirm).toBeEnabled();
  await confirm.click();
  await expect(dialog).toContainText("That password is not right.");
  if (SHOTS) await dialog.screenshot({ path: `${SHOTS}/delete-account-dialog.png` });

  await dialog.locator("input[name=password]").fill(TEST_PASSWORD);
  await confirm.click();
  await page.waitForURL((u) => new URL(u).pathname === "/" && new URL(u).search.includes("account=deleted"), {
    timeout: 20000,
  });
  await expect(page.getByTestId("account-deleted")).toBeVisible();
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/account-deleted.png` });

  // The session is gone and the account cannot sign in again.
  expect((await page.request.get("/account/export")).status()).toBe(401);
  await page.goto("/auth/login");
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("main button[type=submit]");
  await expect(page.locator("main")).toContainText("Invalid email or password");
});
