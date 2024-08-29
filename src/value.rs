//! Field/Value api

use serde::Deserialize;

/// Raw value from an data source row
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub inner: TypedValue,
    pub field: Field,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedValue {
    String(Option<String>),
    Integer(Option<i64>),
    Float(Option<f64>),
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
}
