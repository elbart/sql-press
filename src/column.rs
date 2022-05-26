//! Provides functionality for add/modifying columns. Also provides convenience
//! methods for defining different data types of columns.
use std::rc::Rc;

use crate::{
    change::Change,
    index::{IndexAdd, IndexAlter},
    sql_dialect::SqlDialect,
    table::Table,
};

#[derive(Debug, Clone)]
pub enum ColumnChangeOp {
    Create,
    Drop,
}

#[derive(Debug, Clone)]
pub struct Constraints {
    pub(crate) primary: bool,
    pub(crate) not_null: bool,
    pub(crate) unique: bool,
    pub(crate) default: DefaultConstraint,
}

#[derive(Debug, Clone)]
pub enum DefaultConstraint {
    None,
    Plain(String),
}

impl Constraints {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            primary: false,
            not_null: false,
            unique: false,
            default: DefaultConstraint::None,
        }
    }
}

#[derive(Debug)]
pub struct ColumnRenameChange {
    pub(crate) name: String,
    pub(crate) new_name: String,
}

impl Change for ColumnRenameChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.rename_column(&self.name, &self.new_name)
    }
}

#[derive(Debug)]
pub struct ColumnAlterChange {
    pub(crate) name: String,
    pub(crate) ct: ColumnType,
    pub(crate) conversion_method: Option<String>,
}

impl Change for ColumnAlterChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.alter_column(&self.name, &self.ct, self.conversion_method.as_deref())
    }
}

#[derive(Debug)]
pub struct ColumnDropChange {
    pub(crate) name: String,
    pub(crate) if_exists: bool,
}

impl Change for ColumnDropChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.drop_column(&self.name, self.if_exists)
    }
}

#[derive(Debug, Clone)]
pub struct ColumnAddChange {
    pub(crate) name: String,
    pub(crate) ct: ColumnType,
    pub(crate) with_prefix: bool,
    constraints: Constraints,
}

impl ColumnAddChange {
    pub fn new(name: &str, ct: ColumnType) -> Self {
        Self {
            name: name.into(),
            ct,
            with_prefix: false,
            constraints: Constraints::new(),
        }
    }
}

impl Change for ColumnAddChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.add_column(&self.name, self.with_prefix, &self.ct, &self.constraints)
    }
}

pub struct ColumnAddBuilder {
    inner: ColumnAddChange,
}

impl ColumnAddBuilder {
    pub fn new(name: &str, ct: ColumnType) -> Self {
        Self {
            inner: ColumnAddChange::new(name, ct),
        }
    }

    pub fn primary(mut self, primary: bool) -> Self {
        self.inner.constraints.primary = primary;

        self
    }

    pub fn not_null(mut self, not_null: bool) -> Self {
        self.inner.constraints.not_null = not_null;

        self
    }

    pub fn unique(mut self, unique: bool) -> Self {
        self.inner.constraints.unique = unique;

        self
    }

    pub fn default(mut self, default: DefaultConstraint) -> Self {
        self.inner.constraints.default = default;

        self
    }

    pub fn build(self) -> ColumnAddChange {
        self.inner
    }
}

pub fn uuid(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::UUID)
}

pub fn bool(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::BOOL)
}

pub fn varchar(name: &str, size: Option<usize>) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::VARCHAR(size.unwrap_or(255)))
}

pub fn real(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::REAL)
}

pub fn text(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::TEXT)
}

pub fn timestamp(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::TIMESTAMP)
}

pub fn timestamp_tz(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::TIMESTAMPTZ)
}

pub fn integer(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::INTEGER)
}

pub fn jsonb(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::JSONB)
}

/// Available column types (still partially postgres specific). The crates user
/// needs to be made aware of this fact.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    UUID,
    BOOL,
    VARCHAR(usize),
    REAL,
    INTEGER,
    TEXT,
    TIMESTAMP,
    TIMESTAMPTZ,
    JSONB,
}

pub trait ColumnAdd {
    fn add_column(&mut self, column: ColumnAddChange);
}

pub trait ColumnDrop {
    fn drop_column(&mut self, name: &str);
    fn drop_column_if_exists(&mut self, name: &str);
}

impl ColumnAdd for Table {
    fn add_column(&mut self, column: ColumnAddChange) {
        self.changes.push(Box::new(column));
    }
}

impl ColumnDrop for Table {
    fn drop_column(&mut self, name: &str) {
        self.changes.push(Box::new(ColumnDropChange {
            name: name.into(),
            if_exists: false,
        }))
    }

