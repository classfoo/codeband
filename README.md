# Codeband

Codeband is a Rust-first desktop and web application scaffold based on Tauri 2.x.  
It supports two runtime modes:

- Desktop integrated mode (Tauri app + embedded backend server)
- Backend-only mode (Rust server + browser frontend)

## Project Layout

```text
.
├── apps
│   ├── desktop
│   │   └── src-tauri          # Tauri host application (desktop shell)
│   └── web                    # React + Vite frontend
├── crates
│   ├── domain                 # Core business models and domain types
│   ├── application            # Use-cases and application services
│   └── server                 # HTTP API adapter (Axum), standalone server binary
├── configs
│   └── runtime.env.example    # Runtime environment template
├── scripts                    # Dev/build/init scripts for all targets
├── Cargo.toml                 # Rust workspace root
└── package.json               # Node workspace scripts
```

## Architecture (Layered / Tier-I)

### 1) Domain Layer (`crates/domain`)

- Holds pure business data structures and domain concepts.
- No framework or UI dependency.
- Should remain stable and reusable across all delivery channels.

### 2) Application Layer (`crates/application`)

- Implements use-cases and orchestration logic.
- Depends on `domain`.
- No direct dependency on HTTP, Tauri, or frontend frameworks.

### 3) Delivery Layer

- `crates/server`: HTTP API delivery (Axum), exposes endpoints such as `/api/health`.
- `apps/desktop/src-tauri`: desktop runtime host, starts local backend and opens the app window.
- `apps/web`: web UI client, can run in browser-only mode and call the backend API.

## Runtime Modes

### Desktop Integrated Mode

Use a single command to start Tauri desktop app:

```bash
npm run dev:desktop
```

Flow:

1. Frontend dev server starts (Vite).
2. Tauri host launches.
3. Embedded backend service runs locally.
4. UI communicates with local API.

### Backend-Only Browser Mode

Run backend + web frontend in parallel:

```bash
npm run dev:browser
```

Flow:

1. Rust server runs as standalone process.
2. Frontend runs in browser.
3. Frontend accesses API via `VITE_API_BASE`.

## Build and Packaging

### Common Build Commands

```bash
npm run build:web
npm run build:server
npm run build:desktop
```

### Multi-Platform Packaging

```bash
npm run build:linux
npm run build:macos
npm run build:windows
npm run build:ios
npm run build:android
```

### Platform-Aware Entry

Auto-detect host desktop platform:

```bash
npm run build:platform
```

Or pass mobile target explicitly:

```bash
bash ./scripts/build_all_platforms.sh ios
bash ./scripts/build_all_platforms.sh android
```

### Mobile Initialization (first-time setup)

```bash
npm run init:ios
npm run init:android
```

## Environment Configuration

Copy and adjust values from:

- `configs/runtime.env.example`

Key variables:

- `CODEBAND_HOST`: backend bind host (default `127.0.0.1`)
- `CODEBAND_PORT`: backend bind port (default `8080`)
- `VITE_API_BASE`: frontend API base URL

## Development Notes

- Rust code is organized as a workspace, so backend/domain evolution remains modular.
- Frontend and desktop packaging are decoupled from core business logic.
- Add new business capabilities by extending:
  - `domain` for core types/rules
  - `application` for use-cases
  - `server` for API routes/adapters
  - `web` for UI integration
