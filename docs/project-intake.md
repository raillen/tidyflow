# Project Intake: AutoFlow

Preencha este documento antes de escrever o PRD tecnico.

---

## 1. Identificacao

- Nome do projeto: AutoFlow
- Nome curto ou slug: autoflow
- Tipo de produto: `desktop` (Windows/Linux)
- Status atual: `ideia` / `roadmap`

---

## 2. Problema

- Qual problema real este projeto resolve?
- Automação de cópia, movimentação e backup de arquivos baseada em eventos (Watch Folder) ou agendamento, com interface amigável.
- O que acontece hoje sem ele?
- Usuários precisam fazer backups manuais ou usar scripts complexos/ferramentas sem interface visual intuitiva para monitoramento em tempo real.
- Por que isso importa agora?
- Necessidade de uma ferramenta cross-platform (Windows/Linux) moderna, performática e extensível em C#.

---

## 3. Usuario e contexto

- Quem vai usar?
- Usuários de desktop (Windows/Linux) que precisam organizar arquivos ou realizar backups automáticos.
- Em que ambiente o produto sera usado?
- Desktop local.
- Qual e o principal caso de uso?
- Monitorar uma pasta de origem e copiar/mover novos arquivos para um destino seguindo regras de filtro.
- Qual e o menor resultado util que esse usuario espera?
- Um arquivo movido/copiado automaticamente da origem para o destino após detecção ou agendamento.

---

## 4. Escopo

- O que obrigatoriamente precisa entrar no MVP?
- Avalonia UI, CRUD de Jobs, Watch Folder, Agendamento simples, Persistência JSON, Logs básicos.
- O que seria desejavel, mas nao essencial?
- SQLite, Quartz.NET, Rollback sofisticado, Hash SHA256.
- O que deve ficar explicitamente fora do MVP?
- Versão mobile, sincronização em nuvem nativa (S3/Drive), interface web.

---

## 5. Restricoes

- Prazo: N/A
- Orcamento: N/A
- Restricoes de stack: .NET 8, C#, Avalonia UI.
- Restricoes de seguranca: Permissões de sistema de arquivos.
- Plataformas obrigatorias: Windows, Linux.
- Integracoes obrigatorias: Sistema de arquivos nativo.

---

## 6. Estado tecnico atual

- Linguagem principal atual: C#
- Frameworks atuais: .NET 8, Avalonia UI (proposto)
- Banco ou persistencia atual: Nenhuma (JSON planejado para MVP)
- Estrutura atual do repositorio: Documentação inicial.
- Principais lacunas tecnicas: Implementação da engine de execução e watcher cross-platform robusto.
- Principais riscos tecnicos: Consistência do FileSystemWatcher em diferentes SOs, performance em pastas com muitos arquivos.

---

## 7. Decisao de MVP

Preencha sem deixar varias opcoes em aberto.

- Arquitetura do MVP: Clean Architecture leve (4 camadas: Presentation, Application, Domain, Infrastructure).
- Stack principal: .NET 8, C# 12, Avalonia UI, MVVM.
- Persistencia do MVP: JSON (settings, jobs, profiles, history).
- Estrategia de deploy ou distribuicao: Binários nativos para Windows/Linux.
- O que fica para pos-MVP: SQLite, Quartz.NET, Rollback avançado, Hash SHA256, UI polida final.

---

## 8. Backlog bruto

Liste aqui todas as capacidades desejadas sem se preocupar com prioridade ainda.

- Item 1: CRUD de Jobs (Origem, Destino, Modo, Filtros).
- Item 2: Engine de execução (Copy, Move).
- Item 3: Watch Service (Native/Polling).
- Item 4: Scheduler Service (Recorrência, Intervalos).
- Item 5: Log Service (Real-time, Arquivos).
- Item 6: Preview Engine (Simulação de ações).
- Item 7: Notificações de sistema.
- Item 8: Gerenciamento de conflitos (Overwrite, Skip, Rename).

---

## 9. Fechamento

Depois de preencher este intake, responda em uma frase:

- Qual e a menor versao do produto que ainda gera valor real?
- Um app desktop que monitora uma pasta e copia arquivos filtrados para outra pasta automaticamente com logs de sucesso/erro.
