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

export const TEMPLATE_SYNTAX_LEGEND = [
  { label: "Token", syntax: "{nome}", hint: "Insere valor dinâmico do arquivo ou data." },
  { label: "Transformação", syntax: "[nome]", hint: "Aplica estilo ao trecho acumulado à esquerda." },
  { label: "Regex preset", syntax: "/nome/", hint: "Limpeza por regex pré-definida." },
  { label: "Literal", syntax: "texto livre", hint: "Separadores como / ou _ entre tokens." },
] as const;

export const TEMPLATE_QUICK_PATTERNS = [
  { label: "Ano / mês / arquivo", value: "{year}/{month}/{stem}{ext}" },
  { label: "Data / pasta pai", value: "{date}/{parent}/{stem}{ext}" },
  { label: "Pasta pai / nome", value: "{parent}/{stem}{ext}" },
  { label: "Avó / pai / nome", value: "{grandparent}/{parent}/{stem}{ext}" },
  { label: "Contador sequencial", value: "{year}/{counter}_{stem}{ext}" },
] as const;

export const TEMPLATE_TOKEN_GROUPS: TemplateToolboxGroup[] = [
  {
    id: "datetime",
    label: "Data e hora",
    items: [
      { name: "date", label: "Data", syntax: "{date}", hint: "Data atual no formato YYYY-MM-DD.", example: "2026-06-19" },
      { name: "year", label: "Ano", syntax: "{year}", hint: "Ano com 4 dígitos (relógio local).", example: "2026" },
      { name: "month", label: "Mês", syntax: "{month}", hint: "Mês com 2 dígitos.", example: "06" },
      { name: "day", label: "Dia", syntax: "{day}", hint: "Dia do mês com 2 dígitos.", example: "19" },
      { name: "hour", label: "Hora", syntax: "{hour}", hint: "Hora 00–23.", example: "14" },
      { name: "minute", label: "Minuto", syntax: "{minute}", hint: "Minuto 00–59.", example: "05" },
      { name: "second", label: "Segundo", syntax: "{second}", hint: "Segundo 00–59.", example: "42" },
    ],
  },
  {
    id: "file",
    label: "Arquivo",
    items: [
      { name: "original", label: "Nome original", syntax: "{original}", hint: "Nome completo com extensão.", example: "relatorio.PDF" },
      { name: "stem", label: "Nome sem ext.", syntax: "{stem}", hint: "Nome sem extensão.", example: "relatorio" },
      { name: "ext", label: "Extensão", syntax: "{ext}", hint: "Extensão com ponto.", example: ".pdf" },
      { name: "parent", label: "Pasta pai", syntax: "{parent}", hint: "Nome da pasta imediatamente acima do arquivo.", example: "Docs" },
      { name: "grandparent", label: "Pasta avó", syntax: "{grandparent}", hint: "Nome da pasta acima da pasta pai.", example: "Entrada" },
    ],
  },
  {
    id: "sequence",
    label: "Sequência",
    items: [
      { name: "counter", label: "Contador", syntax: "{counter}", hint: "Número sequencial conforme escopo configurado.", example: "001" },
      { name: "index", label: "Índice", syntax: "{index}", hint: "Posição do item no lote atual (1-based).", example: "3" },
      { name: "guid", label: "GUID", syntax: "{guid}", hint: "Identificador único por avaliação.", example: "a1b2…" },
    ],
  },
];

