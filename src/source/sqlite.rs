//! Sqlite driver implementation
use super::{Driver, Query};
use crate::value::{FieldType, TypedValue, Value};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime};
use sqlite::{self, Connection, Error, State};

pub struct SqliteDriver {
    pub conn: Option<Connection>,
}

impl SqliteDriver {
    pub fn init() -> Self {
        Self { conn: None }
    }
}

#[async_trait]
impl Driver for SqliteDriver {
    async fn connect(&mut self, sconn: String) -> Result<(), String> {
        let conn = sqlite::open(sconn)
            .map_err(|e| format!("Sqlite connection failed: {}", e.to_string()))?;

        self.conn = Some(conn);

        Ok(())
    }

    async fn fetch(&mut self, query: Query) -> Result<Vec<Vec<Value>>, String> {
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
                let efmt = |e: Error| {
                    format!(
                        "Read column {} row {} failed: {}",
                        col.field,
                        row.len(),
                        e.to_string()
                    )
                };
                let inner = match &col.kind {
                    FieldType::Integer => statement
                        .read::<Option<i64>, _>(col.field.as_str())
                        .map_err(efmt)?
                        .map(|v| TypedValue::Integer(v)),
                    FieldType::String => statement
                        .read::<Option<String>, _>(col.field.as_str())
                        .map_err(efmt)?
                        .map(|v| TypedValue::String(v)),
                    FieldType::Float => statement
                        .read::<Option<f64>, _>(col.field.as_str())
                        .map_err(efmt)?
                        .map(|v| TypedValue::Float(v)),
                    FieldType::Time => {
                        let raw = statement
                            .read::<Option<String>, _>(col.field.as_str())
                            .map_err(efmt)?;
                        if let Some(raw) = raw {
                            let dt = NaiveTime::parse_from_str(&raw, "%H:%M:%S").map_err(|e| {
                                format!("Error on parse the {} to time: {}", raw, e.to_string())
                            })?;

                            Some(TypedValue::Time(dt))
                        } else {
                            None
                        }
                    }
                    FieldType::Date => {
                        let raw = statement
                            .read::<Option<String>, _>(col.field.as_str())
                            .map_err(efmt)?;
                        if let Some(raw) = raw {
                            let dt = NaiveDate::parse_from_str(&raw, "%Y-%m-%d").map_err(|e| {
                                format!("Error on parse the {} to date: {}", raw, e.to_string())
                            })?;

                            Some(TypedValue::Date(dt))
                        } else {
                            None
                        }
                    }
                    FieldType::DateTime => {
                        let raw = statement
                            .read::<Option<String>, _>(col.field.as_str())
                            .map_err(efmt)?;
                        if let Some(raw) = raw {
                            let dt = DateTime::parse_from_rfc3339(&raw).map_err(|e| {
                                format!("Error on parse the {} to datetime: {}", raw, e.to_string())
                            })?;

                            Some(TypedValue::DateTime(dt))
                        } else {
                            None
                        }
                    }
                };

                row.push(Value {
                    inner,
                    field: col.clone(),
                });
            }

            values.push(row);
        }

        Ok(values)
    }
}

#[cfg(test)]
#[allow(deprecated)]
pub mod tests {
    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone};

    use crate::{
        source::{sqlite::SqliteDriver, Driver, Query},
        value::{Field, FieldType, TypedValue},
    };

    #[tokio::test]
    async fn supported_types() -> Result<(), String> {
        let mut driver = SqliteDriver::init();
        driver.connect(":memory:".to_string()).await?;

        let query = "
                    CREATE TABLE test (a TEXT, b INTEGER, c REAL, d TEXT, e TEXT, f TEXT);
                    INSERT INTO test VALUES (null, null, null, null, null, null);
                    INSERT INTO test VALUES ('Olá mundo', 2024, 123.45, '23:55:19', '2024-05-15', '1996-12-19T16:39:57-08:00');
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
                Field {
                    title: "d".to_string(),
                    field: "d".to_string(),
                    kind: FieldType::Time,
                },
                Field {
                    title: "e".to_string(),
                    field: "e".to_string(),
                    kind: FieldType::Date,
                },
                Field {
                    title: "f".to_string(),
                    field: "f".to_string(),
                    kind: FieldType::DateTime,
                },
            ],
        };

        let result = driver.fetch(query.clone()).await?;
        assert_eq!(2, result.len());

        let row = &result[0];
        assert_eq!(None, row[0].inner);
        assert_eq!(None, row[1].inner);
        assert_eq!(None, row[2].inner);
        assert_eq!(None, row[3].inner);
        assert_eq!(None, row[4].inner);
        assert_eq!(None, row[5].inner);

        let row = &result[1];
        assert_eq!(
            Some(TypedValue::String("Olá mundo".to_string())),
            row[0].inner
        );
        assert_eq!(Some(TypedValue::Integer(2024)), row[1].inner);
        assert_eq!(Some(TypedValue::Float(123.45)), row[2].inner);
        assert_eq!(
            Some(TypedValue::Time(NaiveTime::from_hms(23, 55, 19))),
            row[3].inner
        );
        assert_eq!(
            Some(TypedValue::Date(NaiveDate::from_ymd(2024, 05, 15))),
            row[4].inner
        );
        assert_eq!(
            Some(TypedValue::DateTime(
                FixedOffset::west(8 * 3600)
                    .with_ymd_and_hms(1996, 12, 19, 16, 39, 57)
                    .unwrap()
            )),
            row[5].inner
        );

        Ok(())
    }

    #[tokio::test]
    async fn column_not_found() -> Result<(), String> {
        let mut driver = SqliteDriver::init();
        driver.connect(":memory:".to_string()).await?;

        let query = "
                CREATE TABLE test (a TEXT, b INTEGER, c REAL, d TEXT, e TEXT, f TEXT);
                INSERT INTO test VALUES (null, null, null, null, null, null);
                INSERT INTO test VALUES ('Olá mundo', 2024, 123.45, '23:55:19', '2024-05-15', '1996-12-19T16:39:57-08:00');
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
                    title: "g".to_string(),
                    field: "g".to_string(),
                    kind: FieldType::Integer,
                },
            ],
        };

        let result = driver.fetch(query.clone()).await;
        assert_eq!(
            Some("Read column g row 1 failed: the index is out of range (g)".to_string()),
            result.err()
        );

        Ok(())
    }
}
