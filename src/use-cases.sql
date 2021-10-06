CREATE TABLE [IF IF NOT EXISTS] { schema }.{ table_name } (
    { column_name } { column_type } { column_constraints }
);
ALTER TABLE { schema }.{ table_name }
ADD COLUMN { column_name } { column_type } { column_constraints },
    ADD COLUMN { column_name } { column_type } { column_constraints },
    ALTER COLUMN { column_name } TYPE { column_type } [USING {conversion_method}],
    DROP COLUMN [IF EXISTS] { column_name } [CASCADE],
    RENAME COLUMN { column_name } TO { new_column_name };
ALTER TABLE [IF EXISTS] { table_name }
    RENAME TO { new_table_name };
ALTER TABLE @schema @.accounts
ADD COLUMN IF NOT EXISTS subject_id VARCHAR;
UPDATE @schema @.accounts
SET subject_id = 'no-subject'
WHERE subject_id IS NULL;
ALTER TABLE @schema @.accounts
ALTER column subject_id
SET NOT NULL;
TRUNCATE TABLE { table_name };
DROP TABLE [IF EXISTS] { table_name } [CASCADE | RESTRICT];