    fn drop_column_if_exists(&mut self, name: &str) {
        self.changes.push(Box::new(ColumnDropChange {
            name: name.into(),
            if_exists: true,
        }))
    }
}

pub trait ColumnCreate: ColumnAdd + IndexAdd {}
impl ColumnCreate for Table {}

pub trait ColumnAlter: ColumnDrop + IndexAlter {
    fn add_column(&mut self, column: ColumnAddChange);

    fn rename_column(&mut self, column_name: &str, new_column_name: &str);

    fn alter_column(
        &mut self,
        column_name: &str,
        new_column_type: ColumnType,
        conversion_method: Option<String>,
    );
}

impl ColumnAlter for Table {
    fn add_column(&mut self, column: ColumnAddChange) {
        let mut alter_column = column;
        alter_column.with_prefix = true;
        self.changes.push(Box::new(alter_column));
    }

    fn rename_column(&mut self, name: &str, new_name: &str) {
        self.changes.push(Box::new(ColumnRenameChange {
            name: name.into(),
            new_name: new_name.into(),
        }))
    }

    fn alter_column(
        &mut self,
        column_name: &str,
        new_column_type: ColumnType,
        conversion_method: Option<String>,
    ) {
        self.changes.push(Box::new(ColumnAlterChange {
            name: column_name.into(),
            ct: new_column_type,
            conversion_method,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column;

    fn get_downcasted_column_change<C: Change>(table: &Table, idx: usize) -> &C {
        table
            .changes
            .get(idx)
            .unwrap()
            .as_any()
            .downcast_ref::<C>()
            .unwrap()
    }

    #[test]
    fn column_add_change() {
        let mut t = Table::default();
        column::ColumnAdd::add_column(&mut t, uuid("id").primary(true).build());
        column::ColumnAdd::add_column(&mut t, uuid("id2").primary(false).build());
        column::ColumnAlter::add_column(&mut t, varchar("id2", None).build());
        assert!(t.changes.len() == 3);

        let col: &ColumnAddChange = get_downcasted_column_change(&t, 0);
        let col2: &ColumnAddChange = get_downcasted_column_change(&t, 2);
        assert!(col.with_prefix == false);
        assert!(col2.with_prefix == true);
    }

    #[test]
    fn column_alter_change() {
        let mut t = Table::default();
        column::ColumnAlter::add_column(&mut t, varchar("id2", None).build());
        column::ColumnAlter::alter_column(&mut t, "id2", ColumnType::UUID, None);
        column::ColumnAlter::rename_column(&mut t, "id2", "id3");
        assert!(t.changes.len() == 3);

        let col: &ColumnAddChange = get_downcasted_column_change(&t, 0);
        let col2: &ColumnAlterChange = get_downcasted_column_change(&t, 1);
        let col3: &ColumnRenameChange = get_downcasted_column_change(&t, 2);

        assert!(col.ct == ColumnType::VARCHAR(255));
        assert!(col2.ct == ColumnType::UUID);
        assert!(col3.new_name == "id3".to_string());
    }

    #[test]
    fn column_drop_change() {
        let mut t = Table::default();
        column::ColumnDrop::drop_column(&mut t, "test");
        column::ColumnDrop::drop_column_if_exists(&mut t, "test");
        assert!(t.changes.len() == 2);

        let col: &ColumnDropChange = get_downcasted_column_change(&t, 0);
        let col2: &ColumnDropChange = get_downcasted_column_change(&t, 1);
        assert!(col.if_exists == false);
        assert!(col.name == "test".to_string());
        assert!(col2.if_exists == true);
        assert!(col2.name == "test".to_string());
    }

    #[test]
    fn column_add_builder() {
        let cb = ColumnAddBuilder::new("id", ColumnType::UUID);

        assert_eq!(cb.inner.name, "id");
        assert_eq!(cb.inner.ct, ColumnType::UUID);

        assert_eq!(cb.inner.constraints.primary, false);
        assert_eq!(cb.inner.constraints.not_null, false);
        assert_eq!(cb.inner.constraints.unique, false);

        let cb = cb.primary(true);
        assert_eq!(cb.inner.constraints.primary, true);

        let cb = cb.not_null(true);
        assert_eq!(cb.inner.constraints.not_null, true);

        let cb = cb.unique(true);
        assert_eq!(cb.inner.constraints.unique, true);
    }
}
