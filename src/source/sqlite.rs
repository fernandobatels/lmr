//! Sqlite driver implementation
use super::{Driver, Query, QueryResult};
use crate::value::{FieldType, TypedValue, Value};
use sqlite::{self, Connection, State};

pub struct SqliteDriver {
    pub conn: Option<Connection>,
}

impl SqliteDriver {
    pub fn init() -> Self {
        Self { conn: None }
    }
}

impl Driver for SqliteDriver {
    async fn connect(&mut self, sconn: String) -> Result<(), String> {
        let conn = sqlite::open(sconn)
            .map_err(|e| format!("Sqlite connection failed: {}", e.to_string()))?;

        self.conn = Some(conn);

        Ok(())
    }

    async fn fetch(&mut self, query: Query) -> Result<QueryResult, String> {
        let conn = self
            .conn
            .as_ref()
            .ok_or("Connection not established".to_string())?;

        let mut statement = conn
            .prepare(query.sql)
            .map_err(|e| format!("Prepare statement failed: {}", e.to_string()))?;

        let mut values = vec![];

        while let Ok(State::Row) = statement.next() {
            let mut row = vec![];

            for col in &query.fields {
                let inner = match &col.kind {
                    FieldType::Integer => TypedValue::Integer(
                        statement
                            .read::<Option<i64>, _>(col.field.as_str())
                            .map_err(|e| {
                                format!(
                                    "Read column {} row {} failed: {}",
                                    col.field,
                                    row.len(),
                                    e.to_string()
                                )
                            })?,
                    ),
                    FieldType::String => TypedValue::String(
                        statement
                            .read::<Option<String>, _>(col.field.as_str())
                            .map_err(|e| {
                                format!(
                                    "Read column {} row {} failed: {}",
                                    col.field,
                                    row.len(),
                                    e.to_string()
                                )
                            })?,
                    ),
                    FieldType::Float => TypedValue::Float(
                        statement
                            .read::<Option<f64>, _>(col.field.as_str())
                            .map_err(|e| {
                                format!(
                                    "Read column {} row {} failed: {}",
                                    col.field,
                                    row.len(),
                                    e.to_string()
                                )
                            })?,
                    ),
                };

                row.push(Value {
                    inner,
                    field: col.clone(),
                });
            }

            values.push(row);
        }

        Ok(QueryResult { values })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        source::{sqlite::SqliteDriver, Driver, Query},
        value::{Field, FieldType, TypedValue},
    };

    #[tokio::test]
    async fn supported_types() -> Result<(), String> {
        let mut driver = SqliteDriver::init();
        driver.connect(":memory:".to_string()).await?;

        let query = "
                CREATE TABLE test (a TEXT, b INTEGER, c REAL);
                INSERT INTO test VALUES (null, null, null);
                INSERT INTO test VALUES ('Olá mundo', 2024, 123.45);
            ";
        driver.conn.as_ref().unwrap().execute(query).unwrap();

        let query = Query {
            title: "Test".to_string(),
            sql: "select * from test".to_string(),
            fields: vec![
                Field {
                    title: "a".to_string(),
                    field: "a".to_string(),
                    kind: FieldType::String,
                },
                Field {
                    title: "b".to_string(),
                    field: "b".to_string(),
                    kind: FieldType::Integer,
                },
                Field {
                    title: "c".to_string(),
                    field: "c".to_string(),
                    kind: FieldType::Float,
                },
            ],
        };

        let result = driver.fetch(query.clone()).await?;
        assert_eq!(2, result.values.len());

        let row = &result.values[0];
        assert_eq!(TypedValue::String(None), row[0].inner);
        assert_eq!(TypedValue::Integer(None), row[1].inner);
        assert_eq!(TypedValue::Float(None), row[2].inner);

        let row = &result.values[1];
        assert_eq!(TypedValue::String(Some("Olá mundo".to_string())), row[0].inner);
        assert_eq!(TypedValue::Integer(Some(2024)), row[1].inner);
        assert_eq!(TypedValue::Float(Some(123.45)), row[2].inner);

        Ok(())
    }
}
