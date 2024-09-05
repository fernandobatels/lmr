//! Export/Presentation api

use crate::{source::Query, value::Value};
use formats::OutputFormat;
use log::*;
use table::TableComponent;

pub mod formats;
pub mod table;

#[derive(Clone, Debug, PartialEq)]
pub struct DataPresented {
    pub is_html: bool,
    pub content: String,
}

pub trait Component {
    fn render(&self, query: Query, data: Vec<Vec<Value>>, format: OutputFormat) -> String;
}

/// Export the querys results into specified format
pub fn present_as(
    data: Vec<(Query, Result<Vec<Vec<Value>>, String>)>,
    title: String,
    format: OutputFormat,
) -> Result<DataPresented, String> {
    info!("Generating the presentation");

    let mut r = String::new();

    r.push_str(&format.title1(&format!("The {} results are here!", title)));

    for (query, result) in data {
        r.push_str(&format.break_line());

        let rquery = present_query_as(query, result, format.clone())?;
        r.push_str(&rquery);
        r.push_str(&format.break_line());
        r.push_str(&format.break_line());
    }

    r.push_str(
        &format.simple("Consider support the project at https://github.com/fernandobatels/lmr"),
    );

    let r = format.body(&r);

    Ok(DataPresented {
        is_html: format == OutputFormat::Html,
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

    r.push_str(&format.title2(&format!("Query: {}", query.title)));

    if let Ok(rows) = data {
        if rows.len() > 0 {
            let table = TableComponent {}.render(query, rows, format.clone());

            r.push_str(&format.simple(&table));
        } else {
            r.push_str(&format.simple("Empty result"));
        }
    } else {
        r.push_str(&format.simple(&format!("Query falied: {}", data.err().unwrap())));
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

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The Project Name results are here!


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


Consider support the project at https://github.com/fernandobatels/lmr
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

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The Project Name results are here!


Query: Title test

Query falied: Table 'users' not found


Consider support the project at https://github.com/fernandobatels/lmr
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

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                content: r#"
The Project Name results are here!


Query: Title test

Empty result


Consider support the project at https://github.com/fernandobatels/lmr
"#
                .to_string()
            },
            exported.clone()
        );

        Ok(())
    }
}
