# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

agent-gateway is a unified gateway for managing multiple AI coding tools (Claude Code, Kimi Code, OpenCode, Kilo CLI) with provider-plan-model-agent hierarchy, automatic fallback, quota control, and protocol translation (Anthropic ↔ OpenAI). Supports CLI, GUI (Tauri), API, and Library modes.

## Technology Stack

| Component        | Technology                   |
|------------------|------------------------------|
| Core Language    | Rust (2021 edition)          |
| Async Runtime    | Tokio                        |
| HTTP Server      | Axum 0.7                     |
| HTTP Client      | Reqwest 0.12                 |
| GUI Framework    | Tauri v2                     |
| GUI Frontend     | Vue3 + Element Plus          |
| CLI Framework    | Clap v4.5                    |
| Data Storage     | SQLite (rusqlite)            |
| Config Format    | YAML + Serde                 |
| Logging          | tracing + tracing-subscriber |
| Encryption       | AES-GCM                      |
| Global State     | Arc + RwLock + DashMap       |
| Plugins          | wasmtime + WASI              |
| Node.js Bindings | NAPI-RS                      |

## Architecture

```txt
┌───────────────────────────────────────
──────────┐
│  CLI (agw-cli) │ GUI (agw-gui/Tauri) │ API (agw-api) │
└──────────────────────┬──────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│               agw-core (shared crate)           │
│  ┌──────────────────────────────────────────┐  │
│  │  Provider → Plan → Model → Agent 4-layer │  │
│  │  (Provider holds: Base URL, API format,  │  │
│  │   endpoints, model list, agent tools)    │  │
│  └──────────────────────────────────────────┘  │
│  ┌─────────────┬─────────────┬─────────────┐  │
│  │ Business    │   Proxy     │   Storage   │  │
│  │ (Plan/      │   (Axum     │   (SQLite   │  │
│  │  Fallback/  │    HTTP     │    + YAML)  │  │
│  │  Quota)     │    Gateway) │             │  │
│  └─────────────┴─────────────┴─────────────┘  │
└─────────────────────────────────────────────────┘
```

## Directory Structure

```txt
agent-gateway/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── agw-core/                 # Core library
│   │   └── src/
│   │       ├── model.rs          # Core data models
│   │       ├── business/         # Plan, Provider, Fallback, Quota
│   │       ├── core/             # Gateway, handlers, forwarder, converter
│   │       ├── storage/          # Config (YAML) + SQLite
│   │       ├── security/         # AES-GCM encryption
│   │       └── plugin/           # WASM plugin engine
│   ├── agw-cli/                  # CLI binary
│   │   └── src/commands/         # serve, plan, provider, agent, fallback, quota, plugin, config, log, completion
│   ├── agw-gui/                  # Tauri desktop app
│   └── agw-api/                  # REST API server (Axum)
├── frontend/                     # Vue3 + Element Plus GUI
│   └── src/
│       ├── views/                # Dashboard, Plans, Providers, Agents, Fallback, Quota, Logs, Settings
│       ├── components/           # Reusable UI components
│       └── composables/          # Vue composables
├── packages/                 # @agent-gateway/* (node, core, cli)
└── scripts/                      # build.sh, build-npm.sh
```

## Build Commands

```bash
# Build all workspace crates
cargo build

# Build specific binary (release mode)
cargo build -p agw-cli --release     # CLI only
cargo build -p agw-gui --release     # Desktop GUI
cargo build -p agw-api --release     # REST API server
```

## Running the API Server

```bash
# Development mode (auto-recompile)
cargo run -p agw-api
# Listens on http://127.0.0.1:8081

# Production mode (background)
cargo build -p agw-api --release
./target/release/agw-api.exe &

# Verify service
curl http://127.0.0.1:8081/health
# Returns: {"status":"ok","version":"0.1.0"}

# Stop service
taskkill //PID <PID> //F  # Windows
kill <PID>                # Linux/macOS
```

**API Endpoints:**

- `GET /health` - Health check
- `GET /api/v1/plans` - List plans
- `POST /api/v1/plans` - Create plan
- `GET /api/v1/providers` - List providers
- `GET /api/v1/quota` - Get quota status
- `GET /api/v1/fallback` - Get fallback config

## Running the GUI (Unified Server)

```bash
# Development mode
cargo run -p agw-gui
# Embedded unified server starts automatically on http://127.0.0.1:8080

# Production mode
cargo build -p agw-gui --release
./target/release/agw-gui.exe
```

**Embedded Server Endpoints (port 8080):**

- `GET /health` - Health check
- `POST /v1/messages` - Anthropic Messages API proxy
- `POST /v1/chat/completions` - OpenAI Chat Completions proxy
- `GET /api/v1/plans` - Management API (same as agw-api)
- All `/api/v1/*` endpoints from agw-api

**Server Mode Configuration** (`~/.agent-gateway/agw-gui/server.yaml`):

