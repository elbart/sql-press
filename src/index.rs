use std::rc::Rc;

use crate::{change::Change, sql_dialect::SqlDialect, table::Table};

pub trait IndexAdd {
    fn add_foreign_index(
        &mut self,
        column_name: &str,
        foreign_table_name: &str,
        foreign_column_name: &str,
        idx_name: Option<String>,
    );

    fn add_primary_index(&mut self, columns: Vec<&str>);
}

impl IndexAdd for Table {
    fn add_foreign_index(
        &mut self,
        column_name: &str,
        foreign_table_name: &str,
        foreign_column_name: &str,
        idx_name: Option<String>,
    ) {
        self.idx_changes.push(Box::new(IndexAddForeignChange {
            column_name: column_name.into(),
            foreign_table_name: foreign_table_name.into(),
            foreign_column_name: foreign_column_name.into(),
            idx_name,
        }));
    }

    fn add_primary_index(&mut self, columns: Vec<&str>) {
        self.idx_changes.push(Box::new(IndexAddPrimaryChange {
            columns: columns.iter().map(|i| i.to_string()).collect(),
        }))
    }
}

#[derive(Debug)]
pub struct IndexAddCombinedChange {
    table_name: String,
    columns: Vec<String>,
    idx_name: Option<String>,
}

#[derive(Debug)]
pub struct IndexAddForeignChange {
    column_name: String,
    foreign_table_name: String,
    foreign_column_name: String,
    idx_name: Option<String>,
}

#[derive(Debug)]
pub struct IndexAddPrimaryChange {
    columns: Vec<String>,
}

impl Change for IndexAddPrimaryChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.add_primary_index(&self.columns)
    }
}

impl Change for IndexAddCombinedChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.add_index(&self.table_name, &self.columns, &self.idx_name)
    }
}

impl Change for IndexAddForeignChange {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        dialect.add_foreign_index(
            &self.column_name,
            &self.foreign_table_name,
            &self.foreign_column_name,
            self.idx_name.clone(),
        )
    }
}
