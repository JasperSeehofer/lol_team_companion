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

/*
 * M149 B: "Select an area of the page" (desktop only). The button exists
 * only with getDisplayMedia, at md width or wider with a fine pointer,
 * decided after hydration. Below, getDisplayMedia is stubbed with a
 * canvas stream so the overlay can be driven without a share prompt; the
 * stub records the tracks so the test can see they were all stopped.
 */
const AREA_BUTTON = "Select an area of the page";

async function stubDisplayMedia(page: Page, mode: "stream" | "deny") {
  await page.addInitScript((mode) => {
    const w = window as unknown as { __areaTracks: MediaStreamTrack[] };
    w.__areaTracks = [];
    const getDisplayMedia = async () => {
      if (mode === "deny") {
        throw new DOMException("Permission denied by system", "NotAllowedError");
      }
      const c = document.createElement("canvas");
      c.width = 1280;
      c.height = 720;
      const ctx = c.getContext("2d")!;
      const paint = () => {
        ctx.fillStyle = "rgb(40, 120, 90)";
        ctx.fillRect(0, 0, c.width, c.height);
      };
      paint();
      setInterval(paint, 50);
      const stream = c.captureStream(30);
      w.__areaTracks.push(...stream.getTracks());
      return stream;
    };
    if (!navigator.mediaDevices) {
      Object.defineProperty(navigator, "mediaDevices", { value: {}, configurable: true });
    }
    Object.defineProperty(navigator.mediaDevices, "getDisplayMedia", {
      value: getDisplayMedia,
      configurable: true,
      writable: true,
    });
  }, mode);
}

async function tracksAllEnded(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const t = (window as unknown as { __areaTracks: MediaStreamTrack[] }).__areaTracks;
    return t.length > 0 && t.every((x) => x.readyState === "ended");
  });
}

test("at phone width there is no area button, and the file floor still works", async ({ browser }) => {
  const page = await (await browser.newContext({ viewport: { width: 390, height: 844 } })).newPage();
  await stubDisplayMedia(page, "stream");
  await authenticatePage(page);
  const d = await openDialog(page);
  await expect(d.getByRole("button", { name: "Attach an image" })).toBeVisible();
  await page.waitForTimeout(500);
  await expect(d.getByRole("button", { name: AREA_BUTTON })).toHaveCount(0);
  await d.locator("#bug-report-file").setInputFiles({ name: "phone.png", mimeType: "image/png", buffer: PNG });
  await expect(d.getByTestId("report-attachment-attached")).toContainText("phone.png");
});

test("at 1280 the area button is there exactly when getDisplayMedia is", async ({ browser }) => {
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage();
  await authenticatePage(page);
  const d = await openDialog(page);
  await expect(d.getByRole("button", { name: "Attach an image" })).toBeVisible();
  await page.waitForTimeout(500);
  const api = await page.evaluate(() => typeof navigator.mediaDevices?.getDisplayMedia === "function");
  await expect(d.getByRole("button", { name: AREA_BUTTON })).toHaveCount(api ? 1 : 0);
});

test("a selected area is attached with Enter, and Esc returns to the dialog", async ({ browser }) => {
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage();
  await stubDisplayMedia(page, "stream");
  await authenticatePage(page);
  const d = await openDialog(page);

  // Enter keeps the starting area: centred, half the viewport each way.
  await d.getByRole("button", { name: AREA_BUTTON }).click();
  const picker = page.getByRole("dialog", { name: "Select an area of the page" });
  await expect(picker).toBeVisible();
  await expect(d).toBeHidden();
  expect(await tracksAllEnded(page)).toBe(true);
  await expect(picker.getByTestId("area-size")).toHaveText("640 x 360");
  // Arrow keys move the area; Alt + arrows resize it.
  await page.keyboard.press("Alt+ArrowRight");
  await expect(picker.getByTestId("area-size")).toHaveText("648 x 360");
  await page.keyboard.press("Enter");
  await expect(picker).toHaveCount(0);
  await expect(d).toBeVisible();
  await expect(d.getByTestId("report-attachment-attached")).toContainText("Selected area");
  await expect(d.getByRole("img", { name: "Screenshot preview" })).toBeVisible();

  // Esc: back to the dialog with the plain message, nothing new attached.
  await d.getByTestId("report-attachment-attached").getByRole("button", { name: "Remove" }).click();
  await d.getByRole("button", { name: AREA_BUTTON }).click();
  await expect(picker).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(picker).toHaveCount(0);
  await expect(d).toBeVisible();
  await expect(d).toContainText("Screen capture was cancelled. Nothing was attached.");
  await expect(d.getByTestId("report-attachment-attached")).toHaveCount(0);
  expect(await tracksAllEnded(page)).toBe(true);
});

test("a refused screen capture reopens the dialog with the message", async ({ browser }) => {
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage();
  await stubDisplayMedia(page, "deny");
  await authenticatePage(page);
  const d = await openDialog(page);
  await d.getByRole("button", { name: AREA_BUTTON }).click();
  await expect(d).toBeVisible();
  await expect(d).toContainText("The browser did not allow screen capture. You can still attach an image or paste one.");
  await expect(page.getByRole("dialog", { name: "Select an area of the page" })).toHaveCount(0);
});
