use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NullabilitySpec {
    pub table_name: String,
    pub column_name: String,
    pub is_not_null: bool,
    pub has_default: bool,
    pub table_is_empty_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NullabilityProofResult {
    pub table_name: String,
    pub column_name: String,
    pub satisfied: bool,
    pub obligations: Vec<String>,
}

pub struct NullabilityContract;

impl NullabilityContract {
    /// Formally evaluates whether column addition satisfies the Nullability Invariant:
    /// is_not_null => (has_default || table_is_empty)
    pub fn verify(spec: &NullabilitySpec) -> NullabilityProofResult {
        let mut obligations = Vec::new();
        let satisfied;

        if !spec.is_not_null {
            satisfied = true;
            obligations.push(format!(
                "Trivial: Column '{}.{}' is nullable (⊥ ∈ Dom(C)) [VERIFIED]",
                spec.table_name, spec.column_name
            ));
        } else if spec.has_default {
            satisfied = true;
            obligations.push(format!(
                "Lemma: NOT NULL column '{}.{}' has valid DEFAULT; existing rows backfilled automatically [VERIFIED]",
                spec.table_name, spec.column_name
            ));
        } else if spec.table_is_empty_proven {
            satisfied = true;
            obligations.push(format!(
                "Lemma: Table '{}' has zero existing records (|T| = 0); NOT NULL holds vacuously [VERIFIED]",
                spec.table_name
            ));
        } else {
            satisfied = false;
            obligations.push(format!(
                "Violation: Adding NOT NULL column '{}.{}' without DEFAULT on populated table violates invariant [FAILED]",
                spec.table_name, spec.column_name
            ));
        }

        NullabilityProofResult {
            table_name: spec.table_name.clone(),
            column_name: spec.column_name.clone(),
            satisfied,
            obligations,
        }
    }
}

