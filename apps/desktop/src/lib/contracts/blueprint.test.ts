import { describe, expect, it } from "vitest";
import {
  parseTemplateDisplayString,
  pipelineToDisplayString,
  templateRegexGroups,
  templateTokenGroups,
  templateTransformGroups,
} from "./blueprint";

describe("blueprint template display parser", () => {
  it("round-trips tokens, parameterized transforms and regex presets", () => {
    const display = "{grandparent}/{date}/{stem}[take(12)][skip(2, end)]/extract_date/";
    const pipeline = parseTemplateDisplayString(display);

    expect(pipeline.segments).toEqual([
      { kind: "token", name: "grandparent" },
      { kind: "literal", value: "/" },
      { kind: "token", name: "date" },
      { kind: "literal", value: "/" },
      { kind: "token", name: "stem" },
      { kind: "transform", name: "take(12)" },
      { kind: "transform", name: "skip(2, end)" },
      { kind: "regexPreset", name: "extract_date" },
    ]);
    expect(pipelineToDisplayString(pipeline)).toBe(display);
  });

  it("keeps ordinary path segments and unknown brackets literal", () => {
    expect(parseTemplateDisplayString("{parent}/docs/{stem}").segments).toEqual([
      { kind: "token", name: "parent" },
      { kind: "literal", value: "/docs/" },
      { kind: "token", name: "stem" },
    ]);
    expect(parseTemplateDisplayString("[2024] report").segments).toEqual([
      { kind: "literal", value: "[2024] report" },
    ]);
  });

  it("preserves windows separators while still parsing tokens", () => {
    expect(parseTemplateDisplayString("C:\\in\\Docs\\{stem}").segments).toEqual([
      { kind: "literal", value: "C:\\in\\Docs\\" },
      { kind: "token", name: "stem" },
    ]);
  });

  it("escapes literals so they round-trip", () => {
    const pipeline = {
      segments: [
        { kind: "literal" as const, value: "{projectName} [final]/remove_digits/" },
        { kind: "token" as const, name: "stem" },
      ],
    };
    const display = pipelineToDisplayString(pipeline);
    expect(parseTemplateDisplayString(display)).toEqual(pipeline);
  });
});

describe("blueprint template toolbox groups", () => {
  it("exposes only backend-supported tokenizer entries", () => {
    const tokenNames = templateTokenGroups().flatMap((group) => group.items.map((item) => item.name));
    const transformSyntaxes = templateTransformGroups().flatMap((group) =>
      group.items.map((item) => item.syntax),
    );
    const regexNames = templateRegexGroups().flatMap((group) => group.items.map((item) => item.name));

    expect(tokenNames).toEqual(
      expect.arrayContaining(["grandparent", "date", "year", "stem", "counter"]),
    );
    expect(transformSyntaxes).toEqual(
      expect.arrayContaining(["[capitalized]", "[camel]", "[take(12)]", "[slice(0, 8)]"]),
    );
    expect(regexNames).toEqual(
      expect.arrayContaining(["extract_date", "remove_parens", "digits_only", "letters_only"]),
    );
  });
});
