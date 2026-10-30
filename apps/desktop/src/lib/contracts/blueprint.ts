import { z } from "zod";
import { t } from "$lib/i18n";
import {
  conflictStrategySchema,
  countActiveFilters,
  fileFilterSchema,
  watchConfigSchema,
  templateSegmentSchema,
  templatePipelineSchema,
  type FileFilter,
  type TemplateSegment,
  type TemplatePipeline,
} from "$lib/contracts/job";

export { templateSegmentSchema, templatePipelineSchema };
export type { TemplateSegment, TemplatePipeline };

export const blueprintKindSchema = z.enum(["file", "folder"]);
export const blueprintOperationSchema = z.enum(["move", "copy"]);
export const counterScopeSchema = z.enum(["global", "perDay", "perFolder", "perParent"]);

export type CounterScope = z.infer<typeof counterScopeSchema>;

export const routingConfigSchema = z.object({
  pathTemplate: templatePipelineSchema.default({ segments: [] }),
  createIntermediateDirs: z.boolean().default(true),
});

export const folderActionSchema = z.enum(["create", "rename", "move", "copy", "delete"]);

export const folderPlanVariableSchema = z.object({
  name: z.string().min(1),
  label: z.string().default(""),
  defaultValue: z.string().default(""),
  required: z.boolean().default(true),
});

export type FolderActionNode = {
  action: z.infer<typeof folderActionSchema>;
  name: string;
  sourcePath: string;
  targetPath: string;
  children: FolderActionNode[];
};

export const folderActionNodeSchema: z.ZodType<FolderActionNode, z.ZodTypeDef, unknown> = z.lazy(() =>
  z.object({
    action: folderActionSchema.default("create"),
    name: z.string().default(""),
    sourcePath: z.string().default(""),
    targetPath: z.string().default(""),
    children: z.array(folderActionNodeSchema).default([]),
  }),
);

export const folderNodeSchema = folderActionNodeSchema;

export const folderPlanSchema = z.object({
  version: z.number().int().positive().default(1),
  variables: z.array(folderPlanVariableSchema).default([]),
  allowDeleteNonEmpty: z.boolean().default(false),
  confirmDeleteNonEmpty: z.boolean().default(false),
  nodes: z.array(folderActionNodeSchema).default([]),
});

export const counterConfigSchema = z.object({
  scope: counterScopeSchema.default("global"),
  start: z.number().int().nonnegative().default(1),
  padding: z.number().int().nonnegative().default(0),
});

export const blueprintSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1).max(120),
  kind: blueprintKindSchema,
  rootPath: z.string(),
  search: fileFilterSchema.default({}),
  routing: routingConfigSchema.default({}),
  operation: blueprintOperationSchema,
  recursive: z.boolean().default(true),
  conflict: conflictStrategySchema,
  renameTemplate: templatePipelineSchema.nullable().optional(),
  folderPlan: folderPlanSchema.nullable().optional(),
  watch: watchConfigSchema.nullable().optional(),
  counter: counterConfigSchema.default({}),
  enabled: z.boolean(),
  lastRun: z.string().datetime().nullable(),
});

export const blueprintSummarySchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  kind: blueprintKindSchema,
  rootPath: z.string(),
  operation: blueprintOperationSchema,
  enabled: z.boolean(),
  lastRun: z.string().datetime().nullable(),
  watchEnabled: z.boolean(),
});

export const blueprintPlanSampleSchema = z.object({
  source: z.string(),
  target: z.string(),
  action: z.string(),
});

export const blueprintCollisionSchema = z.object({
  source: z.string(),
  target: z.string(),
});

export const blueprintApplyResultSchema = z.object({
  processed: z.number().int(),
  failed: z.number().int(),
  skipped: z.number().int(),
});

export const blueprintSimulationReportSchema = z.object({
  matched: z.number().int(),
  skipped: z.number().int(),
  planSample: z.array(blueprintPlanSampleSchema),
  warnings: z.array(z.string()),
  collisions: z.array(blueprintCollisionSchema),
});

export const templatePreviewSchema = z.object({
  resultPath: z.string(),
  resultName: z.string(),
  valid: z.boolean(),
  warnings: z.array(z.string()),
});

