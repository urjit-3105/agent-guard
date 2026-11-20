use leptos::*;
use crate::state::LogStreamEvent;

#[component]
pub fn ConsoleView(
    forward_sql: Signal<Vec<String>>,
    rollback_sql: Signal<Vec<String>>,
    verus_raw_output: Signal<Option<String>>,
    sandbox_logs: Signal<Vec<String>>,
    #[prop(optional)] live_stream_logs: Option<Signal<Vec<LogStreamEvent>>>,
    #[prop(optional)] ws_connected: Option<Signal<bool>>,
) -> impl IntoView {
    let active_tab = create_rw_signal("forward_sql".to_string());

    view! {
        <div class="bg-slate-900/80 border border-slate-800 rounded-xl overflow-hidden shadow-xl backdrop-blur-md">
            // Tab Header
            <div class="flex items-center justify-between px-6 pt-4 border-b border-slate-800 bg-slate-950/40">
                <div class="flex space-x-1">
                    <button
                        on:click=move |_| active_tab.set("forward_sql".to_string())
                        class=move || {
                            let active = active_tab.get() == "forward_sql";
                            format!(
                                "px-3.5 py-2 text-xs font-mono font-semibold border-b-2 transition-all {}",
                                if active {
                                    "border-cyan-400 text-cyan-400 bg-slate-900/60"
                                } else {
                                    "border-transparent text-slate-400 hover:text-slate-200"
                                }
                            )
                        }
                    >
                        "Forward SQL DDL"
                    </button>
                    <button
                        on:click=move |_| active_tab.set("rollback_sql".to_string())
                        class=move || {
                            let active = active_tab.get() == "rollback_sql";
                            format!(
                                "px-3.5 py-2 text-xs font-mono font-semibold border-b-2 transition-all {}",
                                if active {
                                    "border-cyan-400 text-cyan-400 bg-slate-900/60"
                                } else {
                                    "border-transparent text-slate-400 hover:text-slate-200"
                                }
                            )
                        }
                    >
                        "Rollback Scripts"
                    </button>
                    <button
                        on:click=move |_| active_tab.set("verus_output".to_string())
                        class=move || {
                            let active = active_tab.get() == "verus_output";
                            format!(
                                "px-3.5 py-2 text-xs font-mono font-semibold border-b-2 transition-all {}",
                                if active {
                                    "border-purple-400 text-purple-400 bg-slate-900/60"
                                } else {
                                    "border-transparent text-slate-400 hover:text-slate-200"
                                }
                            )
                        }
                    >
                        "Verus SMT Proof Output"
                    </button>
                    <button
                        on:click=move |_| active_tab.set("sandbox_logs".to_string())
                        class=move || {
                            let active = active_tab.get() == "sandbox_logs";
                            format!(
                                "px-3.5 py-2 text-xs font-mono font-semibold border-b-2 transition-all {}",
                                if active {
                                    "border-blue-400 text-blue-400 bg-slate-900/60"
                                } else {
                                    "border-transparent text-slate-400 hover:text-slate-200"
                                }
                            )
                        }
                    >
                        "SQLite Execution Logs"
                    </button>
                    <button
                        on:click=move |_| active_tab.set("live_stream".to_string())
                        class=move || {
                            let active = active_tab.get() == "live_stream";
                            format!(
                                "px-3.5 py-2 text-xs font-mono font-semibold border-b-2 transition-all flex items-center space-x-1.5 {}",
                                if active {
                                    "border-emerald-400 text-emerald-400 bg-slate-900/60"
                                } else {
                                    "border-transparent text-slate-400 hover:text-slate-200"
                                }
                            )
                        }
                    >
                        <span class=move || {
                            let conn = ws_connected.map(|c| c.get()).unwrap_or(false);
                            format!(
                                "w-2 h-2 rounded-full {}",
                                if conn { "bg-emerald-400 animate-pulse" } else { "bg-slate-500" }
                            )
                        }></span>
                        <span>"Live WebSocket Stream"</span>
                    </button>
                </div>

                <div class="hidden sm:flex items-center space-x-2 text-[11px] font-mono pb-2 text-slate-400">
                    <span class=move || {
                        let conn = ws_connected.map(|c| c.get()).unwrap_or(false);
                        format!(
                            "inline-flex items-center px-2 py-0.5 rounded text-[10px] font-medium border {}",
                            if conn {
                                "bg-emerald-950/60 text-emerald-400 border-emerald-800/80"
                            } else {
                                "bg-slate-900 text-slate-400 border-slate-800"
                            }
                        )
                    }>
                        {move || {
                            let conn = ws_connected.map(|c| c.get()).unwrap_or(false);
                            if conn { "WS: Connected" } else { "WS: Standby" }
                        }}
                    </span>
                </div>
            </div>

            // Console Content
            <div class="p-5 font-mono text-xs leading-relaxed max-h-80 overflow-y-auto bg-slate-950/80">
                {move || match active_tab.get().as_str() {
                    "forward_sql" => {
                        let sqls = forward_sql.get();
                        if sqls.is_empty() {
                            view! { <span class="text-slate-500">"-- No migration plan generated yet."</span> }.into_view()
                        } else {
                            view! {
                                <div class="space-y-3">
                                    {sqls.into_iter().enumerate().map(|(i, stmt)| {
                                        view! {
                                            <div class="flex items-start space-x-3 text-cyan-300">
                                                <span class="text-slate-600 select-none">{format!("{:02}", i + 1)}</span>
                                                <pre class="whitespace-pre-wrap">{stmt}</pre>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    },
                    "rollback_sql" => {
                        let sqls = rollback_sql.get();
                        if sqls.is_empty() {
                            view! { <span class="text-slate-500">"-- No rollback scripts generated yet."</span> }.into_view()
                        } else {
                            view! {
                                <div class="space-y-3">
                                    {sqls.into_iter().enumerate().map(|(i, stmt)| {
                                        view! {
                                            <div class="flex items-start space-x-3 text-amber-300">
                                                <span class="text-slate-600 select-none">{format!("{:02}", i + 1)}</span>
                                                <pre class="whitespace-pre-wrap">{stmt}</pre>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    },
                    "verus_output" => {
                        match verus_raw_output.get() {
                            Some(out) => view! {
                                <pre class="text-purple-300 whitespace-pre-wrap">{out}</pre>
                            }.into_view(),
                            None => view! {
                                <span class="text-slate-500">"Verus formal verification output will appear here after verification gate."</span>
                            }.into_view(),
                        }
                    },
                    "sandbox_logs" => {
                        let logs = sandbox_logs.get();
                        if logs.is_empty() {
                            view! { <span class="text-slate-500">"Sandbox execution logs will appear here during execution."</span> }.into_view()
                        } else {
                            view! {
                                <div class="space-y-1.5">
                                    {logs.into_iter().map(|log| {
                                        view! {
                                            <div class="text-slate-300 flex items-start space-x-2">
                                                <span class="text-blue-400">"›"</span>
                                                <span>{log}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    },
                    "live_stream" => {
                        let events = live_stream_logs.map(|s| s.get()).unwrap_or_default();
                        if events.is_empty() {
                            view! {
                                <span class="text-slate-500">
                                    "No live streaming events received yet. WebSocket connected to ws://127.0.0.1:3000/api/v1/ws."
                                </span>
                            }.into_view()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {events.into_iter().rev().map(|evt| {
                                        let channel_color = match evt.channel.as_str() {
                                            "sandbox" => "text-blue-400 bg-blue-950/60 border-blue-800",
                                            "verus" => "text-purple-400 bg-purple-950/60 border-purple-800",
                                            "system" => "text-emerald-400 bg-emerald-950/60 border-emerald-800",
                                            _ => "text-slate-400 bg-slate-900 border-slate-700",
                                        };
                                        let text_color = match evt.level.as_str() {
                                            "error" => "text-rose-400",
                                            "success" => "text-emerald-300",
                                            "warn" => "text-amber-300",
                                            _ => "text-slate-300",
                                        };
                                        view! {
                                            <div class="flex items-start space-x-2 text-xs">
                                                <span class="text-slate-600 select-none text-[10px]">
                                                    {evt.timestamp.format("%H:%M:%S%.3f").to_string()}
                                                </span>
                                                <span class=format!("px-1.5 py-0.2 rounded border text-[10px] uppercase font-bold {}", channel_color)>
                                                    {evt.channel}
                                                </span>
                                                <span class=format!("flex-1 {}", text_color)>
                                                    {evt.message}
                                                </span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                    },
                    _ => view! { <div></div> }.into_view(),
                }}
            </div>
        </div>
    }
}
