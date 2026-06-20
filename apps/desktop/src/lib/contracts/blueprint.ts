import { z } from "zod";
import {
  conflictStrategySchema,
  countActiveFilters,
  fileFilterSchema,
  watchConfigSchema,
  type FileFilter,
} from "$lib/contracts/job";

export const blueprintKindSchema = z.enum(["file", "folder"]);
export const blueprintOperationSchema = z.enum(["move", "copy"]);
export const counterScopeSchema = z.enum(["global", "perDay", "perFolder", "perParent"]);

export const templateSegmentSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("literal"), value: z.string() }),
  z.object({ kind: z.literal("token"), name: z.string() }),
  z.object({ kind: z.literal("transform"), name: z.string() }),
  z.object({ kind: z.literal("regexPreset"), name: z.string() }),
]);

export const templatePipelineSchema = z.object({
  segments: z.array(templateSegmentSchema).default([]),
});

export const routingConfigSchema = z.object({
  pathTemplate: templatePipelineSchema.default({ segments: [] }),
  createIntermediateDirs: z.boolean().default(true),
});

export type FolderNode = {
  name: string;
  children: FolderNode[];
};

export const folderNodeSchema: z.ZodType<FolderNode> = z.lazy(() =>
  z.object({
    name: z.string(),
    children: z.array(folderNodeSchema),
  }),
);

export const folderPlanSchema = z.object({
  nodes: z.array(folderNodeSchema).default([]),
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

export const folderPlanPreviewNodeSchema: z.ZodType<FolderPlanPreviewNode> = z.lazy(() =>
  z.object({
    name: z.string(),
    relativePath: z.string(),
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

export type TemplateSegment = z.infer<typeof templateSegmentSchema>;
export type TemplatePipeline = z.infer<typeof templatePipelineSchema>;
export type RoutingConfig = z.infer<typeof routingConfigSchema>;
export type FolderPlan = z.infer<typeof folderPlanSchema>;
export type CounterConfig = z.infer<typeof counterConfigSchema>;
export type Blueprint = z.infer<typeof blueprintSchema>;
export type BlueprintSummary = z.infer<typeof blueprintSummarySchema>;
export type BlueprintSimulationReport = z.infer<typeof blueprintSimulationReportSchema>;
export type TemplatePreview = z.infer<typeof templatePreviewSchema>;
export type FolderPlanPreviewNode = {
  name: string;
  relativePath: string;
  children: FolderPlanPreviewNode[];
};
export type FolderPlanPreview = z.infer<typeof folderPlanPreviewSchema>;
export type BlueprintKind = z.infer<typeof blueprintKindSchema>;
export type BlueprintOperation = z.infer<typeof blueprintOperationSchema>;

export const TEMPLATE_TOKENS = [
  { name: "year", label: "Ano" },
  { name: "month", label: "Mês" },
  { name: "day", label: "Dia" },
  { name: "hour", label: "Hora" },
  { name: "minute", label: "Minuto" },
  { name: "second", label: "Segundo" },
  { name: "original", label: "Nome original" },
  { name: "stem", label: "Nome sem extensão" },
  { name: "ext", label: "Extensão" },
  { name: "parent", label: "Pasta pai" },
  { name: "counter", label: "Contador" },
  { name: "guid", label: "GUID" },
  { name: "index", label: "Índice" },
] as const;

export const TEMPLATE_TRANSFORMS = [
  { name: "lower", label: "minúsculas" },
  { name: "upper", label: "MAIÚSCULAS" },
  { name: "snake", label: "snake_case" },
  { name: "kebab", label: "kebab-case" },
  { name: "trim", label: "trim" },
  { name: "collapse_spaces", label: "colapsar espaços" },
  { name: "sanitize_windows_filename", label: "sanitizar Windows" },
] as const;

export const TEMPLATE_REGEX_PRESETS = [
  { name: "removeDigits", label: "remover dígitos" },
  { name: "stripSpecial", label: "remover especiais" },
] as const;

export function createEmptyBlueprint(kind: BlueprintKind = "file"): Blueprint {
  return blueprintSchema.parse({
    id: crypto.randomUUID(),
    name: kind === "file" ? "Novo blueprint de arquivos" : "Novo blueprint de pastas",
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
    folderPlan: kind === "folder" ? { nodes: [] } : null,
    watch: null,
    counter: counterConfigSchema.parse({}),
    enabled: true,
    lastRun: null,
  });
}

export function countBlueprintFilters(search: FileFilter): number {
  return countActiveFilters(search);
}

export function pipelineToDisplayString(pipeline: TemplatePipeline): string {
  return pipeline.segments
    .map((segment) => {
      switch (segment.kind) {
        case "literal":
          return segment.value;
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
  const pattern = /\{([a-zA-Z_]+)\}|\[([a-zA-Z_]+)\]|\/([a-zA-Z_]+)\//g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(input)) !== null) {
    if (match.index > lastIndex) {
      segments.push({ kind: "literal", value: input.slice(lastIndex, match.index) });
    }
    if (match[1]) {
      segments.push({ kind: "token", name: match[1] });
    } else if (match[2]) {
      segments.push({ kind: "transform", name: match[2] });
    } else if (match[3]) {
      segments.push({ kind: "regexPreset", name: match[3] });
    }
    lastIndex = pattern.lastIndex;
  }

  if (lastIndex < input.length) {
    segments.push({ kind: "literal", value: input.slice(lastIndex) });
  }

  return { segments };
}

export function insertTokenInDisplayString(current: string, token: string, cursor: number): string {
  const insertion = `{${token}}`;
  return current.slice(0, cursor) + insertion + current.slice(cursor);
}

export function blueprintKindLabel(kind: BlueprintKind): string {
  return kind === "file" ? "Arquivos" : "Pastas";
}

export function blueprintOperationLabel(operation: BlueprintOperation): string {
  return operation === "copy" ? "Cópia" : "Mover";
}
