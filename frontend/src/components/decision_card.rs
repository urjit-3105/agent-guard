use leptos::*;
use pdd_core::{DecisionCard, DecisionStatus, Severity};
use uuid::Uuid;

#[component]
pub fn DecisionCardItem<F>(
    card: DecisionCard,
    on_resolve: F,
) -> impl IntoView
where
    F: Fn(Uuid, String) + 'static + Copy,
{
    let selected_option = create_rw_signal(
        card.options
            .iter()
            .find(|o| o.is_recommended)
            .map(|o| o.id.clone())
            .unwrap_or_else(|| card.options[0].id.clone()),
    );

    let is_resolved = card.is_resolved();
    let card_id = card.id;

    view! {
        <div class="border rounded-xl p-5 transition-all bg-slate-900/90 border-slate-800 shadow-md">
            // Card Header
            <div class="flex items-start justify-between">
                <div>
                    <div class="flex items-center space-x-2">
                        {match card.severity {
                            Severity::Critical => view! {
                                <span class="px-2 py-0.5 rounded text-[10px] font-bold tracking-wider uppercase bg-red-950 text-red-400 border border-red-800">
                                    "Critical Ambiguity"
                                </span>
                            },
                            Severity::Blocking => view! {
                                <span class="px-2 py-0.5 rounded text-[10px] font-bold tracking-wider uppercase bg-rose-950 text-rose-400 border border-rose-800">
                                    "Blocking Ambiguity"
                                </span>
                            },
                            Severity::Warning => view! {
                                <span class="px-2 py-0.5 rounded text-[10px] font-bold tracking-wider uppercase bg-amber-950 text-amber-400 border border-amber-800">
                                    "Warning"
                                </span>
                            },
                            Severity::Info => view! {
                                <span class="px-2 py-0.5 rounded text-[10px] font-bold tracking-wider uppercase bg-slate-800 text-slate-300">
                                    "Info"
                                </span>
                            },
                        }}
                        <span class="text-xs font-mono text-slate-400">
                            "Table: " <span class="text-slate-200 font-semibold">{card.affected_table.clone()}</span>
                            {card.affected_column.as_ref().map(|col| format!(" · Col: {}", col))}
                        </span>
                    </div>
                    <h4 class="text-base font-semibold text-slate-100 mt-2">{card.title.clone()}</h4>
                </div>

                <div>
                    {if is_resolved {
                        view! {
                            <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-950 text-emerald-400 border border-emerald-800">
                                <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                                </svg>
                                "Signed-Off"
                            </span>
                        }
                    } else {
                        view! {
                            <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-amber-950 text-amber-400 border border-amber-800 animate-pulse">
                                "Action Required"
                            </span>
                        }
                    }}
                </div>
            </div>

            // Card Prompt
            <p class="text-sm text-slate-300 mt-3 font-medium">{card.prompt.clone()}</p>

            // Options List
            <div class="mt-4 space-y-2.5">
                {card.options.into_iter().map(|opt| {
                    let opt_id = opt.id.clone();
                    let opt_id_for_check = opt.id.clone();
                    let opt_id_for_click = opt.id.clone();
                    let is_rec = opt.is_recommended;

                    view! {
                        <div
                            on:click=move |_| {
                                if !is_resolved {
                                    selected_option.set(opt_id_for_click.clone());
                                }
                            }
                            class=move || {
                                let selected = selected_option.get() == opt_id_for_check;
                                format!(
                                    "p-3 rounded-lg border cursor-pointer transition-all {}",
                                    if selected {
                                        "bg-slate-800/90 border-cyan-500 ring-1 ring-cyan-500 shadow-sm"
                                    } else {
                                        "bg-slate-950/60 border-slate-800 hover:border-slate-700"
                                    }
                                )
                            }
                        >
                            <div class="flex items-start justify-between">
                                <div class="flex items-center space-x-2.5">
                                    <input
                                        type="radio"
                                        name=format!("decision_opt_{}", card_id)
                                        checked=move || selected_option.get() == opt_id
                                        disabled=is_resolved
                                        class="h-4 w-4 text-cyan-500 focus:ring-cyan-400 border-slate-700 bg-slate-900"
                                    />
                                    <span class="text-sm font-semibold text-slate-200">{opt.label}</span>
                                </div>
                                {if is_rec {
                                    view! {
                                        <span class="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800">
                                            "Recommended"
                                        </span>
                                    }
                                } else {
                                    view! { <span class="hidden"></span> }
                                }}
                            </div>
                            <p class="text-xs text-slate-400 mt-1 ml-6">{opt.description}</p>
                            <div class="mt-2 ml-6 text-[11px] font-mono text-cyan-400/90">
                                "Tradeoff: " <span class="text-slate-400">{opt.safety_tradeoff}</span>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Action Button
            {if !is_resolved {
                view! {
                    <div class="mt-4 flex justify-end">
                        <button
                            on:click=move |_| {
                                on_resolve(card_id, selected_option.get());
                            }
                            class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 active:bg-cyan-700 text-white rounded-lg text-xs font-semibold tracking-wide uppercase transition shadow-lg shadow-cyan-600/20 flex items-center"
                        >
                            <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                            </svg>
                            "Sign-Off & Unblock Pipeline"
                        </button>
                    </div>
                }
            } else {
                let status = card.status.clone();
                view! {
                    <div class="mt-3 pt-3 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400 font-mono">
                        <span>"Resolution signed off by: " <strong class="text-slate-300">
                            {match status {
                                DecisionStatus::Resolved { resolved_by, .. } => resolved_by,
                                _ => "System".to_string(),
                            }}
                        </strong></span>
                        <span class="text-emerald-400">"Verified Compatible"</span>
                    </div>
                }
            }}
        </div>
    }
}

