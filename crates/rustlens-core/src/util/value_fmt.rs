use sqlx::postgres::PgRow;
use sqlx::Row;

/// Best-effort cell formatting for display.
/// This intentionally prefers clarity over completeness.
/// Extend with type-aware formatting as new types are encountered.
pub fn cell_to_string(row: &PgRow, i: usize) -> String {
    if let Ok(v) = row.try_get::<Option<String>, _>(i) {
        return v.unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
        return v
            .map(|x| x.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
        return v
            .map(|x| x.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
        return v
            .map(|x| x.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }

    "<unsupported>".to_string()
}