export const folderPlanPreviewNodeSchema: z.ZodType<FolderPlanPreviewNode, z.ZodTypeDef, unknown> = z.lazy(() =>
  z.object({
    name: z.string(),
    relativePath: z.string(),
    action: folderActionSchema.default("create"),
    sourcePath: z.string().default(""),
    children: z.array(folderPlanPreviewNodeSchema),
  }),
);

export const folderPlanPreviewSchema = z.object({
  rootPath: z.string(),
  nodes: z.array(folderPlanPreviewNodeSchema).default([]),
  folderCount: z.number().int(),
  valid: z.boolean(),
  warnings: z.array(z.string()),
});

export type RoutingConfig = z.infer<typeof routingConfigSchema>;
export type FolderPlan = z.infer<typeof folderPlanSchema>;
export type FolderPlanVariable = z.infer<typeof folderPlanVariableSchema>;
export type FolderAction = z.infer<typeof folderActionSchema>;
export type FolderNode = FolderActionNode;
export type CounterConfig = z.infer<typeof counterConfigSchema>;
export type Blueprint = z.infer<typeof blueprintSchema>;
export type BlueprintSummary = z.infer<typeof blueprintSummarySchema>;
export type BlueprintApplyResult = z.infer<typeof blueprintApplyResultSchema>;
export type BlueprintSimulationReport = z.infer<typeof blueprintSimulationReportSchema>;
export type TemplatePreview = z.infer<typeof templatePreviewSchema>;
export type FolderPlanPreviewNode = {
  name: string;
  relativePath: string;
  action: FolderAction;
  sourcePath: string;
  children: FolderPlanPreviewNode[];
};
export type FolderPlanPreview = z.infer<typeof folderPlanPreviewSchema>;
export type BlueprintKind = z.infer<typeof blueprintKindSchema>;
export type BlueprintOperation = z.infer<typeof blueprintOperationSchema>;

export type TemplateToolboxItem = {
  name: string;
  label: string;
  syntax: string;
  hint: string;
  example?: string;
};

export type TemplateToolboxGroup = {
  id: string;
  label: string;
  items: TemplateToolboxItem[];
};

export function templateSyntaxLegend(): { label: string; syntax: string; hint: string }[] {
  return [
    {
      label: t("blueprints.template.legend.token.label"),
      syntax: "{nome}",
      hint: t("blueprints.template.legend.token.hint"),
    },
    {
      label: t("blueprints.template.legend.transform.label"),
      syntax: "[nome]",
      hint: t("blueprints.template.legend.transform.hint"),
    },
    {
      label: t("blueprints.template.legend.regexPreset.label"),
      syntax: "/nome/",
      hint: t("blueprints.template.legend.regexPreset.hint"),
    },
    {
      label: t("blueprints.template.legend.literal.label"),
      syntax: "texto livre",
      hint: t("blueprints.template.legend.literal.hint"),
    },
  ];
}

export function templateQuickPatterns(): { label: string; value: string }[] {
  return [
    { label: t("blueprints.template.quick.yearMonthFile"), value: "{year}/{month}/{stem}{ext}" },
    { label: t("blueprints.template.quick.dateParentFile"), value: "{date}/{parent}/{stem}{ext}" },
    { label: t("blueprints.template.quick.parentName"), value: "{parent}/{stem}{ext}" },
    {
      label: t("blueprints.template.quick.grandparentParentName"),
      value: "{grandparent}/{parent}/{stem}{ext}",
    },
    { label: t("blueprints.template.quick.sequentialCounter"), value: "{year}/{counter}_{stem}{ext}" },
  ];
}

function token(name: string, syntax: string, example: string): TemplateToolboxItem {
  return {
    name,
    label: t(`blueprints.template.token.${name}.label`),
    syntax,
    hint: t(`blueprints.template.token.${name}.hint`),
    example,
  };
}

function transform(name: string, syntax: string, example: string): TemplateToolboxItem {
  return {
    name,
    label: t(`blueprints.template.transform.${name}.label`),
    syntax,
    hint: t(`blueprints.template.transform.${name}.hint`),
    example,
  };
}

function regexPreset(name: string, example: string): TemplateToolboxItem {
  return {
    name,
    label: t(`blueprints.template.regex.${name}.label`),
    syntax: `/${name}/`,
    hint: t(`blueprints.template.regex.${name}.hint`),
    example,
  };
}

