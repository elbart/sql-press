use crate::{
    column::{TableAlter, TableCreate},
    sql_dialect::SqlDialect,
    table::{Table, TableChange, TableChangeOp},
};
use std::{any::Any, fmt::Debug, rc::Rc};

/// Convenience type alias, which holds a list of Changes.
pub(crate) type Changes = Vec<Box<dyn Change>>;

#[doc(hidden)]
/// Trait which add support to downcast references of subtraits (especially [Change][crate::change::Change]).
/// This is solely used for testing purposes.
pub trait ChangeToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

#[doc(hidden)]
impl<T: 'static> ChangeToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Central trait, which is used to convert structured data to Data Definition
/// Language of the given [SqlDialect][crate::sql_dialect::SqlDialect].
pub trait Change: Debug + ChangeToAny {
    /// Convert self-contained structured SQL changes to Data Definition
    /// Language of the given [SqlDialect][crate::sql_dialect::SqlDialect].
    fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String;
}

/// Holds a set of changes, which shall be converted to DDL
#[derive(Debug)]
pub struct ChangeSet {
    /// Database Schema (postgres specific feature)
    schema: String,
    /// List of Changes, to be applied within this `ChangeSet`
    changes: Changes,
}

impl ChangeSet {
    /// Create a new ChangeSet
    ///
    /// ```
    /// use sql_press::change::ChangeSet;
    /// let mut cs = ChangeSet::new();
    /// cs.rename_table("old", "new");
    /// ```
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Add a new `CREATE TABLE` command to the current [ChangeSet] with the
    /// given `name` argument. The `handler` is a closure which adds individual
    /// colum changes to the `CREATE TABLE` command. The `create_table` function
    /// allows the following commands derived from the trait [TableCreate]:
    /// - add_column,
    /// - add_foreign_index,
    /// - add_primary_index.
    ///
    /// # Example
    /// ```
    /// use sql_press::{
    ///     change::ChangeSet,
    ///     column::{uuid, varchar}
    /// };
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.create_table("my_table", |t| {
    ///     t.add_column(uuid("id").build());
    ///     t.add_column(varchar("name", Some(255)).build());
    /// });
    /// ```
    pub fn create_table<H>(&mut self, name: &str, handler: H)
    where
        H: FnOnce(&mut dyn TableCreate),
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

    /// Add a new `ALTER TABLE` command to the current [ChangeSet] for the
    /// given table name. The `handler` is a closure which allows to add individual
    /// colum changes to the `ALTER TABLE` command. The `alter_table` function
    /// explicitly allows a few more commands to be executed on the table
    /// derived from the trait [TableAlter]:
    /// - [TableAlter::add_column],
    /// - [TableAlter::rename_column],
    /// - [TableAlter::alter_column],
    /// - [IndexAlter::add_primary_index][crate::index::IndexAlter::add_primary_index],
    /// - [IndexAlter::add_foreign_index][crate::index::IndexAlter::add_foreign_index],
    /// - [ColumnDrop::drop_column][crate::column::ColumnDrop::drop_column],
    /// - [ColumnDrop::drop_column_if_exists][crate::column::ColumnDrop::drop_column_if_exists].
    ///
    /// # Example
    /// ```
    /// use sql_press::{
    ///     change::ChangeSet,
    ///     column::{uuid, varchar}
    /// };
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.alter_table("my_table", |t| {
    ///     t.rename_column("name", "slug");
    /// });
    /// ```
    pub fn alter_table<H>(&mut self, name: &str, handler: H)
    where
        H: FnOnce(&mut dyn TableAlter),
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

    /// Add a new `DROP TABLE` command to the current [ChangeSet] for the given
    /// table name.
    ///
    /// # Example
    /// ```
    /// use sql_press::change::ChangeSet;
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.drop_table("my_table");
    /// ```
    pub fn drop_table(&mut self, name: &str) {
        self.changes.push(TableChange::new(
            TableChangeOp::Drop,
            self.schema.clone(),
            name.into(),
            Vec::new(),
        ))
    }

    /// Add a new `ALTER TABLE ... RENAME TO ...` command to the current
    /// [ChangeSet] for the given table name.
    ///
    /// # Example
    /// ```
    /// use sql_press::change::ChangeSet;
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.rename_table("my_table", "my_actual_table");
    /// ```
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

    /// Adds a plain string Change to the current [ChangeSet]. This string is
    /// executed with no transformation etc. This means the script which is run
    /// is potentially bound to a specific database type (e.g. postgres, mysql, ...);
    ///
    /// # Example
    /// ```
    /// use sql_press::change::ChangeSet;
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.run_script("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";");
    /// ```
    pub fn run_script(&mut self, script: &str) {
        self.changes.push(Box::new(Script::new(script)))
    }

    /// Generates DDL for the given [SqlDialect] recursively for all changes in
    /// the current [ChangeSet].
    ///
    /// # Example
    /// ```
    /// use sql_press::{change::ChangeSet, sql_dialect::Postgres};
    ///
    /// let mut cs = ChangeSet::new();
    /// cs.drop_table("my_table");
    /// cs.run_script("DDL INSTRUCTION;");
    ///
    /// assert_eq!(r#"DROP TABLE public."my_table";
    ///
    /// DDL INSTRUCTION;
    /// "#, cs.get_ddl(Postgres::new_rc()));
    /// ```
    pub fn get_ddl(&self, dialect: Rc<dyn SqlDialect>) -> String {
        self.changes
            .iter()
            .map(|c| c.get_ddl(dialect.clone()))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

/// Plain change which is run on the database without additional transformation.
#[derive(Debug)]
pub struct Script {
    script: String,
}

impl Script {
    pub fn new(script: &str) -> Self {
        Self {
            script: script.into(),
        }
    }
}

impl Change for Script {
    fn get_ddl(&self, _dialect: Rc<dyn SqlDialect>) -> String {
        format!("{}\n", self.script)
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
        column::{uuid, varchar, ColumnType, DefaultConstraint},
        sql_dialect::postgres::Postgres,
    };

    use super::*;

    #[test]
    fn create_table() {
        let mut cs = ChangeSet::new();

        cs.create_table("xxx", |t| {
            t.add_foreign_index("tag_id", "tag", "id", Some("fk_blubbi".into()));
            t.add_column(
                uuid("id")
                    .primary(true)
                    .default(DefaultConstraint::Plain("uuid_generate_v4()".into()))
                    .build(),
            );
            t.add_column(varchar("description", None).build());
            t.add_primary_index(vec!["id", "id2"]);
        });

        let _d = Rc::new(Postgres::new());
        // println!("{}", cs.get_ddl(_d));
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

        cs.run_script("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";");

        let _d = Postgres::new_rc();
        // println!("{}", cs.get_ddl(_d));
    }

    #[test]
    fn rename_table() {
        let mut cs = ChangeSet::new();

        cs.rename_table("tags", "tag");

        let _d = Rc::new(Postgres::new());
        // println!("{}", cs.get_ddl(d));
    }

    #[test]
    fn drop_table() {
        let mut cs = ChangeSet::new();

        cs.drop_table("tag");

        let _d = Rc::new(Postgres::new());
        // println!("{}", cs.get_ddl(d));
    }
}
