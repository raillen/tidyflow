import { test, expect } from "./fixtures/test.js";

test.describe("admin", () => {
  test("loads the fleet snapshot with summary metrics", async ({ page }) => {
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));

    await page.goto("/admin");
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.getByText("demo-instance")).toBeVisible();
    await expect(page.getByText("Instâncias").first()).toBeVisible();

    expect(errors).toEqual([]);
  });

  test("refresh re-reads the snapshot", async ({ page }) => {
    await page.goto("/admin");
    await expect(page.getByText("demo-instance")).toBeVisible();

    await page.getByRole("button", { name: /atualizar|refresh/i }).click();
    await expect(page.getByText("demo-instance")).toBeVisible();
  });

  test("instance details expose hardware, management and network", async ({ page }) => {
    await page.goto("/admin");
    const details = page.getByRole("region", { name: /detalhes da instância|instance details/i });

    await expect(details.getByText("linux", { exact: true })).toBeVisible();
    await expect(details.getByText("x86_64")).toBeVisible();
    await expect(details.getByText("8 threads").first()).toBeVisible();
    await expect(details.getByText("eth0: 10.0.0.2")).toBeVisible();
  });

  test("planned capabilities are surfaced as unavailable", async ({ page }) => {
    await page.goto("/admin");
    // The mock declares `applySettingsPolicy` and `requestLogs` as planned.
    await expect(page.getByText(/planejad|planned/i).first()).toBeVisible();
  });

  test("start job dispatches a command and reports the result", async ({ page }) => {
    await page.goto("/admin");
    const details = page.getByRole("region", { name: /detalhes da instância|instance details/i });

    await details.getByRole("button", { name: /iniciar|start/i }).first().click();
    await expect(page.getByText(/accepted|aceit/i).first()).toBeVisible();
  });

  test("enqueue job adds to the command queue", async ({ page }) => {
    await page.goto("/admin");
    const details = page.getByRole("region", { name: /detalhes da instância|instance details/i });

    await details.getByRole("button", { name: /enfileirar|enqueue/i }).first().click();
    await expect(page.getByText(/enfileirad|queued|pending/i).first()).toBeVisible();
  });

  test("a failing command surfaces an error banner instead of dying silently", async ({ page }) => {
    await page.addInitScript(() => {
      const original = window.__TAURI_INTERNALS__.invoke;
      window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
        if (cmd === "admin_dispatch_command") {
          throw "backend offline";
        }
        return original(cmd, args);
      };
    });

    const pageErrors = [];
    page.on("pageerror", (error) => pageErrors.push(error.message));

    await page.goto("/admin");
    const details = page.getByRole("region", { name: /detalhes da instância|instance details/i });
    await details.getByRole("button", { name: /iniciar|start/i }).first().click();

    await expect(page.getByRole("status")).toHaveText(/offline/i);
    expect(pageErrors).toEqual([]);
  });
});
