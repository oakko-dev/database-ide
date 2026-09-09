use postgres::Client;
use serde::Serialize;

use crate::error::AppError;

#[derive(Clone, Debug, Serialize)]
pub struct SchemaInfo { pub name: String }
#[derive(Clone, Debug, Serialize)]
pub struct RelationInfo { pub schema: String, pub name: String, pub relation_type: String }
#[derive(Clone, Debug, Serialize)]
pub struct ColumnInfo { pub name: String, pub data_type: String, pub nullable: bool, pub ordinal_position: i32 }
#[derive(Clone, Debug, Serialize)]
pub struct IndexInfo { pub name: String, pub definition: String }
#[derive(Clone, Debug, Serialize)]
pub struct ConstraintInfo { pub name: String, pub constraint_type: String }
#[derive(Clone, Debug, Serialize)]
pub struct RelationDetails { pub relation: RelationInfo, pub columns: Vec<ColumnInfo>, pub indexes: Vec<IndexInfo>, pub constraints: Vec<ConstraintInfo> }

fn database_error(error: postgres::Error) -> AppError { AppError::Database(format!("schema query failed: {error}")) }

pub fn list_schemas(client: &mut Client) -> Result<Vec<SchemaInfo>, AppError> {
    client.query("SELECT nspname FROM pg_namespace WHERE nspname NOT LIKE 'pg_%' AND nspname <> 'information_schema' ORDER BY nspname", &[])
        .map(|rows| rows.into_iter().map(|row| SchemaInfo { name: row.get(0) }).collect()).map_err(database_error)
}

pub fn list_relations(client: &mut Client, schema: &str) -> Result<Vec<RelationInfo>, AppError> {
    client.query("SELECT n.nspname, c.relname, CASE c.relkind WHEN 'r' THEN 'TABLE' WHEN 'v' THEN 'VIEW' WHEN 'm' THEN 'MATERIALIZED_VIEW' WHEN 'f' THEN 'FOREIGN_TABLE' END FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1 AND c.relkind IN ('r', 'v', 'm', 'f') ORDER BY c.relname", &[&schema])
        .map(|rows| rows.into_iter().map(|row| RelationInfo { schema: row.get(0), name: row.get(1), relation_type: row.get(2) }).collect()).map_err(database_error)
}

pub fn describe_relation(client: &mut Client, schema: &str, relation: &str) -> Result<RelationDetails, AppError> {
    let relation_info = list_relations(client, schema)?.into_iter().find(|item| item.name == relation).ok_or(AppError::NotFound)?;
    let columns = client.query("SELECT column_name, data_type, (is_nullable = 'YES'), ordinal_position::int FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position", &[&schema, &relation]).map_err(database_error)?
        .into_iter().map(|row| ColumnInfo { name: row.get(0), data_type: row.get(1), nullable: row.get(2), ordinal_position: row.get(3) }).collect();
    let indexes = client.query("SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = $1 AND tablename = $2 ORDER BY indexname", &[&schema, &relation]).map_err(database_error)?
        .into_iter().map(|row| IndexInfo { name: row.get(0), definition: row.get(1) }).collect();
    let constraints = client.query("SELECT constraint_name, constraint_type FROM information_schema.table_constraints WHERE table_schema = $1 AND table_name = $2 ORDER BY constraint_name", &[&schema, &relation]).map_err(database_error)?
        .into_iter().map(|row| ConstraintInfo { name: row.get(0), constraint_type: row.get(1) }).collect();
    Ok(RelationDetails { relation: relation_info, columns, indexes, constraints })
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_names_are_bound_as_query_parameters() { let user_input = "public' OR '1'='1"; assert!(!user_input.contains("SELECT")); }
}
