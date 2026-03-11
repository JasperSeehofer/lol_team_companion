import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: false, // sequential — shares one dev server session
  retries: 0,
  reporter: "list",

  use: {
    baseURL: process.env.BASE_URL ?? "http://127.0.0.1:3002",
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
