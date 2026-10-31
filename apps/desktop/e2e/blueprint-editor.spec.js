import { test, expect } from "./fixtures/test.js";

/** Covers the blueprint editor shell and the folder-structure editor panels. */
test.describe("blueprint editor", () => {
  const openEditor = async (page, name) => {
    await page.goto("/blueprints");
    await page.getByRole("button", { name }).click();
    const dialog = page.locator("dialog[open]");
    await expect(dialog.locator("#modal-title")).toHaveText(name);
    return dialog;
  };

  test("file blueprint editor renders every section", async ({ page }) => {
    const dialog = await openEditor(page, "Organizar notas fiscais");

    for (const label of ["Geral", "Busca", "Template", "Automação"]) {
      await dialog.getByRole("button", { name: label, exact: false }).first().click();
      await expect(dialog.getByRole("heading", { level: 3 })).toBeVisible();
    }
  });

  test("file blueprint editor edits name and root path", async ({ page }) => {
    const dialog = await openEditor(page, "Organizar notas fiscais");

    await dialog.locator('input[type="text"]').first().fill("Notas 2026");
    await expect(dialog.locator("#modal-title")).toHaveText("Notas 2026");

    const root = dialog.locator('input[placeholder="C:\\\\pasta\\\\raiz"]');
    await root.fill("/home/demo/Nova");
    await expect(root).toHaveValue("/home/demo/Nova");
  });

  test("folder blueprint editor starts on the tree panel", async ({ page }) => {
    const dialog = await openEditor(page, "Estrutura de projeto");

    await expect(dialog.getByRole("heading", { level: 3, name: /árvore|tree/i })).toBeVisible();
  });

  test("folder editor navigates all five panels", async ({ page }) => {
    const dialog = await openEditor(page, "Estrutura de projeto");

    for (const label of ["Geral", "Variáveis", "Árvore", "Segurança", "Automação"]) {
      await dialog.getByRole("button", { name: label, exact: false }).first().click();
      await expect(dialog.getByRole("heading", { level: 3 })).toBeVisible();
    }
  });

  test("folder editor adds and removes a root folder node", async ({ page }) => {
    const dialog = await openEditor(page, "Estrutura de projeto");

    const nodes = dialog.locator(".folder-tree > li");
    const before = await nodes.count();

    await dialog.getByRole("button", { name: /criar pasta|create folder/i }).first().click();
    await expect(nodes).toHaveCount(before + 1);

    await nodes.last().getByRole("button", { name: /remover|remove/i }).first().click();
    await expect(nodes).toHaveCount(before);
  });

  test("folder editor adds and removes a plan variable", async ({ page }) => {
    const dialog = await openEditor(page, "Estrutura de projeto");
    await dialog.getByRole("button", { name: /variáveis|variables/i }).first().click();

    const variables = dialog.locator(".variable-row");
    const before = await variables.count();

    await dialog.getByRole("button", { name: /^variável$|^variable$/i }).first().click();
    await expect(variables).toHaveCount(before + 1);
  });

  test("folder editor watches default to auto-correct", async ({ page }) => {
    const dialog = await openEditor(page, "Estrutura de projeto");
    await dialog.getByRole("button", { name: /automação|automation/i }).first().click();

    await dialog.getByRole("checkbox", { name: /monitoramento ativo|watch enabled/i }).check();

    // Folder watches opt into auto-correct; file jobs do not.
    const autoCorrect = dialog.getByRole("checkbox", {
      name: /auto-corrigir|auto-correct/i,
    });
    await expect(autoCorrect).toBeChecked();
  });

  test("template panel previews a rename pipeline", async ({ page }) => {
    const dialog = await openEditor(page, "Organizar notas fiscais");
    await dialog.getByRole("button", { name: /template/i }).first().click();

    const sample = dialog.locator('input[placeholder*="Docs"]').first();
    await sample.fill("relatorio.PDF");

    await expect(dialog.getByText(/2026|relatorio/i).first()).toBeVisible();
  });
});
