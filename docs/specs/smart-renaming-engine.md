# Especificação: Motor de Renomeação Inteligente (Smart Renaming Engine)

Este documento detalha as novas funcionalidades do motor de renomeação de arquivos do AutoFlow, elevando o sistema de templates básicos para um motor de regras robusto.

## 1. Tokens Expandidos
Além dos tokens básicos, o sistema suportará:
- `{Date}`: Data atual (yyyy-MM-dd).
- `{Time}`: Hora atual (HH-mm-ss).
- `{DateTime}`: Data e Hora completa.
- `{FileName}`: Nome original do arquivo (sem extensão).
- `{Ext}`: Extensão original (sem o ponto).
- `{Parent}`: Nome da pasta pai.
- `{GUID}`: Identificador único universal.
- `{Counter}`: Contador sequencial (útil para resolver conflitos).
- `{JobName}`: Nome do Fluxo ou Blueprint.

## 2. Modificadores de Texto
Os tokens de texto (`{FileName}`, `{Parent}`, `{JobName}`) suportarão modificadores após o caractere `:`:
- `Upper`: Transforma em MAIÚSCULO.
- `Lower`: Transforma em minúsculo.
- `Title`: Transforma em Title Case (Primeira Letra De Cada Palavra Maiúscula).
- `Snake`: Transforma em snake_case.
- `Kebab`: Transforma em kebab-case.

Exemplo: `{FileName:Upper}_{Date}` -> `PROJETO_2024-05-20.pdf`

## 3. Preview em Tempo Real (UI)
- O editor de Blueprint exibirá um exemplo dinâmico conforme o usuário digita.
- Validação visual imediata para evitar erros de sintaxe.

## 4. Resolução de Conflitos Automática
- Se o arquivo final já existir, o sistema tentará anexar um sufixo numérico automaticamente ou usar o token `{Counter}` se presente no template.

## 5. Implementação Técnica
- Atualização do `OrganizationService.cs` para suportar o novo parser de tokens.
- Atualização do `BlueprintEditorViewModel.cs` para fornecer a lógica de preview.
- Atualização do `BlueprintEditorWindow.axaml` para exibir o preview formatado.