export const TEMPLATE_TRANSFORM_GROUPS: TemplateToolboxGroup[] = [
  {
    id: "case",
    label: "Caixa e formato",
    items: [
      { name: "lower", label: "minúsculas", syntax: "[lower]", hint: "Converte todo o trecho para minúsculas.", example: "report" },
      { name: "upper", label: "MAIÚSCULAS", syntax: "[upper]", hint: "Converte todo o trecho para maiúsculas.", example: "REPORT" },
      { name: "capitalized", label: "Capitalizado", syntax: "[capitalized]", hint: "Primeira letra em maiúscula e restante em minúscula.", example: "Relatorio mensal" },
      { name: "title", label: "Title Case", syntax: "[title]", hint: "Primeira letra de cada palavra em maiúscula.", example: "Relatorio Mensal" },
      { name: "snake", label: "snake_case", syntax: "[snake]", hint: "Espaços e hífens viram underscore.", example: "meu_arquivo" },
      { name: "camel", label: "camelCase", syntax: "[camel]", hint: "Formato camelCase para nomes técnicos.", example: "meuArquivo" },
      { name: "pascal", label: "PascalCase", syntax: "[pascal]", hint: "Formato PascalCase para nomes técnicos.", example: "MeuArquivo" },
      { name: "kebab", label: "kebab-case", syntax: "[kebab]", hint: "Formato com hífens.", example: "meu-arquivo" },
    ],
  },
  {
    id: "substring",
    label: "Recorte",
    items: [
      { name: "take", label: "Pegar início", syntax: "[take(12)]", hint: "Mantém os primeiros N caracteres.", example: "relatorio-fin" },
      { name: "take_end", label: "Pegar fim", syntax: "[take(8, end)]", hint: "Mantém os últimos N caracteres.", example: "2026.pdf" },
      { name: "skip", label: "Pular início", syntax: "[skip(3)]", hint: "Remove os primeiros N caracteres.", example: "atorio" },
      { name: "skip_end", label: "Pular fim", syntax: "[skip(4, end)]", hint: "Remove os últimos N caracteres.", example: "relatorio" },
      { name: "slice", label: "Fatiar", syntax: "[slice(0, 8)]", hint: "Mantém caracteres do índice inicial até o final exclusivo.", example: "relatori" },
    ],
  },
  {
    id: "cleanup",
    label: "Limpeza",
    items: [
      { name: "trim", label: "trim", syntax: "[trim]", hint: "Remove espaços no início e fim.", example: "arquivo" },
      { name: "collapse_spaces", label: "colapsar espaços", syntax: "[collapse_spaces]", hint: "Múltiplos espaços viram um só.", example: "meu doc" },
      { name: "strip_symbols", label: "remover símbolos", syntax: "[strip_symbols]", hint: "Remove símbolos e preserva letras, números e separadores comuns.", example: "arquivo.pdf" },
      { name: "spaces_to_underscore", label: "espaços para _", syntax: "[spaces_to_underscore]", hint: "Troca espaços por underscore.", example: "meu_arquivo" },
      { name: "sanitize_windows_filename", label: "sanitizar Windows", syntax: "[sanitize_windows_filename]", hint: "Substitui caracteres inválidos no Windows.", example: "arquivo_seguro" },
    ],
  },
];

export const TEMPLATE_REGEX_GROUPS: TemplateToolboxGroup[] = [
  {
    id: "presets",
    label: "Presets",
    items: [
      { name: "remove_digits", label: "remover dígitos", syntax: "/remove_digits/", hint: "Remove todos os números do trecho.", example: "doc" },
      { name: "strip_special", label: "remover especiais", syntax: "/strip_special/", hint: "Mantém letras, números, ponto, hífen e underscore.", example: "arquivo_1" },
      { name: "extract_date", label: "extrair data", syntax: "/extract_date/", hint: "Retorna a primeira data encontrada no trecho.", example: "2026-06-19" },
      { name: "remove_parens", label: "remover parênteses", syntax: "/remove_parens/", hint: "Remove blocos entre parênteses.", example: "arquivo final" },
      { name: "digits_only", label: "somente dígitos", syntax: "/digits_only/", hint: "Mantém apenas números.", example: "20260619" },
      { name: "letters_only", label: "somente letras", syntax: "/letters_only/", hint: "Mantém apenas letras.", example: "relatorio" },
    ],
  },
];

/** @deprecated Use TEMPLATE_TOKEN_GROUPS — mantido para compatibilidade. */
export const TEMPLATE_TOKENS = TEMPLATE_TOKEN_GROUPS.flatMap((group) => group.items).map(({ name, label }) => ({
  name,
  label,
}));

/** @deprecated Use TEMPLATE_TRANSFORM_GROUPS */
export const TEMPLATE_TRANSFORMS = TEMPLATE_TRANSFORM_GROUPS.flatMap((group) => group.items).map(({ name, label }) => ({
  name,
  label,
}));

/** @deprecated Use TEMPLATE_REGEX_GROUPS */
export const TEMPLATE_REGEX_PRESETS = TEMPLATE_REGEX_GROUPS.flatMap((group) => group.items).map(({ name, label }) => ({
  name,
  label,
}));

export const COUNTER_SCOPE_OPTIONS = [
  { value: "global", label: "Global", hint: "Um contador único para todo o blueprint." },
  { value: "perDay", label: "Por dia", hint: "Reinicia a cada dia (YYYY-MM-DD)." },
  { value: "perFolder", label: "Por pasta destino", hint: "Sequência independente por pasta de destino." },
  { value: "perParent", label: "Por pasta pai", hint: "Sequência por pasta pai do arquivo de origem." },
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
  const pattern = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}|\[([^\]\r\n]+)\]|\/([a-zA-Z_][a-zA-Z0-9_]*)\//g;
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
