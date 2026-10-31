import { test, expect } from "./fixtures/test.js";

const settingsLoaded = async (page) => {
  await expect(page.getByRole("button", { name: /salvar|save/i }).first()).toBeEnabled();
};

test.describe("settings", () => {
  test("every tab renders without errors", async ({ page }) => {
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));

    await page.goto("/settings");
    await settingsLoaded(page);

    const tabs = [
      "Geral",
      "Performance",
      "Segurança",
      "Notificações",
      "Admin",
      "Manutenção",
      "Suporte",
      "Sobre",
      "Changelog",
    ];

    for (const tab of tabs) {
      await page.getByRole("button", { name: tab, exact: true }).click();
      await expect(page.locator("section.panel").first()).toBeVisible();
    }

    expect(errors).toEqual([]);
  });

  test("theme buttons switch the document theme", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);

    await page.getByRole("button", { name: "Claro", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");

    await page.getByRole("button", { name: "Escuro", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
    await expect(page.locator("html")).toHaveClass(/dark/);

    await page.getByRole("button", { name: "Sistema", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "system");
  });

  test("accent preset updates the CSS variable", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);

    await page.getByRole("button", { name: "Rosa", exact: true }).click();

    const accent = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--accent").trim(),
    );
    expect(accent.toLowerCase()).toBe("#e11d48");
  });

  test("saving persists and confirms", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);

    await page.getByRole("button", { name: /salvar|save/i }).first().click();
    await expect(page.getByText(/configured|salvas|saved/i).last()).toBeVisible();
    await expect(page.locator('[aria-live="polite"]').filter({ hasText: /salv/i })).toHaveCount(1);
  });

  test("pin validation rejects a short pin", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);
    await page.getByRole("button", { name: "Segurança", exact: true }).click();

    await page.locator('section.panel input[type="checkbox"]').first().check();

    const pins = page.locator('section.panel input[type="password"]');
    await pins.nth(0).fill("12");
    await pins.nth(1).fill("12");
    await page.getByRole("button", { name: /salvar|save/i }).first().click();

    await expect(page.getByText(/pelo menos 4|at least 4/i)).toBeVisible();
  });

  test("pin validation rejects mismatched confirmation", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);
    await page.getByRole("button", { name: "Segurança", exact: true }).click();

    await page.locator('section.panel input[type="checkbox"]').first().check();
    const pins = page.locator('section.panel input[type="password"]');
    await pins.nth(0).fill("1234");
    await pins.nth(1).fill("9999");
    await page.getByRole("button", { name: /salvar|save/i }).first().click();

    await expect(page.getByText(/não confere|does not match/i)).toBeVisible();
  });

  test("webhooks can be added and removed", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);
    await page.getByRole("button", { name: "Notificações", exact: true }).click();

    const webhookCards = page.locator("section.panel .nested-card");
    await expect(webhookCards).toHaveCount(0);

    await page.getByRole("button", { name: /^Adicionar$|^Add$/i }).click();
    await expect(webhookCards).toHaveCount(1);
    await expect(webhookCards.first().locator("input").nth(1)).toHaveValue(
      "https://example.com/webhook",
    );

    await webhookCards.first().getByRole("button", { name: /remover|remove/i }).click();
    await expect(webhookCards).toHaveCount(0);
  });

  test("changelog search filters entries", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);
    await page.getByRole("button", { name: "Changelog", exact: true }).click();

    await expect(page.getByText("v2.1.0")).toBeVisible();
    await page.locator("section.panel input").first().fill("zzzz-inexistente");
    await expect(page.getByText("v2.1.0")).toHaveCount(0);
  });

  test("language selector offers both locales", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);

    const select = page.locator("section.panel select").first();
    await expect(select.locator("option")).toHaveCount(2);
    await expect(select.locator("option").nth(0)).toHaveText("Português (BR)");
    await expect(select.locator("option").nth(1)).toHaveText("English (US)");
  });

  test("admin secret save is blocked below 32 characters", async ({ page }) => {
    await page.goto("/settings");
    await settingsLoaded(page);
    await page.getByRole("button", { name: "Admin", exact: true }).click();

    const secretField = page.locator('section.panel input[type="password"]').last();
    const saveSecret = page.getByRole("button", { name: /salvar no cofre|save to vault/i });

    await secretField.fill("curto");
    await expect(saveSecret).toBeDisabled();

    await secretField.fill("s".repeat(32));
    await expect(saveSecret).toBeEnabled();
  });
});
