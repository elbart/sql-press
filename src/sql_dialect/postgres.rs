use crate::column::{ColumnType, Constraints};

use super::SqlDialect;

#[derive(Debug, Clone)]
pub struct Postgres {}
impl SqlDialect for Postgres {
    fn create_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String {
        format!(
            "CREATE TABLE {}.\"{}\" (\n{}\n);",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn alter_table(&self, schema: &str, name: &str, changes: Vec<String>) -> String {
        format!(
            "ALTER TABLE {}.\"{}\"\n{};",
            schema,
            name,
            changes.join(",\n")
        )
    }

    fn rename_table(&self, schema: &str, name: &str, new_table_name: &str) -> String {
        format!(
            "ALTER TABLE {}.\"{}\" RENAME TO {}.\"{}\";",
            schema, name, schema, new_table_name,
        )
    }

    fn drop_table(&self, schema: &str, name: &str) -> String {
        format!("DROP TABLE {}.\"{}\";", schema, name,)
    }

    fn add_column(
        &self,
        name: &str,
        with_prefix: bool,
        ct: &ColumnType,
        _constraints: &Constraints,
    ) -> String {
        let t = match ct {
            ColumnType::VARCHAR(s) => format!("VARCHAR({})", s),
            ColumnType::UUID => "uuid".into(),
        };

        format!(
            "{}\"{}\" {}",
            with_prefix.then(|| "ADD COLUMN ").unwrap_or(""),
            name,
            &t
        )
    }

    fn rename_column(&self, name: &str, new_name: &str) -> String {
        format!("RENAME COLUMN \"{}\" TO \"{}\"", name, new_name)
    }

    fn alter_column(&self, name: &str, ct: &ColumnType, conversion_method: Option<&str>) -> String {
        let t = match ct {
            ColumnType::VARCHAR(s) => format!("VARCHAR({})", s),
            ColumnType::UUID => "uuid".into(),
        };
        format!(
            "ALTER COLUMN \"{}\" TYPE {}{}",
            name,
            &t,
            conversion_method
                .map(|u| format!(" USING {}", u))
                .unwrap_or_else(|| "".into())
        )
    }

    fn drop_column(&self, name: &str, if_exists: bool) -> String {
        format!(
            "DROP COLUMN {}\"{}\"",
            if_exists.then(|| "IF EXISTS ").unwrap_or(""),
            name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_table() {
        let d = Box::new(Postgres {});
        let ddl = d.create_table("public", "tag", Vec::new());
        assert_eq!(ddl, format!("CREATE TABLE public.\"tag\" (\n\n);"));

        let ddl = d.create_table("public", "tag", vec!["CHANGE 1".into(), "CHANGE 2".into()]);
        assert_eq!(
            ddl,
            format!("CREATE TABLE public.\"tag\" (\nCHANGE 1,\nCHANGE 2\n);")
        );
    }

    #[test]
    fn rename_table() {
        let d = Box::new(Postgres {});
        let ddl = d.rename_table("public", "tags", "tag");
        assert_eq!(
            ddl,
            format!("ALTER TABLE public.\"tags\" RENAME TO public.\"tag\";")
        );
    }

    #[test]
    fn alter_table() {
        let d = Box::new(Postgres {});
        let ddl = d.alter_table("public", "tags", Vec::new());
        assert_eq!(ddl, format!("ALTER TABLE public.\"tags\"\n;"));

        let ddl = d.alter_table("public", "tags", vec!["CHANGE 1".into(), "CHANGE 2".into()]);
        assert_eq!(
            ddl,
            format!("ALTER TABLE public.\"tags\"\nCHANGE 1,\nCHANGE 2;")
        );
    }

    #[test]
    fn drop_table() {
        let d = Box::new(Postgres {});
        let ddl = d.drop_table("public", "tags");
        assert_eq!(ddl, format!("DROP TABLE public.\"tags\";"));
    }

    #[test]
    fn add_column() {
        let d = Box::new(Postgres {});
        let ddl = d.add_column("id", false, &ColumnType::UUID, &Constraints::new());
        assert_eq!(ddl, format!("\"id\" uuid"));

        let ddl = d.add_column("id", true, &ColumnType::UUID, &Constraints::new());
        assert_eq!(ddl, format!("ADD COLUMN \"id\" uuid"));
    }

    #[test]
    fn rename_column() {
        let d = Box::new(Postgres {});
        let ddl = d.rename_column("id", "id2");
        assert_eq!(ddl, format!("RENAME COLUMN \"id\" TO \"id2\""));
    }

    #[test]
    fn drop_column() {
        let d = Box::new(Postgres {});
        let ddl = d.drop_column("id", false);
        assert_eq!(ddl, format!("DROP COLUMN \"id\""));

        let ddl = d.drop_column("id", true);
        assert_eq!(ddl, format!("DROP COLUMN IF EXISTS \"id\""));
    }
}
