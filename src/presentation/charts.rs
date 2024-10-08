//! Charts component

use super::{formats::OutputFormat, Component, ImagePresented, RenderedContent};
use crate::{source::Query, value::Value};
use charts_rs::{self, BarChart, Box, LineChart, PieChart, Series};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Pizza,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ChartComponent {
    pub kind: ChartType,
    #[serde(default)]
    pub keys_by: Option<String>,
    #[serde(default)]
    pub series_by: Option<ChartSeriesBy>,
    #[serde(default)]
    pub series: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ChartSeriesBy {
    pub key: String,
    pub values: String,
}

impl ChartComponent {
    pub fn prepare_series(
        &self,
        query: &Query,
        keys: &Vec<String>,
        data: &Vec<Vec<Value>>,
    ) -> Result<Vec<Series>, String> {
        if self.series.is_none() && self.series_by.is_none() {
            return Err("Series must be defined".to_string());
        }

        let mut series = vec![];

        if let Some(dseries) = &self.series {
            for serie in dseries {
                let col = query
                    .fields
                    .iter()
                    .find(|f| f.field == *serie)
                    .ok_or_else(|| format!("Field {} not found", serie))?;

                let mut values = vec![];
                for row in data {
                    let value = get_value_by(col.field.clone(), row)?;
                    values.push(value);
                }
                series.push(Series::new(col.title.clone(), values));
            }
        }

        if let Some(series_by) = self.series_by.clone() {
            if self.keys_by.is_none() {
                return Err("Keys must be defined".to_string());
            }
            let keys_by = self.keys_by.clone().unwrap();

            let mut dseries = vec![];
            for row in data {
                let serie = get_key_by(series_by.key.clone(), row)?;

                if !dseries.contains(&serie) {
                    dseries.push(serie);
                }
            }

            for serie in dseries {
                let mut values: Vec<(String, f32)> =
                    keys.iter().map(|k| (k.clone(), 0.0)).collect();
                for row in data {
                    let serie_key = get_key_by(series_by.key.clone(), row)?;
                    if serie_key == serie {
                        let value = get_value_by(series_by.values.clone(), row)?;
                        let key = get_key_by(keys_by.clone(), row)?;

                        if let Some(v) = values.iter_mut().find(|(k, _)| k == &key) {
                            *v = (key, value);
                        }
                    }
                }

                series.push(Series::new(serie, values.iter().map(|(_, v)| *v).collect()));
            }
        }

        Ok(series)
    }

    pub fn prepare_keys(
        &self,
        _query: &Query,
        data: &Vec<Vec<Value>>,
    ) -> Result<Vec<String>, String> {
        if self.keys_by.is_none() && self.kind != ChartType::Pizza {
            return Err("Keys must be defined".to_string());
        }

        let mut keys = vec![];

        if let Some(by) = self.keys_by.clone() {
            for row in data {
                let value = get_key_by(by.clone(), row)?;

                if !keys.contains(&value) {
                    keys.push(value);
                }
            }
        }

        Ok(keys)
    }
}

impl Component for ChartComponent {
    fn render(
        &self,
        query: Query,
        data: Vec<Vec<Value>>,
        format: OutputFormat,
    ) -> Result<RenderedContent, String> {
        if format != OutputFormat::Html {
            return Err("Output format without chart support".to_string());
        }

        let keys = self.prepare_keys(&query, &data)?;
        let series = self.prepare_series(&query, &keys, &data)?;

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
            ChartType::Pizza => {
                let mut chart = PieChart::new(series);
                chart.margin = margin;
                chart.svg()
            }
        }
        .map_err(|e| format!("Error generating chart: {}", e))?;

        let png = charts_rs::svg_to_png(&svg)
            .map_err(|e| format!("Error converting SVG to PNG: {}", e))?;

        let cid = Uuid::new_v4().to_string();

        let img_tag = format!(
            "<img class=\"lmr-img\" title=\"{}\" src=\"cid:{}\">",
            query.title, cid
        );

        Ok(RenderedContent {
            content: img_tag,
            images: vec![ImagePresented {
                mime: "image/png".to_string(),
                data: png,
                cid: cid,
            }],
        })
    }
}

fn get_key_by(by: String, row: &Vec<Value>) -> Result<String, String> {
    let col = row
        .iter()
        .find(|v| v.field.field == by)
        .ok_or_else(|| format!("Field {} not found", by))?;
    let value = if let Some(v) = &col.inner {
        v.to_string()
    } else {
        "".to_string()
    };

    Ok(value)
}

