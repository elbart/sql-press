use crate::column::{ColumnType, Constraints};

use super::SqlDialect;

#[derive(Debug, Clone)]
pub struct Postgres {
    pub(crate) schema: String,
}

impl Postgres {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Postgres {
    fn default() -> Self {
        Self {
            schema: "public".into(),
        }
    }
}

impl SqlDialect for Postgres {
    fn create_table(&self, name: &str, changes: Vec<String>, if_not_exists: bool) -> String {
        format!(
            "CREATE TABLE {}{}.\"{}\" (\n{}\n);",
            if_not_exists.then(|| "IF NOT EXISTS ").unwrap_or(""),
            self.schema,
            name,
            changes.join(",\n")
        )
    }

    fn alter_table(&self, name: &str, changes: Vec<String>) -> String {
        format!(
            "ALTER TABLE {}.\"{}\"\n{};",
            self.schema,
            name,
            changes.join(",\n")
        )
    }

    fn rename_table(&self, name: &str, new_table_name: &str) -> String {
        format!(
            "ALTER TABLE {}.\"{}\" RENAME TO {}.\"{}\";",
            self.schema, name, self.schema, new_table_name,
        )
    }

    fn drop_table(&self, name: &str) -> String {
        format!("DROP TABLE {}.\"{}\";", self.schema, name,)
    }

    fn add_column(
        &self,
        name: &str,
        with_prefix: bool,
        ct: &ColumnType,
        constraints: &Constraints,
    ) -> String {
        format!(
            "{}\"{}\" {}{}",
            with_prefix.then(|| "ADD COLUMN ").unwrap_or(""),
            name,
            self.column_type(ct),
            self.constraints(constraints)
        )
    }

    fn rename_column(&self, name: &str, new_name: &str) -> String {
        format!("RENAME COLUMN \"{}\" TO \"{}\"", name, new_name)
    }

