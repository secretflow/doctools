import { fileURLToPath } from "node:url";

import { defineConfig, devices } from "@playwright/test";

const isCI = Boolean(process.env["CI"]);

const pythonPath = JSON.stringify(relpath(".venv/bin/python"));

export default defineConfig({
  testDir: relpath("tests/specs"),
  testMatch: /.*\.ts/,
  testIgnore: /util\/.*/,

  snapshotPathTemplate: "{testDir}/snapshots/{testFilePath}/{arg}-{projectName}{ext}",

  fullyParallel: true,
  forbidOnly: isCI,
  retries: isCI ? 3 : 0,
  workers: isCI ? 1 : undefined,

  use: {
    baseURL: "http://127.0.0.1:5173",
    trace: "on-first-retry",
    screenshot: "on-first-failure",
  },
  expect: {
    toMatchSnapshot: {
      maxDiffPixels: 16,
    },
  },

  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1440, height: 900 },
      },
    },
    {
      name: "firefox",
      use: {
        ...devices["Desktop Firefox"],
        viewport: { width: 1440, height: 900 },
        launchOptions: { firefoxUserPrefs: { "network.proxy.type": 0 } },
      },
    },
    {
      name: "webkit",
      use: {
        ...devices["Desktop Safari"],
        viewport: { width: 1440, height: 900 },
      },
    },
    {
      name: "iphone",
      use: {
        ...devices["iPhone 12"],
      },
    },
  ],

  webServer: {
    port: 5173,
    command: `${pythonPath} -m secretflow_doctools preview -i tests/demo -- -p 5173`,
    env: {
      LOGURU_LEVEL: "WARNING",
    },
    gracefulShutdown: { signal: "SIGINT", timeout: 500 },
    reuseExistingServer: !isCI,
  },

  reporter: [["html", { open: "never" }]],
});

function relpath(path: string) {
  return fileURLToPath(new URL(path, import.meta.url));
}
