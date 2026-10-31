import { test, expect } from "./fixtures/test.js";

test.describe("history", () => {
  test("lists audit entries with summary metrics", async ({ page }) => {
    await page.goto("/history");
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.getByText("Backup de Documentos").first()).toBeVisible();
    await expect(page.getByText("Organizar notas fiscais").first()).toBeVisible();
  });

  test("status filter narrows the result set", async ({ page }) => {
    await page.goto("/history");
    await expect(page.getByText("Organizar notas fiscais").first()).toBeVisible();

    await page.locator("select").first().selectOption("FAILED");
    await page.getByRole("button", { name: /filtrar|filter/i }).click();

    await expect(page.getByText("Organizar notas fiscais")).toHaveCount(0);
    await expect(page.getByText("Backup de Documentos").first()).toBeVisible();
  });

  test("search narrows the result set", async ({ page }) => {
    await page.goto("/history");
    await page.locator('input[placeholder]').first().fill("notas fiscais");
    await page.getByRole("button", { name: /filtrar|filter/i }).click();

    await expect(page.getByText("Organizar notas fiscais").first()).toBeVisible();
    await expect(page.getByText("Backup de Documentos")).toHaveCount(0);
  });

  test("reset clears every filter", async ({ page }) => {
    await page.goto("/history");
    await page.locator('input[placeholder]').first().fill("inexistente");
    await page.getByRole("button", { name: /filtrar|filter/i }).click();
    await expect(page.getByText("Backup de Documentos")).toHaveCount(0);

    await page.getByRole("button", { name: /limpar filtros|reset/i }).click();
    await expect(page.getByText("Backup de Documentos").first()).toBeVisible();
  });

  test("details button opens the entry inspector", async ({ page }) => {
    await page.goto("/history");
    await page.getByRole("button", { name: /detalhes|details/i }).first().click();

    await expect(page.getByText("/home/demo/Documentos/relatorio.pdf").first()).toBeVisible();
  });

  test("csv and json export produce a download", async ({ page }) => {
    await page.goto("/history");

    for (const [label, ext] of [
      [/^CSV$/, "csv"],
      [/^JSON$/, "json"],
    ]) {
      const downloadPromise = page.waitForEvent("download");
      await page.getByRole("button", { name: label }).click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toContain(ext);
    }
  });

  test("clear asks for confirmation and empties the table", async ({ page }) => {
    page.on("dialog", (d) => void d.accept());

    await page.goto("/history");
    await expect(page.getByText("Backup de Documentos").first()).toBeVisible();

    await page.getByRole("button", { name: "Limpar", exact: true }).click();
    await expect(page.getByText("Backup de Documentos")).toHaveCount(0);
  });

  test("pagination controls stay bounded", async ({ page }) => {
    await page.goto("/history");
    const previous = page.getByRole("button", { name: /anterior|previous/i });
    await expect(previous).toBeDisabled();
  });
});
