import { test, expect } from "./fixtures/test.js";

/** Covers the editor panels that were split out of the old monolithic component. */
test.describe("job editor panels", () => {
  const openEditor = async (page) => {
    await page.goto("/flows");
    await page.getByRole("button", { name: "Backup de Documentos" }).click();
    const dialog = page.locator("dialog[open]");
    await expect(dialog.locator("#modal-title")).toHaveText("Backup de Documentos");
    return dialog;
  };

  const navTo = async (dialog, label) => {
    await dialog.getByRole("button", { name: label, exact: false }).first().click();
  };

  test("every navigation section renders its panel header", async ({ page }) => {
    const dialog = await openEditor(page);

    const sections = [
      ["Geral", /geral|general/i],
      ["Filtros", /filtros|filters/i],
      ["Automação", /automação|automation/i],
      ["Monitoramento", /monitoramento|monitoring/i],
      ["Segurança", /segurança|security/i],
      ["Avançado", /avançado|advanced/i],
    ];

    for (const [label, heading] of sections) {
      await navTo(dialog, label);
      await expect(dialog.getByRole("heading", { level: 3, name: heading })).toBeVisible();
    }
  });

  test("general panel edits name, mode and conflict", async ({ page }) => {
    const dialog = await openEditor(page);

    const nameInput = dialog.locator('input[type="text"]').first();
    await nameInput.fill("Backup renomeado");
    await expect(dialog.locator("#modal-title")).toHaveText("Backup renomeado");

    await dialog.getByRole("button", { name: /mover|move/i }).click();
    await expect(dialog.getByRole("button", { name: /mover|move/i })).toHaveAttribute(
      "aria-pressed",
      "true",
    );

    await dialog.locator("select").first().selectOption("overwrite");
    await expect(dialog.locator("select").first()).toHaveValue("overwrite");
  });

  test("filters panel keeps typed text with trailing comma", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Filtros");

    const extensions = dialog.locator('input[placeholder=".pdf, .docx"]');
    await extensions.fill(".pdf, .docx, ");

    // The raw text must survive: trimming it mid-typing would fight the user.
    await expect(extensions).toHaveValue(".pdf, .docx, ");
    await expect(dialog.locator(".nav-item.active .badge")).toHaveText("1");
  });

  test("filters panel toggles exclusion presets", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Filtros");

    // Nothing is filtered on a fresh job, so the nav badge starts hidden.
    const badge = dialog.locator(".nav-item.active .badge");
    await expect(badge).toHaveCount(0);

    const preset = dialog.getByRole("checkbox", { name: "node_modules" });
    await preset.check();
    await expect(preset).toBeChecked();
    await expect(badge).toHaveText("1");

    await preset.uncheck();
    await expect(preset).not.toBeChecked();
    await expect(badge).toHaveCount(0);

  });

  test("automation panel switches rule kind and keeps typed hours", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Automação");

    await dialog.getByRole("checkbox", { name: /agendamento ativo|schedule enabled/i }).first().check();

    const ruleSelect = dialog.locator("select").filter({ has: page.locator('option[value="weekly"]') });
    await ruleSelect.selectOption("daily");

    const hourInput = dialog.locator('input[type="number"]').first();
    await hourInput.fill("18");
    await expect(hourInput).toHaveValue("18");
  });

  test("automation panel clamps out-of-range minutes", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Automação");
    await dialog.getByRole("checkbox", { name: /agendamento ativo|schedule enabled/i }).first().check();

    // Interval is the default rule: a single numeric field holds the minutes.
    const minutes = dialog.locator('input[type="number"]').first();
    await minutes.fill("0");
    await expect(minutes).toHaveValue("1");
  });

  test("monitoring panel requires at least one event", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Monitoramento");

    await dialog.getByRole("checkbox", { name: /monitoramento ativo|watch enabled/i }).check();

    const onlyEvent = dialog.getByRole("checkbox", { name: /criação|created/i }).first();
    await expect(onlyEvent).toBeChecked();
    // The last remaining event is disabled instead of silently refusing the
    // change, so the checkbox cannot drift out of sync with the model.
    await expect(onlyEvent).toBeDisabled();

    // Enabling a second event re-enables the first one for removal.
    await dialog.getByRole("checkbox", { name: /modificação|modified/i }).check();
    await expect(onlyEvent).toBeEnabled();
    await onlyEvent.uncheck();
    await expect(onlyEvent).not.toBeChecked();
  });

  test("security panel reveals the encryption options", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Segurança");

    // The encryption options live in a collapsed section by default.
    await dialog.locator("summary", { hasText: /criptografia do pacote|pack encryption/i }).click();
    await dialog.getByRole("checkbox", { name: /criptografar saída|encrypt output/i }).check();
    await expect(dialog.locator('input[type="password"]')).toBeVisible();
    await expect(dialog.locator('input[type="password"]')).toHaveAttribute(
      "autocomplete",
      "new-password",
    );
  });

  test("advanced panel masks the telegram bot token", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Avançado");

    await dialog.getByRole("checkbox", { name: /notificações|notifications/i }).first().check();
    await dialog.getByRole("button", { name: "Telegram" }).click();

    const secret = dialog.locator('input[type="password"]');
    await expect(secret).toHaveCount(1);
    await expect(secret).toHaveAttribute("autocomplete", "off");
  });

  test("advanced panel adds and removes notification channels", async ({ page }) => {
    const dialog = await openEditor(page);
    await navTo(dialog, "Avançado");
    await dialog.getByRole("checkbox", { name: /notificações|notifications/i }).first().check();

    const channels = dialog.locator(".channel-card");
    await expect(channels).toHaveCount(0);

    await dialog.getByRole("button", { name: "Discord" }).click();
    await expect(channels).toHaveCount(1);

    await dialog.getByRole("button", { name: "Telegram" }).click();
    await expect(channels).toHaveCount(2);

    await channels.first().getByRole("button").click();
    await expect(channels).toHaveCount(1);
  });
});
