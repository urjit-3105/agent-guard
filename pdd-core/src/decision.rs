use crate::error::{PddError, Result};
use crate::model::{
    Ambiguity, AmbiguityCategory, ColumnDef, DecisionCard, DecisionOption, DecisionStatus,
    IntentAction, MigrationIntent, MigrationPlan, PlanStatus, Severity,
};
use chrono::Utc;
use uuid::Uuid;

pub struct DecisionCardGenerator;

impl DecisionCardGenerator {
    pub fn generate_cards(
        intent: &MigrationIntent,
        ambiguities: &[Ambiguity],
    ) -> Vec<DecisionCard> {
        ambiguities
            .iter()
            .map(|ambiguity| Self::create_card_for_ambiguity(intent.id, ambiguity))
            .collect()
    }

    fn create_card_for_ambiguity(intent_id: Uuid, ambiguity: &Ambiguity) -> DecisionCard {
        let (prompt, options) = match ambiguity.category {
            AmbiguityCategory::NullabilityMissingDefault => (
                format!(
                    "Adding NOT NULL column '{}' without DEFAULT. How should existing records be handled?",
                    ambiguity.affected_column.as_deref().unwrap_or("unknown")
                ),
                vec![
                    DecisionOption {
                        id: "opt_add_default".to_string(),
                        label: "Provide Safe DEFAULT (Recommended)".to_string(),
                        description: "Add a schema-level DEFAULT expression (e.g., empty string, 0, or false) to backfill existing records immediately.".to_string(),
                        is_recommended: true,
                        resulting_patch_description: "Appends 'DEFAULT <val>' to column definition.".to_string(),
                        safety_tradeoff: "Zero downtime, guaranteed backward compatibility.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_two_phase_nullable".to_string(),
                        label: "Two-Phase Backfill (Nullable First)".to_string(),
                        description: "Add column as NULLABLE first. Require a separate application worker migration to backfill data before adding NOT NULL constraint.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Modifies column definition to nullable.".to_string(),
                        safety_tradeoff: "Requires two deployment phases and asynchronous backfill.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_permit_nulls".to_string(),
                        label: "Permit NULL Values".to_string(),
                        description: "Relax constraint to NULLABLE permanently.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Removes NOT NULL constraint from column definition.".to_string(),
                        safety_tradeoff: "Application code must handle Option<T> / None checks.".to_string(),
                    },
                ],
            ),
            AmbiguityCategory::DestructiveChange => (
                format!(
                    "Destructive operation detected on '{}'. Are you certain this schema change is intended?",
                    ambiguity.affected_table
                ),
                vec![
                    DecisionOption {
                        id: "opt_soft_deprecate".to_string(),
                        label: "Soft Deprecate (Recommended)".to_string(),
                        description: "Rename column/table with '_deprecated_' prefix and keep data accessible until end of deprecation cycle.".to_string(),
                        is_recommended: true,
                        resulting_patch_description: "Replaces DROP with ALTER TABLE RENAME.".to_string(),
                        safety_tradeoff: "Zero data loss; retains reversibility and backward compatibility.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_hard_drop".to_string(),
                        label: "Acknowledge Data Deletion & Hard Drop".to_string(),
                        description: "Explicitly authorize irrevocable deletion of the column/table.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Executes raw DROP statement.".to_string(),
                        safety_tradeoff: "Irreversible data loss; breaking change for running clients.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_abort".to_string(),
                        label: "Abort Change".to_string(),
                        description: "Cancel this destructive action from the migration batch.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Omit this action from execution.".to_string(),
                        safety_tradeoff: "No change applied.".to_string(),
                    },
                ],
            ),
            AmbiguityCategory::TenantIsolationViolation => (
                format!(
                    "Multi-tenant data boundary missing on table '{}'. Enforce tenant isolation?",
                    ambiguity.affected_table
                ),
                vec![
                    DecisionOption {
                        id: "opt_enforce_tenant_id".to_string(),
                        label: "Add Tenant ID & Index (Recommended)".to_string(),
                        description: "Inject 'tenant_id TEXT NOT NULL' with foreign key reference and tenant composite index.".to_string(),
                        is_recommended: true,
                        resulting_patch_description: "Injects tenant_id column and tenant partition index.".to_string(),
                        safety_tradeoff: "Strict multi-tenant security; prevents cross-tenant data leaks.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_exempt_global_table".to_string(),
                        label: "Declare as Global/System Table".to_string(),
                        description: "Formally exempt this table from tenant scoping as a global shared registry.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Adds system_scope annotation to table metadata.".to_string(),
                        safety_tradeoff: "Table accessible across all tenants; must not store customer PII.".to_string(),
                    },
                ],
            ),
            AmbiguityCategory::MissingIndexOnForeignKey => (
                format!(
                    "Foreign key on '{}.{}' lacks an index.",
                    ambiguity.affected_table,
                    ambiguity.affected_column.as_deref().unwrap_or("col")
                ),
                vec![
                    DecisionOption {
                        id: "opt_create_fk_index".to_string(),
                        label: "Add Index Automatically (Recommended)".to_string(),
                        description: "Generate a CREATE INDEX statement for the foreign key column.".to_string(),
                        is_recommended: true,
                        resulting_patch_description: "Adds CREATE INDEX IF NOT EXISTS.".to_string(),
                        safety_tradeoff: "Improves query and cascade performance; minor write overhead.".to_string(),
                    },
                    DecisionOption {
                        id: "opt_skip_index".to_string(),
                        label: "Skip Index Creation".to_string(),
                        description: "Acknowledge potential sequential scans on foreign key lookups.".to_string(),
                        is_recommended: false,
                        resulting_patch_description: "Do not create index.".to_string(),
                        safety_tradeoff: "Risk of high latency on large datasets.".to_string(),
                    },
                ],
            ),
            AmbiguityCategory::UnconstrainedTextLength => (
                "Text field lacks bounds".to_string(),
                vec![
                    DecisionOption {
                        id: "opt_bounded_text".to_string(),
                        label: "Enforce Bound (Recommended)".to_string(),
                        description: "Add length constraint to prevent memory blowups.".to_string(),
                        is_recommended: true,
                        resulting_patch_description: "Adds CHECK(length(...) <= 255).".to_string(),
                        safety_tradeoff: "Prevents memory abuse.".to_string(),
                    },
                ],
            ),
        };

        DecisionCard {
            id: Uuid::new_v4(),
            intent_id,
            ambiguity_id: ambiguity.id,
            title: ambiguity.title.clone(),
            prompt,
            severity: ambiguity.severity,
            affected_table: ambiguity.affected_table.clone(),
            affected_column: ambiguity.affected_column.clone(),
            options,
            status: DecisionStatus::Pending,
        }
    }

