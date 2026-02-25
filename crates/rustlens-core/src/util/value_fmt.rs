use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo};

pub fn cell_to_string(row: &PgRow, i: usize) -> String {
    let type_name = row.columns()[i].type_info().name();
    match type_name {
        "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => match row.try_get::<Option<String>, _>(i) {
            Ok(Some(v)) => v,
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "INT2" | "INT4" | "INT8" => match row.try_get::<Option<i64>, _>(i) {
            Ok(Some(v)) => v.to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "FLOAT4" | "FLOAT8" => match row.try_get::<Option<f64>, _>(i) {
            Ok(Some(v)) => v.to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "BOOL" => match row.try_get::<Option<bool>, _>(i) {
            Ok(Some(v)) => v.to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "UUID" => match row.try_get::<Option<uuid::Uuid>, _>(i) {
            Ok(Some(v)) => v.to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "JSON" | "JSONB" => match row.try_get::<Option<serde_json::Value>, _>(i) {
            Ok(Some(v)) => v.to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "TIMESTAMPTZ" => match row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(i) {
            Ok(Some(v)) => v.to_rfc3339(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "TIMESTAMP" => match row.try_get::<Option<chrono::NaiveDateTime>, _>(i) {
            Ok(Some(v)) => v.format("%Y-%m-%d %H:%M:%S").to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "DATE" => match row.try_get::<Option<chrono::NaiveDate>, _>(i) {
            Ok(Some(v)) => v.format("%Y-%m-%d").to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "TIME" => match row.try_get::<Option<chrono::NaiveTime>, _>(i) {
            Ok(Some(v)) => v.format("%H:%M:%S").to_string(),
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        "BYTEA" => match row.try_get::<Option<Vec<u8>>, _>(i) {
            Ok(Some(bytes)) => {
                let preview = hex_preview(&bytes, 16);
                if bytes.len() > 16 {
                    format!("<bytea {}B {}…>", bytes.len(), preview)
                } else {
                    format!("<bytea {}B {}>", bytes.len(), preview)
                }
            }
            Ok(None) => "NULL".into(),
            Err(_) => "<error>".into(),
        },

        _ => match row.try_get::<Option<String>, _>(i) {
            Ok(Some(v)) => v,
            Ok(None) => "NULL".into(),
            Err(_) => format!("<{}>", type_name),
        },
    }
}

fn hex_preview(bytes: &[u8], max: usize) -> String {
    let n = bytes.len().min(max);
    let mut out = String::with_capacity(n * 2);
    for b in &bytes[..n] {
        use std::fmt::Write;
        let _ = write!(out, "{:02x}", b);
    }
    out
}
