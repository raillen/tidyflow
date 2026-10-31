import { test, expect } from "./fixtures/test.js";

test.describe("flows", () => {
  test("lists jobs from the backend", async ({ page }) => {
    await page.goto("/flows");
    await expect(page.getByRole("button", { name: "Backup de Documentos" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Fluxo desativado" })).toBeVisible();
  });

  test("disabled job cannot be run", async ({ page }) => {
    await page.goto("/flows");
    const card = page.locator(".job-card", { hasText: "Fluxo desativado" });
    await expect(card.getByRole("button", { name: /executar|run/i })).toBeDisabled();
  });

  test("create modal opens and saves a complete flow", async ({ page }) => {
    await page.goto("/flows");
    await page.getByRole("button", { name: "Novo fluxo" }).first().click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog.locator("#modal-title")).toHaveText(/novo fluxo|new flow/i);

    // The name field is prefilled with the localized default.
    const nameInput = dialog.locator('input[type="text"]').first();
    await expect(nameInput).toHaveValue(/nova transferência|new transfer/i);

    // Source and target are required: an incomplete flow must not submit.
    await dialog.getByRole("button", { name: /salvar|save/i }).click();
    await expect(page.locator("dialog[open]")).toHaveCount(1);

    await dialog.locator('input[placeholder="C:\\\\pasta\\\\origem"]').fill("/home/demo/origem");
    await dialog.locator('input[placeholder="D:\\\\pasta\\\\destino"]').fill("/home/demo/destino");
    await dialog.getByRole("button", { name: /salvar|save/i }).click();
    await expect(page.locator("dialog[open]")).toHaveCount(0);
    await expect(page.getByRole("button", { name: /nova transferência|new transfer/i })).toBeVisible();
  });

  test("close button dismisses the modal", async ({ page }) => {
    await page.goto("/flows");
    await page.getByRole("button", { name: "Novo fluxo" }).first().click();
    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();

    await dialog.getByRole("button", { name: /fechar|close/i }).click();
    await expect(page.locator("dialog[open]")).toHaveCount(0);
  });

  test("escape closes the modal", async ({ page }) => {
    await page.goto("/flows");
    await page.getByRole("button", { name: "Novo fluxo" }).first().click();
    await expect(page.locator("dialog[open]")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.locator("dialog[open]")).toHaveCount(0);
  });

  test("edit modal hydrates the job from the backend", async ({ page }) => {
    await page.goto("/flows");
    await page.getByRole("button", { name: "Backup de Documentos" }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog.locator("#modal-title")).toHaveText("Backup de Documentos");

    const sourceInput = dialog.locator('input[placeholder="C:\\\\pasta\\\\origem"]');
    await expect(sourceInput).toHaveValue("/home/demo/Documentos");
  });

  test("simulate shows a report modal", async ({ page }) => {
    await page.goto("/flows");
    const card = page.locator(".job-card", { hasText: "Backup de Documentos" });
    await card.getByRole("button", { name: /simular|simulate/i }).click();

    const dialog = page.locator("dialog[open]");
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText(/3|simulação|simulation/i);
  });

  test("rollback asks for confirmation and reports the result", async ({ page }) => {
    const dialogs = [];
    page.on("dialog", async (d) => {
      dialogs.push(d.message());
      await d.accept();
    });

    await page.goto("/flows");
    const card = page.locator(".job-card", { hasText: "Backup de Documentos" });
    await card.getByRole("button", { name: /desfazer|rollback/i }).click();

    await expect.poll(() => dialogs.length).toBeGreaterThanOrEqual(1);
    await expect(page.getByRole("status")).toHaveText(/rollback concluído|rollback complete/i);
  });

  test("delete asks for confirmation and removes the card", async ({ page }) => {
    page.on("dialog", (d) => void d.accept());

    await page.goto("/flows");
    const card = page.locator(".job-card", { hasText: "Fluxo desativado" });
    await card.getByRole("button", { name: /excluir|delete/i }).click();

    await expect(page.getByRole("button", { name: "Fluxo desativado" })).toHaveCount(0);
  });

  test("queue pause and resume toggle the label", async ({ page }) => {
    await page.goto("/flows");
    await expect(page.getByRole("button", { name: /pausar|pause/i })).toBeVisible();
    await page.getByRole("button", { name: /pausar|pause/i }).click();
    await expect(page.getByRole("button", { name: /retomar|resume/i })).toBeVisible();
    await page.getByRole("button", { name: /retomar|resume/i }).click();
    await expect(page.getByRole("button", { name: /pausar|pause/i })).toBeVisible();
  });

  test("stop all asks for confirmation", async ({ page }) => {
    const messages = [];
    page.on("dialog", async (d) => {
      messages.push(d.message());
      await d.accept();
    });

    await page.goto("/flows");
    await page.getByRole("button", { name: /parar tudo|stop all/i }).click();
    await expect.poll(() => messages.length).toBe(1);
  });

  test("live execution progress renders and can be cancelled", async ({ page }) => {
    await page.goto("/flows");
    const ids = await page.evaluate(() => window.__TIDYFLOW_MOCK__.ids);

    await page.evaluate(
      ({ jobId, executionId }) => {
        window.__TIDYFLOW_MOCK__.pushExecution({
          executionId,
          jobId,
          jobName: "Backup de Documentos",
          currentFile: "arquivo-1.pdf",
          percent: 42,
          bytesPerSec: 1024,
          recentLog: [],
        });
      },
      { jobId: ids.JOB_ID, executionId: ids.EXECUTION_ID },
    );

    await expect(page.getByText("arquivo-1.pdf")).toBeVisible();
    await expect(page.getByText("42%")).toBeVisible();

    await page.getByRole("button", { name: /cancelar|cancel/i }).click();
    await expect(page.getByText("arquivo-1.pdf")).toHaveCount(0);
  });
});
