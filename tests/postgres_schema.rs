use database_ide::{
    schema::{describe_relation, list_relations, list_schemas},
    AppError,
};
use postgres::{Client, NoTls};
use std::env;

#[test]
#[ignore = "requires a disposable PostgreSQL instance; run scripts/test-postgres-schema.sh"]
fn explores_schemas_relations_and_details() {
    let url =
        env::var("DATABASE_URL").expect("DATABASE_URL must point to the disposable test database");
    let mut client = Client::connect(&url, NoTls).expect("connect to disposable PostgreSQL");
    client.batch_execute("DROP SCHEMA IF EXISTS ide_schema_test CASCADE; CREATE SCHEMA ide_schema_test; CREATE TABLE ide_schema_test.customers (id integer PRIMARY KEY, email text NOT NULL); CREATE INDEX customers_email_idx ON ide_schema_test.customers (email);").expect("create schema fixture");
    let schemas = list_schemas(&mut client).expect("list schemas");
    assert!(schemas
        .iter()
        .any(|schema| schema.name == "ide_schema_test"));
    let relations = list_relations(&mut client, "ide_schema_test").expect("list relations");
    assert_eq!(
        relations
            .iter()
            .find(|relation| relation.name == "customers")
            .map(|relation| relation.relation_type.as_str()),
        Some("TABLE")
    );
    let details =
        describe_relation(&mut client, "ide_schema_test", "customers").expect("describe relation");
    assert!(details
        .columns
        .iter()
        .any(|column| column.name == "email" && !column.nullable));
    assert!(details
        .indexes
        .iter()
        .any(|index| index.name == "customers_email_idx"));
    assert!(details
        .constraints
        .iter()
        .any(|constraint| constraint.constraint_type == "PRIMARY KEY"));
    assert!(matches!(
        describe_relation(&mut client, "ide_schema_test", "missing"),
        Err(AppError::NotFound)
    ));
}
