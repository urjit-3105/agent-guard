pub mod nullability_invariants;
pub mod schema_safety;
pub mod tenant_isolation;

pub use nullability_invariants::{NullabilityContract, NullabilityProofResult, NullabilitySpec};
pub use schema_safety::{SchemaMutationSpec, SchemaSafetyContract, SchemaSafetyProofResult};
pub use tenant_isolation::{
    TenantIsolationContract, TenantIsolationProofResult, TenantTableSpec,
};

impl TenantIsolationProofResult {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// Verus specification macro shim.
/// Enables seamless compilation under standard rustc while preserving
/// authentic Verus macro syntax for formal SMT solver runs.
#[macro_export]
macro_rules! verus_spec {
    ($($item:tt)*) => {
        $($item)*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_boundary_verification() {
        let valid_spec = TenantTableSpec {
            table_name: "billing_invoices".to_string(),
            has_tenant_id_column: true,
            is_tenant_id_not_null: true,
            references_tenants_table: true,
            has_tenant_index: true,
        };

        let result = TenantIsolationContract::verify_tenant_boundary(&valid_spec);
        assert!(result.lemma_satisfied);
        assert_eq!(result.proof_obligations.len(), 3);

        let invalid_spec = TenantTableSpec {
            table_name: "billing_invoices".to_string(),
            has_tenant_id_column: false,
            is_tenant_id_not_null: false,
            references_tenants_table: false,
            has_tenant_index: false,
        };

        let invalid_result = TenantIsolationContract::verify_tenant_boundary(&invalid_spec);
        assert!(!invalid_result.lemma_satisfied);
    }

    #[test]
    fn test_schema_safety_soft_deprecate() {
        let spec = SchemaMutationSpec {
            target_table: "users".to_string(),
            is_drop_column: true,
            is_drop_table: false,
            is_soft_deprecated: true,
            has_archive_backup: false,
        };

        let result = SchemaSafetyContract::verify_safety(&spec);
        assert!(result.non_destructive_proved);
        assert!(result.backward_compatible);
    }

    #[test]
    fn test_nullability_default_evaluation() {
        let spec_with_default = NullabilitySpec {
            table_name: "orders".to_string(),
            column_name: "status".to_string(),
            is_not_null: true,
            has_default: true,
            table_is_empty_proven: false,
        };
        assert!(NullabilityContract::verify(&spec_with_default).satisfied);

        let spec_without_default = NullabilitySpec {
            table_name: "orders".to_string(),
            column_name: "status".to_string(),
            is_not_null: true,
            has_default: false,
            table_is_empty_proven: false,
        };
        assert!(!NullabilityContract::verify(&spec_without_default).satisfied);
    }
}
