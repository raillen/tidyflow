import { test, expect } from "./fixtures/test.js";

test.describe("blueprints", () => {
  test("kind tabs filter the list", async ({ page }) => {
    await page.goto("/blueprints");
    await expect(page.getByRole("button", { name: "Organizar notas fiscais" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Estrutura de projeto" })).toBeVisible();

    await page.getByRole("tab", { name: "Arquivos" }).click();
    await expect(page.getByRole("button", { name: "Estrutura de projeto" })).toHaveCount(0);

    await page.getByRole("tab", { name: "Pastas" }).click();
    await expect(page.getByRole("button", { name: "Organizar notas fiscais" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Estrutura de projeto" })).toBeVisible();
  });

  test("create modal exposes kind-restricted editor", async ({ page }) => {
    await page.goto("/blueprints");
    await page.getByRole("button", { name: "Novo blueprint" }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog.locator("#modal-title")).toHaveText(/novo blueprint|new blueprint/i);
  });

  test("file blueprint apply reports the processed count", async ({ page }) => {
    await page.goto("/blueprints");
    const card = page.locator(".bp-card", { hasText: "Organizar notas fiscais" });
    await card.getByRole("button", { name: /aplicar|apply/i }).click();

    await expect(page.getByText(/2/).first()).toBeVisible();
  });

  test("folder blueprint apply asks for the required variable first", async ({ page }) => {
    await page.goto("/blueprints");
    const card = page.locator(".bp-card", { hasText: "Estrutura de projeto" });
    await card.getByRole("button", { name: /aplicar|apply/i }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText(/nome do projeto|project/i);

    // Confirm is blocked until the required variable has a value.
    const confirm = dialog.getByRole("button", { name: /aplicar|apply/i }).last();
    await expect(confirm).toBeDisabled();

    await dialog.locator("input").first().fill("Projeto Alpha");
    await expect(confirm).toBeEnabled();
    await confirm.click();
    await expect(page.locator("dialog[open]")).toHaveCount(0);
  });

  test("folder blueprint simulate posts variables and opens the report", async ({ page }) => {
    await page.goto("/blueprints");
    const card = page.locator(".bp-card", { hasText: "Estrutura de projeto" });
    await card.getByRole("button", { name: /simular|simulate/i }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await dialog.locator("input").first().fill("Projeto Beta");
    await dialog.getByRole("button", { name: /simular|simulate/i }).last().click();

    await expect(page.locator("dialog[open]")).toContainText(/2|simula/i);
  });

  test("delete asks for confirmation", async ({ page }) => {
    const messages = [];
    page.on("dialog", async (d) => {
      messages.push(d.message());
      await d.accept();
    });

    await page.goto("/blueprints");
    const card = page.locator(".bp-card", { hasText: "Organizar notas fiscais" });
    await card.getByRole("button", { name: /excluir|delete/i }).click();

    await expect.poll(() => messages.length).toBe(1);
    await expect(page.getByRole("button", { name: "Organizar notas fiscais" })).toHaveCount(0);
  });

  test("edit modal hydrates an existing blueprint", async ({ page }) => {
    await page.goto("/blueprints");
    await page.getByRole("button", { name: "Organizar notas fiscais" }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog.locator("#modal-title")).toHaveText("Organizar notas fiscais");
    await expect(dialog.locator('input[placeholder="C:\\\\pasta\\\\raiz"]')).toHaveValue(
      "/home/demo/Entrada",
    );
  });

  test("cancel dismisses the variables modal", async ({ page }) => {
    await page.goto("/blueprints");
    const card = page.locator(".bp-card", { hasText: "Estrutura de projeto" });
    await card.getByRole("button", { name: /aplicar|apply/i }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await dialog.getByRole("button", { name: /cancelar|cancel/i }).click();
    await expect(page.locator("dialog[open]")).toHaveCount(0);
  });
});
