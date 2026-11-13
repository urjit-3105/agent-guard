use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    BigInt,
    Text,
    Boolean,
    Timestamp,
    Json,
    Uuid,
    Real,
    Blob,
    Custom(String),
}

impl DataType {
    pub fn to_sqlite_type(&self) -> &'static str {
        match self {
            DataType::Integer | DataType::BigInt | DataType::Boolean => "INTEGER",
            DataType::Text | DataType::Json | DataType::Uuid => "TEXT",
            DataType::Timestamp => "DATETIME",
            DataType::Real => "REAL",
            DataType::Blob => "BLOB",
            DataType::Custom(_) => "TEXT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}

impl ForeignKeyAction {
    pub fn to_sql(&self) -> &'static str {
        match self {
            ForeignKeyAction::Cascade => "CASCADE",
            ForeignKeyAction::SetNull => "SET NULL",
            ForeignKeyAction::Restrict => "RESTRICT",
            ForeignKeyAction::NoAction => "NO ACTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub foreign_key_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnModification {
    SetNullable(bool),
    SetDefault(Option<String>),
    ChangeType(DataType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentAction {
    CreateTable {
        table_name: String,
        columns: Vec<ColumnDef>,
        is_tenant_scoped: bool,
    },
    AddColumn {
        table_name: String,
        column: ColumnDef,
    },
    DropColumn {
        table_name: String,
        column_name: String,
    },
    AlterColumn {
        table_name: String,
        column_name: String,
        modifications: Vec<ColumnModification>,
    },
    DropTable {
        table_name: String,
    },
    AddIndex {
        table_name: String,
        column_names: Vec<String>,
        is_unique: bool,
    },
    AddForeignKey {
        from_table: String,
        from_column: String,
        to_table: String,
        to_column: String,
        on_delete: ForeignKeyAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationIntent {
    pub id: Uuid,
    pub raw_prompt: String,
    pub actions: Vec<IntentAction>,
    pub created_at: DateTime<Utc>,
}

impl MigrationIntent {
    pub fn new(raw_prompt: impl Into<String>, actions: Vec<IntentAction>) -> Self {
        Self {
            id: Uuid::new_v4(),
            raw_prompt: raw_prompt.into(),
            actions,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmbiguityCategory {
    DestructiveChange,
    TenantIsolationViolation,
    NullabilityMissingDefault,
    MissingIndexOnForeignKey,
    UnconstrainedTextLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Blocking,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ambiguity {
    pub id: Uuid,
    pub category: AmbiguityCategory,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub affected_table: String,
    pub affected_column: Option<String>,
    pub recommended_action: String,
}

impl Ambiguity {
    pub fn new(
        category: AmbiguityCategory,
        severity: Severity,
        title: impl Into<String>,
        description: impl Into<String>,
        affected_table: impl Into<String>,
        affected_column: Option<String>,
        recommended_action: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            severity,
            title: title.into(),
            description: description.into(),
            affected_table: affected_table.into(),
            affected_column,
            recommended_action: recommended_action.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub is_recommended: bool,
    pub resulting_patch_description: String,
    pub safety_tradeoff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Pending,
    Resolved {
        selected_option_id: String,
        resolved_by: String,
        resolved_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionCard {
    pub id: Uuid,
    pub intent_id: Uuid,
    pub ambiguity_id: Uuid,
    pub title: String,
    pub prompt: String,
    pub severity: Severity,
    pub affected_table: String,
    pub affected_column: Option<String>,
    pub options: Vec<DecisionOption>,
    pub status: DecisionStatus,
}

impl DecisionCard {
    pub fn is_resolved(&self) -> bool {
        matches!(self.status, DecisionStatus::Resolved { .. })
    }

    pub fn selected_option(&self) -> Option<&DecisionOption> {
        match &self.status {
            DecisionStatus::Resolved {
                selected_option_id, ..
            } => self.options.iter().find(|opt| &opt.id == selected_option_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    NeedsClarification,
    ReadyForVerification,
    Verified,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: Uuid,
    pub intent_id: Uuid,
    pub status: PlanStatus,
    pub pending_decisions: Vec<DecisionCard>,
    pub resolved_decisions: Vec<DecisionCard>,
    pub forward_sql: Vec<String>,
    pub rollback_sql: Vec<String>,
    pub formal_invariants: Vec<String>,
    pub summary: String,
}

impl MigrationPlan {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn to_json_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn schema_mapping_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "plan_id": self.id,
            "intent_id": self.intent_id,
            "status": format!("{:?}", self.status),
            "pending_count": self.pending_decisions.len(),
            "resolved_count": self.resolved_decisions.len(),
            "forward_statement_count": self.forward_sql.len(),
            "rollback_statement_count": self.rollback_sql.len(),
            "invariants_count": self.formal_invariants.len(),
        })
    }
}

