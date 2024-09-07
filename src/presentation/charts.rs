//! Charts component

use super::{formats::OutputFormat, Component};
use crate::{source::Query, value::Value};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use charts_rs::{BarChart, Box, LineChart, Series};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ChartComponent {
    pub kind: ChartType,
    pub keys: String,
    pub series: Vec<String>,
}

impl Component for ChartComponent {
    fn render(
        &self,
        query: Query,
        data: Vec<Vec<Value>>,
        format: OutputFormat,
    ) -> Result<String, String> {
        if format != OutputFormat::Html {
            return Err("Output format without chart support".to_string());
        }

        let mut series = vec![];

        for serie in &self.series {
            let mut values = vec![];
            for row in &data {
                let col = row
                    .iter()
                    .find(|v| v.field.field == *serie)
                    .ok_or_else(|| format!("Field {} not found", serie))?;
                let value = if let Some(v) = &col.inner {
                    v.to_float()
                } else {
                    Ok(0.0)
                }?;

                values.push(value);
            }
            let col = query
                .fields
                .iter()
                .find(|f| f.field == *serie)
                .ok_or_else(|| format!("Field {} not found", serie))?;

            series.push(Series::new(col.title.clone(), values));
        }

        let mut keys = vec![];

        for row in &data {
            let col = row
                .iter()
                .find(|v| v.field.field == self.keys)
                .ok_or_else(|| format!("Field {} not found", self.keys))?;
            let value = if let Some(v) = &col.inner {
                v.to_string()
            } else {
                "".to_string()
            };

            keys.push(value);
        }

        let margin = Box {
            top: 10.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        };

        let svg = match self.kind {
            ChartType::Bar => {
                let mut chart = BarChart::new(series, keys);
                chart.margin = margin;
                chart.svg()
            }
            ChartType::Line => {
                let mut chart = LineChart::new(series, keys);
                chart.margin = margin;
                chart.svg()
            }
        }
        .map_err(|e| format!("Error generating chart: {}", e))?;

        let svg = STANDARD.encode(svg);

        Ok(format!(
            "<img class=\"lmr-img\" src=\"data:image/svg+xml;base64, {}\">",
            svg
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use super::{ChartComponent, ChartType};
    use crate::{
        presentation::{formats::OutputFormat, Component},
        source::Query,
        value::{Field, FieldType, TypedValue, Value},
    };

    #[test]
    pub fn non_html_format() {
        let query = Query {
            sql: "SELECT * FROM table".to_string(),
            title: "Test".to_string(),
            fields: vec![],
        };

        let data = vec![];

        let chart = ChartComponent {
            kind: ChartType::Bar,
            keys: "key".to_string(),
            series: vec!["field".to_string()],
        };

        let result = chart.render(query.clone(), data.clone(), OutputFormat::Plain);
        assert_eq!(
            Err("Output format without chart support".to_string()),
            result
        );

        let result = chart.render(query, data, OutputFormat::Markdown);
        assert_eq!(
            Err("Output format without chart support".to_string()),
            result
        );
    }

    #[test]
    pub fn html_format() {
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

        let chart = ChartComponent {
            kind: ChartType::Bar,
            keys: "name".to_string(),
            series: vec!["age".to_string()],
        };

        let result = chart.render(query.clone(), data.clone(), OutputFormat::Html);
        assert_eq!(
            Err("Output format without chart support".to_string()),
            result
        );
    }
}
