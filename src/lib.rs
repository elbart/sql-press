//! sql_press has the sole purpose of defining database changes (e.g. for
//! schema migrations) as code instead of using plain SQL. The use of this crate
//! defines one or multiple [ChangeSet][crate::change::ChangeSet]'s with individual changes.
//! Those changes will be converted to DDL (effectively a plain
//! [String][std::string::String]) with a supported SQL Dialect.
//!
//! # Examples
//!
//! ## Create a new Table
//!
//! ```
//! use sql_press::{
//!     change::ChangeSet,
//!     column::{varchar, uuid},
//!     sql_dialect::Postgres,
//! };
//!
//! let mut cs = ChangeSet::new();
//!
//! // Adds a custom defined script to be executed.
//! cs.run_script("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";");
//!
//! cs.create_table("my_new_table", |t| {
//!     t.add_column(uuid("id").primary(true).build());
//!     t.add_column(varchar("name", Some(255)).not_null(true).build());
//! });
//!
//! let ddl = Postgres::new_rc();
//! println!("{}", cs.get_ddl(ddl));
//! ```
//!
//! ## Rename an existing Table
//!
//! ```
//! use sql_press::{
//!     change::ChangeSet,
//!     sql_dialect::Postgres,
//! };
//!
//! let mut cs = ChangeSet::new();
//!
//! cs.rename_table("my_new_table", "my_actual_table");
//!
//! let ddl = Postgres::new_rc();
//! println!("{}", cs.get_ddl(ddl));
//! ```
//!
//! ## Alter (change) columns within an existing table
//!
//! ```
//! use sql_press::{
//!     change::ChangeSet,
//!     column::{varchar, ColumnType},
//!     sql_dialect::Postgres,
//! };
//!
//! let mut cs = ChangeSet::new();
//!
//! cs.alter_table("my_actual_table", |t| {
//!     t.add_column(varchar("description", Some(255)).build());
//!     t.rename_column("name", "slug");
//!     t.alter_column("slug", ColumnType::TEXT, None);
//!     t.drop_column_if_exists("not_found");
//! });
//!
//! let ddl = Postgres::new_rc();
//! println!("{}", cs.get_ddl(ddl));
//! ```
//!
//! ## Delete / Drop a table
//!
//! ```
//! use sql_press::{
//!     change::ChangeSet,
//!     sql_dialect::Postgres,
//! };
//!
//! let mut cs = ChangeSet::new();
//!
//! cs.drop_table("my_actual_table");
//!
//! let ddl = Postgres::new_rc();
//! println!("{}", cs.get_ddl(ddl));
//! ```

pub mod change;
pub mod column;
pub mod index;
pub mod sql_dialect;
pub mod table;
