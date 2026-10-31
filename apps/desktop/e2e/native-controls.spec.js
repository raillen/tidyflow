import { test, expect } from "./fixtures/test.js";

/**
 * Native form controls (select, date pickers, checkboxes) are painted by the
 * platform, not by CSS `background`. Their scheme comes from `color-scheme`, so
 * a themed app that omits it gets light controls under dark text — the select
 * in Settings rendered near-white text on a light-grey popup.
 *
 * These assertions pin the contract: every themed control must resolve to the
 * matching `color-scheme`, and text must contrast with its own background.
 */

/** Relative luminance per WCAG 2.x, for a computed `rgb(...)` colour. */
function luminance(color) {
  const [r, g, b] = color.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(fg, bg) {
  const a = luminance(fg);
  const b = luminance(bg);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

for (const theme of ["light", "dark"]) {
  test.describe(`native controls in ${theme}`, () => {
    test.beforeEach(async ({ page }) => {
      await page.addInitScript((value) => {
        localStorage.setItem("tidyflow-mock-settings", JSON.stringify({ language: "pt-BR", theme: value }));
      }, theme);
      await page.goto("/settings");
      await expect(page.getByRole("button", { name: /salvar|save/i }).first()).toBeEnabled();
    });

    test("root declares the active color scheme", async ({ page }) => {
      const scheme = await page.evaluate(
        () => getComputedStyle(document.documentElement).colorScheme,
      );
      expect(scheme).toBe(theme);
    });

    test("select text is readable against its own background", async ({ page }) => {
      const styles = await page.evaluate(() => {
        const el = document.querySelector("select.field-input");
        const computed = getComputedStyle(el);
        return {
          color: computed.color,
          background: computed.backgroundColor,
          scheme: computed.colorScheme,
        };
      });

      // `color-scheme: normal` means the platform paints the control in the
      // system scheme, ignoring our `background`.
      expect(styles.scheme).toBe(theme);
      expect(contrast(styles.color, styles.background)).toBeGreaterThanOrEqual(4.5);
    });

    test("select options are readable", async ({ page }) => {
      const styles = await page.evaluate(() => {
        const option = document.querySelector("select.field-input option");
        const computed = getComputedStyle(option);
        return { color: computed.color, background: computed.backgroundColor };
      });

      // A transparent option background lets the platform popup show through,
      // so there is nothing to compare against.
      expect(styles.background).not.toBe("rgba(0, 0, 0, 0)");
      expect(contrast(styles.color, styles.background)).toBeGreaterThanOrEqual(4.5);
    });

    test("date input is readable", async ({ page }) => {
      // The history page owns the date filters. `goto` alone only updates the
      // SPA route, so the assertion may run against the previous page.
      await page.goto("/history");
      const dateInput = page.locator('input[type="datetime-local"]').first();
      await expect(dateInput).toBeVisible();

      const styles = await dateInput.evaluate((el) => {
        const computed = getComputedStyle(el);
        return {
          color: computed.color,
          background: computed.backgroundColor,
          scheme: computed.colorScheme,
        };
      });

      expect(styles.scheme).toBe(theme);
      expect(contrast(styles.color, styles.background)).toBeGreaterThanOrEqual(4.5);
    });

    test("custom select arrow is not the platform default", async ({ page }) => {
      // `appearance: auto` paints the native control over our colours, which is
      // what produced the unreadable popup. The arrow is redrawn in CSS instead.
      const appearance = await page.evaluate(
        () => getComputedStyle(document.querySelector("select.field-input")).appearance,
      );
      expect(appearance).toBe("none");
    });
  });
}
