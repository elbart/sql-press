use crate::column::{ColumnType, Constraints};

pub mod postgres;

pub trait SqlDialect {
    fn create_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String;

    fn alter_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String;

    fn rename_table(&self, schema: &str, name: &str, new_table_name: &str) -> String;

    fn drop_table(&self, schema: &str, name: &str) -> String;

    fn add_column(
        &self,
        name: &str,
        with_prefix: bool,
        ct: &ColumnType,
        constraints: &Constraints,
    ) -> String;

    fn rename_column(&self, name: &str, new_name: &str) -> String;

    fn alter_column(&self, name: &str, ct: &ColumnType, conversion_method: Option<&str>) -> String;

    fn drop_column(&self, name: &str, if_exists: bool) -> String;

    fn column_type(&self, ct: &ColumnType) -> String;

    fn constraints(&self, constraints: &Constraints) -> String;
}
