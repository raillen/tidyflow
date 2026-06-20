# Documentation Kit

Este diretorio contem templates reutilizaveis para montar um pacote de documentacao tecnica em qualquer projeto.

## Arquivos incluidos

- `project-intake-template.md`
- `prd-tecnico-template.md`
- `ADR-001-template.md`
- `spec-contratos-template.md`
- `sprint-0-plan-template.md`

## Ordem de uso

1. preencher `project-intake-template.md`
2. fechar a arquitetura do MVP
3. preencher `prd-tecnico-template.md`
4. preencher `ADR-001-template.md`
5. preencher `spec-contratos-template.md`
6. preencher `sprint-0-plan-template.md`

## Regra pratica

Se algum template ficar com muitos `OU`, `TALVEZ` ou multiplas opcoes abertas, o projeto ainda nao tomou decisoes suficientes para virar plano executavel.

## Estrutura recomendada no projeto de destino

```text
docs/
  adr/
    ADR-001-<tema>.md
  v2/                          # reimplementação (ex.: Tauri + Svelte)
    README.md
    ARCHITECTURE.md
    STACK.md
    DOMAIN.md
    API-IPC.md
    UI-DESIGN-SYSTEM.md
    PROJECT-STRUCTURE.md
    ROADMAP.md
  specs/
    <contrato-principal>.md
    schemas/
  plans/
    sprint-0-plan.md
```

## AutoFlow v2

Pacote completo da reimplementação Tauri + Svelte + Rust: [`docs/v2/README.md`](../v2/README.md).
