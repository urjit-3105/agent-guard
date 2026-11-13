fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use frontend::App;
        use leptos::*;
        mount_to_body(|| view! { <App/> });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("AgentGuard Leptos WebAssembly Client initialized.");
    }
}
