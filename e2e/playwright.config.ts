import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  // Increased from 30s to 60s: audit tests register users and create teams per test,
  // requiring multiple page loads of a 52MB WASM bundle. 60s gives enough headroom
  // on the dev server. Run audit tests from a fresh browser session for best results.
  timeout: 60_000,
  expect: { timeout: 5_000 },
  fullyParallel: false, // sequential — shares one dev server session
  // One worker: files used to run two at a time against the same dev server,
  // and with the large debug WASM bundle that made page loads and the
  // register redirect time out now and then (the "flaky" audit-draft and
  // pages failures).
  workers: 1,
  retries: 0,
  reporter: "list",

  use: {
    baseURL: process.env.BASE_URL ?? "http://127.0.0.1:3020",
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  // No webServer block — assumes `cargo leptos watch` is already running.
});
