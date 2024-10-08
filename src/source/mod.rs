//! Data sources drivers

use crate::value::{Field, Value};
use async_trait::async_trait;
use log::*;
use serde::Deserialize;

#[cfg(feature = "postgres")]
pub mod postgres;
pub mod sqlite;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum SourceType {
    Sqlite,
    Postgres,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Source {
    pub kind: SourceType,
    pub conn: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Query {
    pub sql: String,
    pub title: String,
    pub fields: Vec<Field>,
}

/// Data source driver definitions
#[async_trait]
pub trait Driver {
    // Establish the connection and prepare for fetch
    async fn connect(&mut self, conn: String) -> Result<(), String>;

    // Query and fetch the data
    async fn fetch(&mut self, query: Query) -> Result<Vec<Vec<Value>>, String>;
}

/// Setup the driver of specified kind
#[allow(unreachable_patterns)]
fn get_driver(kind: SourceType) -> Result<Box<dyn Driver + Send>, String> {
    debug!("Preparing the driver for {:?}", kind);

    match kind {
        SourceType::Sqlite => Ok(Box::new(sqlite::SqliteDriver::init())),
        #[cfg(feature = "postgres")]
        SourceType::Postgres => Ok(Box::new(postgres::PostgresDriver::init())),
        _ => Err("Not supported kind".to_string()),
    }
}

/// Query and fetch the data from the database
pub async fn fetch(
    source: Source,
    querys: Vec<Query>,
) -> Result<Vec<(Query, Result<Vec<Vec<Value>>, String>)>, String> {
    let mut driver = get_driver(source.kind)?;

    info!("Connecting on database");

    driver.connect(source.conn).await?;

    debug!("Database connected");

    let mut r = vec![];

    for query in querys {
        info!("Fetching '{}' query", query.title);

        let result = driver.fetch(query.clone()).await;

        r.push((query, result));
    }

    Ok(r)
}

#[cfg(test)]
pub mod tests {
    use crate::{
        source::{Query, Source, SourceType},
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

        let source = Source {
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
        assert_eq!(None, result.as_ref().err());
        let result = result.clone().unwrap();
        assert_eq!(2, result.len());

        let row = &result[0];
        assert_eq!(query.fields[0], row[0].field);
        assert_eq!(Some(TypedValue::String("Alice".to_string())), row[0].inner);
        assert_eq!(query.fields[1], row[1].field);
        assert_eq!(Some(TypedValue::Integer(42)), row[1].inner);

        let row = &result[1];
        assert_eq!(query.fields[0], row[0].field);
        assert_eq!(Some(TypedValue::String("Bob".to_string())), row[0].inner);
        assert_eq!(query.fields[1], row[1].field);
        assert_eq!(Some(TypedValue::Integer(69)), row[1].inner);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_with_failed_query() -> Result<(), String> {
        let query = "
            drop table if exists users;
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ";
        sqlite::Connection::open_with_flags(
            "/tmp/test-lmr2.db",
            sqlite::OpenFlags::new().with_create().with_read_write(),
        )
        .unwrap()
        .execute(query)
        .unwrap();

        let source = Source {
            conn: "/tmp/test-lmr2.db".to_string(),
            kind: SourceType::Sqlite,
        };

        let query1 = Query {
            title: "Test".to_string(),
            sql: "select * from users".to_string(),
            fields: vec![Field {
                title: "User name".to_string(),
                field: "name".to_string(),
                kind: FieldType::String,
            }],
        };

        let query2 = Query {
            title: "Test 2".to_string(),
            sql: "select * from tusers".to_string(),
            fields: vec![Field {
                title: "User name".to_string(),
                field: "name".to_string(),
                kind: FieldType::String,
            }],
        };

        let querys = vec![query1.clone(), query2.clone(), query1.clone()];

        let results = super::fetch(source, querys).await?;
        assert_eq!(3, results.len());

        let (rquery, result) = &results[0];
        assert_eq!(query1.clone(), rquery.clone());
        assert_eq!(None, result.as_ref().err());

        let (rquery, result) = &results[1];
        assert_eq!(query2.clone(), rquery.clone());
        assert_eq!(
            Some("Prepare statement failed: no such table: tusers (code 1)".to_string()),
            result.clone().err()
        );

        let (rquery, result) = &results[2];
        assert_eq!(query1.clone(), rquery.clone());
        assert_eq!(None, result.as_ref().err());

        Ok(())
    }
}
