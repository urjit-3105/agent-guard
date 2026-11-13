use leptos::*;
use crate::state::PipelineStage;

#[component]
pub fn IntentMapFlowchart(
    current_stage: Signal<PipelineStage>,
    has_blocking_decisions: Signal<bool>,
    is_verified: Signal<bool>,
    is_committed: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="bg-slate-900/80 border border-slate-800 rounded-xl p-6 shadow-xl backdrop-blur-md">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h3 class="text-sm font-semibold uppercase tracking-wider text-cyan-400">
                        "AgentGuard Live Intent Map"
                    </h3>
                    <p class="text-xs text-slate-400">
                        "Proof-Driven Verification Pipeline & Execution Boundary"
                    </p>
                </div>
                <div class="flex items-center space-x-2 text-xs">
                    <span class="inline-flex items-center px-2 py-1 rounded bg-slate-800 text-slate-300 font-mono">
                        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse mr-1.5"></span>
                        "Active Topology: SQLite Multi-Tenant Invariants"
                    </span>
                </div>
            </div>

            // Pipeline Grid / Flowchart Nodes
            <div class="grid grid-cols-1 md:grid-cols-5 gap-3 relative">
                // Step 1: Intent Ingestion
                <div class=move || {
                    let active = current_stage.get() == PipelineStage::Ingestion;
                    format!(
                        "p-4 rounded-lg border transition-all duration-300 {}",
                        if active {
                            "bg-cyan-950/40 border-cyan-500 shadow-lg shadow-cyan-500/10 ring-1 ring-cyan-500"
                        } else {
                            "bg-slate-950/60 border-slate-800"
                        }
                    )
                }>
                    <div class="flex items-center justify-between mb-2">
                        <span class="text-xs font-mono text-cyan-400 font-semibold">"01. INGEST"</span>
                        <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
                    </div>
                    <h4 class="font-medium text-sm text-slate-200">"Raw Intent"</h4>
                    <p class="text-xs text-slate-400 mt-1">"Agent-generated DDL / NL mutations"</p>
                    <div class="mt-3 text-[11px] font-mono text-slate-500">"Input Schema Parser"</div>
                </div>

                // Step 2: PDD Ambiguity Analysis
                <div class=move || {
                    let active = current_stage.get() == PipelineStage::PddAnalysis;
                    format!(
                        "p-4 rounded-lg border transition-all duration-300 {}",
                        if active {
                            "bg-indigo-950/40 border-indigo-500 shadow-lg shadow-indigo-500/10 ring-1 ring-indigo-500"
                        } else {
                            "bg-slate-950/60 border-slate-800"
                        }
                    )
                }>
                    <div class="flex items-center justify-between mb-2">
                        <span class="text-xs font-mono text-indigo-400 font-semibold">"02. PDD ANALYSIS"</span>
                        <span class="w-2 h-2 rounded-full bg-indigo-400"></span>
                    </div>
                    <h4 class="font-medium text-sm text-slate-200">"Rule Engine"</h4>
                    <p class="text-xs text-slate-400 mt-1">"Tenant boundary & nullability inspection"</p>
                    <div class="mt-3 text-[11px] font-mono text-slate-500">"Ambiguity Detection"</div>
                </div>

                // Step 3: Human Sign-off (Decision Cards)
                <div class=move || {
                    let is_blocked = has_blocking_decisions.get();
                    let active = current_stage.get() == PipelineStage::DecisionSignOff;
                    format!(
                        "p-4 rounded-lg border transition-all duration-300 {}",
                        if is_blocked {
                            "bg-rose-950/40 border-rose-500 shadow-lg shadow-rose-500/20 ring-1 ring-rose-500"
                        } else if active {
                            "bg-amber-950/40 border-amber-500 ring-1 ring-amber-500"
                        } else {
                            "bg-slate-950/60 border-slate-800"
                        }
                    )
                }>
                    <div class="flex items-center justify-between mb-2">
                        <span class=move || {
                            if has_blocking_decisions.get() {
                                "text-xs font-mono text-rose-400 font-semibold"
                            } else {
                                "text-xs font-mono text-amber-400 font-semibold"
                            }
                        }>
                            "03. SIGN-OFF"
                        </span>
                        <span class=move || {
                            if has_blocking_decisions.get() {
                                "w-2 h-2 rounded-full bg-rose-500 animate-ping"
                            } else {
                                "w-2 h-2 rounded-full bg-slate-600"
                            }
                        }></span>
                    </div>
                    <h4 class="font-medium text-sm text-slate-200">"Decision Cards"</h4>
                    <p class="text-xs text-slate-400 mt-1">"Explicit Human-In-The-Loop Sign-Off"</p>
                    <div class="mt-3 text-[11px] font-mono">
                        {move || {
                            if has_blocking_decisions.get() {
                                view! { <span class="text-rose-400 font-semibold">"PIPELINE BLOCKED"</span> }
                            } else {
                                view! { <span class="text-emerald-400">"RESOLVED / READY"</span> }
                            }
                        }}
                    </div>
                </div>

                // Step 4: Verus Formal Proof Gate
                <div class=move || {
                    let active = current_stage.get() == PipelineStage::VerusProofGate;
                    let verified = is_verified.get();
                    format!(
                        "p-4 rounded-lg border transition-all duration-300 {}",
                        if verified {
                            "bg-emerald-950/40 border-emerald-500 shadow-lg shadow-emerald-500/10 ring-1 ring-emerald-500"
                        } else if active {
                            "bg-purple-950/40 border-purple-500 ring-1 ring-purple-500"
                        } else {
                            "bg-slate-950/60 border-slate-800"
                        }
                    )
                }>
                    <div class="flex items-center justify-between mb-2">
                        <span class="text-xs font-mono text-purple-400 font-semibold">"04. VERUS PROOF"</span>
                        <span class=move || {
                            if is_verified.get() {
                                "w-2 h-2 rounded-full bg-emerald-400"
                            } else {
                                "w-2 h-2 rounded-full bg-purple-400"
                            }
                        }></span>
                    </div>
                    <h4 class="font-medium text-sm text-slate-200">"Formal Verification"</h4>
                    <p class="text-xs text-slate-400 mt-1">"SMT Z3 mathematical proof generation"</p>
                    <div class="mt-3 text-[11px] font-mono">
                        {move || {
                            if is_verified.get() {
                                view! { <span class="text-emerald-400 font-semibold">"Q.E.D. PROVED"</span> }
                            } else {
                                view! { <span class="text-slate-500">"SMT Gate Idle"</span> }
                            }
                        }}
                    </div>
                </div>

                // Step 5: SQLite Sandboxing & Commit
                <div class=move || {
                    let committed = is_committed.get();
                    format!(
                        "p-4 rounded-lg border transition-all duration-300 {}",
                        if committed {
                            "bg-emerald-950/40 border-emerald-500 shadow-lg shadow-emerald-500/10 ring-1 ring-emerald-500"
                        } else {
                            "bg-slate-950/60 border-slate-800"
                        }
                    )
                }>
                    <div class="flex items-center justify-between mb-2">
                        <span class="text-xs font-mono text-emerald-400 font-semibold">"05. SANDBOX & COMMIT"</span>
                        <span class=move || {
                            if is_committed.get() {
                                "w-2 h-2 rounded-full bg-emerald-400"
                            } else {
                                "w-2 h-2 rounded-full bg-slate-600"
                            }
                        }></span>
                    </div>
                    <h4 class="font-medium text-sm text-slate-200">"SQLite Sandbox"</h4>
                    <p class="text-xs text-slate-400 mt-1">"Dry-run & rollback verification"</p>
                    <div class="mt-3 text-[11px] font-mono">
                        {move || {
                            if is_committed.get() {
                                view! { <span class="text-emerald-400 font-semibold">"SAFE COMMIT DONE"</span> }
                            } else {
                                view! { <span class="text-slate-500">"Pre-flight Isolation"</span> }
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
