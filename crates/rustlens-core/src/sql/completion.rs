use std::collections::HashMap;

const KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "JOIN",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "ON",
    "GROUP",
    "BY",
    "ORDER",
    "LIMIT",
    "OFFSET",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "TABLE",
    "VIEW",
    "INDEX",
    "ALTER",
    "DROP",
    "AND",
    "OR",
    "NOT",
    "NULL",
    "TRUE",
    "FALSE",
    "DISTINCT",
    "AS",
    "UNION",
    "ALL",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "RETURNING",
    "ILIKE",
    "SIMILAR",
    "WITH",
    "RECURSIVE",
    "LATERAL",
    "UNNEST",
    "ANY",
    "ARRAY",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Table,
    Column,
    Snippet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub insert_text: String,
    pub kind: CompletionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResult {
    pub visible: bool,
    pub prefix_start: usize,
    pub items: Vec<CompletionItem>,
}

pub fn complete(
    text: &str,
    cursor: usize,
    tables: &[String],
    columns: &HashMap<String, Vec<String>>,
) -> CompletionResult {
    let (start, prefix) = current_prefix(text, cursor.min(text.len()));
    let ctx = context_before_cursor(text, start);
    let dot = dot_qualifier(text, start);
    let allow_empty =
        dot.is_some() || matches!(ctx.as_deref(), Some("FROM" | "JOIN" | "INTO" | "UPDATE"));

    if prefix.is_empty() && !allow_empty {
        return CompletionResult {
            visible: false,
            prefix_start: start,
            items: Vec::new(),
        };
    }

    let mut items = Vec::new();
    if let Some(table_like) = dot {
        if let Some(cols) = columns.get(table_like) {
            push_prefix(
                &mut items,
                cols.iter().map(String::as_str),
                prefix,
                CompletionKind::Column,
                40,
            );
            return finish(start, items);
        }
    }

    if matches!(ctx.as_deref(), Some("FROM" | "JOIN" | "INTO" | "UPDATE")) {
        push_prefix(
            &mut items,
            tables.iter().map(String::as_str),
            prefix,
            CompletionKind::Table,
            40,
        );
        return finish(start, items);
    }

    push_prefix(
        &mut items,
        KEYWORDS.iter().copied(),
        prefix,
        CompletionKind::Keyword,
        40,
    );
    push_prefix(
        &mut items,
        tables.iter().map(String::as_str),
        prefix,
        CompletionKind::Table,
        40,
    );

    if "select".starts_with(&prefix.to_ascii_lowercase()) {
        items.push(CompletionItem {
            label: "SELECT * FROM".to_string(),
            insert_text: "SELECT * FROM ".to_string(),
            kind: CompletionKind::Snippet,
        });
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label && a.kind == b.kind);
    items.truncate(40);
    finish(start, items)
}

pub fn apply_completion(
    text: &mut String,
    cursor: &mut usize,
    result: &CompletionResult,
    selected: usize,
) {
    let Some(item) = result.items.get(selected) else {
        return;
    };
    let end = (*cursor).min(text.len());
    text.replace_range(result.prefix_start..end, &item.insert_text);
    *cursor = result.prefix_start + item.insert_text.len();
}

fn finish(prefix_start: usize, items: Vec<CompletionItem>) -> CompletionResult {
    CompletionResult {
        visible: !items.is_empty(),
        prefix_start,
        items,
    }
}

fn context_before_cursor(text: &str, start: usize) -> Option<String> {
    let before = &text[..start];
    let mut it = before.split_whitespace();
    Some(it.next_back()?.to_ascii_uppercase())
}

fn dot_qualifier(text: &str, prefix_start: usize) -> Option<&str> {
    if prefix_start == 0 {
        return None;
    }
    let bytes = text.as_bytes();
    if bytes[prefix_start - 1] != b'.' {
        return None;
    }
    let mut i = prefix_start - 1;
    while i > 0 {
        let b = bytes[i - 1];
        let ok = (b as char).is_ascii_alphanumeric() || b == b'_';
        if !ok {
            break;
        }
        i -= 1;
    }
    Some(&text[i..prefix_start - 1])
}

fn push_prefix<'a>(
    out: &mut Vec<CompletionItem>,
    iter: impl Iterator<Item = &'a str>,
    prefix: &str,
    kind: CompletionKind,
    limit: usize,
) {
    let prefix_upper = prefix.to_ascii_uppercase();
    for value in iter {
        if prefix.is_empty() || value.to_ascii_uppercase().starts_with(&prefix_upper) {
            out.push(CompletionItem {
                label: value.to_string(),
                insert_text: value.to_string(),
                kind,
            });
            if out.len() >= limit {
                return;
            }
        }
    }
}

fn current_prefix(text: &str, cursor: usize) -> (usize, &str) {
    let mut i = cursor;
    let bytes = text.as_bytes();
    while i > 0 {
        let b = bytes[i - 1];
        let ok = (b as char).is_ascii_alphanumeric() || b == b'_';
        if !ok {
            break;
        }
        i -= 1;
    }
    (i, &text[i..cursor])
}

#[cfg(test)]
mod tests {
    use super::{apply_completion, complete, CompletionKind};
    use std::collections::HashMap;

    #[test]
    fn completes_tables_after_from() {
        let result = complete(
            "select * from u",
            15,
            &["users".to_string()],
            &HashMap::new(),
        );
        assert!(result.visible);
        assert_eq!(result.items[0].label, "users");
        assert_eq!(result.items[0].kind, CompletionKind::Table);
    }

    #[test]
    fn completes_columns_after_dot() {
        let mut columns = HashMap::new();
        columns.insert("users".to_string(), vec!["email".to_string()]);
        let result = complete("select users.e", 14, &[], &columns);
        assert_eq!(result.items[0].label, "email");
        assert_eq!(result.items[0].kind, CompletionKind::Column);
    }

    #[test]
    fn applies_completion_to_prefix() {
        let mut text = "select * from u".to_string();
        let mut cursor = text.len();
        let result = complete(&text, cursor, &["users".to_string()], &HashMap::new());
        apply_completion(&mut text, &mut cursor, &result, 0);
        assert_eq!(text, "select * from users");
    }
}
