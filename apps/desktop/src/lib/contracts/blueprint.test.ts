import { describe, expect, it } from "vitest";
import {
  TEMPLATE_REGEX_GROUPS,
  TEMPLATE_TOKEN_GROUPS,
  TEMPLATE_TRANSFORM_GROUPS,
  parseTemplateDisplayString,
  pipelineToDisplayString,
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
});

describe("blueprint template toolbox groups", () => {
  it("exposes only backend-supported tokenizer entries", () => {
    const tokenNames = TEMPLATE_TOKEN_GROUPS.flatMap((group) => group.items.map((item) => item.name));
    const transformSyntaxes = TEMPLATE_TRANSFORM_GROUPS.flatMap((group) =>
      group.items.map((item) => item.syntax),
    );
    const regexNames = TEMPLATE_REGEX_GROUPS.flatMap((group) => group.items.map((item) => item.name));

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
