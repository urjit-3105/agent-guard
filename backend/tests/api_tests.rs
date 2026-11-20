use backend::engine::{SqliteSandbox, VerusRunner};
use backend::state::AppState;
use backend::create_router;
use pdd_core::{process_migration_intent, DecisionCardGenerator, PlanStatus};

#[tokio::test]
async fn test_sqlite_sandbox_dry_run_and_rollback() {
    let sandbox = SqliteSandbox::create_in_memory()
        .await
        .expect("Failed to create in-memory sandbox");

    let forward_sql = vec![
        "CREATE TABLE audit_logs (id TEXT PRIMARY KEY, tenant_id TEXT NOT NULL REFERENCES tenants(id), message TEXT NOT NULL);".to_string(),
        "CREATE INDEX idx_audit_logs_tenant ON audit_logs(tenant_id);".to_string(),
    ];
    let rollback_sql = vec![
        "DROP TABLE IF EXISTS audit_logs;".to_string(),
    ];

    let plan_id = uuid::Uuid::new_v4();

    // 1. Test sandboxed execution with rollback testing
    let result = sandbox
        .execute_sandboxed_migration(plan_id, &forward_sql, &rollback_sql, true)
        .await
        .expect("Sandbox execution failed");

    assert!(result.success);
    assert!(result.rollback_tested);
    assert!(result.rollback_successful);
    assert!(result.tables_created.contains(&"audit_logs".to_string()));

    // 2. Commit sandboxed execution
    let commit_result = sandbox
        .execute_sandboxed_migration(plan_id, &forward_sql, &rollback_sql, false)
        .await
        .expect("Sandbox commit failed");

    assert!(commit_result.success);
    assert!(!commit_result.rollback_tested);

    // Verify table exists via pragma
    let cols = sandbox
        .inspect_table("audit_logs")
        .await
        .expect("Failed to inspect table");

    assert!(cols.iter().any(|c| c.name == "tenant_id" && c.not_null));
    assert!(cols.iter().any(|c| c.name == "message" && c.not_null));
}

#[tokio::test]
async fn test_verus_runner_formal_proof_verification() {
    let raw = "Create table customer_billing with columns (id uuid primary key, amount int not null);";
    let (intent, cards, mut plan) = process_migration_intent(raw).expect("PDD parse failed");

    assert_eq!(plan.status, PlanStatus::NeedsClarification);

    // Resolve tenant isolation decision card to enforce tenant_id
    let card_id = cards[0].id;
    DecisionCardGenerator::resolve_decision(
        &mut plan,
        &intent,
        card_id,
        "opt_enforce_tenant_id",
        "compliance_officer",
    )
    .expect("Resolution failed");

    assert_eq!(plan.status, PlanStatus::ReadyForVerification);

    // Trigger Verus verification runner
    let report = VerusRunner::run_verification(&plan, &intent).await;

    assert!(report.verified, "Expected Verus formal proof to succeed");
    assert!(report.formal_proofs_count > 0);
    assert!(report.raw_verus_output.contains("Verus Formal Verification Engine"));
    assert!(report.raw_verus_output.contains("Q.E.D."));
    assert!(report.verdict.contains("PASS"));
}

#[tokio::test]
async fn test_backend_state_and_router_initialization() {
    let sandbox = SqliteSandbox::create_in_memory()
        .await
        .expect("Failed to initialize sandbox");

    let state = AppState::new(sandbox.pool().clone()).await;
    let _app = create_router(state);
    // Router successfully created with /api/v1/ws and state bound
}

#[tokio::test]
async fn test_realtime_websocket_log_broadcast_and_sandbox_streaming() {
    let sandbox = SqliteSandbox::create_in_memory()
        .await
        .expect("Failed to initialize sandbox");

    let state = AppState::new(sandbox.pool().clone()).await;
    let mut rx = state.log_sender.subscribe();

    // 1. Test manual log broadcast
    state.emit_log(
        "system",
        "info",
        None,
        "System initializing real-time test event",
    );

    let event = rx.recv().await.expect("Failed to receive broadcast event");
    assert_eq!(event.channel, "system");
    assert_eq!(event.level, "info");
    assert_eq!(event.message, "System initializing real-time test event");

    // 2. Test sandbox execution live streaming
    let plan_id = uuid::Uuid::new_v4();
    let forward_sql = vec![
        "CREATE TABLE live_orders (id TEXT PRIMARY KEY, tenant_id TEXT NOT NULL REFERENCES tenants(id), amount REAL);".to_string(),
    ];
    let rollback_sql = vec![
        "DROP TABLE IF EXISTS live_orders;".to_string(),
    ];

    let result = sandbox
        .execute_sandboxed_migration_with_sender(
            plan_id,
            &forward_sql,
            &rollback_sql,
            true,
            Some(&state.log_sender),
        )
        .await
        .expect("Sandbox execution failed");

    assert!(result.success);

    // Collect streamed events from channel
    let mut streamed_messages = Vec::new();
    while let Ok(evt) = rx.try_recv() {
        assert_eq!(evt.channel, "sandbox");
        assert_eq!(evt.plan_id, Some(plan_id));
        streamed_messages.push(evt.message);
    }

    assert!(
        !streamed_messages.is_empty(),
        "Expected live sandbox log messages to be streamed"
    );
    assert!(streamed_messages
        .iter()
        .any(|m| m.contains("Initializing sandboxed SQLite transaction pool")));
    assert!(streamed_messages
        .iter()
        .any(|m| m.contains("Executing DDL: CREATE TABLE live_orders")));
    assert!(streamed_messages
        .iter()
        .any(|m| m.contains("Testing Rollback DDL: DROP TABLE IF EXISTS live_orders")));
    assert!(streamed_messages
        .iter()
        .any(|m| m.contains("Rollback executed cleanly")));
}

