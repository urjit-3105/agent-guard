use crate::model::{
    Ambiguity, AmbiguityCategory, IntentAction, MigrationIntent, Severity,
};

pub struct AmbiguityAnalyzer;

impl AmbiguityAnalyzer {
    pub fn analyze(intent: &MigrationIntent) -> Vec<Ambiguity> {
        let mut ambiguities = Vec::new();

        for action in &intent.actions {
            match action {
                IntentAction::CreateTable {
                    table_name,
                    columns,
                    is_tenant_scoped,
                } => {
                    // Check Tenant Isolation
                    let is_tenant_candidate = *is_tenant_scoped
                        || Self::is_likely_tenant_table(table_name);

                    let has_tenant_col = columns.iter().any(|c| {
                        let name_lower = c.name.to_lowercase();
                        name_lower == "tenant_id" || name_lower == "organization_id" || name_lower == "org_id"
                    });

                    if is_tenant_candidate && !has_tenant_col {
                        ambiguities.push(Ambiguity::new(
                            AmbiguityCategory::TenantIsolationViolation,
                            Severity::Blocking,
                            format!("Missing tenant isolation in '{}'", table_name),
                            format!(
                                "Table '{}' appears to store tenant/customer data but lacks an explicit non-nullable 'tenant_id' column.",
                                table_name
                            ),
                            table_name.clone(),
                            None,
                            "Add 'tenant_id TEXT NOT NULL' with foreign key reference to tenants table.",
                        ));
                    }
                }

                IntentAction::AddColumn { table_name, column } => {
                    // Check Nullability without default on existing table
                    if !column.is_nullable && column.default_value.is_none() {
                        ambiguities.push(Ambiguity::new(
                            AmbiguityCategory::NullabilityMissingDefault,
                            Severity::Blocking,
                            format!("NOT NULL column '{}' has no DEFAULT value", column.name),
                            format!(
                                "Adding non-nullable column '{}' to table '{}' without a DEFAULT value will fail on existing rows.",
                                column.name, table_name
                            ),
                            table_name.clone(),
                            Some(column.name.clone()),
                            "Provide a fallback DEFAULT value or introduce column as NULLABLE followed by a two-phase backfill.",
                        ));
                    }
                }

                IntentAction::DropColumn {
                    table_name,
                    column_name,
                } => {
                    // Check Destructive drop column
                    ambiguities.push(Ambiguity::new(
                        AmbiguityCategory::DestructiveChange,
                        Severity::Blocking,
                        format!("Destructive drop of column '{}.{}'", table_name, column_name),
                        format!(
                            "Dropping column '{}' from table '{}' permanently deletes historical data and causes breaking schema changes.",
                            column_name, table_name
                        ),
                        table_name.clone(),
                        Some(column_name.clone()),
                        "Soft-deprecate the column with a prefix (e.g. '_deprecated_') or create a backward-compatible view.",
                    ));
                }

                IntentAction::DropTable { table_name } => {
                    // Check Destructive drop table
                    ambiguities.push(Ambiguity::new(
                        AmbiguityCategory::DestructiveChange,
                        Severity::Critical,
                        format!("Critical table deletion: '{}'", table_name),
                        format!(
                            "Permanently dropping table '{}' destroys all associated records and cascades.",
                            table_name
                        ),
                        table_name.clone(),
                        None,
                        "Archive table to cold storage before dropping, or perform soft deletion.",
                    ));
                }

                IntentAction::AddForeignKey {
                    from_table,
                    from_column,
                    ..
                } => {
                    // Check for index on foreign key
                    ambiguities.push(Ambiguity::new(
                        AmbiguityCategory::MissingIndexOnForeignKey,
                        Severity::Warning,
                        format!("Unindexed foreign key '{}.{}'", from_table, from_column),
                        format!(
                            "Foreign key on '{}.{}' should have a supporting index to prevent full-table scans during joins and deletion cascades.",
                            from_table, from_column
                        ),
                        from_table.clone(),
                        Some(from_column.clone()),
                        format!("Create index 'idx_{}_{}' concurrently.", from_table, from_column),
                    ));
                }

                _ => {}
            }
        }

        ambiguities
    }

    fn is_likely_tenant_table(name: &str) -> bool {
        let lower = name.to_lowercase();
        let tenant_keywords = [
            "billing", "invoice", "order", "project", "task", "document", "file",
            "account", "user", "audit", "subscription", "member", "customer",
            "setting", "item", "workspace", "org", "metric",
        ];

        // System tables exempt
        if lower.starts_with("system_") || lower.starts_with("schema_") || lower == "tenants" || lower == "migrations" {
            return false;
        }

        tenant_keywords.iter().any(|&k| lower.contains(k))
    }
}

