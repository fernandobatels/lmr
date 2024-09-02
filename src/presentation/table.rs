//! Table component

use super::formats::OutputFormat;
use super::Component;
use crate::{
    source::Query,
    value::{TypedValue, Value},
};
use tabled::{builder::Builder, settings::Style, Table};

pub struct TableComponent {}

fn stylefy(table: &mut Table, format: OutputFormat) {
    match format {
        OutputFormat::Plain => table.with(Style::ascii()),
    };
}

impl Component for TableComponent {
    fn render(
        &self,
        query: Query,
        rows: Vec<Vec<Value>>,
        format: OutputFormat,
    ) -> Result<String, String> {
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

        let mut table = btable.build();

        stylefy(&mut table, format);

        Ok(format!("{}", table.to_string()))
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
    pub fn basic_table() {
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
            result,
            Ok(r#"+-----------+-----+
| User name | Age |
+-----------+-----+
| john.abc  | 30  |
+-----------+-----+
| jane.abc  | 25  |
+-----------+-----+"#
                .to_string())
        );
    }
}
