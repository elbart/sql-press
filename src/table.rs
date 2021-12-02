use std::rc::Rc;

use crate::{
    change::{Change, Changes},
    sql_dialect::SqlDialect,
};

pub struct Table {
    pub(crate) changes: Changes,
    pub(crate) idx_changes: Changes,
}

impl Table {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            idx_changes: Vec::new(),
        }
    }

    pub fn get_changes(self) -> Changes {
        self.changes
            .into_iter()
            .chain(self.idx_changes.into_iter())
            .collect()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum TableChangeOp {
    Create,
    CreateIfNotExists,
    Alter,
    Rename { new_table_name: String },
    Drop,
}

#[derive(Debug)]
pub struct TableChange {
    operation: TableChangeOp,
    name: String,
    changes: Changes,
}

impl TableChange {
    pub fn new(
        operation: TableChangeOp,
        _schema: String,
        name: String,
        changes: Changes,
    ) -> Box<Self> {
        Box::new(Self {
            operation,
            name,
            changes,
        })
    }
}

impl Change for TableChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        match &self.operation {
            TableChangeOp::Create => {
                let c = self
                    .changes
                    .iter()
                    .map(|c| c.get_ddl(dialect.clone()))
                    .collect();
                dialect.create_table(&self.name, c, false)
            }
            TableChangeOp::CreateIfNotExists => {
                let c = self
                    .changes
                    .iter()
                    .map(|c| c.get_ddl(dialect.clone()))
                    .collect();
                dialect.create_table(&self.name, c, true)
            }
            TableChangeOp::Alter => {
                let c = self
                    .changes
                    .iter()
                    .map(|c| c.get_ddl(dialect.clone()))
                    .collect();
                dialect.alter_table(&self.name, c)
            }
            TableChangeOp::Drop => dialect.drop_table(&self.name),
            TableChangeOp::Rename { new_table_name } => {
                dialect.rename_table(&self.name, new_table_name)
            }
        }
    }
}
