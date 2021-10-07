use crate::{
    column::{ColumnAlter, ColumnCreate},
    sql_dialect::SqlDialect,
    table::{Table, TableChange, TableChangeOp},
};
use std::{any::Any, fmt::Debug, rc::Rc};

pub(crate) type Changes = Vec<Box<dyn Change>>;

pub trait ChangeToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> ChangeToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Change: Debug + ChangeToAny {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String;
}

#[derive(Debug)]
pub struct ChangeSet {
    schema: String,
    changes: Changes,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn create_table<H>(&mut self, name: &str, handler: H)
    where
        H: FnOnce(&mut dyn ColumnCreate),
    {
        let mut t: Table = Default::default();
        handler(&mut t);
        self.changes.push(TableChange::new(
            TableChangeOp::Create,
            self.schema.clone(),
            name.into(),
            t.get_changes(),
        ));
    }

    pub fn alter_table<H>(&mut self, name: &str, handler: H)
    where
        H: FnOnce(&mut dyn ColumnAlter),
    {
        let mut t: Table = Default::default();
        handler(&mut t);
        self.changes.push(TableChange::new(
            TableChangeOp::Alter,
            self.schema.clone(),
            name.into(),
            t.get_changes(),
        ));
    }

    pub fn drop_table(&mut self, name: &str) {
        self.changes.push(TableChange::new(
            TableChangeOp::Drop,
            self.schema.clone(),
            name.into(),
            Vec::new(),
        ))
    }

    pub fn rename_table(&mut self, name: &str, new_name: &str) {
        self.changes.push(TableChange::new(
            TableChangeOp::Rename {
                new_table_name: new_name.into(),
            },
            self.schema.clone(),
            name.into(),
            Vec::new(),
        ))
    }

    pub fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        self.changes
            .iter()
            .map(|c| c.get_ddl(dialect.clone()))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self {
            schema: "public".into(),
            changes: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::{uuid, varchar, ColumnType},
        sql_dialect::postgres::Postgres,
    };

    use super::*;

    #[test]
    fn create_table() {
        let mut cs = ChangeSet::new();

        cs.create_table("xxx", |t| {
            t.add_column(uuid("id").primary(true).build());
            t.add_column(varchar("description", None).build())
        });

        let _d = Rc::new(Postgres {});
        // println!("{}", cs.get_ddl(d));
    }

    #[test]
    fn alter_table() {
        let mut cs = ChangeSet::new();

        cs.alter_table("xxx", |t| {
            t.add_column(uuid("id2").build());
            t.drop_column("description");
            t.rename_column("id2", "id3");
            t.drop_column_if_exists("id3");
            t.add_column(varchar("description2", None).build());
            t.alter_column("description2", ColumnType::UUID, None);
            t.alter_column(
                "description2",
                ColumnType::UUID,
                Some("%%%conversion_method%%%".into()),
            );
        });

        let _d = Rc::new(Postgres {});
        // println!("{}", cs.get_ddl(d));
    }

    #[test]
    fn rename_table() {
        let mut cs = ChangeSet::new();

        cs.rename_table("tags", "tag");

        let _d = Rc::new(Postgres {});
        // println!("{}", cs.get_ddl(d));
    }

    #[test]
    fn drop_table() {
        let mut cs = ChangeSet::new();

        cs.drop_table("tag");

        let _d = Rc::new(Postgres {});
        // println!("{}", cs.get_ddl(d));
    }
}
