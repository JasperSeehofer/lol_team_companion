/**
 * Phase 19 plan 04 — Bug-report widget e2e regression suite.
 *
 * Covers SC-3 (label-driven capture) and SC-7 (no dark patterns
 * regression) for the bug-report widget shipped in plan 19-02 and
 * the page-attribute rollout completed in plan 19-04 (Tasks 1 + 2).
 *
 * Tests (5 functional + 1 negative-content gate):
 *
 * 1. `visible on auth-required pages` — authedPage, goto /draft,
 *    expect floating Report button by aria-label visible.
 * 2. `hidden on public pages` — bare page (no auth), goto /auth/login,
 *    expect the same button has count 0.
 * 3. `select mode captures the nearest data-feedback-label` —
 *    authedPage, goto /draft, click Report, click a tagged element
 *    inside the draft view, expect modal opens with the captured
 *    label visible in the "Element" field.
 * 4. `esc cancels select mode without opening modal` — authedPage,
 *    goto /draft, click Report, press Escape, expect modal count 0.
 * 5. `submit persists and shows toast` — authedPage, goto /draft,
 *    click Report, click tagged element, fill description, choose
 *    Bug, click Submit, expect "Your report is in" toast within 3000
 *    ms, expect modal closes. Deep persistence verification is left
 *    to the manual checkpoint (Task 4) since `list_bug_reports`
 *    returns Forbidden in v1.
 * 6. `widget DOM has no forbidden characters` (Warning 9 gate) —
 *    authedPage, goto /draft, click Report, click tagged element,
 *    read the modal's innerHTML, hard-assert it contains none of:
 *      "🐛"  — bug emoji
 *      "🎉"  — celebration emoji
 *      "Thanks!" — exclamation form (period form is the canonical
 *                  toast string and is allowed)
 *    Belt-and-suspenders on top of plan 19-02's source-code grep.
 *
 * Per wasm rule 56, every navigation is followed by
 * `waitForTimeout(500)` so the WASM hydration Effect can settle
 * before assertions fire.
 *
 * Auth gate test (#2) uses the bare `page` fixture so it does NOT
 * trigger the auth flow — the floating button must genuinely be
 * absent on /auth/login.
 */
import { test, expect } from "./fixtures";
import { captureErrors, filterRealErrors } from "./helpers";

// Locator strings drawn from src/components/bug_report_widget.rs
// — kept here as constants so a future markup tweak that breaks the
// suite produces an obvious diff in one place.
const REPORT_BUTTON = 'button[aria-label="Report a bug or wishlist item"]';
const MODAL_DIALOG = '[role="dialog"]';
const BUG_TOGGLE = 'button:has-text("Bug")';
const SUBMIT_BUTTON = 'button:has-text("Submit")';
const TOAST_REGEX = /Your report is in/;
// Tagged element on the /draft page from plan 19-04 Task 1 — Blue side
// column carries `data-feedback-label="Draft → Blue side"`. The
// `*=` partial-match selector is resilient to label tweaks.
const TAGGED_DRAFT_ELEMENT = '[data-feedback-label*="Draft"]';