export function templateTokenGroups(): TemplateToolboxGroup[] {
  return [
    {
      id: "datetime",
      label: t("blueprints.template.group.datetime"),
      items: [
        token("date", "{date}", "2026-06-19"),
        token("year", "{year}", "2026"),
        token("month", "{month}", "06"),
        token("day", "{day}", "19"),
        token("hour", "{hour}", "14"),
        token("minute", "{minute}", "05"),
        token("second", "{second}", "42"),
      ],
    },
    {
      id: "file",
      label: t("blueprints.template.group.file"),
      items: [
        token("original", "{original}", "relatorio.PDF"),
        token("stem", "{stem}", "relatorio"),
        token("ext", "{ext}", ".pdf"),
        token("parent", "{parent}", "Docs"),
        token("grandparent", "{grandparent}", "Entrada"),
      ],
    },
    {
      id: "sequence",
      label: t("blueprints.template.group.sequence"),
      items: [
        token("counter", "{counter}", "001"),
        token("index", "{index}", "3"),
        token("guid", "{guid}", "a1b2…"),
      ],
    },
  ];
}

export function templateTransformGroups(): TemplateToolboxGroup[] {
  return [
    {
      id: "case",
      label: t("blueprints.template.group.case"),
      items: [
        transform("lower", "[lower]", "report"),
        transform("upper", "[upper]", "REPORT"),
        transform("capitalized", "[capitalized]", "Relatorio mensal"),
        transform("title", "[title]", "Relatorio Mensal"),
        transform("snake", "[snake]", "meu_arquivo"),
        transform("camel", "[camel]", "meuArquivo"),
        transform("pascal", "[pascal]", "MeuArquivo"),
        transform("kebab", "[kebab]", "meu-arquivo"),
      ],
    },
    {
      id: "substring",
      label: t("blueprints.template.group.substring"),
      items: [
        transform("take", "[take(12)]", "relatorio-fin"),
        transform("take_end", "[take(8, end)]", "2026.pdf"),
        transform("skip", "[skip(3)]", "atorio"),
        transform("skip_end", "[skip(4, end)]", "relatorio"),
        transform("slice", "[slice(0, 8)]", "relatori"),
      ],
    },
    {
      id: "cleanup",
      label: t("blueprints.template.group.cleanup"),
      items: [
        transform("trim", "[trim]", "arquivo"),
        transform("collapse_spaces", "[collapse_spaces]", "meu doc"),
        transform("strip_symbols", "[strip_symbols]", "arquivo.pdf"),
        transform("spaces_to_underscore", "[spaces_to_underscore]", "meu_arquivo"),
        transform("sanitize_windows_filename", "[sanitize_windows_filename]", "arquivo_seguro"),
      ],
    },
  ];
}

export function templateRegexGroups(): TemplateToolboxGroup[] {
  return [
    {
      id: "presets",
      label: t("blueprints.template.group.presets"),
      items: [
        regexPreset("remove_digits", "doc"),
        regexPreset("strip_special", "arquivo_1"),
        regexPreset("extract_date", "2026-06-19"),
        regexPreset("remove_parens", "arquivo final"),
        regexPreset("digits_only", "20260619"),
        regexPreset("letters_only", "relatorio"),
      ],
    },
  ];
}

export function counterScopeOptions(): { value: CounterScope; label: string; hint: string }[] {
  return [
    {
      value: "global",
      label: t("blueprints.template.counter.global.label"),
      hint: t("blueprints.template.counter.global.hint"),
    },
    {
      value: "perDay",
      label: t("blueprints.template.counter.perDay.label"),
      hint: t("blueprints.template.counter.perDay.hint"),
    },
    {
      value: "perFolder",
      label: t("blueprints.template.counter.perFolder.label"),
      hint: t("blueprints.template.counter.perFolder.hint"),
    },
    {
      value: "perParent",
      label: t("blueprints.template.counter.perParent.label"),
      hint: t("blueprints.template.counter.perParent.hint"),
    },
  ];
}

