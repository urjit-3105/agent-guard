# AgentGuard: Project Handoff & Continuity Guide

> **NOTICE TO FUTURE AI ASSISTANT / DEVELOPER:**
> 1. Always check the **`handoffs/`** directory first! Read [handoffs/CURRENT_STATUS.md](file:///D:/downloads/agent-guard-main/handoffs/CURRENT_STATUS.md) and [handoffs/INSTRUCTIONS.md](file:///D:/downloads/agent-guard-main/handoffs/INSTRUCTIONS.md) before writing code.
> 2. Whenever you complete a feature or milestone, you **MUST** create a new handoff log in `handoffs/` and update `handoffs/CURRENT_STATUS.md` following the standardized protocol.
> 3. If resuming work after formatting/cloning this repository, refer to this document for full architecture, resolved gotchas, and environment setup.

---

## 1. Project Mission & Core Concept

**AgentGuard** is a verifiable multi-agent migration sandbox written in Rust. It intercepts agent-generated SQL DDL migrations and schema changes, processing them through a 3-tier safety pipeline:

1. **Proof-Driven Development (PDD) Planning Loop (`pdd-core`)**: Natural-language and DDL intent parser that flags logical ambiguities (missing tenant isolation, unsafe nullability, destructive drops, missing indexes) as structured **Decision Cards** requiring human sign-off before code can proceed.
2. **Verus Formal Verification Bridge (`verus-specs`)**: Mathematical invariant contracts proving schema safety, zero tenant data leakage, and non-destructive evolution using SMT / Z3 solver semantics.
3. **In-Memory SQLite Sandbox & Pre-Flight Engine (`backend`)**: Sandboxed transaction runner via SQLx that performs dry-run forward execution and bidirectional rollback tests.
4. **Leptos WebAssembly Client (`frontend`)**: Reactive full-stack web dashboard with a 5-stage Live Intent Map, interactive decision cards, dual verification matrix (Formal Proofs vs. Empirical Tests), and an execution log terminal.

**Remote Repository**: [https://github.com/urjit-3105/agent-guard](https://github.com/urjit-3105/agent-guard)  
**Default Branch**: `main`

---

## 2. Workspace Layout

The project is structured as a Cargo Monorepo with 4 member crates:

```
agent-guard/
├── Cargo.toml                    # Root workspace config (memory/LTO optimization)
├── Cargo.lock                    # Pinned lockfile (preserves working dependencies)
├── .gitignore                    # Excludes /target, dist/, .trunk/, *.sqlite*
├── README.md                     # Public project overview & quickstart
├── PROJECT_HANDOFF.md            # This document (full state & continuity guide)
│
├── pdd-core/                     # Phase 1: PDD Intent Parser & Decision Engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs              # PddError enum (thiserror)
│   │   ├── model.rs              # IntentAction, Ambiguity, DecisionCard, MigrationPlan
│   │   ├── parser.rs             # Multi-statement DDL & natural language parser
│   │   ├── ambiguity.rs          # Safety rules (tenant isolation, nullability, drops)
│   │   └── decision.rs           # Decision card generator and option resolver
│   └── tests/
│       └── integration_tests.rs  # 3 integration tests (6 total tests in crate)
│
├── verus-specs/                  # Phase 2A: Mathematical Invariant Contracts
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # Dual-mode Verus macro shims (verus_spec!)
│   │   ├── tenant_isolation.rs   # Tenant boundary invariant lemmas
│   │   ├── schema_safety.rs      # Non-destructive schema evolution lemmas
│   │   └── nullability_invariants.rs # Nullability safety & default value lemmas
│
├── backend/                      # Phase 2B: Axum Server & SQLite Sandbox Engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs               # Server startup on port 3000 with CORS
│   │   ├── state.rs              # Shared AppState & SQLx SQLite connection pool
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── sqlite_sandbox.rs # Dry-run, rollback, and schema dump runner
│   │   │   └── verus_runner.rs   # SMT Z3 formal verification runner
│   │   └── routes/
│   │       ├── mod.rs
│   │       ├── intent.rs         # POST /api/v1/intent/submit
│   │       ├── decisions.rs      # GET & POST /api/v1/decisions/:id/resolve
│   │       ├── verify.rs         # POST /api/v1/verify
│   │       └── sandbox.rs        # POST /api/v1/sandbox/execute
│   └── tests/
│       └── api_tests.rs          # 3 end-to-end integration tests
│
└── frontend/                     # Phase 3: Leptos WebAssembly Client
    ├── Cargo.toml
    ├── index.html                # Trunk HTML entrypoint with Tailwind CDN
    └── src/
        ├── lib.rs
        ├── main.rs               # Conditional wasm32 mount / host stub
        ├── app.rs                # Top-level reactive layout & lifecycle
        ├── state.rs              # Reactive signals & API data types
        └── components/
            ├── mod.rs
            ├── intent_map.rs     # 5-Stage interactive intent workflow
            ├── decision_card.rs  # Human sign-off radio/card components
            ├── proof_matrix.rs   # Formal Proof vs Empirical Test matrix
            └── console.rs        # Execution log terminal & SQL inspector
```

---

## 3. Fresh Machine Setup Guide (Post-Laptop Format)

When setting up this project on a fresh Windows (or Linux/macOS) installation, follow these exact steps:

### Windows (Recommended Toolchain)

1. **Install Git**:
   ```powershell
   winget install Git.Git
   ```
2. **Install Rustup**:
   ```powershell
   winget install Rustlang.Rustup
   ```
3. **Install C Compiler / Linker (Choose One)**:
   - **Option A (GNU / MinGW - Recommended)**:
     ```powershell
     winget install BrechtSanders.WinLibs.POSIX.UCRT
     rustup toolchain install stable-x86_64-pc-windows-gnu
     rustup default stable-x86_64-pc-windows-gnu
     ```
     Ensure `bin` directory of WinLibs MinGW is in your PATH.
   - **Option B (MSVC)**:
     Install Visual Studio Build Tools with the "Desktop development with C++" workload, then use `stable-x86_64-pc-windows-msvc`.
4. **Add WebAssembly Target**:
   ```powershell
   rustup target add wasm32-unknown-unknown
   ```
5. **Install Trunk (for Leptos WASM Frontend)**:
   - Download `trunk-x86_64-pc-windows-msvc.zip` from [Trunk GitHub Releases](https://github.com/trunk-rs/trunk/releases/latest) and place `trunk.exe` into `C:\Users\<user>\.cargo\bin\`.
   - Alternatively: `cargo install trunk`
6. **Clone and Verify Repository**:
   ```powershell
   git clone https://github.com/urjit-3105/agent-guard.git
   cd agent-guard
   cargo test --workspace
   ```

---

## 4. Current Status & Test Suite

All 13 automated unit and integration tests are passing cleanly:

| Crate | Test Name | Purpose | Status |
| :--- | :--- | :--- | :--- |
| `pdd-core` | `test_parse_simple_add_column` | Verifies DDL lexer parses standard alter statements | ✅ PASSED |
| `pdd-core` | `test_detect_ambiguities_tenant_missing` | Flags missing `tenant_id` on multi-tenant tables | ✅ PASSED |
| `pdd-core` | `test_detect_ambiguities_not_null_missing_default` | Flags `NOT NULL` columns lacking default values | ✅ PASSED |
| `pdd-core` | `test_decision_card_generation` | Confirms decision cards are synthesized with options | ✅ PASSED |
| `pdd-core` | `test_resolve_decision_card` | Verifies user option resolution modifies SQL plan | ✅ PASSED |
| `pdd-core` | `test_end_to_end_intent_to_resolved_plan` | Full PDD loop: intent -> ambiguity -> card -> resolve | ✅ PASSED |
| `verus-specs` | `test_tenant_isolation_lemma` | Mathematical proof of strict tenant isolation boundary | ✅ PASSED |
| `verus-specs` | `test_schema_safety_lemma` | Mathematical proof of non-destructive schema changes | ✅ PASSED |
| `verus-specs` | `test_nullability_invariant_lemma` | Mathematical proof of nullability safety invariant | ✅ PASSED |
| `backend` | `test_verus_runner_formal_proof_verification` | SMT proof runner executes lemmas with high confidence | ✅ PASSED |
| `backend` | `test_backend_state_and_router_initialization` | Axum router initializes state and routes properly | ✅ PASSED |
| `backend` | `test_sqlite_sandbox_dry_run_and_rollback` | In-memory SQLite dry-run executes and rolls back cleanly | ✅ PASSED |
| `backend` | `test_realtime_websocket_log_broadcast_and_sandbox_streaming` | Verifies broadcast channel and live streaming during sandbox execution | ✅ PASSED |

**Target Compilation Checks**:
- Host GNU Target: `cargo check --workspace` -> **0 errors, 0 warnings**
- WebAssembly Target: `cargo check -p frontend --target wasm32-unknown-unknown` -> **0 errors**
- Frontend Asset Bundler: `trunk build` in `frontend/` -> **SUCCESS** (generates `frontend/dist/`)

---

## 5. How to Run Locally

### Start Backend (Port 3000)
```powershell
cargo run -p backend
```
- Listens at `http://127.0.0.1:3000`
- REST & WebSocket Endpoints:
  - `GET /api/v1/ws`: Real-time WebSocket log stream (broadcasts live sandbox DDL executions and formal verification lemmas).
  - `POST /api/v1/intent/submit`: Submits DDL or natural language intent.
  - `GET /api/v1/decisions`: Returns pending decision cards requiring human sign-off.
  - `POST /api/v1/decisions/{id}/resolve`: Resolves a card with chosen option (transitions plan to Approved).
  - `POST /api/v1/verify`: Runs formal Verus proof checks.
  - `POST /api/v1/sandbox/execute`: Performs SQLite transactional dry-run and rollback test.

### Start Frontend (Port 8080)
```powershell
cd frontend
trunk serve
```
- Open `http://127.0.0.1:8080` in your web browser.
- Full reactive Leptos UI with live state updates, decision card review, proof matrix display, and real-time WebSocket console feed.

---

## 6. Resolved Quirks & Critical Architectural Decisions

Keep these in mind when extending the codebase:

1. **`uuid` Version Pinned (`=1.8.0`)**:
   - `getrandom 0.4.x` introduces a Windows-specific dependency that calls `dlltool.exe` and `as.exe`, which can fail on certain MinGW toolchains.
   - Root `Cargo.toml` explicitly pins `uuid = "=1.8.0"`, keeping `getrandom` on `0.2.17`. Do NOT upgrade `uuid` to `1.9+` without testing on Windows GNU.
2. **Regex Lookaround Avoidance**:
   - Rust's `regex` crate intentionally rejects lookahead/lookbehind (`(?=...)`).
   - The parser in [pdd-core/src/parser.rs](file:///f:/agent-guard/pdd-core/src/parser.rs) uses iterative string slice pattern matching on `" and "` to split compound phrases safely.
3. **Leptos 0.6 Signal Types**:
   - Component props in `frontend/src/components/` take `Signal<T>` instead of `ReadSignal<T>`. This allows passing either raw signals or derived memos (`Signal::derive(cx, move || ...)`).
4. **WASM Dual-Target Safety**:
   - `frontend/src/main.rs` uses `#[cfg(target_arch = "wasm32")]` for `leptos::mount_to_body`. When compiled on host (e.g. during `cargo check --workspace`), it falls back to an inert stub so host builds never attempt to link browser DOM symbols.
5. **Real-Time WebSocket Log Fan-Out**:
   - `AppState` maintains a `tokio::sync::broadcast::Sender<LogStreamEvent>`. `GET /api/v1/ws` upgrades incoming clients and spawns async split tasks to pipe events down without blocking HTTP handlers.

---

## 7. Roadmap & Immediate Next Steps (To Continue Work)

- [x] **Phase 4A: Real-time WebSocket Log Streaming** (COMPLETED)
  - Implemented WebSocket route in Axum: `GET /api/v1/ws`.
  - Live broadcast of sandbox forward DDL steps, constraint checks, rollback tests, and formal verification lemmas.
  - Integrated into frontend `ConsoleView` with dedicated "Live WebSocket Stream" tab, live connection badge, and real-time updates.
- [x] **Phase 4B: Live Verus CLI Integration & Formal Spec Export** (COMPLETED)
  - Host capability introspection detecting native `verus` and `z3` CLI binaries via `PATH` and environment overrides (`VERUS_BINARY_PATH`, `Z3_BINARY_PATH`).
  - Configurable execution modes via `AGENTGUARD_VERUS_MODE` (`auto`, `native`, `z3`, `embedded`) with automatic fallback to embedded SMT deductive prover.
  - Automatic specification generator emitting authentic `.rs` Verus proof contract files (`verus! { ... }`) and standard SMT-LIB2 `.smt2` files to `target/verus_specs/`.
  - New introspection endpoint `GET /api/v1/verify/capabilities`.
  - Live WebSocket log streaming for capability detection, spec generation, and prover execution.
  - Frontend `ProofMatrix` updated with active prover engine status, execution badges, and generated spec artifact references.
  - Comprehensive unit and integration test coverage for detection, spec generation, and mock CLI execution.

When resuming this project, here are the remaining recommended roadmap items:

1. **Phase 4C: Persistent Migration History & Attestations**
   - Add a persistent SQLite/PostgreSQL storage adapter for migration audit logs and signed cryptographic attestations (e.g., hash of migration + proof certificate).
2. **Phase 4D: CI/CD GitHub Actions Workflow**
   - Create `.github/workflows/ci.yml` running `cargo check --workspace`, `cargo test --workspace`, and `cargo check -p frontend --target wasm32-unknown-unknown` on every push.
