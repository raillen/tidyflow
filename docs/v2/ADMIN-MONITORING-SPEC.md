# Painel Administrativo e Monitoramento

Este documento define a direção do painel que gerencia instâncias do AutoFlow em várias máquinas da rede.

## Objetivo

Permitir que um administrador veja, monitore e opere instalações do AutoFlow a partir de um painel local ou web.

O modelo esperado é:

1. Cada instalação roda como uma instância/agent.
2. A instância envia heartbeat, inventário, logs e status.
3. O painel central web recebe os dados e registra comandos.
4. O agent busca ou recebe comandos e executa apenas ações permitidas.

## Primeira entrega implementada

A primeira fundação já existe no app desktop:

- contrato `AdminFleetSnapshot`;
- contrato `AdminCommandRequest`;
- contrato `AdminSignedEnvelope`;
- rota `/admin`;
- comando IPC `admin_fleet_snapshot`;
- comando IPC `admin_heartbeat_payload`;
- comando IPC `admin_signed_heartbeat_payload`;
- comando IPC `admin_send_signed_heartbeat_once`;
- comandos IPC `admin_agent_secret_generate`, `admin_agent_secret_set` e `admin_agent_secret_clear`;
- comando IPC `admin_dispatch_command`;
- comandos IPC para enfileirar, listar, resumir e processar a fila local;
- leitura local de máquina, jobs, execuções ativas e capacidades;
- execução local de `startJob`;
- cancelamento local de `cancelExecution`;
- execução local de `pauseJob`, `resumeJob`, `stopJob` e `deleteJob`;
- configurações persistidas do modo Admin/Agent;
- segredo do agent salvo no keyring/cofre do sistema operacional;
- auditoria local para comandos admin diretos, enfileirados e processados;
- comandos que dependem de servidor web ainda modelados como planejados.

## Agent e fila local

Tambem ja existem os contratos:

- `AdminEnrollmentRequest`;
- `AdminEnrollmentResponse`;
- `AdminHeartbeatPayload`;
- `AdminQueuedCommand`;
- `AdminCommandQueueSummary`.

O heartbeat contem o snapshot da instancia local e a quantidade de comandos pendentes.

O envelope assinado (`AdminSignedEnvelope`) registra:

- versao do contrato;
- tipo do payload;
- instancia emissora;
- data de emissao;
- data de expiracao;
- nonce;
- hash do payload;
- assinatura `blake3`.

O segredo de assinatura vem do cofre do sistema operacional. Ele nao e salvo em texto claro no SQLite. O SQLite guarda apenas o indicador `enrollmentTokenConfigured` para a UI.

O envio manual de heartbeat assinado usa:

```text
POST {serverUrl}/api/agents/{instanceId}/heartbeat
```

O corpo enviado e o `AdminSignedEnvelope<AdminHeartbeatPayload>`.

A fila local usa SQLite e registra:

- id do comando;
- origem (`local-ui`, `server`, etc.);
- payload do comando;
- status: pendente, executando, concluido, falhou ou ignorado;
- resultado final;
- datas de criacao e atualizacao.

O painel local ja consegue enfileirar comandos e processar o proximo pendente. No modo web, o servidor central deve inserir comandos assinados nessa fila.

## Servidor admin minimo

O crate `autoflow-admin-server` inicia a base HTTP do painel central.

Nesta etapa ele pode rodar de duas formas:

- estado em memoria para testes rapidos;
- SQLite para persistir agents cadastrados e o ultimo snapshot recebido.

Isso permite validar o contrato entre agent e servidor antes de fixar RBAC, deployment web e o painel central completo.

Endpoints implementados:

```text
GET /health
GET /api/fleet
POST /api/enrollments
POST /api/agents/{instanceId}/heartbeat
```

O endpoint de matricula recebe `AdminEnrollmentTokenRequest`, valida o token de convite e cadastra o segredo inicial enviado pelo agent.

