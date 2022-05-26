//! Central trait definition for what an [SqlDialect] implementation has to support.
use crate::column::{ColumnType, Constraints};

pub mod postgres;
pub use postgres::Postgres;

pub trait SqlDialect {
    fn create_table(&self, name: &str, changes: Vec<String>, if_not_exists: bool) -> String;

    fn alter_table(&self, name: &str, changes: Vec<String>) -> String;

    fn rename_table(&self, name: &str, new_table_name: &str) -> String;

    fn drop_table(&self, name: &str) -> String;

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

    fn add_index(&self, table_name: &str, columns: &[String], idx_name: &Option<String>) -> String;

    fn add_foreign_index(
        &self,
        column_name: &str,
        foreign_table_name: &str,
        foreign_column_name: &str,
        idx_name: Option<String>,
        add_clause: &bool,
    ) -> String;

    fn add_primary_index(&self, columns: &Vec<String>) -> String;

    fn column_type(&self, ct: &ColumnType) -> String;

    fn constraints(&self, constraints: &Constraints) -> String;
}
