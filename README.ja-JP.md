<p align="center">
  <img src="./icon.svg" alt="TidyFlow ロゴ" width="88" height="88">
</p>

<h1 align="center">TidyFlow</h1>

<p align="center">
  ファイルの整理、移動、コピー、監査をローカルで扱うデスクトップ自動化アプリです。
</p>

<p align="center">
  <a href="./README.md">English</a>
  ·
  <a href="./README.pt-BR.md">Português</a>
  ·
  <a href="./README.ja-JP.md">日本語</a>
</p>

<p align="center">
  <img alt="バージョン" src="https://img.shields.io/badge/version-0.2.1--alpha-blue">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB">
  <img alt="Svelte" src="https://img.shields.io/badge/Svelte-5-FF3E00">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-000000">
  <img alt="ライセンス" src="https://img.shields.io/badge/license-package--level-green">
</p>

<p align="center">
  <img src="./interface-uiv2.png" alt="ダッシュボードと自動化パネルを表示した TidyFlow のデスクトップ画面">
</p>

## メニュー

- [概要](#概要)
- [プロジェクトの状態](#プロジェクトの状態)
- [TidyFlow でできること](#tidyflow-でできること)
- [アーキテクチャ](#アーキテクチャ)
- [リポジトリ構成](#リポジトリ構成)
- [技術スタック](#技術スタック)
- [はじめかた](#はじめかた)
- [開発コマンド](#開発コマンド)
- [ドキュメント一覧](#ドキュメント一覧)
- [セキュリティとプライバシー](#セキュリティとプライバシー)
- [ロードマップ](#ロードマップ)
- [コントリビューション](#コントリビューション)
- [ライセンス](#ライセンス)
- [サポート](#サポート)

## 概要

TidyFlow は、繰り返し発生するファイル操作を管理し、何が実行されたのか、何が実行中なのか、次に何が実行されるのかを明確に把握したい人のためのデスクトップアプリです。

UI は Svelte、デスクトップシェルは Tauri、コアは Rust で構成されています。アプリはローカルで動作し、運用データを SQLite に保存します。ファイル操作は明示的な Rust command を通して行い、フロントエンドがファイルシステムを直接操作しない設計です。

このリポジトリは v2 の再実装です。一部の内部 crate や古い技術ドキュメントでは、まだ `autoflow-*` という名前空間が使われています。公開プロダクト名は **TidyFlow** です。

## プロジェクトの状態

| 項目 | 状態 |
| --- | --- |
| リリース | `0.2.1-alpha` |
| プロダクト名 | TidyFlow |
| Rust 内部名前空間 | `autoflow-*` |
| デスクトップシェル | Tauri 2 |
| フロントエンド | Svelte 5 + TypeScript |
| バックエンドコア | Rust workspace |
| 永続化 | `sqlx` 経由の SQLite |
| CI | UI チェック、Rust テスト、UI テストを実行する GitLab pipeline |

現在は alpha 版です。主要な流れは存在しますが、安定版までに API、UI の細部、パッケージングは変わる可能性があります。

## TidyFlow でできること

| 領域 | 説明 |
| --- | --- |
| 転送ジョブ | コピーまたは移動の job を作成し、コピー元、コピー先、フィルター、競合ルール、実行オプションを設定できます。 |
| シミュレーション | 実際にファイルを変更する前に、job や blueprint の結果を確認できます。 |
| フォルダ監視 | ディレクトリを監視し、ファイルイベントが安定した後に作業を queue に追加します。 |
| スケジュール | interval または日時ベースの schedule で job を実行します。 |
| Blueprints | template、counter、folder plan、variable を使ってファイルやフォルダを整理します。 |
| 監査ログ | 処理済みイベントを検索し、詳細を確認し、集計し、CSV または JSON に export できます。 |
| Admin panel | ローカルインスタンス情報、実行中の job、command queue、managed-agent workflow 用の signed heartbeat payload を確認できます。 |
| 設定 | 外観、言語、パフォーマンス、セキュリティ、通知、メンテナンス、サポートを調整できます。 |

TidyFlow は、隠れた background 処理よりも、見える自動化を重視します。実行状況、最近の activity、failure、次回実行を UI 上で確認できます。

## アーキテクチャ

```text
Svelte UI
  -> Tauri commands and events
    -> Rust application services
      -> Domain rules
      -> SQLite repositories
      -> Filesystem, watcher, scheduler, notifications
```

主な境界:

| レイヤー | 場所 | 役割 |
| --- | --- | --- |
| Desktop app | `apps/desktop` | Svelte routes、UI contracts、Tauri shell、app configuration。 |
| Domain | `crates/autoflow-domain` | Jobs、blueprints、filters、schedules、settings、audit types、tokenizer、path rules。 |
| Application | `crates/autoflow-application` | Jobs、execution、blueprints、scripts、notifications、packages の use case。 |
| Infrastructure | `crates/autoflow-infrastructure` | SQLite stores、migrations、persisted settings、audit、secrets、UI state。 |
| Core runtime | `crates/autoflow-core` | App state、queue、scheduler、watchers、admin actions、event emission。 |
| Admin server | `crates/autoflow-admin-server` | リモート管理 workflow 向けの HTTP administrative primitives。 |

## リポジトリ構成

```text
.
├── apps/
│   └── desktop/                 # Tauri 2 + Svelte 5 デスクトップアプリ
├── crates/
│   ├── autoflow-domain/         # 純粋な domain model と rules
│   ├── autoflow-application/    # Use cases と orchestration
│   ├── autoflow-infrastructure/ # SQLite, settings, audit, secrets, storage
│   ├── autoflow-core/           # Runtime, queue, scheduler, watchers
│   └── autoflow-admin-server/   # Admin server primitives
├── docs/
│   ├── v2/                      # Architecture, IPC, roadmap, stack notes
│   ├── WIKI/                    # ユーザー向け wiki pages
│   ├── adr/                     # Architecture decision records
│   ├── plans/                   # Planning notes と feature analysis
│   └── specs/                   # Contracts と schemas
├── tests/
│   └── e2e/                     # Playwright tests
├── Cargo.toml                   # Rust workspace
├── package.json                 # pnpm workspace scripts
├── pnpm-workspace.yaml
└── README.md
```

## 技術スタック

| 領域 | ツール |
| --- | --- |
| Desktop | Tauri 2 |
| UI | Svelte 5, SvelteKit, TypeScript, Vite |
| UI validation | Zod, Vitest, Playwright |
| Rust async runtime | Tokio |
| 永続化 | SQLite, `sqlx`, migrations |
| File watching | `notify`, `notify-debouncer-full` |
| HTTP/admin | Axum, Reqwest |
| Packaging | Tauri bundler |
| Workspace | pnpm workspaces + Cargo workspace |

## はじめかた

### 前提条件

- Node.js 20 LTS 以上
- Corepack 経由の pnpm
- Rust stable toolchain
- 利用する OS で Tauri が必要とする system dependencies

### インストール

```bash
git clone https://github.com/raillen/tidyflow.git
cd tidyflow
corepack enable
pnpm install
```

### デスクトップアプリを起動

```bash
pnpm dev
```

root の `dev` script は、`@tidyflow/desktop` workspace package を通して Tauri desktop app を起動します。

## 開発コマンド

| コマンド | 用途 |
| --- | --- |
| `pnpm dev` | Tauri + Svelte desktop app を development mode で起動します。 |
| `pnpm build` | Desktop application bundle を作成します。 |
| `pnpm check` | Desktop package の Svelte / TypeScript check を実行します。 |
| `pnpm test` | Frontend unit tests を実行します。 |
| `pnpm test:rust` | `cargo test --workspace` を実行します。 |
| `pnpm --filter @tidyflow/desktop check` | Desktop workspace package だけを check します。 |
| `cargo test --workspace` | Rust tests を直接すべて実行します。 |

## ドキュメント一覧

| ドキュメント | 用途 |
| --- | --- |
| [`docs/v2/ARCHITECTURE.md`](./docs/v2/ARCHITECTURE.md) | Runtime architecture、layers、flows。 |
| [`docs/v2/API-IPC.md`](./docs/v2/API-IPC.md) | Tauri command と event contracts。 |
| [`docs/v2/DOMAIN.md`](./docs/v2/DOMAIN.md) | Domain entities、invariants、rules。 |
| [`docs/v2/ROADMAP.md`](./docs/v2/ROADMAP.md) | Planned phases と delivery criteria。 |
| [`docs/v2/STACK.md`](./docs/v2/STACK.md) | Technical stack と library choices。 |
| [`docs/WIKI/User-Guide.md`](./docs/WIKI/User-Guide.md) | User-facing guide。 |
| [`docs/adr/`](./docs/adr/) | Architecture decisions。 |

v2 ドキュメントは、現在の alpha code と整合するよう更新中です。差異がある場合、public overview については code、manifest、この root README を優先してください。

## セキュリティとプライバシー

TidyFlow は、ローカル実行と明示的な境界を重視して設計されています。

- Runtime data は local SQLite に保存されます。
- File operations は Rust commands を通ります。
- Path validation は domain/core rules に集約されます。
- UI は低レベルの filesystem behavior を直接所有しません。
- Admin-agent secrets は application settings と platform keyring path を通して扱われます。
- 破壊的な admin command は明示的な command request として表現されます。

Alpha note: sensitive production workflow で使う前に、package manifests、settings、deployment configuration を確認してください。

## ロードマップ

短期的な作業は v2 foundation の安定化に集中しています。

- transfer execution と cancellation を強化する;
- job、watch、schedule、blueprint flow の整合性を保つ;
- supported locale の UI text を改善する;
- packaging と updater configuration を仕上げる;
- desktop workflow 全体の Playwright coverage を広げる;
- 古い AutoFlow document を現在の TidyFlow product name に合わせる。

詳細は [`docs/v2/ROADMAP.md`](./docs/v2/ROADMAP.md) を参照してください。

## コントリビューション

Alpha 期間中の contribution は歓迎します。変更は小さく、検証しやすくしてください。

Pull request の前に:

1. 関連する check を実行する。
2. behavior、setup、command、contract が変わる場合は documentation を更新する。
3. UI copy は短く、具体的で、読みやすくする。
4. 未完成 feature に placeholder UI を残さない。
5. 既知の limitation を明確に書く。

推奨 check:

```bash
pnpm check
pnpm test
pnpm test:rust
```

## ライセンス

TidyFlow は package-level licensing を使います。各 package manifest が、その部分の source of truth です。

| 領域 | ライセンス |
| --- | --- |
| `crates/autoflow-domain` | `MIT OR Apache-2.0` |
| `crates/autoflow-core` | `MIT OR Apache-2.0` |
| Workspace metadata を継承する workspace crates と desktop application | `GPL-3.0-only` |

[`LICENSE`](./LICENSE)、[`LICENSE-MIT`](./LICENSE-MIT)、[`LICENSE-APACHE`](./LICENSE-APACHE) を参照してください。

## サポート

TidyFlow は [Raillen Santos](https://github.com/raillen) により開発されています。

- Website: [raillen.site](https://raillen.site)
- GitHub: [github.com/raillen](https://github.com/raillen)
- Buy Me a Coffee: [buymeacoffee.com/raillen](https://www.buymeacoffee.com/raillen)