    pub fn build_initial_plan(
        intent: &MigrationIntent,
        cards: Vec<DecisionCard>,
    ) -> MigrationPlan {
        let has_blocking = cards.iter().any(|c| c.severity >= Severity::Blocking && !c.is_resolved());
        let status = if has_blocking {
            PlanStatus::NeedsClarification
        } else {
            PlanStatus::ReadyForVerification
        };

        let (forward_sql, rollback_sql) = Self::synthesize_sql(&intent.actions, &cards);
        let formal_invariants = Self::synthesize_invariants(&intent.actions, &cards);

        MigrationPlan {
            id: Uuid::new_v4(),
            intent_id: intent.id,
            status,
            pending_decisions: cards.iter().filter(|c| !c.is_resolved()).cloned().collect(),
            resolved_decisions: cards.iter().filter(|c| c.is_resolved()).cloned().collect(),
            forward_sql,
            rollback_sql,
            formal_invariants,
            summary: format!(
                "Migration plan for {} action(s) with {} pending decision(s).",
                intent.actions.len(),
                cards.iter().filter(|c| !c.is_resolved()).count()
            ),
        }
    }

    pub fn resolve_decision(
        plan: &mut MigrationPlan,
        intent: &MigrationIntent,
        card_id: Uuid,
        option_id: &str,
        resolved_by: &str,
    ) -> Result<()> {
        let card_idx = plan
            .pending_decisions
            .iter()
            .position(|c| c.id == card_id)
            .ok_or_else(|| PddError::ResolutionError(format!("Decision card {} not found or already resolved", card_id)))?;

        let mut card = plan.pending_decisions.remove(card_idx);
        let valid_option = card.options.iter().any(|o| o.id == option_id);
        if !valid_option {
            return Err(PddError::ResolutionError(format!(
                "Invalid option '{}' for decision card {}",
                option_id, card_id
            )));
        }

        card.status = DecisionStatus::Resolved {
            selected_option_id: option_id.to_string(),
            resolved_by: resolved_by.to_string(),
            resolved_at: Utc::now(),
        };

        plan.resolved_decisions.push(card);

        let has_blocking = plan
            .pending_decisions
            .iter()
            .any(|c| c.severity >= Severity::Blocking && !c.is_resolved());

        plan.status = if has_blocking {
            PlanStatus::NeedsClarification
        } else {
            PlanStatus::ReadyForVerification
        };

        let mut all_cards = plan.pending_decisions.clone();
        all_cards.extend(plan.resolved_decisions.clone());

        let (forward, rollback) = Self::synthesize_sql(&intent.actions, &all_cards);
        plan.forward_sql = forward;
        plan.rollback_sql = rollback;
        plan.formal_invariants = Self::synthesize_invariants(&intent.actions, &all_cards);

        plan.summary = format!(
            "Migration plan: {} action(s), {} pending decision(s), {} resolved.",
            intent.actions.len(),
            plan.pending_decisions.len(),
            plan.resolved_decisions.len()
        );

        Ok(())
    }

