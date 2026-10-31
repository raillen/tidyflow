import { test as base, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const mockSource = readFileSync(join(here, "..", "fixtures", "tauri-mock.js"), "utf8");

/** Every test gets the in-memory Tauri backend installed before the app boots. */
export const test = base.extend({
  page: async ({ page }, use) => {
    await page.addInitScript({ content: mockSource });
    await use(page);
  },
});

export { expect };
