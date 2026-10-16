/**
 * M141 H2 (bug_report y1abg605): a screenshot attached to feedback by file
 * (the guaranteed floor) is re-encoded to WebP on the server, shown to
 * admins on /admin/bugs and served only to admins; anything that is not a
 * PNG, JPEG or WebP is refused in the dialog. CI runs it.
 */
import { test, expect, type Page } from "@playwright/test";
import { authenticatePage, signInAsAdmin } from "./fixtures";

// A 2x2 PNG.
const PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAEklEQVR4nGM4IScnd0KOAUIBAB3SBBHfPB1bAAAAAElFTkSuQmCC",
  "base64"
);

async function openDialog(page: Page) {
  await page.goto("/draft");
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(500);
  await page.locator('button[aria-label="Give Feedback"]').click();
  await page.locator("[data-feedback-hint]").getByRole("button", { name: "Skip" }).click();
  return page.getByRole("dialog", { name: "Give Feedback" });
}

test("a file attached to feedback reaches the admin as WebP, and only the admin", async ({ browser }) => {
  const stamp = Date.now();
  const words = `Screenshot report ${stamp}`;
  const user = await (await browser.newContext()).newPage();
  await authenticatePage(user);
  const d = await openDialog(user);
  await d.locator("textarea").fill(words);
  await d.locator("#bug-report-file").setInputFiles({ name: "shot.png", mimeType: "image/png", buffer: PNG });
  await expect(d.getByTestId("report-attachment-attached")).toContainText("shot.png");
  await expect(d.getByRole("img", { name: "Screenshot preview" })).toBeVisible();
  await d.getByRole("button", { name: "Send feedback" }).click();
  await expect(user.getByText("Thanks. Your feedback is in.")).toBeVisible({ timeout: 10000 });
  await expect(user.getByText("the screenshot was not saved")).toHaveCount(0);

  const admin = await (await browser.newContext()).newPage();
  await signInAsAdmin(admin);
  await admin.goto("/admin/bugs");
  await admin.waitForLoadState("networkidle");
  await admin.locator('[data-segmented="bug-filter"]').getByRole("radio", { name: "New", exact: true }).click();
  await admin.getByRole("link", { name: new RegExp(words) }).click();
  const shot = admin.getByTestId("report-screenshot").getByRole("img", { name: "Screenshot attached to the report" });
  await expect(shot).toBeVisible();
  const src = (await shot.getAttribute("src")) ?? "";
  expect(src).toMatch(/^\/admin\/attachment\/[A-Za-z0-9_-]+$/);

  const asAdmin = await admin.request.get(src);
  expect(asAdmin.status()).toBe(200);
  expect(asAdmin.headers()["content-type"]).toBe("image/webp");
  expect(asAdmin.headers()["x-content-type-options"]).toBe("nosniff");
  const body = await asAdmin.body();
  expect(body.subarray(0, 4).toString()).toBe("RIFF");
  expect(body.subarray(8, 12).toString()).toBe("WEBP");

  // The reporter, and anyone else who is not an admin, gets a 404.
  expect((await user.request.get(src)).status()).toBe(404);

  // The admin can delete it.
  await admin.getByTestId("report-screenshot").getByRole("button", { name: "Delete screenshot" }).click();
  await expect(admin.getByText("Screenshot deleted.")).toBeVisible();
  expect((await admin.request.get(src)).status()).toBe(404);
});

test("a file that is not an image is refused in the dialog", async ({ browser }) => {
  const page = await (await browser.newContext()).newPage();
  await authenticatePage(page);
  const d = await openDialog(page);
  await d.locator("#bug-report-file").setInputFiles({
    name: "notes.svg",
    mimeType: "image/svg+xml",
    buffer: Buffer.from("<svg xmlns='http://www.w3.org/2000/svg'></svg>"),
  });
  await expect(d).toContainText("Attach a PNG, JPEG or WebP image.");
  await expect(d.getByTestId("report-attachment-attached")).toHaveCount(0);
});

test("posting bytes to a report that is not yours is refused", async ({ browser }) => {
  const page = await (await browser.newContext()).newPage();
  await authenticatePage(page);
  const r = await page.request.post("/feedback/attachment/doesnotexist", {
    data: PNG,
    headers: { "content-type": "image/png" },
  });
  expect(r.status()).toBe(404);
});
