# AI Model & Developer Handover Protocol

> **CRITICAL INSTRUCTION FOR ALL AI MODELS (AND DEVELOPERS) WORKING ON AGENTGUARD:**
> Whenever you start or finish a task in this repository, you **MUST** follow the protocol defined in this document.

---

## 1. When Starting Work (Mandatory Pre-Flight)

Before writing or modifying any code:
1. **Read the Latest Status**: Read `handoffs/CURRENT_STATUS.md` and the most recent dated log in `handoffs/` to understand the current architecture, completed phases, and immediate priorities.
2. **Review Critical Architectural Gotchas**:
   - `uuid` is strictly pinned to `=1.8.0` in root `Cargo.toml`. Never bump to `1.9+` without testing MinGW/Windows `getrandom` compatibility.
   - Rust's `regex` crate does not support lookarounds (`(?=...)` / `(?!...)`). Use iterative slice matching.
   - Leptos 0.6 component props use `Signal<T>` instead of raw `ReadSignal<T>` to allow derived memos.
   - Dual WASM target safety: Host code runs with native Axum/Tokio; frontend WASM code uses `#[cfg(target_arch = "wasm32")]` stubs so `cargo check --workspace` never breaks on host.
3. **Verify Baseline Tests**:
   Always execute `cargo test --workspace` to confirm that all tests pass before making any changes.

---

## 2. When Finishing Work (Mandatory Handover Requirement)

Every model or engineer who completes a unit of work, feature, or roadmap phase **MUST**:

1. **Create a New Handoff Log** in this `handoffs/` directory named:
   `handoffs/YYYY-MM-DD_<phase_or_feature>_handoff.md`
   (e.g., `handoffs/2026-11-20_phase_4c_history_handoff.md`).

2. **Update `handoffs/CURRENT_STATUS.md`** so it reflects the latest overall state of the project.

3. **Follow the Required Handoff Template**:
   Each new log must include:
   - **Header**: Phase/feature name, date, model/author identity.
   - **Executive Summary**: 2-3 sentences explaining what was built and why.
   - **Exact Changes Made**: List of all modified/created files with functions and logic described.
   - **Testing & Verification**: Exact test commands executed, number of passing tests, and build status (`wasm32-unknown-unknown` and host).
   - **Git Status & Commit**: Commit hash and confirmation of whether changes were pushed to GitHub.
   - **Immediate Next Steps for the Next Model**: A concrete, step-by-step roadmap for what the next engineer/model should implement next, including target file paths and architecture recommendations.

4. **Run Verification Before Logging**:
   - `cargo test --workspace`
   - `cargo check -p frontend --target wasm32-unknown-unknown`
   - Commit all changes cleanly with descriptive conventional commit messages (`feat: ...`, `fix: ...`, `docs: ...`).

---

## 3. Directory Index

- [INSTRUCTIONS.md](file:///D:/downloads/agent-guard-main/handoffs/INSTRUCTIONS.md) — This operating protocol.
- [CURRENT_STATUS.md](file:///D:/downloads/agent-guard-main/handoffs/CURRENT_STATUS.md) — The active master state of the repository.
- `2026-11-20_phase_4b_complete.md` — Phase 4B milestone log (Live Verus CLI & Spec Export).
