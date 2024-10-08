//! Export/Presentation api

use crate::{source::Query, value::Value};
use formats::OutputFormat;
use log::*;

pub mod charts;
pub mod formats;
pub mod table;

#[derive(Clone, Debug, PartialEq)]
pub struct DataPresented {
    pub is_html: bool,
    pub content: String,
    pub images: Vec<ImagePresented>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImagePresented {
    pub cid: String,
    pub mime: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderedContent {
    pub content: String,
    pub images: Vec<ImagePresented>,
}

pub trait Component {
    fn render(
        &self,
        query: Query,
        data: Vec<Vec<Value>>,
        format: OutputFormat,
    ) -> Result<RenderedContent, String>;
}

/// Export the querys results into specified format
pub fn present_as(
    data: Vec<(Query, Box<dyn Component>, Result<Vec<Vec<Value>>, String>)>,
    title: String,
    format: OutputFormat,
) -> Result<DataPresented, String> {
    info!("Generating the presentation");

    let mut r = String::new();
    let mut images = vec![];

    r.push_str(&format.title1(&format!("The {} results are here!", title)));

    for (query, comp, result) in data {
        r.push_str(&format.break_line());

        let rquery = present_query_as(query, comp, result, format.clone())?;
        r.push_str(&rquery.content);
        r.push_str(&format.break_line());
        r.push_str(&format.break_line());
        images.extend(rquery.images);
    }

    r.push_str(
        &format.simple("Consider support the project at https://github.com/fernandobatels/lmr"),
    );

    let r = format.body(&r);

    Ok(DataPresented {
        is_html: format == OutputFormat::Html,
        content: r,
        images,
    })
}

/// Export the query result
fn present_query_as(
    query: Query,
    component: Box<dyn Component>,
    data: Result<Vec<Vec<Value>>, String>,
    format: OutputFormat,
) -> Result<RenderedContent, String> {
    debug!("Generating for '{}' query", query.title);

    let mut r = RenderedContent {
        content: String::new(),
        images: vec![],
    };

    r.content
        .push_str(&format.title2(&format!("Query: {}", query.title)));

    if let Ok(rows) = data {
        if rows.len() > 0 {
            let table = component.render(query, rows, format.clone());

            if let Ok(table) = table {
                r.content.push_str(&format.simple(&table.content));
                r.images.extend(table.images);
            } else {
                r.content.push_str(
                    &format.simple(&format!("Error on rendering: {}", table.err().unwrap())),
                );
            }
        } else {
            r.content.push_str(&format.simple("Empty result"));
        }
    } else {
        r.content
            .push_str(&format.simple(&format!("Query falied: {}", data.err().unwrap())));
    }

    Ok(r)
}

#[cfg(test)]
pub mod tests {
    use crate::{
        presentation::{charts::ChartComponent, charts::*, table::TableComponent, Component},
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
            Box::new(TableComponent {}) as Box<dyn Component>,
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
                images: vec![],
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

        let data = vec![(
            query.clone(),
            Box::new(TableComponent {}) as Box<dyn Component>,
            Err("Table 'users' not found".to_string()),
        )];

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                images: vec![],
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

        let data = vec![(
            query.clone(),
            Box::new(TableComponent {}) as Box<dyn Component>,
            Ok(vec![]),
        )];

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                images: vec![],
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

    #[test]
    fn present_as_txt_with_failed_render() -> Result<(), String> {
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
            Box::new(ChartComponent {
                kind: ChartType::Bar,
                keys_by: Some("name".to_string()),
                series_by: None,
                series: Some(vec![]),
            }) as Box<dyn Component>,
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
            ]),
        )];

        let exported = super::present_as(data, "Project Name".to_string(), OutputFormat::Plain)?;

        assert_eq!(
            DataPresented {
                is_html: false,
                images: vec![],
                content: r#"
The Project Name results are here!


Query: Title test

Error on rendering: Output format without chart support


Consider support the project at https://github.com/fernandobatels/lmr
"#
                .to_string()
            },
            exported.clone()
        );

        Ok(())
    }
}