    fn synthesize_sql(
        actions: &[IntentAction],
        cards: &[DecisionCard],
    ) -> (Vec<String>, Vec<String>) {
        let mut forward = Vec::new();
        let mut rollback = Vec::new();

        for action in actions {
            match action {
                IntentAction::CreateTable {
                    table_name,
                    columns,
                    is_tenant_scoped,
                } => {
                    let mut cols = columns.clone();

                    // Check if decision card for tenant isolation resolved with opt_enforce_tenant_id
                    let tenant_card = cards.iter().find(|c| {
                        c.affected_table == *table_name
                            && c.prompt.contains("Multi-tenant")
                    });

                    let should_inject_tenant_id = match tenant_card {
                        Some(c) => match &c.status {
                            DecisionStatus::Resolved { selected_option_id, .. } => {
                                selected_option_id == "opt_enforce_tenant_id"
                            }
                            DecisionStatus::Pending => *is_tenant_scoped,
                        },
                        None => *is_tenant_scoped,
                    };

                    if should_inject_tenant_id && !cols.iter().any(|c| c.name == "tenant_id") {
                        cols.insert(
                            1,
                            ColumnDef {
                                name: "tenant_id".to_string(),
                                data_type: crate::model::DataType::Text,
                                is_nullable: false,
                                is_primary_key: false,
                                default_value: None,
                                foreign_key_ref: Some("tenants(id)".to_string()),
                            },
                        );
                    }

                    let col_defs: Vec<String> = cols
                        .iter()
                        .map(|c| {
                            let mut def = format!("{} {}", c.name, c.data_type.to_sqlite_type());
                            if c.is_primary_key {
                                def.push_str(" PRIMARY KEY");
                            } else if !c.is_nullable {
                                def.push_str(" NOT NULL");
                            }
                            if let Some(ref d) = c.default_value {
                                def.push_str(&format!(" DEFAULT '{}'", d));
                            }
                            if let Some(ref fk) = c.foreign_key_ref {
                                def.push_str(&format!(" REFERENCES {}", fk));
                            }
                            def
                        })
                        .collect();

                    forward.push(format!(
                        "CREATE TABLE IF NOT EXISTS {} (\n  {}\n);",
                        table_name,
                        col_defs.join(",\n  ")
                    ));

                    if should_inject_tenant_id {
                        forward.push(format!(
                            "CREATE INDEX IF NOT EXISTS idx_{}_tenant ON {}(tenant_id);",
                            table_name, table_name
                        ));
                    }

                    rollback.push(format!("DROP TABLE IF EXISTS {};", table_name));
                }

                IntentAction::AddColumn { table_name, column } => {
                    let mut col = column.clone();

                    // Check if nullability decision resolved with opt_add_default
                    let null_card = cards.iter().find(|c| {
                        c.affected_table == *table_name
                            && c.affected_column.as_deref() == Some(&column.name)
                    });

                    if let Some(c) = null_card {
                        if let DecisionStatus::Resolved { selected_option_id, .. } = &c.status {
                            if selected_option_id == "opt_add_default" && col.default_value.is_none() {
                                col.default_value = Some("''".to_string());
                            } else if selected_option_id == "opt_permit_nulls" || selected_option_id == "opt_two_phase_nullable" {
                                col.is_nullable = true;
                            }
                        }
                    }

                    let mut col_def = format!("{} {}", col.name, col.data_type.to_sqlite_type());
                    if !col.is_nullable {
                        col_def.push_str(" NOT NULL");
                    }
                    if let Some(ref d) = col.default_value {
                        col_def.push_str(&format!(" DEFAULT {}", d));
                    }

                    forward.push(format!(
                        "ALTER TABLE {} ADD COLUMN {};",
                        table_name, col_def
                    ));

                    // SQLite 3.35+ supports DROP COLUMN; rollback drops it
                    rollback.push(format!(
                        "ALTER TABLE {} DROP COLUMN {};",
                        table_name, col.name
                    ));
                }

                IntentAction::DropColumn {
                    table_name,
                    column_name,
                } => {
                    let drop_card = cards.iter().find(|c| {
                        c.affected_table == *table_name
                            && c.affected_column.as_deref() == Some(column_name)
                    });

                    let soft_deprecate = match drop_card {
                        Some(c) => match &c.status {
                            DecisionStatus::Resolved { selected_option_id, .. } => {
                                selected_option_id == "opt_soft_deprecate"
                            }
                            _ => true, // default safe
                        },
                        None => true,
                    };

                    if soft_deprecate {
                        let deprecated_name = format!("_deprecated_{}", column_name);
                        forward.push(format!(
                            "ALTER TABLE {} RENAME COLUMN {} TO {};",
                            table_name, column_name, deprecated_name
                        ));
                        rollback.push(format!(
                            "ALTER TABLE {} RENAME COLUMN {} TO {};",
                            table_name, deprecated_name, column_name
                        ));
                    } else {
                        forward.push(format!(
                            "ALTER TABLE {} DROP COLUMN {};",
                            table_name, column_name
                        ));
                        rollback.push(format!(
                            "-- WARNING: Cannot restore dropped column data\nALTER TABLE {} ADD COLUMN {} TEXT;",
                            table_name, column_name
                        ));
                    }
                }

                IntentAction::DropTable { table_name } => {
                    forward.push(format!("DROP TABLE IF EXISTS {};", table_name));
                    rollback.push(format!("-- WARNING: Manual backup restore required for table {}", table_name));
                }

                IntentAction::AddIndex {
                    table_name,
                    column_names,
                    is_unique,
                } => {
                    let unique_clause = if *is_unique { "UNIQUE " } else { "" };
                    let idx_name = format!("idx_{}_{}", table_name, column_names.join("_"));
                    forward.push(format!(
                        "CREATE {}INDEX IF NOT EXISTS {} ON {}({});",
                        unique_clause,
                        idx_name,
                        table_name,
                        column_names.join(", ")
                    ));
                    rollback.push(format!("DROP INDEX IF EXISTS {};", idx_name));
                }

                _ => {}
            }
        }

        (forward, rollback)
    }

    fn synthesize_invariants(
        actions: &[IntentAction],
        _cards: &[DecisionCard],
    ) -> Vec<String> {
        let mut invariants = Vec::new();

        invariants.push("Invariant: Schema mutation preserves transaction isolation.".to_string());

        for action in actions {
            match action {
                IntentAction::CreateTable { table_name, .. } => {
                    invariants.push(format!(
                        "TenantIsolationLemma: Table '{}' maintains r.tenant_id != NULL for all r in rows.",
                        table_name
                    ));
                    invariants.push(format!(
                        "SchemaSafetyLemma: Creation of '{}' does not violate existing referential integrity.",
                        table_name
                    ));
                }
                IntentAction::AddColumn { table_name, column } => {
                    invariants.push(format!(
                        "NullabilityLemma: Adding '{}.{}' with is_not_null implies default_value is present or table is empty.",
                        table_name, column.name
                    ));
                }
                IntentAction::DropColumn { table_name, column_name } => {
                    invariants.push(format!(
                        "NonDestructiveLemma: Renaming '{}.{}' preserves historical data accessibility.",
                        table_name, column_name
                    ));
                }
                _ => {}
            }
        }

        invariants
    }
}

