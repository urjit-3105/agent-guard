# AgentGuard: Master Project Status & Architecture Guide

> **Current Timestamp**: 2026-11-20  
> **Repository**: [https://github.com/urjit-3105/agent-guard](https://github.com/urjit-3105/agent-guard)  
> **Default Branch**: `main`  
> **Latest Milestone**: Phase 4B (Live Verus CLI Integration & Formal Spec Export)  
> **Test Suite**: 17 / 17 tests passing across workspace monorepo  

---

## 1. Project Overview & Core Mission

**AgentGuard** is a verifiable multi-agent migration sandbox written in Rust. It intercepts agent-generated SQL DDL schema changes, processing them through a multi-tier safety pipeline before any change can reach production:

```
[Agent SQL DDL / Natural Language]
             │
             ▼
  ┌───────────────────────┐
  │  pdd-core             │  Phase 1: Proof-Driven Development Intent Parser
  │  (Intent & Ambiguity) │  Flags missing tenants, nullability risks, destructive drops
  └──────────┬────────────┘  Emits structured Decision Cards for human sign-off
             │
             ▼ (Decision Cards Signed Off)
  ┌───────────────────────┐
  │  verus-specs &        │  Phase 2A & 4B: Mathematical Formal Invariant Contracts
  │  verus_runner         │  Proves zero tenant leakage, non-destructive evolution, safe backfills
  └──────────┬────────────┘  Emits .rs Verus specs & .smt2 Z3 benchmarks (Native or Embedded)
             │
             ▼ (Formal Proofs PASS)
  ┌───────────────────────┐
  │  backend              │  Phase 2B & 4A: SQLite Transaction Sandbox & Rollback Engine
  │  (Sandbox Runner)     │  Dry-runs forward DDL, tests reverse DDL rollback, streams logs via WS
  └──────────┬────────────┘
             │
             ▼
  ┌───────────────────────┐
  │  frontend             │  Phase 3 & 4A/4B: Leptos 0.6 WASM Reactive Dashboard
  │  (WebAssembly UI)     │  5-stage Live Intent Map, Decision Cards, Dual Matrix, Live Console
  └───────────────────────┘
```

---

## 2. Monorepo Crate Layout

| Crate | Purpose | Key Files |
| :--- | :--- | :--- |
| `pdd-core` | Intent parser, ambiguity detection, and decision card resolver | `src/parser.rs`, `src/ambiguity.rs`, `src/decision.rs`, `src/model.rs` |
| `verus-specs` | Formal mathematical invariant contracts | `src/tenant_isolation.rs`, `src/schema_safety.rs`, `src/nullability_invariants.rs` |
| `backend` | Axum HTTP/WebSocket server, SQLite sandbox, Verus CLI runner | `src/engine/verus_runner.rs`, `src/engine/sqlite_sandbox.rs`, `src/routes/` |
| `frontend` | Full-stack Leptos WebAssembly client UI | `src/app.rs`, `src/components/`, `src/state.rs` |

---

## 3. What Has Been Completed So Far

### ✅ Phase 1: PDD Planning Loop (`pdd-core`)
- Parses natural language and DDL statements (`CREATE TABLE`, `DROP TABLE`, `DROP COLUMN`, `ADD COLUMN`).
- Detects safety ambiguities (missing `tenant_id NOT NULL`, unsafe drops without soft-deprecation, `NOT NULL` without `DEFAULT`).
- Generates structured, blocking `DecisionCard`s with concrete resolution options (`opt_enforce_tenant_id`, `opt_soft_deprecate_column`, etc.).
- Resolving all blocking cards transitions plan status from `NeedsClarification` to `ReadyForVerification`.

### ✅ Phase 2A: Verus Mathematical Contracts (`verus-specs`)
- Multi-tenant boundary isolation lemma ($\forall r \in T, r.\text{tenant\_id} \neq \bot \land r.\text{tenant\_id} \in \text{Dom}(\text{Tenants})$).
- Non-destructive schema evolution lemma ($\text{Dom}(S_t) \subseteq \text{Dom}(S_{t+1})$).
- Nullability preservation lemma ($\text{is\_not\_null} \implies (\text{has\_default} \lor \text{table\_is\_empty})$).
- Dual-mode `verus_spec!` macro shim allowing compilation under standard `rustc` while remaining authentic for formal Verus solvers.

### ✅ Phase 2B: In-Memory SQLite Sandbox Engine (`backend`)
- In-memory SQLite transaction sandbox powered by SQLx.
- Forward execution dry-runs, PRAGMA schema introspection (`table_info`), and atomic rollback verification testing reverse DDL cleanly without database state drift.

### ✅ Phase 3: Reactive WebAssembly Frontend (`frontend`)
- Reactive dashboard written in Leptos 0.6 and Tailwind CSS.
- 5-stage visual Intent Map (`Ingestion` → `PddAnalysis` → `DecisionSignOff` → `VerusProofGate` → `SandboxCommit`).
- Interactive Decision Cards interface with live option resolution.
- Dual Verification Matrix comparing deductive formal proofs against empirical sandbox tests.

### ✅ Phase 4A: Real-time WebSocket Log Streaming
- Axum WebSocket endpoint `GET /api/v1/ws` with broadcast channel fan-out.
- Asynchronous streaming of live sandbox execution, constraint checks, and formal proof obligations.
- Frontend `ConsoleView` with dedicated "Live WebSocket Stream" tab, real-time connection badge, and event stream listener.

### ✅ Phase 4B: Live Verus CLI Integration & Spec Code Export
- **Capability Detection**: `VerusRunner::detect_capabilities()` auto-detects `verus` and `z3` CLI binaries in `PATH` and environment variable overrides (`VERUS_BINARY_PATH`, `Z3_BINARY_PATH`).
- **Configurable Mode Toggle**: `AGENTGUARD_VERUS_MODE` (`auto`, `native`, `z3`, `embedded`) with automatic fallback to the embedded SMT deductive prover.
- **Formal Spec Generator**: Automatically generates authentic `.rs` Verus proof specifications (`verus! { ... }`) and standard `.smt2` SMT-LIB2 benchmark files into `target/verus_specs/`.
- **Introspection Route**: Added `GET /api/v1/verify/capabilities` endpoint.
- **WebSocket Integration**: Live streaming of prover detection, spec export, and lemma proofs.
- **UI Enhancements**: Updated `ProofMatrix` with active engine badges and generated specification artifact paths.
- **Tests**: 4 new integration tests (8 total in backend, 17 across workspace).

---

## 4. How to Build & Run Locally

### Run Tests:
```bash
cargo test --workspace
```
*(All 17 unit and integration tests must pass).*

### Check Frontend WASM Compilation:
```bash
cargo check -p frontend --target wasm32-unknown-unknown
```

### Start Backend (Port 3000):
```bash
cargo run -p backend
```
- Base URL: `http://127.0.0.1:3000`
- Endpoints:
  - `GET /health`
  - `GET /api/v1/ws`
  - `GET /api/v1/verify/capabilities`
  - `POST /api/v1/intent/submit`
  - `GET /api/v1/decisions`
  - `POST /api/v1/decisions/:id/resolve`
  - `POST /api/v1/verify`
  - `POST /api/v1/sandbox/execute`

### Start Frontend (Port 8080):
```bash
cd frontend
trunk serve
```
- Open `http://127.0.0.1:8080` in any web browser.

---

## 5. Critical Invariants & Gotchas (Do Not Break!)

1. **`uuid` Version Strictly Pinned (`=1.8.0`)**:
   - `getrandom 0.4.x` pulls Windows-specific MinGW dependencies that can fail compilation on host.
   - Root `Cargo.toml` pins `uuid = "=1.8.0"`. **Never** bump to `1.9+`.
2. **No Regex Lookarounds**:
   - Rust's `regex` crate explicitly forbids lookarounds (`(?=...)`, `(?!...)`). Use iterative slice matching (`split(" and ")`).
3. **Leptos 0.6 Signal Props**:
   - Component props in `frontend/src/components/` must take `Signal<T>` instead of `ReadSignal<T>` to allow derived memos.
4. **WASM Dual-Target Safety**:
   - Host checks use native Axum; WASM uses `#[cfg(target_arch = "wasm32")]` so `cargo check --workspace` works cleanly on both host and browser targets.

---

## 6. Immediate Next Steps: Roadmap for the Next Model

The next items on the roadmap are:

### 🎯 Immediate Priority: Phase 4C — Persistent Migration History & Attestations

#### Goal:
Persist completed migrations, verified plans, decision logs, and cryptographically signed formal verification attestations into a persistent SQLite/PostgreSQL database adapter so that migrations survive server restarts and produce verifiable audit certificates.

#### Recommended Implementation Architecture:

1. **Storage Schema & Adapter (`backend/src/storage/` or `backend/src/db/`)**:
   - Create tables in SQLite:
     ```sql
     CREATE TABLE IF NOT EXISTS migration_history (
         id TEXT PRIMARY KEY,
         intent_id TEXT NOT NULL,
         plan_id TEXT NOT NULL,
         status TEXT NOT NULL,
         forward_sql TEXT NOT NULL,
         rollback_sql TEXT NOT NULL,
         verified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
         attestation_hash TEXT NOT NULL,
         attestation_signature TEXT NOT NULL,
         raw_verus_proofs TEXT NOT NULL
     );

     CREATE TABLE IF NOT EXISTS decision_audit_log (
         id TEXT PRIMARY KEY,
         plan_id TEXT NOT NULL,
         card_id TEXT NOT NULL,
         decision_category TEXT NOT NULL,
         selected_option TEXT NOT NULL,
         resolved_by TEXT NOT NULL,
         resolved_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
     );
     ```

2. **Cryptographic Attestation Generator**:
   - Compute a deterministic SHA-256 hash over:
     `Hash(plan.forward_sql + plan.rollback_sql + verus_report.raw_verus_output + resolved_decisions)`
   - Generate an attestation certificate containing:
     - Certificate UUID
     - Plan ID
     - Timestamp
     - Prover Engine & Solver Time
     - Mathematical Invariant Proof Summary
     - SHA-256 Digest Signature

3. **New API Endpoints**:
   - `GET /api/v1/history` — List past verified and committed migrations with timestamps and status.
   - `GET /api/v1/history/:plan_id` — Retrieve full historical record.
   - `GET /api/v1/history/:plan_id/attestation` — Export formal cryptographic verification certificate (JSON / Markdown / PEM).

4. **Frontend Integration**:
   - Add a "History & Attestations" tab or modal in the frontend dashboard.
   - Display past migrations, execution status badges, and a "Download Proof Certificate" button.

5. **Testing**:
   - Add unit/integration tests in `backend/tests/` asserting persistence across app state reloads and cryptographic hash determinism.

---

### 🎯 Secondary Priority: Phase 4D — CI/CD GitHub Actions Workflow

Create `.github/workflows/ci.yml` running:
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo check -p frontend --target wasm32-unknown-unknown`
on every pull request and push to `main`.
