use pdd_core::{
    process_migration_intent, DecisionCardGenerator, PlanStatus, Severity,
};

#[test]
fn test_complex_multi_statement_migration() {
    let raw = r#"
        Create table user_billing (id uuid primary key, amount int not null);
        Add column risk_score int not null to accounts;
        Drop column legacy_ssn from users;
        Add index on orders (tenant_id, created_at);
    "#;

    let (intent, cards, plan) = process_migration_intent(raw).expect("Failed to process multi-statement intent");

    assert_eq!(intent.actions.len(), 4);
    assert!(cards.len() >= 3);
    assert_eq!(plan.status, PlanStatus::NeedsClarification);

    // Verify tenant isolation card on user_billing
    let billing_card = cards
        .iter()
        .find(|c| c.affected_table == "user_billing")
        .expect("Missing tenant card for user_billing");
    assert_eq!(billing_card.severity, Severity::Blocking);

    // Verify nullability card on accounts.risk_score
    let risk_card = cards
        .iter()
        .find(|c| c.affected_table == "accounts" && c.affected_column.as_deref() == Some("risk_score"))
        .expect("Missing nullability card for accounts.risk_score");
    assert_eq!(risk_card.severity, Severity::Blocking);

    // Verify destructive drop card on users.legacy_ssn
    let drop_card = cards
        .iter()
        .find(|c| c.affected_table == "users" && c.affected_column.as_deref() == Some("legacy_ssn"))
        .expect("Missing drop card for users.legacy_ssn");
    assert_eq!(drop_card.severity, Severity::Blocking);
}

#[test]
fn test_resolving_all_blocking_cards_reaches_ready_status() {
    let raw = "Add column tier text not null to organizations; Drop column temp_token from sessions";
    let (intent, cards, mut plan) = process_migration_intent(raw).expect("Parse failed");

    assert_eq!(plan.status, PlanStatus::NeedsClarification);

    for card in cards {
        let opt = if card.prompt.contains("NOT NULL") {
            "opt_add_default"
        } else if card.prompt.contains("Destructive") {
            "opt_soft_deprecate"
        } else {
            &card.options[0].id
        };

        DecisionCardGenerator::resolve_decision(&mut plan, &intent, card.id, opt, "security_officer")
            .expect("Failed to resolve card");
    }

    assert_eq!(plan.status, PlanStatus::ReadyForVerification);
    assert!(plan.pending_decisions.is_empty());
    assert_eq!(plan.resolved_decisions.len(), 2);

    // Ensure soft-deprecate produced an ALTER TABLE RENAME statement
    assert!(plan.forward_sql.iter().any(|s| s.contains("RENAME COLUMN temp_token TO _deprecated_temp_token")));
}

#[test]
fn test_system_tables_are_exempt_from_tenant_isolation() {
    let raw = "Create table system_health_metrics (id uuid primary key, cpu_usage int, recorded_at timestamp)";
    let (_intent, cards, plan) = process_migration_intent(raw).expect("Parse failed");

    // system_health_metrics is a system table, so no tenant isolation ambiguity should be raised
    let tenant_card = cards.iter().find(|c| c.prompt.contains("Multi-tenant"));
    assert!(tenant_card.is_none());
    assert_eq!(plan.status, PlanStatus::ReadyForVerification);
}
