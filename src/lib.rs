use column::{Column, ColumnType};
use instruction::{ColumnInstruction, TableInstruction};

pub trait DialectGenerator {
    fn create_table(table: &Table) -> String;
    fn drop_table(table: &Table) -> String;
    fn join_table_separator() -> &'static str;
    fn join_table_column_separator() -> &'static str;
    fn add_column(col: &Column) -> String;
}

#[derive(Debug)]
pub struct Postgres {}

impl DialectGenerator for Postgres {
    fn create_table(table: &Table) -> String {
        format!(
            "CREATE TABLE {}.\"{}\" (\n{}\n);",
            table.schema,
            table.name,
            table
                .columns
                .iter()
                .filter(|c| matches!(c, ColumnInstruction::Create(_)))
                .map(|c| c.get_ddl::<Self>())
                .collect::<Vec<String>>()
                .join(Self::join_table_column_separator())
        )
    }

    fn drop_table(table: &Table) -> String {
        format!("DROP TABLE {}.\"{}\";", table.schema, table.name)
    }

    fn join_table_separator() -> &'static str {
        "\n"
    }

    fn join_table_column_separator() -> &'static str {
        ",\n"
    }

    fn add_column(col: &Column) -> String {
        match col.ct {
            ColumnType::UUID => {
                format!("\"{}\" uuid", col.name)
            }
        }
    }
}

pub mod index {
    #[derive(Debug, Clone)]
    pub struct Index {
        name: String,
        it: IndexType,
    }

    #[derive(Debug, Clone)]
    pub enum IndexType {}
}

pub mod column {

    #[derive(Debug, Clone)]
    pub struct Column {
        pub(crate) name: String,
        pub(crate) ct: ColumnType,
        primary: bool,
    }

    impl Column {
        pub fn new(name: &str, ct: ColumnType) -> Self {
            Self {
                name: name.into(),
                ct,
                primary: false,
            }
        }

        pub fn primary(mut self, primary: bool) -> Self {
            self.primary = primary;

            self
        }
    }

    pub fn uuid(name: &str) -> Column {
        Column::new(name, ColumnType::UUID)
    }

    #[derive(Debug, Clone)]
    pub enum ColumnType {
        UUID,
    }
}

pub mod instruction {
    use crate::{column::Column, DialectGenerator, Table};

    #[derive(Debug)]
    pub enum ColumnInstruction {
        Create(Column),
        Alter(Column),
        Drop(Column),
    }

    impl ColumnInstruction {
        pub fn get_ddl<T: DialectGenerator>(&self) -> String {
            match self {
                Self::Create(c) => T::add_column(c),
                _ => todo!(),
            }
        }
    }

    #[derive(Debug)]
    pub enum TableInstruction {
        Create(Table),
        Alter(Table),
        Drop(Table),
    }

    impl TableInstruction {
        pub fn get_ddl<T: DialectGenerator>(&self) -> String {
            match self {
                Self::Create(t) => T::create_table(t),
                Self::Drop(t) => T::drop_table(t),
                _ => {
                    todo!()
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Migration {
    tables: Vec<TableInstruction>,
    schema: String,
}

#[derive(Debug)]
pub struct Table {
    name: String,
    schema: String,
    columns: Vec<ColumnInstruction>,
    // indexes: Vec<Index>,
}

impl Table {
    pub fn new(name: &str, schema: &str) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(), // indexes: Vec::new(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(ColumnInstruction::Create(column));
    }
    // pub fn add_index(&mut self, index: Index) {
    //     self.indexes.push(index);
    // }
}

impl Migration {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn create_table<H>(&mut self, name: &str, handler: H)
    where
        H: FnOnce(&mut Table),
    {
        let mut t = Table::new(name, &self.schema);
        handler(&mut t);

        self.tables.push(TableInstruction::Create(t));
    }

    pub fn drop_table(&mut self, name: &str) {
        self.tables
            .push(TableInstruction::Drop(Table::new(name, &self.schema)));
    }

    fn get_ddl<T: DialectGenerator>(&self) -> String {
        self.tables
            .iter()
            .map(|t| t.get_ddl::<T>())
            .collect::<Vec<String>>()
            .join(T::join_table_separator())
    }
}

impl Default for Migration {
    fn default() -> Self {
        Self {
            tables: Default::default(),
            schema: "public".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_table() {
        let mut m = Migration::new();

        m.create_table("tag", |t| {
            t.add_column(column::uuid("id").primary(true));
            t.add_column(column::uuid("description"));
        });

        println!("{}", m.get_ddl::<Postgres>());
    }

    #[test]
    fn drop_table() {
        let mut m = Migration::new();

        m.drop_table("tag");

        println!("{}", m.get_ddl::<Postgres>());
    }
}
