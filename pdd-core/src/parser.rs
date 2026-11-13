use crate::error::{PddError, Result};
use crate::model::{ColumnDef, DataType, ForeignKeyAction, IntentAction, MigrationIntent};
use regex::Regex;

pub struct IntentParser;

impl IntentParser {
    pub fn parse(raw_prompt: &str) -> Result<MigrationIntent> {
        let cleaned = raw_prompt.trim();
        if cleaned.is_empty() {
            return Err(PddError::ParseError("Prompt cannot be empty".to_string()));
        }

        let mut actions = Vec::new();

        // Split statements by semicolons or newline boundaries
        let statements: Vec<&str> = cleaned
            .split([';', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for stmt in statements {
            // Also split compound statements connected by " and " if it begins an action phrase
            let sub_statements = Self::split_compound_phrases(stmt);
            for sub in sub_statements {
                if let Some(action) = Self::parse_single_statement(sub)? {
                    actions.push(action);
                }
            }
        }

        if actions.is_empty() {
            // Try generic fallback parser
            if let Some(action) = Self::parse_heuristic_fallback(cleaned)? {
                actions.push(action);
            } else {
                return Err(PddError::ParseError(format!(
                    "Could not extract database migration actions from input: \"{}\"",
                    cleaned
                )));
            }
        }

        Ok(MigrationIntent::new(cleaned, actions))
    }

    fn split_compound_phrases(statement: &str) -> Vec<&str> {
        let keywords = ["add ", "create ", "drop ", "alter ", "remove "];
        let mut result = Vec::new();
        let mut current = statement;

        while let Some(idx) = current.to_lowercase().find(" and ") {
            let after_and = current[idx + 5..].trim_start();
            let after_lower = after_and.to_lowercase();
            let starts_action = keywords.iter().any(|kw| after_lower.starts_with(kw));
            if starts_action {
                let left = current[..idx].trim();
                if !left.is_empty() {
                    result.push(left);
                }
                current = after_and;
            } else {
                break;
            }
        }
        let rem = current.trim();
        if !rem.is_empty() {
            result.push(rem);
        }
        result
    }

    fn parse_single_statement(stmt: &str) -> Result<Option<IntentAction>> {
        let s = stmt.trim();

        // 1. Create table
        let create_table_re = Regex::new(r"(?i)^(?:create\s+table|add\s+(?:a\s+)?(?:tenant-aware\s+|multi-tenant\s+)?table)\s+([a-zA-Z0-9_]+)(?:\s+(?:with(?:\s+columns)?|\()?\s*(.*)\)?)?$").unwrap();
        if let Some(caps) = create_table_re.captures(s) {
            let table_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let raw_cols = caps.get(2).map_or("", |m| m.as_str());
            let is_tenant_flag = s.to_lowercase().contains("tenant");

            let columns = if raw_cols.is_empty() {
                // If no columns specified, at least default to primary key id
                vec![ColumnDef {
                    name: "id".to_string(),
                    data_type: DataType::Uuid,
                    is_nullable: false,
                    is_primary_key: true,
                    default_value: None,
                    foreign_key_ref: None,
                }]
            } else {
                Self::parse_column_list(raw_cols)
            };

            return Ok(Some(IntentAction::CreateTable {
                table_name,
                columns,
                is_tenant_scoped: is_tenant_flag,
            }));
        }

        // 2. Drop table
        let drop_table_re = Regex::new(r"(?i)^(?:drop|delete|remove)\s+table\s+([a-zA-Z0-9_]+)").unwrap();
        if let Some(caps) = drop_table_re.captures(s) {
            let table_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            return Ok(Some(IntentAction::DropTable { table_name }));
        }

        // 3. Drop column
        let drop_col_re = Regex::new(r"(?i)^(?:drop|delete|remove)\s+(?:column\s+)?([a-zA-Z0-9_]+)\s+from\s+(?:table\s+)?([a-zA-Z0-9_]+)").unwrap();
        if let Some(caps) = drop_col_re.captures(s) {
            let column_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let table_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
            return Ok(Some(IntentAction::DropColumn {
                table_name,
                column_name,
            }));
        }

        // 4. Add column
        let add_col_re = Regex::new(r"(?i)^add\s+(?:column\s+)?([a-zA-Z0-9_]+)(?:\s+([a-zA-Z0-9_]+))?(?:\s+(as\s+not\s+null|not\s+null|nullable))?(?:\s+with\s+default\s+([^\s]+|\'[^\']*\'))?\s+to\s+(?:table\s+)?([a-zA-Z0-9_]+)").unwrap();
        if let Some(caps) = add_col_re.captures(s) {
            let col_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let type_str = caps.get(2).map(|m| m.as_str()).unwrap_or("text");
            let null_clause = caps.get(3).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
            let default_val = caps.get(4).map(|m| m.as_str().trim_matches('\'').to_string());
            let table_name = caps.get(5).map_or("", |m| m.as_str()).to_string();

            let is_nullable = !null_clause.contains("not null");
            let data_type = Self::parse_data_type(type_str);

            return Ok(Some(IntentAction::AddColumn {
                table_name,
                column: ColumnDef {
                    name: col_name,
                    data_type,
                    is_nullable,
                    is_primary_key: false,
                    default_value: default_val,
                    foreign_key_ref: None,
                },
            }));
        }

        // 5. Alter column
        let alter_col_re = Regex::new(r"(?i)^alter\s+(?:column\s+)?([a-zA-Z0-9_]+)\s+in\s+(?:table\s+)?([a-zA-Z0-9_]+)\s+to\s+(.*)").unwrap();
        if let Some(caps) = alter_col_re.captures(s) {
            let column_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let table_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
            let details = caps.get(3).map_or("", |m| m.as_str());

            let mut modifications = Vec::new();
            if details.to_lowercase().contains("not null") {
                modifications.push(crate::model::ColumnModification::SetNullable(false));
            } else if details.to_lowercase().contains("nullable") {
                modifications.push(crate::model::ColumnModification::SetNullable(true));
            }

            return Ok(Some(IntentAction::AlterColumn {
                table_name,
                column_name,
                modifications,
            }));
        }

        // 6. Add index
        let add_idx_re = Regex::new(r"(?i)^add\s+(unique\s+)?index\s+on\s+([a-zA-Z0-9_]+)\s*\((.*?)\)").unwrap();
        if let Some(caps) = add_idx_re.captures(s) {
            let is_unique = caps.get(1).is_some();
            let table_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
            let col_names = caps
                .get(3)
                .map_or("", |m| m.as_str())
                .split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();

            return Ok(Some(IntentAction::AddIndex {
                table_name,
                column_names: col_names,
                is_unique,
            }));
        }

        // 7. Add foreign key
        let add_fk_re = Regex::new(r"(?i)^add\s+foreign\s+key\s+from\s+([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)\s+to\s+([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)").unwrap();
        if let Some(caps) = add_fk_re.captures(s) {
            return Ok(Some(IntentAction::AddForeignKey {
                from_table: caps.get(1).map_or("", |m| m.as_str()).to_string(),
                from_column: caps.get(2).map_or("", |m| m.as_str()).to_string(),
                to_table: caps.get(3).map_or("", |m| m.as_str()).to_string(),
                to_column: caps.get(4).map_or("", |m| m.as_str()).to_string(),
                on_delete: ForeignKeyAction::Cascade,
            }));
        }

        Ok(None)
    }

    fn parse_column_list(cols_str: &str) -> Vec<ColumnDef> {
        let cleaned = cols_str.trim_matches(|c| c == '(' || c == ')' || c == '{' || c == '}');
        let parts: Vec<&str> = cleaned.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();

        parts
            .into_iter()
            .map(|col_spec| {
                let tokens: Vec<&str> = col_spec.split_whitespace().collect();
                let col_name = tokens.first().copied().unwrap_or("unnamed");
                let type_token = tokens.get(1).copied().unwrap_or("text");
                let data_type = Self::parse_data_type(type_token);
                let col_lower = col_spec.to_lowercase();

                let is_primary_key = col_lower.contains("primary key") || col_name == "id";
                let is_nullable = if is_primary_key {
                    false
                } else {
                    !col_lower.contains("not null")
                };

                // Look for default
                let default_value = if let Some(idx) = col_lower.find("default") {
                    let rem = &col_spec[idx + 7..].trim();
                    rem.split_whitespace().next().map(|v| v.trim_matches('\'').to_string())
                } else {
                    None
                };

                ColumnDef {
                    name: col_name.to_string(),
                    data_type,
                    is_nullable,
                    is_primary_key,
                    default_value,
                    foreign_key_ref: None,
                }
            })
            .collect()
    }

    fn parse_data_type(raw: &str) -> DataType {
        match raw.to_lowercase().as_str() {
            "int" | "integer" | "i32" | "int4" => DataType::Integer,
            "bigint" | "i64" | "int8" => DataType::BigInt,
            "text" | "varchar" | "string" | "char" => DataType::Text,
            "bool" | "boolean" => DataType::Boolean,
            "timestamp" | "datetime" | "date" => DataType::Timestamp,
            "json" | "jsonb" => DataType::Json,
            "uuid" => DataType::Uuid,
            "real" | "float" | "double" | "f64" => DataType::Real,
            "blob" | "bytea" | "bytes" => DataType::Blob,
            other => DataType::Custom(other.to_string()),
        }
    }

    fn parse_heuristic_fallback(input: &str) -> Result<Option<IntentAction>> {
        let lower = input.to_lowercase();

        // Check if intent is adding a column to a table: e.g. "Add priority column to tasks table"
        if lower.contains("add") && (lower.contains("column") || lower.contains("field")) && lower.contains("to") {
            let re = Regex::new(r"(?i)add\s+(?:column\s+|field\s+)?([a-zA-Z0-9_]+)\s+to\s+(?:table\s+)?([a-zA-Z0-9_]+)").unwrap();
            if let Some(caps) = re.captures(input) {
                let col_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let table_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let not_null = lower.contains("not null");
                return Ok(Some(IntentAction::AddColumn {
                    table_name,
                    column: ColumnDef {
                        name: col_name,
                        data_type: DataType::Text,
                        is_nullable: !not_null,
                        is_primary_key: false,
                        default_value: None,
                        foreign_key_ref: None,
                    },
                }));
            }
        }

        // Check if intent is dropping something: e.g. "remove legacy_id from users"
        if (lower.contains("drop") || lower.contains("remove") || lower.contains("delete")) && lower.contains("from") {
            let re = Regex::new(r"(?i)(?:drop|remove|delete)\s+(?:column\s+)?([a-zA-Z0-9_]+)\s+from\s+(?:table\s+)?([a-zA-Z0-9_]+)").unwrap();
            if let Some(caps) = re.captures(input) {
                let col_name = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let table_name = caps.get(2).map_or("", |m| m.as_str()).to_string();
                return Ok(Some(IntentAction::DropColumn {
                    table_name,
                    column_name: col_name,
                }));
            }
        }

        Ok(None)
    }
}
