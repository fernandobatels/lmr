//! Export/Presentation api

use log::*;
use serde::Deserialize;
use tabled::{builder::Builder, settings::Style};

use crate::{
    source::Query,
    value::{TypedValue, Value},
};

#[derive(Clone, Debug, PartialEq)]
pub struct DataPresented {
    pub is_html: bool,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum OutputFormat {
    Plain,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Plain
    }
}

/// Export the querys results into specified format
pub fn present_as(
    data: Vec<(Query, Result<Vec<Vec<Value>>, String>)>,
    format: OutputFormat,
) -> Result<DataPresented, String> {
    info!("Generating the presentation");

    let mut r = String::new();

    r.push_str(&format!("\nThe results of your query are here!\n\n"));

    for (query, result) in data {
        r.push_str("\n");

        let rquery = present_query_as(query, result, format.clone())?;
        r.push_str(&rquery);
        r.push_str("\n");
        r.push_str("\n");
    }

    Ok(DataPresented {
        is_html: false,
        content: r,
    })
}

/// Export the query result
pub fn present_query_as(
    query: Query,
    data: Result<Vec<Vec<Value>>, String>,
    format: OutputFormat,
) -> Result<String, String> {
    debug!("Generating for '{}' query", query.title);

    let mut r = String::new();

    r.push_str(&format!("Query: {}\n\n", query.title));

    if let Ok(rows) = data {
        if rows.len() > 0 {
            let mut btable = Builder::default();

            btable.push_record(
                query
                    .fields
                    .iter()
                    .map(|e| e.title.clone())
                    .collect::<Vec<String>>(),
            );

            for row in rows {
                btable.push_record(
                    row.iter()
                        .map(|e| {
                            e.inner
                                .clone()
                                .unwrap_or(TypedValue::String(String::new()))
                                .to_string()
                        })
                        .collect::<Vec<String>>(),
                );
            }

            let table = btable.build().with(Style::ascii()).to_string();

            r.push_str(&format!("{}\n", table));
        } else {
            r.push_str("Empty result\n");
        }
    } else {
        r.push_str(&format!("Query falied: {}\n", data.err().unwrap()));
    }

    Ok(r)
}

#[cfg(test)]
pub mod tests {
    use crate::{
        source::Query,
        value::{Field, FieldType, TypedValue, Value},
    };

    use super::{DataPresented, OutputFormat};

    #[test]
    fn present_as_txt() -> Result<(), String> {
        let query = Query {
            title: "Title test".to_string(),
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

        let data = vec![(
            query.clone(),
            Ok(vec![
                vec![
                    Value {
                        inner: Some(TypedValue::String("john.abc".to_string())),
                        field: query.fields[0].clone(),
                    },
                    Value {
                        inner: Some(TypedValue::Integer(30)),
                        field: query.fields[1].clone(),
                    },
                ],
                vec![
                    Value {
                        inner: None,
                        field: query.fields[0].clone(),
                    },
                    Value {
                        inner: Some(TypedValue::Integer(28)),
                        field: query.fields[1].clone(),
                    },
                ],
                vec![
                    Value {
                        inner: Some(TypedValue::String("ane.abc".to_string())),
                        field: query.fields[0].clone(),
                    },
                    Value {
                        inner: None,
                        field: query.fields[1].clone(),
                    },
                ],
            ]),
        )];

        let exported = super::present_as(data, OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The results of your query are here!


Query: Title test

+-----------+-----+
| User name | Age |
+-----------+-----+
| john.abc  | 30  |
+-----------+-----+
|           | 28  |
+-----------+-----+
| ane.abc   |     |
+-----------+-----+


"#
                .to_string()
            },
            exported.clone()
        );

        Ok(())
    }

    #[test]
    fn present_as_txt_with_empty_result() -> Result<(), String> {
        let query = Query {
            title: "Title test".to_string(),
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

        let data = vec![(query.clone(), Err("Table 'users' not found".to_string()))];

        let exported = super::present_as(data, OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The results of your query are here!


Query: Title test

Query falied: Table 'users' not found


"#
                .to_string()
            },
            exported.clone()
        );

        Ok(())
    }

    #[test]
    fn present_as_txt_with_failed_result() -> Result<(), String> {
        let query = Query {
            title: "Title test".to_string(),
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

        let data = vec![(query.clone(), Ok(vec![]))];

        let exported = super::present_as(data, OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The results of your query are here!


Query: Title test

Empty result


"#
                .to_string()
            },
            exported.clone()
        );

        Ok(())
    }
}
