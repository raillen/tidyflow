import { test, expect } from "./fixtures/test.js";

/**
 * Smoke coverage: every route renders, the sidebar navigates, and no page
 * throws an uncaught error while loading against the mocked backend.
 */
test.describe("navigation", () => {
  test("dashboard loads with health and audit cards", async ({ page }) => {
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));

    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Dashboard" })).toBeVisible();
    await expect(page.getByText("OK", { exact: true })).toBeVisible();
    await expect(page.getByText("Backup de Documentos").first()).toBeVisible();

    expect(errors).toEqual([]);
  });

  test("sidebar navigates across every module", async ({ page }) => {
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));

    await page.goto("/");

    const routes = [
      { link: /fluxos|flows/i, url: /\/flows$/, heading: /fluxos|flows/i },
      { link: /blueprints/i, url: /\/blueprints$/, heading: /blueprints/i },
      { link: /admin/i, url: /\/admin$/, heading: /painel administrativo|admin/i },
      { link: /histórico|history/i, url: /\/history$/, heading: /auditoria|history/i },
      { link: /configurações|settings/i, url: /\/settings$/, heading: /configurações|settings/i },
      { link: /dashboard/i, url: /\/$/, heading: /dashboard/i },
    ];

    for (const route of routes) {
      await page.getByRole("navigation").getByRole("link", { name: route.link }).click();
      await expect(page).toHaveURL(route.url);
      await expect(page.getByRole("heading", { level: 1, name: route.heading })).toBeVisible();
    }

    expect(errors).toEqual([]);
  });

  test("active route is marked with aria-current", async ({ page }) => {
    await page.goto("/history");
    const active = page.getByRole("navigation").getByRole("link", { name: /histórico|history/i });
    await expect(active).toHaveAttribute("aria-current", "page");
  });

  test("deep link to an unknown flow opens the edit modal", async ({ page }) => {
    const ids = { BLUEPRINT_FILE_ID: "00000000-0000-4000-8000-000000000003" };
    await page.goto("/flows?edit=00000000-0000-4000-8000-000000000001");
    await expect(page.locator("dialog[open] #modal-title")).toHaveText("Backup de Documentos");
    expect(ids.BLUEPRINT_FILE_ID).toBeTruthy();
  });
});