    fn alter_column(&self, name: &str, ct: &ColumnType, conversion_method: Option<&str>) -> String {
        format!(
            "ALTER COLUMN \"{}\" TYPE {}{}",
            name,
            self.column_type(ct),
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

    fn add_index(
        &self,
        _table_name: &str,
        _columns: &[String],
        _idx_name: &Option<String>,
    ) -> String {
        todo!()
    }

    fn add_foreign_index(
        &self,
        column_name: &str,
        foreign_table_name: &str,
        foreign_column_name: &str,
        idx_name: Option<String>,
        add_clause: &bool,
    ) -> String {
        format!(
            "{}{}FOREIGN KEY(\"{}\") REFERENCES \"{}\"(\"{}\")",
            add_clause
                .then(|| format!("ADD "))
                .unwrap_or_else(|| "".into()),
            idx_name
                .map(|x| format!("CONSTRAINT {} ", x))
                .unwrap_or_else(|| "".into()),
            column_name,
            foreign_table_name,
            foreign_column_name
        )
    }

    fn add_primary_index(&self, columns: &Vec<String>) -> String {
        format!(
            "PRIMARY KEY({})",
            columns
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn column_type(&self, ct: &ColumnType) -> String {
        match ct {
            ColumnType::UUID => "uuid".into(),
            ColumnType::BOOL => "boolean".into(),
            ColumnType::VARCHAR(s) => format!("VARCHAR({})", s),
            ColumnType::REAL => "real".into(),
            ColumnType::TEXT => "text".into(),
            ColumnType::TIMESTAMP => "timestamp".into(),
            ColumnType::TIMESTAMPTZ => "timestamp with time zone".into(),
            ColumnType::INTEGER => "integer".into(),
            ColumnType::JSONB => "jsonb".into(),
        }
    }

    fn constraints(&self, constraints: &Constraints) -> String {
        let def_constraint = || match &constraints.default {
            crate::column::DefaultConstraint::None => "".into(),
            crate::column::DefaultConstraint::Plain(s) => format!("DEFAULT {}", s),
        };

        let c = vec![
            constraints.primary.then(|| "PRIMARY KEY").unwrap_or(""),
            constraints.not_null.then(|| "NOT NULL").unwrap_or(""),
            constraints.unique.then(|| "UNIQUE").unwrap_or(""),
            def_constraint().as_ref(),
        ]
        .join(" ");

        let c = c.trim();

        if !c.is_empty() {
            // prefix with a space
            format!(" {}", c)
        } else {
            "".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::column::DefaultConstraint;

    use super::*;

    #[test]
    fn create_table() {
        let d = Box::new(Postgres::new());
        let ddl = d.create_table("tag", Vec::new(), false);
        assert_eq!(ddl, format!("CREATE TABLE public.\"tag\" (\n\n);"));

        let ddl = d.create_table("tag", vec!["CHANGE 1".into(), "CHANGE 2".into()], false);
        assert_eq!(
            ddl,
            format!("CREATE TABLE public.\"tag\" (\nCHANGE 1,\nCHANGE 2\n);")
        );

        let ddl = d.create_table("tag", Vec::new(), true);
        assert_eq!(
            ddl,
            format!("CREATE TABLE IF NOT EXISTS public.\"tag\" (\n\n);")
        );
    }

    #[test]
    fn rename_table() {
        let d = Box::new(Postgres::new());
        let ddl = d.rename_table("tags", "tag");
        assert_eq!(
            ddl,
            format!("ALTER TABLE public.\"tags\" RENAME TO public.\"tag\";")
        );
    }

    #[test]
    fn alter_table() {
        let d = Box::new(Postgres::new());
        let ddl = d.alter_table("tags", Vec::new());
        assert_eq!(ddl, format!("ALTER TABLE public.\"tags\"\n;"));

        let ddl = d.alter_table("tags", vec!["CHANGE 1".into(), "CHANGE 2".into()]);
        assert_eq!(
            ddl,
            format!("ALTER TABLE public.\"tags\"\nCHANGE 1,\nCHANGE 2;")
        );
    }

    #[test]
    fn drop_table() {
        let d = Box::new(Postgres::new());
        let ddl = d.drop_table("tags");
        assert_eq!(ddl, format!("DROP TABLE public.\"tags\";"));
    }

    #[test]
    fn add_column() {
        let d = Box::new(Postgres::new());
        let ddl = d.add_column("id", false, &ColumnType::UUID, &Constraints::new());
        assert_eq!(ddl, format!("\"id\" uuid"));

        let ddl = d.add_column("id", true, &ColumnType::UUID, &Constraints::new());
        assert_eq!(ddl, format!("ADD COLUMN \"id\" uuid"));

        let mut constraints = Constraints::new();
        constraints.primary = true;
        let ddl = d.add_column("id", true, &ColumnType::UUID, &constraints);
        assert_eq!(ddl, format!("ADD COLUMN \"id\" uuid PRIMARY KEY"));

        constraints.primary = false;
        constraints.not_null = true;
        constraints.unique = true;

        let ddl = d.add_column("id", true, &ColumnType::UUID, &constraints);
        assert_eq!(ddl, format!("ADD COLUMN \"id\" uuid NOT NULL UNIQUE"));

        let mut constraints = Constraints::new();
        constraints.default = DefaultConstraint::Plain("uuid_v4_generate()".into());

        let ddl = d.add_column("id", true, &ColumnType::UUID, &constraints);
        assert_eq!(
            ddl,
            format!("ADD COLUMN \"id\" uuid DEFAULT uuid_v4_generate()")
        );
    }

    #[test]
    fn rename_column() {
        let d = Box::new(Postgres::new());
        let ddl = d.rename_column("id", "id2");
        assert_eq!(ddl, format!("RENAME COLUMN \"id\" TO \"id2\""));
    }

    #[test]
    fn drop_column() {
        let d = Box::new(Postgres::new());
        let ddl = d.drop_column("id", false);
        assert_eq!(ddl, format!("DROP COLUMN \"id\""));

        let ddl = d.drop_column("id", true);
        assert_eq!(ddl, format!("DROP COLUMN IF EXISTS \"id\""));
    }

    #[test]
    fn add_foreign_index() {
        let d = Box::new(Postgres::new());
        let ddl = d.add_foreign_index("blubb_id", "blubb", "id", None, &false);
        assert_eq!(
            ddl,
            format!("FOREIGN KEY(\"blubb_id\") REFERENCES \"blubb\"(\"id\")")
        );

        let d = Box::new(Postgres::new());
        let ddl = d.add_foreign_index("blubb_id", "blubb", "id", None, &true);
        assert_eq!(
            ddl,
            format!("ADD FOREIGN KEY(\"blubb_id\") REFERENCES \"blubb\"(\"id\")")
        );

        let ddl = d.add_foreign_index(
            "blubb_id",
            "blubb",
            "id",
            Some("fk_blubb_blubb_id".into()),
            &false,
        );
        assert_eq!(
            ddl,
            format!(
                "CONSTRAINT fk_blubb_blubb_id FOREIGN KEY(\"blubb_id\") REFERENCES \"blubb\"(\"id\")"
            )
        );

        let ddl = d.add_foreign_index(
            "blubb_id",
            "blubb",
            "id",
            Some("fk_blubb_blubb_id".into()),
            &true,
        );
        assert_eq!(
            ddl,
            format!(
                "ADD CONSTRAINT fk_blubb_blubb_id FOREIGN KEY(\"blubb_id\") REFERENCES \"blubb\"(\"id\")"
            )
        );
    }

    #[test]
    fn add_primary_index() {
        let d = Box::new(Postgres::new());
        let ddl = d.add_primary_index(&vec!["id".into(), "id2".into()]);
        assert_eq!(ddl, format!("PRIMARY KEY(\"id\", \"id2\")"));
    }
}