export function folderActionOptions(): { value: FolderAction; label: string; hint: string }[] {
  return [
    { value: "create", label: t("folders.action.create"), hint: t("folders.action.createHint") },
    { value: "rename", label: t("folders.action.rename"), hint: t("folders.action.renameHint") },
    { value: "move", label: t("folders.action.move"), hint: t("folders.action.moveHint") },
    { value: "copy", label: t("folders.action.copy"), hint: t("folders.action.copyHint") },
    { value: "delete", label: t("folders.action.delete"), hint: t("folders.action.deleteHint") },
  ];
}

export function folderActionLabel(action: FolderAction): string {
  return folderActionOptions().find((option) => option.value === action)?.label ?? action;
}

export function createFolderActionNode(action: FolderAction = "create", name = "Nova pasta"): FolderActionNode {
  return folderActionNodeSchema.parse({
    action,
    name,
    sourcePath: "",
    targetPath: "",
    children: [],
  });
}

export function createDefaultFolderPlan(): FolderPlan {
  return folderPlanSchema.parse({
    version: 1,
    variables: [
      {
        name: "projectName",
        label: "Nome do projeto",
        defaultValue: "Projeto exemplo",
        required: true,
      },
    ],
    allowDeleteNonEmpty: false,
    confirmDeleteNonEmpty: false,
    nodes: [
      {
        action: "create",
        name: "{projectName}",
        sourcePath: "",
        targetPath: "",
        children: [
          createFolderActionNode("create", "01 Briefing"),
          createFolderActionNode("create", "02 Execucao"),
          createFolderActionNode("create", "03 Entrega"),
        ],
      },
    ],
  });
}

export function createFolderVariableValues(folderPlan: FolderPlan | null | undefined): Record<string, string> {
  const values: Record<string, string> = {};
  for (const variable of folderPlan?.variables ?? []) {
    const key = variable.name.trim();
    if (!key) continue;
    values[key] = variable.defaultValue ?? "";
  }
  return values;
}

export function createEmptyBlueprint(kind: BlueprintKind = "file"): Blueprint {
  return blueprintSchema.parse({
    id: crypto.randomUUID(),
    name: kind === "file" ? "Nova regra de arquivos" : "Nova estrutura de pastas",
    kind,
    rootPath: "",
    search: fileFilterSchema.parse({}),
    routing: {
      pathTemplate: {
        segments: [
          { kind: "token", name: "year" },
          { kind: "literal", value: "/" },
          { kind: "token", name: "stem" },
          { kind: "token", name: "ext" },
        ],
      },
      createIntermediateDirs: true,
    },
    operation: "move",
    recursive: true,
    conflict: "skip",
    renameTemplate: null,
    folderPlan: kind === "folder" ? createDefaultFolderPlan() : null,
    watch: null,
    counter: counterConfigSchema.parse({}),
    enabled: true,
    lastRun: null,
  });
}

export function countBlueprintFilters(search: FileFilter): number {
  return countActiveFilters(search);
}

const TRANSFORM_NAMES = new Set([
  "upper",
  "lower",
  "capitalized",
  "title",
  "snake",
  "camel",
  "pascal",
  "kebab",
  "trim",
  "collapse_spaces",
  "strip_symbols",
  "spaces_to_underscore",
  "sanitize_windows_filename",
  "take",
  "skip",
  "slice",
]);

const REGEX_PRESET_NAMES = new Set([
  "remove_digits",
  "strip_special",
  "extract_date",
  "remove_parens",
  "digits_only",
  "letters_only",
]);

const ESCAPABLE = new Set(["[", "]", "/", "\\"]);

function normalizeIdentifier(value: string): string {
  let out = "";
  let previousWasSeparator = true;
  for (const ch of value.trim()) {
    if (/[A-Za-z0-9]/.test(ch)) {
      if (ch >= "A" && ch <= "Z" && !previousWasSeparator && !out.endsWith("_")) {
        out += "_";
      }
      out += ch.toLowerCase();
      previousWasSeparator = false;
    } else if (ch === "_" || ch === "-" || /\s/.test(ch)) {
      if (out && !out.endsWith("_")) out += "_";
      previousWasSeparator = true;
    }
  }
  return out.replace(/^_+|_+$/g, "");
}

function isIdentifier(value: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(value);
}

function isKnownTransform(value: string): boolean {
  const base = value.split("(")[0]?.trim() ?? "";
  return TRANSFORM_NAMES.has(normalizeIdentifier(base));
}