fn get_value_by(by: String, row: &Vec<Value>) -> Result<f32, String> {
    let col = row
        .iter()
        .find(|v| v.field.field == by)
        .ok_or_else(|| format!("Field {} not found", by))?;
    let value = if let Some(v) = &col.inner {
        v.to_float()
    } else {
        Ok(0.0)
    }?;

    Ok(value)
}

#[cfg(test)]
pub mod tests {
    use super::{ChartComponent, ChartType};
    use crate::{
        presentation::{charts::ChartSeriesBy, formats::OutputFormat, Component},
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
            keys_by: Some("key".to_string()),
            series: Some(vec!["field".to_string()]),
            series_by: None,
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
            series_by: None,
            keys_by: Some("name".to_string()),
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.render(query.clone(), data.clone(), OutputFormat::Html);
        assert_eq!(true, result.is_ok());
        assert!(result
            .unwrap()
            .content
            .starts_with("<img class=\"lmr-img\" title=\"Title test\" src=\"cid:"));
    }

    #[test]
    pub fn prepare_keys() {
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
            series_by: None,
            keys_by: Some("name".to_string()),
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert_eq!(
            Ok(vec!["john.abc".to_string(), "jane.abc".to_string()]),
            result
        );
    }

    #[test]
    pub fn prepare_keys_duplicate() {
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
            vec![
                Value {
                    inner: Some(TypedValue::String("jane.abc".to_string())),
                    field: query.fields[0].clone(),
                },
                Value {
                    inner: Some(TypedValue::Integer(28)),
                    field: query.fields[1].clone(),
                },
            ],
        ];

        let chart = ChartComponent {
            kind: ChartType::Bar,
            series_by: None,
            keys_by: Some("name".to_string()),
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert_eq!(
            Ok(vec!["john.abc".to_string(), "jane.abc".to_string()]),
            result
        );
    }

    #[test]
    pub fn prepare_keys_not_found() {
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
            series_by: None,
            keys_by: Some("name2".to_string()),
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert_eq!(Err("Field name2 not found".to_string()), result);
    }

    #[test]
    pub fn prepare_series_with_series_option() {
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
            series_by: None,
            keys_by: Some("name".to_string()),
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_series(
            &query,
            &vec!["john.abc".to_string(), "jane.abc".to_string()],
            &data,
        );
        assert!(result.is_ok());
        let series = result.unwrap();

        assert_eq!(series.len(), 1);
        assert_eq!(series[0].name, "Age".to_string());
        assert_eq!(series[0].data, vec![30.0, 25.0]);
    }

    #[test]
    pub fn prepare_series_with_series_by_option() {
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
            series_by: Some(ChartSeriesBy {
                key: "name".to_string(),
                values: "age".to_string(),
            }),
            keys_by: Some("name".to_string()),
            series: None,
        };

        let result = chart.prepare_series(
            &query,
            &vec!["john.abc".to_string(), "jane.abc".to_string()],
            &data,
        );
        assert!(result.is_ok());
        let series = result.unwrap();

        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "john.abc");
        assert_eq!(series[0].data, vec![30.0, 0.0]);
        assert_eq!(series[1].name, "jane.abc");
        assert_eq!(series[1].data, vec![0.0, 25.0]);
    }

    #[test]
    pub fn prepare_series_without_serie_setup() {
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
            series_by: None,
            keys_by: Some("name".to_string()),
            series: None,
        };

        let result = chart.prepare_series(
            &query,
            &vec!["john.abc".to_string(), "jane.abc".to_string()],
            &data,
        );
        assert_eq!(Err("Series must be defined".to_string()), result);
    }

    #[test]
    pub fn prepare_keys_without_keys_setup() {
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
            series_by: None,
            keys_by: None,
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert_eq!(Err("Keys must be defined".to_string()), result);

        let chart = ChartComponent {
            kind: ChartType::Line,
            series_by: None,
            keys_by: None,
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert_eq!(Err("Keys must be defined".to_string()), result);

        let chart = ChartComponent {
            kind: ChartType::Pizza,
            series_by: None,
            keys_by: None,
            series: Some(vec!["age".to_string()]),
        };

        let result = chart.prepare_keys(&query, &data);
        assert!(result.is_ok());
    }
}
