use std::rc::Rc;

use crate::{
    change::{Change, Changes},
    sql_dialect::SqlDialect,
};

pub struct Table {
    pub(crate) changes: Changes,
}

impl Table {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn get_changes(self) -> Changes {
        self.changes
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
    schema: String,
    name: String,
    changes: Changes,
}

impl TableChange {
    pub fn new(
        operation: TableChangeOp,
        schema: String,
        name: String,
        changes: Changes,
    ) -> Box<Self> {
        Box::new(Self {
            operation,
            schema,
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
                dialect.create_table(&self.schema, &self.name, c)
            }
            TableChangeOp::Alter => {
                let c = self
                    .changes
                    .iter()
                    .map(|c| c.get_ddl(dialect.clone()))
                    .collect();
                dialect.alter_table(&self.schema, &self.name, c)
            }
            TableChangeOp::Drop => dialect.drop_table(&self.schema, &self.name),
            TableChangeOp::Rename { new_table_name } => {
                dialect.rename_table(&self.schema, &self.name, new_table_name)
            }
            _ => todo!(),
        }
    }
}