O endpoint de heartbeat recebe `AdminSignedEnvelope<AdminHeartbeatPayload>` e valida:

- agent registrado no servidor;
- tipo do envelope igual a `heartbeat`;
- `instanceId` da rota igual ao `instanceId` do envelope;
- envelope ainda nao expirado;
- assinatura `blake3` com o segredo do agent;
- `instanceId` do payload igual ao da rota.

Quando o heartbeat e aceito, o servidor atualiza o ultimo snapshot da instancia e retorna `AdminHeartbeatAccepted`.

O cadastro de segredo do agent existe como metodo interno do estado do servidor. Ele ainda nao foi exposto por HTTP porque matricula por token/convite precisa de autenticacao propria.

No backend SQLite, a tabela `admin_agents` guarda:

- `instance_id`;
- segredo do agent;
- ultimo snapshot recebido;
- primeira data de cadastro;
- ultima data vista pelo servidor.

Antes de producao, o segredo central deve migrar para cofre, KMS ou criptografia em repouso.

Execucao local do servidor:

```text
cargo run -p autoflow-admin-server
```

Variaveis de configuracao:

```text
AUTOFLOW_ADMIN_BIND=127.0.0.1:7840
AUTOFLOW_ADMIN_DB=autoflow-admin.sqlite
AUTOFLOW_ADMIN_ENROLLMENT_TOKEN=convite-temporario
AUTOFLOW_ADMIN_BOOTSTRAP_INSTANCE_ID=local-...
AUTOFLOW_ADMIN_BOOTSTRAP_SECRET=af_...
```

O bind padrao usa `127.0.0.1` para desenvolvimento local. Para rede ou web, usar HTTPS, firewall e autenticacao antes de expor o servico.

## Dados por instância

Cada instância deve expor:

- `instanceId` estável;
- nome da máquina;
- status online, atenção ou offline;
- último heartbeat;
- sistema operacional;
- arquitetura;
- threads disponíveis;
- memória total quando disponível;
- versão do AutoFlow;
- modo de gerenciamento;
- URL do servidor admin quando configurada;
- permissões de comandos remotos e ações em lote;
- domínio/rede;
- interfaces ou endereços conhecidos;
- fluxos cadastrados;
- execuções ativas;
- capacidades administrativas.

## Comandos administrativos

Comandos modelados:

- iniciar fluxo;
- cancelar execução;
- pausar fluxo;
- continuar fluxo;
- parar fluxo;
- criar fluxo remoto;
- editar fluxo remoto;
- deletar fluxo remoto;
- aplicar política de configuração;
- solicitar logs.

Nem todo comando precisa estar disponível em todas as instâncias. O agent deve publicar suas capacidades.

## Segurança mínima

Este painel controla máquinas reais. Nenhum comando remoto deve ser tratado como simples chamada de UI.

Regras mínimas:

- registro inicial por convite ou token;
- `instanceId` estável por instalação;
- segredo local salvo no cofre do sistema operacional;
- HTTPS obrigatório no modo web;
- comandos assinados ou vinculados a sessão autenticada;
- RBAC no painel central;
- auditoria de quem executou cada comando;
- opção local para aceitar apenas monitoramento;
- confirmação para comandos destrutivos ou em lote.

## Arquitetura recomendada

```text
AutoFlow Agent
  -> heartbeat
  -> inventário
  -> logs
  -> status de fluxos
  <- comandos assinados

Admin Server
  -> API HTTP
  -> WebSocket/SSE
  -> fila de comandos
  -> auditoria central
  -> PostgreSQL

Admin Web
  -> dashboard
  -> máquinas
  -> fluxos
  -> logs
  -> ações em lote
```

## Próximos cortes

1. Implementar rotacao de segredo do agent.
2. Automatizar sincronizacao periodica do heartbeat assinado.
3. Adicionar grupos de maquinas e acoes em lote multi-instancia.
4. Adicionar RBAC e auditoria central.
5. Implementar edicao remota de fluxos com validacao e previa.
