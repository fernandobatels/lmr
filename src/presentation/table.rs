//! Table component

use super::formats::OutputFormat;
use super::Component;
use crate::{
    source::Query,
    value::{TypedValue, Value},
};
use table_to_html::{
    html::{Attribute, HtmlElement, HtmlVisitorMut},
    HtmlTable,
};
use tabled::{builder::Builder, settings::Style};

pub struct TableComponent {}

impl Component for TableComponent {
    fn render(&self, query: Query, rows: Vec<Vec<Value>>, format: OutputFormat) -> String {
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

        match format {
            OutputFormat::Plain => btable.build().with(Style::ascii()).to_string(),
            OutputFormat::Html => {
                let rows: Vec<Vec<String>> = btable.into();
                let mut table = HtmlTable::with_header(rows);
                table.visit_mut(HtmlTableClasses {});

                format!("{}", table)
            }
            OutputFormat::Markdown => btable.build().with(Style::markdown()).to_string(),
        }
    }
}

struct HtmlTableClasses {}

impl HtmlVisitorMut for HtmlTableClasses {
    fn visit_element_mut(&mut self, e: &mut HtmlElement) -> bool {
        if e.tag() == "table" {
            let mut attrs = e.attrs().to_vec();
            attrs.push(Attribute::new("class", "lmr-table"));
            *e = HtmlElement::new("table", attrs, e.value().cloned());
        }

        true
    }
}

#[cfg(test)]
pub mod tests {
    use super::TableComponent;
    use crate::presentation::formats::OutputFormat;
    use crate::presentation::Component;
    use crate::source::Query;
    use crate::value::{Field, FieldType, TypedValue, Value};

    #[test]
    pub fn txt_table() {
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

        let data = vec![
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
                    inner: Some(TypedValue::String("jane.abc".to_string())),
                    field: query.fields[0].clone(),
                },
                Value {
                    inner: Some(TypedValue::Integer(25)),
                    field: query.fields[1].clone(),
                },
            ],
        ];

        let table = TableComponent {};
        let result = table.render(query, data, OutputFormat::Plain);

        assert_eq!(
            r#"+-----------+-----+
| User name | Age |
+-----------+-----+
| john.abc  | 30  |
+-----------+-----+
| jane.abc  | 25  |
+-----------+-----+"#
                .to_string(),
            result
        );
    }

    #[test]
    pub fn markdown_table() {
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

        let data = vec![
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
                    inner: Some(TypedValue::String("jane.abc".to_string())),
                    field: query.fields[0].clone(),
                },
                Value {
                    inner: Some(TypedValue::Integer(25)),
                    field: query.fields[1].clone(),
                },
            ],
        ];

        let table = TableComponent {};
        let result = table.render(query, data, OutputFormat::Markdown);

        assert_eq!(
            r#"| User name | Age |
|-----------|-----|
| john.abc  | 30  |
| jane.abc  | 25  |"#
                .to_string(),
            result
        );
    }

    #[test]
    pub fn html_table() {
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

        let data = vec![
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
                    inner: Some(TypedValue::String("jane.abc".to_string())),
                    field: query.fields[0].clone(),
                },
                Value {
                    inner: Some(TypedValue::Integer(25)),
                    field: query.fields[1].clone(),
                },
            ],
        ];

        let table = TableComponent {};
        let result = table.render(query, data, OutputFormat::Html);

        assert_eq!(
            r#"<table class="lmr-table">
    <thead>
        <tr>
            <th>
                <div>
                    <p>
                        User name
                    </p>
                </div>
            </th>
            <th>
                <div>
                    <p>
                        Age
                    </p>
                </div>
            </th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>
                <div>
                    <p>
                        john.abc
                    </p>
                </div>
            </td>
            <td>
                <div>
                    <p>
                        30
                    </p>
                </div>
            </td>
        </tr>
        <tr>
            <td>
                <div>
                    <p>
                        jane.abc
                    </p>
                </div>
            </td>
            <td>
                <div>
                    <p>
                        25
                    </p>
                </div>
            </td>
        </tr>
    </tbody>
</table>"#
                .to_string(),
            result
        );
    }
}
