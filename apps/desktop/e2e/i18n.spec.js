import { test, expect } from "./fixtures/test.js";

test.describe("i18n", () => {
  test("defaults to pt-BR", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("navigation").getByRole("link", { name: "Dashboard" })).toBeVisible();
    await expect(page.getByRole("navigation").getByRole("link", { name: "Fluxos" })).toBeVisible();
    await expect(
      page.getByRole("navigation").getByRole("link", { name: "Configurações" }),
    ).toBeVisible();
  });

  test("stored en-US setting renders english navigation", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("tidyflow-locale", "en-US");
      localStorage.setItem("tidyflow-mock-settings", JSON.stringify({ language: "en-US" }));
    });
    await page.goto("/");

    await expect(page.getByRole("navigation").getByRole("link", { name: "Flows" })).toBeVisible();
    await expect(
      page.getByRole("navigation").getByRole("link", { name: "Settings" }),
    ).toBeVisible();
  });

  test("changing language through settings persists the locale", async ({ page }) => {
    await page.goto("/settings");
    const select = page.locator("section.panel select").first();
    await expect(select.locator("option")).toHaveCount(2);

    await select.selectOption("en-US");
    await page.getByRole("button", { name: /salvar|save/i }).first().click();

    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("tidyflow-locale")), {
        timeout: 10000,
      })
      .toBe("en-US");
  });

  test("no translation key leaks into the rendered UI", async ({ page }) => {
    const leaks = [];
    page.on("console", (message) => {
      const text = message.text();
      // Unresolved keys surface verbatim, e.g. "settings.fooBar".
      if (/\b[a-z][a-zA-Z]*\.[a-zA-Z.]+\b/.test(text) && message.type() === "warning") {
        leaks.push(text);
      }
    });

    for (const route of ["/", "/flows", "/blueprints", "/admin", "/history", "/settings"]) {
      await page.goto(route);
      const body = await page.locator("body").innerText();
      // A dotted identifier alone in the page usually means a missing key.
      const suspicious = body.match(/\b(?:nav|settings|flows|history|admin|blueprints)\.[a-zA-Z.]+\b/g);
      expect(suspicious ?? []).toEqual([]);
    }

    expect(leaks).toEqual([]);
  });
});
