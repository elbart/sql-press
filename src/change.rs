use crate::column::{
    ColumnAddChange, ColumnDropChange, ColumnRenameChange, ColumnType, Constraints,
};
use std::{fmt::Debug, rc::Rc};

pub trait Change: Debug {
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String;
}

pub trait SqlDialect {
    fn create_table(&self, schema: &String, name: &String, changes: Vec<String>) -> String;
    fn alter_table(&self, schema: &String, name: &String, changes: Vec<String>) -> String;
    fn add_column(
        &self,
        name: &String,
        with_prefix: bool,
        ct: &ColumnType,
        constraints: &Constraints,
    ) -> String;
    fn rename_column(&self, name: &String, new_name: &String) -> String;
    fn drop_column(&self, name: &String) -> String;
}

#[derive(Debug, Clone)]
pub struct Postgres {}
impl SqlDialect for Postgres {
    fn create_table(&self, schema: &String, name: &String, changes: Vec<String>) -> String {
        format!(
            "CREATE TABLE {}.\"{}\" (\n{}\n);",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn alter_table(&self, schema: &String, name: &String, changes: Vec<String>) -> String {
        format!(
            "ALTER TABLE {}.\"{}\"\n{};",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn add_column(
        &self,
        name: &String,
        with_prefix: bool,
        ct: &ColumnType,
        constraints: &Constraints,
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

    fn rename_column(&self, name: &String, new_name: &String) -> String {
        format!("RENAME COLUMN \"{}\" TO \"{}\"", name, new_name)
    }

    fn drop_column(&self, name: &String) -> String {
        format!("DROP COLUMN \"{}\"", name)
    }
}

#[derive(Debug)]
pub struct ChangeSet {
    schema: String,
    changes: Vec<Box<dyn Change>>,
}

#[derive(Debug)]
pub enum TableChangeOp {
    Create,
    CreateIfNotExists,
    Alter,
    Drop,
}

#[derive(Debug)]
pub struct TableChange {
    operation: TableChangeOp,
    schema: String,
    name: String,
    changes: Vec<Box<dyn Change>>,
}

impl TableChange {
    pub fn new(
        operation: TableChangeOp,
        schema: String,
        name: String,
        changes: Vec<Box<dyn Change>>,
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
        match self.operation {
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
            _ => todo!(),
        }
    }
}

pub trait ColumnCreate {
    fn add_column(&mut self, column: ColumnAddChange);
}

pub trait ColumnDrop {
    fn drop_column(&mut self, name: &str);
    fn drop_column_if_exists(&mut self, name: &str);
}

pub struct Table {
    name: String,
    schema: String,
    columns: Vec<ColumnAddChange>,
    changes: Vec<Box<dyn Change>>,
}

impl Table {
    pub fn new(name: &str, schema: &str) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            columns: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn get_changes(self) -> Vec<Box<dyn Change>> {
        self.changes
    }
}

impl ColumnCreate for Table {
    fn add_column(&mut self, column: ColumnAddChange) {
        self.changes.push(Box::new(column));
    }
}

impl ColumnDrop for Table {
    fn drop_column(&mut self, name: &str) {
        self.changes
            .push(Box::new(ColumnDropChange { name: name.into() }))
    }

    fn drop_column_if_exists(&mut self, name: &str) {
        todo!()
    }
}

pub trait ColumnAlter: ColumnDrop {
    fn add_column(&mut self, column: ColumnAddChange);

    fn rename_column(&mut self, column_name: &str, new_column_name: &str);
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
        let mut t = Table::new(name, &self.schema);
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
        let mut t = Table::new(name, &self.schema);
        handler(&mut t);
        self.changes.push(TableChange::new(
            TableChangeOp::Alter,
            self.schema.clone(),
            name.into(),
            t.get_changes(),
        ));
    }

    pub fn drop_table(&mut self) {
        todo!()
    }

    pub fn rename_table(&mut self) {
        todo!()
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
    use crate::column::{uuid, varchar};

    use super::*;

    #[test]
    fn create_table() {
        let mut cs = ChangeSet::new();

        cs.create_table("xxx", |t| {
            t.add_column(uuid("id").primary(true).build());
            t.add_column(varchar("description", None).build())
        });

        cs.alter_table("xxx", |t| {
            t.add_column(uuid("id2").build());
            t.drop_column("description");
            t.rename_column("id2", "id3")
        });

        let d = Rc::new(Postgres {});
        println!("{}", cs.get_ddl(d));
    }
}
