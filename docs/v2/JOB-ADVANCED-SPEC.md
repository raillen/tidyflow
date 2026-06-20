# Spec — Job avançado (Fluxos v2)

**Status:** `Implementado (exceto Watch)`  
**Escopo:** Filtros, smart sync, agendamento, segurança, scripts, webhooks, modal redimensionável.

Watch permanece **módulo separado** — ver ROADMAP Fase 2.1.

---

## 1. Modelo `Job`

| Bloco | Tipo | Descrição |
|-------|------|-----------|
| Core | `name`, `source_path`, `target_path`, `mode`, `conflict`, `enabled` | Igual MVP |
| `filters` | `FileFilter` | Todos os filtros de arquivo |
| `options` | `TransferOptions` | Smart sync, hash, pacote AES |
| `schedule` | `ScheduleConfig?` | Agendamento (sem watch) |
| `scripts` | `ScriptsConfig` | Hooks pré/pós |
| `notify` | `NotifyConfig` | Discord, Telegram, Generic |
| `last_run`, `next_run` | `DateTime?` | UTC |

---

## 2. FileFilter

- **include_extensions** — vazio = todas  
- **exclude_patterns** — glob editável (`**/node_modules/**`)  
- **exclude_preset_ids** — presets fixos + custom salvos no job  
- **name_regex**, **path_regex** — opcionais  
- **min_size_bytes**, **max_size_bytes**  
- **max_depth** — profundidade relativa à origem  
- **modified_after/before**, **created_after/before**  
- **older_than_days**  
- **content_contains** + **content_max_bytes** (default 5MB) + **content_extensions**  
- **recursive**  
- **include_hidden**  
- **symlink_mode**: `follow` \| `copyLink` \| `skip`  
- **skip_empty_files**

Presets fixos: `node_modules`, `git`, `temp`, `system`, `build`.

---

## 3. TransferOptions

- **smart_sync** — pipeline size → mtime → BLAKE3  
- **strict_hash_sync** — sempre BLAKE3 quando smart sync ativo  
- **verify_after_copy** — hash pós-cópia (BLAKE3)  
- **stop_on_integrity_error** — aborta job no primeiro hash fail  
- **encrypt_output** — pacote `.autoflow.zip` AES-256 após transferência  
- **encrypt_password** — enviado na execução manual; persistido via keyring se `remember_encrypt_password`  
- **remove_files_after_pack**  
- **pack_filename** — opcional  

---

## 4. ScheduleConfig

- **enabled**, **timezone** (IANA + `local` = SO)  
- **rule**: `interval { minutes }` \| `daily { hour, minute }` \| `weekly { days[], hour, minute }` (UI: **Dias da Semana**)  
- **missed_run_policy**: notificar tray (não executa catch-up automático no MVP)

---

## 5. ScriptsConfig

- Scripts em `{app_data}/scripts/`  
- **pre_script**, **post_script** — nome do arquivo  
- **timeout_secs** — default 60, range 5–600  
- Pré exit ≠ 0 → aborta; pós exit ≠ 0 → warning no audit  

---

## 6. NotifyConfig

Eventos: `started`, `completed`, `failed`.

Canais:
- **discord** — webhook URL  
- **telegram** — bot_token + chat_id (token no keyring se remember)  
- **generic** — URL + headers opcionais  

Sem WhatsApp. Falha de webhook não falha o job.

---

## 7. UI — Modal de fluxo

- Variante `large`: ~80% viewport, min 640×480  
- Resize 8 vias, persistência em `ui_state` (SQLite)  
- Nav lateral colapsável: Geral | Filtros | Automação | Segurança | Avançado  
- Seções colapsáveis dentro de cada nav  
- Footer: resumo + Simular + Salvar  

---

## 8. IPC

| Command | Descrição |
|---------|-----------|
| `jobs_*` | CRUD, `run`, `simulate` (por id) |
| `jobs_simulate_draft` | Simulação com payload `Job` (criar/editar sem salvar) |
| `ui_state_get` / `ui_state_save` | Persistência modal (tamanho, nav colapsada) |
| `jobs_list_missed_schedules` / `jobs_clear_missed_schedules` | Catch-up tray + banner in-app |

---

## 9. Fora de escopo (esta entrega)

- Módulo Watch  
- WhatsApp webhook  
- SHA-256 alternativo (BLAKE3 padrão)  
- i18n Paraglide completo  