test.describe("bug-report widget", () => {
  test("visible on auth-required pages", async ({ authedPage }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);

    await expect(authedPage.locator(REPORT_BUTTON)).toBeVisible({
      timeout: 5000,
    });

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("hidden on public pages", async ({ page }) => {
    // Bare `page` fixture — NOT authedPage. The widget gates on
    // `is_authed() && !on_pathname_excluded()`; on /auth/login the
    // user is unauthenticated AND the path starts with /auth (which
    // is in the HIDDEN_PREFIXES list). Either gate alone hides the
    // button, but the canonical behavior is that BOTH conditions
    // hold on this route.
    const errors = captureErrors(page);
    await page.goto("/auth/login");
    await page.waitForTimeout(500);

    await expect(page.locator(REPORT_BUTTON)).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("select mode captures the nearest data-feedback-label", async ({
    authedPage,
  }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);

    // Step 1: click the floating Report button to enter Selecting.
    await authedPage.locator(REPORT_BUTTON).click();
    // Brief settle for the global click-capture listener to attach.
    await authedPage.waitForTimeout(200);

    // Step 2: click a tagged element. The click-capture listener
    // walks up via Element::closest("[data-feedback-label]") and
    // transitions to Editing with the resolved label.
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await authedPage.waitForTimeout(300);

    // Modal opens and the resolved label appears in its "Element"
    // field. The label text starts with "Draft" per Task 1 rollout.
    const modal = authedPage.locator(MODAL_DIALOG);
    await expect(modal).toBeVisible({ timeout: 3000 });
    await expect(modal).toContainText(/Draft/);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("esc cancels select mode without opening modal", async ({
    authedPage,
  }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);

    // Enter select mode.
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.waitForTimeout(200);

    // Press Escape — the keydown listener resets to Idle without
    // opening the modal. Brief settle for the listener teardown.
    await authedPage.keyboard.press("Escape");
    await authedPage.waitForTimeout(200);

    // Modal must NOT be open.
    await expect(authedPage.locator(MODAL_DIALOG)).toHaveCount(0);

    // Listener-detach regression gate: after Esc, clicking a tagged
    // element must NOT reopen the modal (the capture-phase click
    // listener must be fully removed, not just visually quiesced).
    // Without the capture-flag match on removeEventListener, this
    // click silently reopens the modal — see widget commit 05697fe.
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await authedPage.waitForTimeout(200);
    await expect(authedPage.locator(MODAL_DIALOG)).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("submit persists and shows toast", async ({ authedPage }) => {
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);

    // Open the modal via the full select-mode flow.
    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.waitForTimeout(200);
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await authedPage.waitForTimeout(300);

    const modal = authedPage.locator(MODAL_DIALOG);
    await expect(modal).toBeVisible({ timeout: 3000 });

    // Fill the description textarea. The modal renders a single
    // <textarea> for the report body.
    await modal.locator("textarea").fill("test report from e2e");

    // Click the Bug toggle (it is the SSR default, but we click it
    // explicitly so the test exercises the toggle interaction).
    await modal.locator(BUG_TOGGLE).click();
    await authedPage.waitForTimeout(100);

    // Submit. The server-fn writes to SurrealDB; on success the
    // ToastContext dispatches a Success toast and the modal closes.
    await modal.locator(SUBMIT_BUTTON).click();

    // Toast appears within 3000 ms — text comes from
    // src/components/bug_report_widget.rs line ~511:
    //   "Thanks. Your report is in."
    await expect(authedPage.locator(`text=${TOAST_REGEX}`)).toBeVisible({
      timeout: 3000,
    });

    // Modal closes.
    await expect(authedPage.locator(MODAL_DIALOG)).toHaveCount(0);

    expect(filterRealErrors(errors)).toHaveLength(0);
  });

  test("widget DOM has no forbidden characters", async ({ authedPage }) => {
    // Warning 9 negative-content gate (SC-7 hard assertion).
    // Belt-and-suspenders on top of plan 19-02's source-code grep:
    // even if a future refactor reintroduces 🐛 / 🎉 / "Thanks!" into
    // the widget, this assertion fires against the rendered DOM.
    const errors = captureErrors(authedPage);
    await authedPage.goto("/draft");
    await authedPage.waitForTimeout(500);

    await authedPage.locator(REPORT_BUTTON).click();
    await authedPage.waitForTimeout(200);
    await authedPage.locator(TAGGED_DRAFT_ELEMENT).first().click();
    await authedPage.waitForTimeout(300);

    const modal = authedPage.locator(MODAL_DIALOG);
    await expect(modal).toBeVisible({ timeout: 3000 });

    const html = await modal.innerHTML();
    expect(html).not.toContain("🐛");
    expect(html).not.toContain("🎉");
    expect(html).not.toContain("Thanks!");

    expect(filterRealErrors(errors)).toHaveLength(0);
  });
});
