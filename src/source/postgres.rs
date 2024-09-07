//! Postgress driver implementation

use super::{Driver, Query};
use crate::value::{FieldType, TypedValue, Value};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use log::*;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use tokio_postgres::{types::Type, Client, NoTls};

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
            let raw = &stmt.columns()[idx];

            columns.push((col, idx, raw));
        }

        let utc = FixedOffset::east_opt(0).ok_or("Invalid timezone".to_string())?;

        for row in qrows {
            let mut r = vec![];

            for (col, idx, rcol) in &columns {
                let inner = match col.kind {
                    FieldType::Integer => match rcol.type_() {
                        &Type::INT2 => row
                            .try_get::<usize, Option<i16>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| ov.map(|v| v.into()).map(TypedValue::Integer)),
                        &Type::INT4 => row
                            .try_get::<usize, Option<i32>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| ov.map(|v| v.into()).map(TypedValue::Integer)),
                        &Type::INT8 => row
                            .try_get::<usize, Option<i64>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|v| v.map(TypedValue::Integer)),
                        _ => Err(format!("Invalid integer type {}", rcol.type_())),
                    },
                    FieldType::String => row
                        .try_get::<usize, Option<String>>(*idx)
                        .map_err(|e| e.to_string())
                        .map(|v| v.map(TypedValue::String)),
                    FieldType::Float => match rcol.type_() {
                        &Type::FLOAT4 => row
                            .try_get::<usize, Option<f32>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| ov.map(|v| v.into()).map(TypedValue::Float)),
                        &Type::FLOAT8 => row
                            .try_get::<usize, Option<f64>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| ov.map(|v| v.into()).map(TypedValue::Float)),
                        &Type::NUMERIC => row
                            .try_get::<usize, Option<Decimal>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| ov.map(|v| v.to_f64().unwrap_or(0.0)).map(TypedValue::Float)),
                        _ => Err(format!("Invalid float type {}", rcol.type_())),
                    },
                    FieldType::Date => row
                        .try_get::<usize, Option<NaiveDate>>(*idx)
                        .map_err(|e| e.to_string())
                        .map(|v| v.map(TypedValue::Date)),
                    FieldType::Time => row
                        .try_get::<usize, Option<NaiveTime>>(*idx)
                        .map_err(|e| e.to_string())
                        .map(|v| v.map(TypedValue::Time)),
                    FieldType::DateTime => match rcol.type_() {
                        &Type::TIMESTAMPTZ => row
                            .try_get::<usize, Option<DateTime<FixedOffset>>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|v| v.map(TypedValue::DateTime)),
                        &Type::TIMESTAMP => row
                            .try_get::<usize, Option<NaiveDateTime>>(*idx)
                            .map_err(|e| e.to_string())
                            .map(|ov| {
                                ov.map(|v| DateTime::from_naive_utc_and_offset(v, utc))
                                    .map(TypedValue::DateTime)
                            }),
                        _ => Err(format!("Invalid datetime type {}", rcol.type_())),
                    },
                }
                .map_err(|e| format!("Column {} row {} error: {}", col.field, r.len(), e))?;

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
    async fn basic_supported_types() -> Result<(), String> {
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
    async fn ints_supported_types() -> Result<(), String> {
        let mut driver = PostgresDriver::init();

        driver
            .connect("postgresql://postgres:123@localhost/lmr_tests".to_string())
            .await?;

        let conn = driver.conn.as_ref().unwrap();

        conn.execute("CREATE temp TABLE test (a smallint, b int, c bigint);", &[])
            .await
            .unwrap();
        conn.execute("INSERT INTO test VALUES (null, null, null);", &[])
            .await
            .unwrap();
        conn.execute("INSERT INTO test VALUES (123, 456, 789);", &[])
            .await
            .unwrap();

        let query = Query {
            title: "Test".to_string(),
            sql: "select * from test".to_string(),
            fields: vec![
                Field {
                    title: "a".to_string(),
                    field: "a".to_string(),
                    kind: FieldType::Integer,
                },
                Field {
                    title: "b".to_string(),
                    field: "b".to_string(),
                    kind: FieldType::Integer,
                },
                Field {
                    title: "c".to_string(),
                    field: "c".to_string(),
                    kind: FieldType::Integer,
                },
            ],
        };

        let result = driver.fetch(query.clone()).await?;
        assert_eq!(2, result.len());

        let row = &result[0];
        assert_eq!(None, row[0].inner);
        assert_eq!(None, row[1].inner);
        assert_eq!(None, row[2].inner);

        let row = &result[1];
        assert_eq!(Some(TypedValue::Integer(123)), row[0].inner);
        assert_eq!(Some(TypedValue::Integer(456)), row[1].inner);
        assert_eq!(Some(TypedValue::Integer(789)), row[2].inner);

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

    #[tokio::test]
    async fn timestamp_supported_types() -> Result<(), String> {
        let mut driver = PostgresDriver::init();

        driver
            .connect("postgresql://postgres:123@localhost/lmr_tests".to_string())
            .await?;

        let conn = driver.conn.as_ref().unwrap();

        conn.execute(
            "CREATE temp TABLE test (a timestamp with time zone, b timestamp without time zone);",
            &[],
        )
        .await
        .unwrap();
        conn.execute("INSERT INTO test VALUES (null, null);", &[])
            .await
            .unwrap();
        conn.execute(
            "INSERT INTO test VALUES ('1996-12-19T16:39:57-08:00', '1996-12-19T16:39:57+00:00');",
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
                    kind: FieldType::DateTime,
                },
                Field {
                    title: "b".to_string(),
                    field: "b".to_string(),
                    kind: FieldType::DateTime,
                },
            ],
        };

        let result = driver.fetch(query.clone()).await?;
        assert_eq!(2, result.len());

        let row = &result[0];
        assert_eq!(None, row[0].inner);
        assert_eq!(None, row[1].inner);

        let row = &result[1];
        assert_eq!(
            Some(TypedValue::DateTime(
                FixedOffset::west(8 * 3600)
                    .with_ymd_and_hms(1996, 12, 19, 16, 39, 57)
                    .unwrap()
            )),
            row[0].inner
        );
        assert_eq!(
            Some(TypedValue::DateTime(
                FixedOffset::west(0)
                    .with_ymd_and_hms(1996, 12, 19, 16, 39, 57)
                    .unwrap()
            )),
            row[1].inner
        );

        Ok(())
    }

    #[tokio::test]
    async fn floats_supported_types() -> Result<(), String> {
        let mut driver = PostgresDriver::init();

        driver
            .connect("postgresql://postgres:123@localhost/lmr_tests".to_string())
            .await?;

        let conn = driver.conn.as_ref().unwrap();

        conn.execute(
            "CREATE temp TABLE test (a real, b double precision, c numeric, d decimal);",
            &[],
        )
        .await
        .unwrap();

        conn.execute("INSERT INTO test VALUES (null, null, null, null);", &[])
            .await
            .unwrap();

        conn.execute(
            "INSERT INTO test VALUES (123.45, 6789.01, 98765.4321, 12345.6789);",
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
                    kind: FieldType::Float,
                },
                Field {
                    title: "b".to_string(),
                    field: "b".to_string(),
                    kind: FieldType::Float,
                },
                Field {
                    title: "c".to_string(),
                    field: "c".to_string(),
                    kind: FieldType::Float,
                },
                Field {
                    title: "d".to_string(),
                    field: "d".to_string(),
                    kind: FieldType::Float,
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

        let row = &result[1];
        assert_eq!(Some(TypedValue::Float(123.44999694824219)), row[0].inner);
        assert_eq!(Some(TypedValue::Float(6789.01)), row[1].inner);
        assert_eq!(Some(TypedValue::Float(98765.4321)), row[2].inner);
        assert_eq!(Some(TypedValue::Float(12345.6789)), row[3].inner);

        Ok(())
    }
}
