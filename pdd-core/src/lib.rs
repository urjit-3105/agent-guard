pub mod ambiguity;
pub mod decision;
pub mod error;
pub mod model;
pub mod parser;

pub use ambiguity::AmbiguityAnalyzer;
pub use decision::DecisionCardGenerator;
pub use error::{PddError, Result};
pub use model::{
    Ambiguity, AmbiguityCategory, ColumnDef, DataType, DecisionCard, DecisionOption,
    DecisionStatus, IntentAction, MigrationIntent, MigrationPlan, PlanStatus, Severity,
};
pub use parser::IntentParser;

/// Complete PDD planning pipeline:
/// Ingests natural language prompt, parses actions, detects ambiguities,
/// generates structured decision cards, and synthesizes an initial migration plan.
pub fn process_migration_intent(
    raw_prompt: &str,
) -> Result<(MigrationIntent, Vec<DecisionCard>, MigrationPlan)> {
    let intent = IntentParser::parse(raw_prompt)?;
    let ambiguities = AmbiguityAnalyzer::analyze(&intent);
    let cards = DecisionCardGenerator::generate_cards(&intent, &ambiguities);
    let plan = DecisionCardGenerator::build_initial_plan(&intent, cards.clone());

    Ok((intent, cards, plan))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table_with_tenant() {
        let prompt = "Create table billing_records with columns (id uuid primary key, amount int not null, status text)";
        let (intent, cards, plan) = process_migration_intent(prompt).expect("Failed to process intent");

        assert_eq!(intent.actions.len(), 1);
        match &intent.actions[0] {
            IntentAction::CreateTable { table_name, .. } => {
                assert_eq!(table_name, "billing_records");
            }
            _ => panic!("Expected CreateTable"),
        }

        // billing_records is a tenant table keyword and lacks tenant_id, so it must generate a blocking decision card
        assert!(!cards.is_empty());
        assert!(cards.iter().any(|c| c.title.contains("tenant isolation")));
        assert_eq!(plan.status, PlanStatus::NeedsClarification);
    }

    #[test]
    fn test_parse_destructive_drop_and_nullability() {
        let prompt = "Add column priority int not null to tasks; drop column legacy_notes from tasks";
        let (intent, cards, plan) = process_migration_intent(prompt).expect("Failed to process intent");

        assert_eq!(intent.actions.len(), 2);
        assert_eq!(cards.len(), 2);

        let nullability_card = cards.iter().find(|c| c.severity == Severity::Blocking && c.prompt.contains("NOT NULL"));
        assert!(nullability_card.is_some());

        let drop_card = cards.iter().find(|c| c.prompt.contains("Destructive operation"));
        assert!(drop_card.is_some());

        assert_eq!(plan.status, PlanStatus::NeedsClarification);
    }

    #[test]
    fn test_resolve_decision_cards_unblocks_plan() {
        let prompt = "Add column priority int not null to tasks";
        let (intent, cards, mut plan) = process_migration_intent(prompt).expect("Failed to process intent");

        assert_eq!(plan.status, PlanStatus::NeedsClarification);
        let card_id = cards[0].id;

        // Resolve card with recommended default option
        DecisionCardGenerator::resolve_decision(
            &mut plan,
            &intent,
            card_id,
            "opt_add_default",
            "lead_architect",
        )
        .expect("Resolution failed");

        assert_eq!(plan.status, PlanStatus::ReadyForVerification);
        assert_eq!(plan.pending_decisions.len(), 0);
        assert_eq!(plan.resolved_decisions.len(), 1);
        assert!(plan.forward_sql[0].contains("DEFAULT"));
    }
}