function isKnownRegexPreset(value: string): boolean {
  return REGEX_PRESET_NAMES.has(normalizeIdentifier(value));
}

function readUntil(
  input: string,
  start: number,
  delimiter: string,
  stopAtNewline: boolean,
): { content: string; next: number } | null {
  for (let index = start; index < input.length; index += 1) {
    const ch = input[index]!;
    if (ch === delimiter) return { content: input.slice(start, index), next: index + 1 };
    if (stopAtNewline && (ch === "\n" || ch === "\r")) return null;
  }
  return null;
}

function needsSlashEscape(value: string, slashIndex: number): boolean {
  const rest = value.slice(slashIndex + 1);
  const end = rest.indexOf("/");
  if (end < 0) return false;
  const candidate = rest.slice(0, end);
  return isIdentifier(candidate) && isKnownRegexPreset(candidate);
}

function formatLiteral(value: string, followedBySegment: boolean): string {
  let out = "";
  for (let index = 0; index < value.length; index += 1) {
    const ch = value[index]!;
    if (ch === "\\") {
      const next = value[index + 1];
      if ((next && ESCAPABLE.has(next)) || (!next && followedBySegment)) out += "\\\\";
      else out += "\\";
    } else if (ch === "{" || ch === "}") {
      out += `${ch}${ch}`;
    } else if (ch === "[" || ch === "]") {
      out += `\\${ch}`;
    } else if (ch === "/" && needsSlashEscape(value, index)) {
      out += "\\/";
    } else {
      out += ch;
    }
  }
  return out;
}

export function pipelineToDisplayString(pipeline: TemplatePipeline): string {
  return pipeline.segments
    .map((segment, index) => {
      switch (segment.kind) {
        case "literal":
          return formatLiteral(segment.value, index < pipeline.segments.length - 1);
        case "token":
          return `{${segment.name}}`;
        case "transform":
          return `[${segment.name}]`;
        case "regexPreset":
          return `/${segment.name}/`;
      }
    })
    .join("");
}

export function parseTemplateDisplayString(input: string): TemplatePipeline {
  const segments: TemplateSegment[] = [];
  let literal = "";
  let index = 0;

  function flush() {
    if (!literal) return;
    segments.push({ kind: "literal", value: literal });
    literal = "";
  }

  while (index < input.length) {
    const ch = input[index]!;

    if (ch === "\\" && index + 1 < input.length && ESCAPABLE.has(input[index + 1]!)) {
      literal += input[index + 1];
      index += 2;
      continue;
    }

    if (ch === "{" && input[index + 1] === "{") {
      literal += "{";
      index += 2;
      continue;
    }

    if (ch === "}" && input[index + 1] === "}") {
      literal += "}";
      index += 2;
      continue;
    }

    if (ch === "{") {
      const found = readUntil(input, index + 1, "}", false);
      if (found && isIdentifier(found.content)) {
        flush();
        segments.push({ kind: "token", name: found.content });
        index = found.next;
        continue;
      }
      literal += ch;
      index += 1;
      continue;
    }

    if (ch === "[") {
      const found = readUntil(input, index + 1, "]", true);
      const transform = found?.content.trim() ?? "";
      if (found && transform && isKnownTransform(transform)) {
        flush();
        segments.push({ kind: "transform", name: transform });
        index = found.next;
        continue;
      }
      literal += ch;
      index += 1;
      continue;
    }

    if (ch === "/") {
      const found = readUntil(input, index + 1, "/", true);
      if (found && isIdentifier(found.content) && isKnownRegexPreset(found.content)) {
        flush();
        segments.push({ kind: "regexPreset", name: found.content });
        index = found.next;
        continue;
      }
      literal += ch;
      index += 1;
      continue;
    }

    literal += ch;
    index += 1;
  }

  flush();
  return { segments };
}

export function insertTokenInDisplayString(current: string, token: string, cursor: number): string {
  const insertion = `{${token}}`;
  return current.slice(0, cursor) + insertion + current.slice(cursor);
}

export function blueprintKindLabel(kind: BlueprintKind): string {
  return kind === "file" ? t("blueprints.files") : t("blueprints.folders");
}

export function blueprintOperationLabel(operation: BlueprintOperation): string {
  return operation === "copy" ? t("flows.copy") : t("flows.move");
}