```yaml
mode: Embedded                 # Embedded or External
embedded_listen: "127.0.0.1:8080"
external_endpoint: null        # For External mode
auto_start: true
```

## Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p agw-core

# Run single test
cargo test test_name_here

# Run with output visible
cargo test -- --nocapture
```

## CLI Commands

```bash
# Gateway control
agw serve              # Start gateway
agw serve --daemon     # Start in background
agw stop               # Stop gateway

# Provider management
agw provider list              # List all providers
agw provider info <provider>   # Show provider details
agw provider update            # Update provider definitions

# Plan management
agw plan add --wizard                          # Interactive wizard
agw plan add --provider <id> --plan <id>       # Add plan non-interactively
agw plan list                  # List all plans
agw plan use <plan_id>         # Set default plan
agw plan test <plan_id>        # Test connectivity

# Agent management
agw agent list                 # List agents
agw agent bind <plan_id> <agent_id>
agw agent unbind <plan_id> <agent_id>
agw agent auto-config <plan_id> <agent_id>

# API Key helper
agw key open-page <provider>   # Open provider's API key page in browser
agw key test <plan_id>         # Test API key

# Fallback control
agw fallback on                # Enable automatic fallback
agw fallback off               # Disable
agw fallback set <plan1,plan2> # Set priority order

# Quota management
agw quota status               # Show quota usage
agw quota set <plan> <limit>   # Set quota

# Plugin management
agw plugin list                # List plugins
agw plugin install <source>    # Install (local/GitHub/remote)
agw plugin uninstall <id>

# Configuration
agw config edit                # Open config in editor
agw config show                # Show current config

# Shell completion
agw completion bash            # Generate bash completion
agw completion zsh             # Generate zsh completion
```

## Key Implementation Patterns

- **Provider-Plan-Model-Agent hierarchy**: Provider holds all tech configs (Base URL, API format, endpoints). Plan is a subscription tier under a provider (Lite/Plus/Max). Model is a specific model (GLM-5, Claude Sonnet 4.5). Agent is a standalone tool executable.
- **State management**: `Arc<RwLock<AppState>>` + `DashMap` for thread-safe shared state
- **Protocol translation**: Bidirectional Anthropic Messages API ↔ OpenAI Chat Completions
- **Fallback triggers**: 429 rate limits, 5xx errors, connection failure, timeout, quota exceeded
- **Fallback limit**: Max 3 attempts (configurable) to prevent infinite loops
- **SSE streaming**: Both Anthropic and OpenAI SSE formats supported
- **Encryption**: AES-256-GCM for API key storage
- **Plugin system**: WASM-based via wasmtime + WASI sandbox
- **Provider builtins**: Built-in provider templates in `provider_builtin.rs`, remote updates via `index.json`
- **API Key flow**: Browser launch to provider key page + clipboard monitoring (detects `sk-`, `sk-ant-`, `sk-proj-`, `AIza`, `gsk_`, `kilo_` prefixes)
- **Unified Router**: `create_unified_app()` merges GatewayState proxy routes with AppState management API routes for embedded server mode in Tauri GUI
- **Server Modes**: GUI supports Embedded (built-in unified server) and External (connect to remote agw-api) modes

## Application Data Paths

All application data is stored in a unified directory under the user's home:

```
~/.agent-gateway/                       # Windows: C:\Users\<用户名>\.agent-gateway
│                                       # macOS:   /Users/<用户名>/.agent-gateway
│                                       # Linux:   /home/<用户名>/.agent-gateway
│                                       # Override via AGENT_GATEWAY_HOME env var
│
├── agw.yaml                            # Public config (gateway settings)
│
├── agw-core/                           # Core module data
│   ├── user_plans.yaml                 # User plans config
│   ├── providers_builtin.yaml          # Provider definitions
│   ├── fallback.yaml                   # Fallback config
│   ├── custom_agents.yaml              # Custom agents
│   ├── custom_providers.yaml           # Custom providers
│   ├── api.yaml                        # API config
│   ├── encryption.key                  # Encryption key
│   ├── gateway.db                      # SQLite database
│   ├── logs/                           # Request logs
│   └── plugins/                        # WASM plugins
│
├── agw-cli/                            # CLI module data
│   ├── config.yaml                     # CLI config
│   └── gateway.pid                     # Runtime PID file
│
├── agw-gui/                            # GUI module data
│   ├── config.yaml                     # GUI config (theme, window state)
│   ├── tray.yaml                       # Tray icon config
│   └── server.yaml                     # Server mode config (Embedded/External)
│
└── .migrated.marker                    # Migration marker (created after first run)
```

**Migration**: On first startup, data from old paths (`AppData/Roaming/agent-gateway` and `AppData/Local/agent-gateway` on Windows) is automatically migrated to the new unified path. Old directories are preserved as backup.

## Development Notes

- Primary development on Linux
- Use `tracing` for structured logging (not `log` crate)
- All async code uses Tokio runtime
- Workspace resolver version "2" required
- Release profile: LTO, codegen-units=1, panic=abort
