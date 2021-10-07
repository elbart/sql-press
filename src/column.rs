use std::rc::Rc;

use crate::{change::Change, sql_dialect::SqlDialect, table::Table};

#[derive(Debug, Clone)]
pub enum ColumnChangeOp {
    Create,
    Drop,
}

#[derive(Debug, Clone)]
pub struct Constraints {
    primary: bool,
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
        Self { primary: false }
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

    pub fn build(self) -> ColumnAddChange {
        self.inner
    }
}

pub fn uuid(name: &str) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::UUID)
}

pub fn varchar(name: &str, size: Option<usize>) -> ColumnAddBuilder {
    ColumnAddBuilder::new(name, ColumnType::VARCHAR(size.unwrap_or(255)))
}

#[derive(Debug, Clone)]
pub enum ColumnType {
    UUID,
    VARCHAR(usize),
}

pub trait ColumnCreate {
    fn add_column(&mut self, column: ColumnAddChange);
}

pub trait ColumnDrop {
    fn drop_column(&mut self, name: &str);
    fn drop_column_if_exists(&mut self, name: &str);
}

impl ColumnCreate for Table {
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

pub trait ColumnAlter: ColumnDrop {
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