#[tokio::test]
async fn test_phase_4b_verus_capabilities_detection() {
    let caps = VerusRunner::detect_capabilities().await;
    assert_eq!(caps.mode_configured, "auto");
    assert!(!caps.spec_export_directory.is_empty());
    // Active mode resolves to EmbeddedSmtProver when native binaries aren't installed in test env
    assert_eq!(caps.active_mode, "EmbeddedSmtProver");
}

#[tokio::test]
async fn test_phase_4b_spec_generation_and_disk_export() {
    let raw = "Create table tenant_workspaces with columns (id uuid primary key, tenant_id text not null, name text not null);";
    let (intent, _cards, plan) = process_migration_intent(raw).expect("PDD parse failed");

    // 1. Verify Rust/Verus spec generation syntax
    let verus_code = VerusRunner::generate_verus_spec_code(&plan, &intent);
    assert!(verus_code.contains("verus!"));
    assert!(verus_code.contains("use vstd::prelude::*;"));
    assert!(verus_code.contains("lemma_tenant_isolation_tenant_workspaces"));
    assert!(verus_code.contains("has_tenant_id_col == true"));

    // 2. Verify SMT-LIB2 benchmark format
    let smt2_code = VerusRunner::generate_smt2_code(&plan, &intent);
    assert!(smt2_code.contains("(set-logic ALL)"));
    assert!(smt2_code.contains("(check-sat)"));

    // 3. Verify disk export
    let temp_export_dir = std::env::temp_dir().join(format!("verus_test_{}", uuid::Uuid::new_v4()));
    let (rs_path, smt2_path) = VerusRunner::export_spec_files(&plan, &intent, &temp_export_dir)
        .await
        .expect("Spec export failed");

    assert!(rs_path.exists());
    assert!(smt2_path.exists());

    let rs_disk = tokio::fs::read_to_string(&rs_path).await.unwrap();
    assert!(rs_disk.contains("lemma_tenant_isolation_tenant_workspaces"));

    let smt2_disk = tokio::fs::read_to_string(&smt2_path).await.unwrap();
    assert!(smt2_disk.contains("(set-logic ALL)"));

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_export_dir).await;
}

#[tokio::test]
async fn test_phase_4b_verus_verification_report_artifacts() {
    let raw = "Create table payments (id uuid primary key, tenant_id text not null references tenants(id), amount int not null);";
    let (intent, _cards, plan) = process_migration_intent(raw).expect("PDD parse failed");

    let report = VerusRunner::run_verification(&plan, &intent).await;

    assert!(report.verified);
    assert!(report.generated_spec_file.is_some());
    assert!(report.generated_smt2_file.is_some());
    assert_eq!(report.capabilities.active_mode, "EmbeddedSmtProver");
    assert!(report.smt_solver_engine.contains("Verus Formal SMT / Z3"));
}

#[tokio::test]
async fn test_phase_4b_mock_cli_prover_execution() {
    let raw = "Create table orders (id uuid primary key, tenant_id text not null references tenants(id), total real not null);";
    let (intent, _cards, plan) = process_migration_intent(raw).expect("PDD parse failed");

    // Create a mock executable/script
    let temp_dir = std::env::temp_dir().join(format!("mock_verus_{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await.unwrap();

    #[cfg(windows)]
    let mock_path = temp_dir.join("mock_verus.cmd");
    #[cfg(not(windows))]
    let mock_path = temp_dir.join("mock_verus.sh");

    #[cfg(windows)]
    let script_content = "@echo off\r\necho verification results:: 3 verified, 0 errors, 3 obligations satisfied\r\nexit /b 0\r\n";
    #[cfg(not(windows))]
    let script_content = "#!/bin/sh\necho \"verification results:: 3 verified, 0 errors, 3 obligations satisfied\"\nexit 0\n";

    tokio::fs::write(&mock_path, script_content).await.unwrap();

    // Set environment variable to force native CLI execution through mock
    std::env::set_var("VERUS_BINARY_PATH", mock_path.to_string_lossy().to_string());
    std::env::set_var("AGENTGUARD_VERUS_MODE", "native");

    let report = VerusRunner::run_verification(&plan, &intent).await;

    // Reset env vars
    std::env::remove_var("VERUS_BINARY_PATH");
    std::env::remove_var("AGENTGUARD_VERUS_MODE");
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    assert!(report.verified);
    assert!(report.cli_execution.is_some());
    let cli_info = report.cli_execution.as_ref().unwrap();
    assert!(cli_info.cli_invoked);
    assert_eq!(cli_info.exit_code, Some(0));
    assert!(cli_info.cli_stdout.contains("verification results"));
}


