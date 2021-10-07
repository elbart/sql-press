use crate::column::{ColumnType, Constraints};

use super::SqlDialect;

#[derive(Debug, Clone)]
pub struct Postgres {}
impl SqlDialect for Postgres {
    fn create_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String {
        format!(
            "CREATE TABLE {}.\"{}\" (\n{}\n);",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn alter_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String {
        format!(
            "ALTER TABLE {}.\"{}\"\n{};",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn rename_table(&self, schema: &str, name: &str, new_table_name: &str) -> String {
        format!(
            "ALTER TABLE {}.\"{}\" RENAME TO {}\"{}\";",
            schema, name, schema, new_table_name,
        )
    }

    fn drop_table(&self, schema: &str, name: &str) -> String {
        format!("DROP TABLE {}.\"{}\";", schema, name,)
    }

    fn add_column(
        &self,
        name: &str,
        with_prefix: bool,
        ct: &ColumnType,
        _constraints: &Constraints,
    ) -> String {
        let t = match ct {
            ColumnType::VARCHAR(s) => format!("VARCHAR({})", s),
            ColumnType::UUID => "uuid".into(),
        };

        format!(
            "{}{} {}",
            with_prefix.then(|| "ADD COLUMN ").unwrap_or(""),
            name,
            &t
        )
    }

    fn rename_column(&self, name: &str, new_name: &str) -> String {
        format!("RENAME COLUMN \"{}\" TO \"{}\"", name, new_name)
    }

    fn alter_column(&self, name: &str, ct: &ColumnType, conversion_method: Option<&str>) -> String {
        let t = match ct {
            ColumnType::VARCHAR(s) => format!("VARCHAR({})", s),
            ColumnType::UUID => "uuid".into(),
        };
        format!(
            "ALTER COLUMN \"{}\" TYPE {}{}",
            name,
            &t,
            conversion_method
                .map(|u| format!(" USING {}", u))
                .unwrap_or_else(|| "".into())
        )
    }

    fn drop_column(&self, name: &str, if_exists: bool) -> String {
        format!(
            "DROP COLUMN {}\"{}\"",
            if_exists.then(|| "IF EXISTS ").unwrap_or(""),
            name
        )
    }
}
