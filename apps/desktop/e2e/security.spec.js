import { test, expect } from "./fixtures/test.js";

/** The root layout registers its `security_lock` listener on mount. */
async function openApp(page) {
  await page.goto("/");
  await expect(page.getByRole("navigation")).toBeVisible();
}

async function lock(page) {
  await page.evaluate(() => window.__TIDYFLOW_MOCK__.emit("security_lock", {}));
  await expect(page.locator(".lock-screen")).toBeVisible();
}

test.describe("security pin", () => {
  test("security_lock event shows the lock screen", async ({ page }) => {
    await openApp(page);
    await lock(page);

    await expect(page.locator("#pin-title")).toHaveText(/bloqueado|locked/i);
    await expect(page.getByRole("navigation")).toHaveCount(0);
  });

  test("wrong pin shows an error and keeps the screen locked", async ({ page }) => {
    await openApp(page);
    await lock(page);

    await page.locator('input[type="password"]').fill("9999");
    await page.getByRole("button", { name: /desbloquear|unlock/i }).click();

    await expect(page.getByRole("alert")).toHaveText(/incorreto|incorrect/i);
    await expect(page.locator(".lock-screen")).toBeVisible();
  });

  test("correct pin unlocks the app", async ({ page }) => {
    await openApp(page);
    await lock(page);

    await page.locator('input[type="password"]').fill("1234");
    await page.getByRole("button", { name: /desbloquear|unlock/i }).click();

    await expect(page.locator(".lock-screen")).toHaveCount(0);
    await expect(page.getByRole("navigation")).toBeVisible();
  });

  test("enter submits the pin", async ({ page }) => {
    await openApp(page);
    await lock(page);

    const input = page.locator('input[type="password"]');
    await input.fill("1234");
    await input.press("Enter");

    await expect(page.locator(".lock-screen")).toHaveCount(0);
  });

  test("unlock button stays disabled while the field is empty", async ({ page }) => {
    await openApp(page);
    await lock(page);

    await expect(page.getByRole("button", { name: /desbloquear|unlock/i })).toBeDisabled();
  });

  test("pin fields are not autocompleted with the current password", async ({ page }) => {
    await openApp(page);
    await lock(page);

    await expect(page.locator('input[type="password"]')).toHaveAttribute(
      "autocomplete",
      "current-password",
    );
  });
});
