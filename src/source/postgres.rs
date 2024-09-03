//! Postgress driver implementation

use super::{Driver, Query};
use crate::value::{FieldType, TypedValue, Value};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use log::*;
use tokio_postgres::{Client, Error, NoTls};

pub struct PostgresDriver {
    pub conn: Option<Client>,
}

impl PostgresDriver {
    pub fn init() -> Self {
        Self { conn: None }
    }
}

#[async_trait]
impl Driver for PostgresDriver {
    async fn connect(&mut self, sconn: String) -> Result<(), String> {
        let (client, connection) = tokio_postgres::connect(&sconn, NoTls)
            .await
            .map_err(|e| format!("Postgres connection failed: {}", e.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Postgres connection error: {}", e);
            }
        });

        self.conn = Some(client);

        Ok(())
    }

    async fn fetch(&mut self, query: Query) -> Result<Vec<Vec<Value>>, String> {
        let conn = self
            .conn
            .as_ref()
            .ok_or("Connection not established".to_string())?;

        let stmt = conn
            .prepare(query.sql.as_str())
            .await
            .map_err(|e| format!("Prepare statement failed: {}", e.to_string()))?;

        let qrows = conn
            .query(&stmt, &[])
            .await
            .map_err(|e| format!("Query failed: {}", e.to_string()))?;

        let mut columns = vec![];
        let mut rows = vec![];

        for col in query.fields {
            let idx = stmt
                .columns()
                .iter()
                .position(|c| c.name() == col.field)
                .ok_or(format!("Column {} not found", col.field))?;

            columns.push((col, idx));
        }

        for row in qrows {
            let mut r = vec![];

            for (col, idx) in &columns {
                let efmt = |e: Error| {
                    format!(
                        "Read column {} row {} failed: {}",
                        col.field,
                        r.len(),
                        e.to_string()
                    )
                };

                let inner = match col.kind {
                    FieldType::Integer => row
                        .try_get::<usize, Option<i32>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::Integer(v)),
                    FieldType::String => row
                        .try_get::<usize, Option<String>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::String(v)),
                    FieldType::Float => row
                        .try_get::<usize, Option<f64>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::Float(v)),
                    FieldType::Date => row
                        .try_get::<usize, Option<NaiveDate>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::Date(v)),
                    FieldType::Time => row
                        .try_get::<usize, Option<NaiveTime>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::Time(v)),
                    FieldType::DateTime => row
                        .try_get::<usize, Option<DateTime<FixedOffset>>>(*idx)
                        .map_err(efmt)?
                        .map(|v| TypedValue::DateTime(v)),
                };

                r.push(Value {
                    inner,
                    field: col.clone(),
                });
            }

            rows.push(r);
        }

        Ok(rows)
    }
}

#[cfg(test)]
#[allow(deprecated)]
pub mod tests {
    use crate::{
        source::{postgres::PostgresDriver, Driver, Query},
        value::{Field, FieldType, TypedValue},
    };
    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone};

    #[tokio::test]
    async fn supported_types() -> Result<(), String> {
        let mut driver = PostgresDriver::init();

        driver
            .connect("postgresql://postgres:123@localhost/lmr_tests".to_string())
            .await?;

        let conn = driver.conn.as_ref().unwrap();

        conn.execute(
            "CREATE temp TABLE test (a varchar(50), b INT, c float, d time, e date, f timestamp with time zone);",
            &[],
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO test VALUES (null, null, null, null, null, null);",
            &[],
        )
        .await
        .unwrap();
        conn.execute("INSERT INTO test VALUES ('Olá mundo', 2024, 123.45, '23:55:19', '2024-05-15', '1996-12-19T16:39:57-08:00');", &[]).await.unwrap();

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
        let mut driver = PostgresDriver::init();

        driver
            .connect("postgresql://postgres:123@localhost/lmr_tests".to_string())
            .await?;

        let conn = driver.conn.as_ref().unwrap();

        conn.execute(
            "CREATE temp TABLE test (a varchar(50), b INT, c numeric(18,4), d time, e date, f timestamp with time zone);",
            &[],
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO test VALUES (null, null, null, null, null, null);",
            &[],
        )
        .await
        .unwrap();

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
        assert_eq!(Some("Column g not found".to_string()), result.err());

        Ok(())
    }
}
