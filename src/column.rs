use std::rc::Rc;

use crate::change::{Change, SqlDialect};

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
pub struct ColumnDropChange {
    pub(crate) name: String,
}

impl Change for ColumnDropChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.drop_column(&self.name)
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
