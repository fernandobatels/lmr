//! Field/Value api

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use serde::Deserialize;

/// Raw value from an data source row
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub inner: Option<TypedValue>,
    pub field: Field,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedValue {
    String(String),
    Integer(i64),
    Float(f64),
    Time(NaiveTime),
    Date(NaiveDate),
    DateTime(DateTime<FixedOffset>),
}

impl ToString for TypedValue {
    fn to_string(&self) -> String {
        match self {
            TypedValue::String(v) => v.clone(),
            TypedValue::Integer(v) => v.to_string(),
            TypedValue::Float(v) => v.to_string(),
            TypedValue::Time(v) => v.to_string(),
            TypedValue::Date(v) => v.to_string(),
            TypedValue::DateTime(v) => v.to_string(),
        }
    }
}

/// Field definition
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Field {
    pub field: String,
    pub title: String,
    pub kind: FieldType,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Time,
    Date,
    DateTime,
}

#[cfg(test)]
pub mod tests {
    use chrono::{DateTime, NaiveDate, NaiveTime};

    use crate::value::TypedValue;

    #[test]
    fn typed_value_to_string() -> Result<(), String> {
        assert_eq!(
            "Some text".to_string(),
            TypedValue::String("Some text".to_string()).to_string()
        );
        assert_eq!("1234".to_string(), TypedValue::Integer(1234).to_string());
        assert_eq!(
            "1234.56".to_string(),
            TypedValue::Float(1234.56).to_string()
        );
        assert_eq!(
            "12:35:25".to_string(),
            TypedValue::Time(NaiveTime::from_hms(12, 35, 25)).to_string()
        );
        assert_eq!(
            "2025-05-12".to_string(),
            TypedValue::Date(NaiveDate::from_ymd(2025, 05, 12)).to_string()
        );
        assert_eq!(
            "2015-05-15 00:00:00 +00:00".to_string(),
            TypedValue::DateTime(
                DateTime::from_timestamp(1431648000, 0)
                    .unwrap()
                    .fixed_offset()
            )
            .to_string()
        );

        Ok(())
    }
}
