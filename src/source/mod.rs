//! Data sources drivers

use crate::config::ConfigSource;
use crate::value::{Field, Value};
use log::*;
use serde::Deserialize;

pub mod sqlite;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum SourceType {
    Sqlite,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Query {
    pub sql: String,
    pub title: String,
    pub fields: Vec<Field>,
}

pub struct QueryResult {
    pub values: Vec<Vec<Value>>,
}

/// Data source driver definitions
pub trait Driver {
    // Establish the connection and prepare for fetch
    async fn connect(&mut self, conn: String) -> Result<(), String>;

    // Query and fetch the data
    async fn fetch(&mut self, query: Query) -> Result<QueryResult, String>;
}

/// Setup the driver of specified kind
fn get_driver(kind: SourceType) -> Result<impl Driver, String> {
    debug!("Preparing the driver for {:?}", kind);

    match kind {
        SourceType::Sqlite => Ok(sqlite::SqliteDriver::init()),
        _ => Err("Not supported kind".to_string()),
    }
}

/// Query and fetch the data from the database
pub async fn fetch(
    source: ConfigSource,
    querys: Vec<Query>,
) -> Result<Vec<(Query, QueryResult)>, String> {
    let mut driver = get_driver(source.kind)?;

    info!("Connecting on database");

    driver.connect(source.conn).await?;

    debug!("Database connected");

    let mut r = vec![];

    for query in querys {
        info!("Fetching '{}' query", query.title);

        let result = driver.fetch(query.clone()).await?;

        r.push((query, result));
    }

    Ok(r)
}

#[cfg(test)]
pub mod tests {
    use crate::{
        config::ConfigSource,
        source::{Query, SourceType},
        value::{Field, FieldType, TypedValue},
    };

    #[tokio::test]
    async fn fetch() -> Result<(), String> {
        let query = "
            drop table if exists users;
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ";
        sqlite::Connection::open_with_flags(
            "/tmp/test-lmr.db",
            sqlite::OpenFlags::new().with_create().with_read_write(),
        )
        .unwrap()
        .execute(query)
        .unwrap();

        let source = ConfigSource {
            conn: "/tmp/test-lmr.db".to_string(),
            kind: SourceType::Sqlite,
        };

        let query = Query {
            title: "Test".to_string(),
            sql: "select * from users".to_string(),
            fields: vec![
                Field {
                    title: "User name".to_string(),
                    field: "name".to_string(),
                    kind: FieldType::String,
                },
                Field {
                    title: "Age".to_string(),
                    field: "age".to_string(),
                    kind: FieldType::Integer,
                },
            ],
        };

        let results = super::fetch(source, vec![query.clone()]).await?;
        assert_eq!(1, results.len());

        let (rquery, result) = &results[0];
        assert_eq!(query.clone(), rquery.clone());
        assert_eq!(2, result.values.len());

        let row = &result.values[0];
        assert_eq!(query.fields[0], row[0].field);
        assert_eq!(TypedValue::String(Some("Alice".to_string())), row[0].inner);
        assert_eq!(query.fields[1], row[1].field);
        assert_eq!(TypedValue::Integer(Some(42)), row[1].inner);

        let row = &result.values[1];
        assert_eq!(query.fields[0], row[0].field);
        assert_eq!(TypedValue::String(Some("Bob".to_string())), row[0].inner);
        assert_eq!(query.fields[1], row[1].field);
        assert_eq!(TypedValue::Integer(Some(69)), row[1].inner);

        Ok(())
    }
}
