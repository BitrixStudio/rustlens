use crate::app::state::SessionState;

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
    // Postgres-flavored
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

pub fn refresh_completion(s: &mut SessionState) {
    let (start, prefix) = current_prefix(&s.sql_text, s.sql_cursor);

    let ctx = context_before_cursor(&s.sql_text, start);
    let dot = dot_qualifier(&s.sql_text, start);

    let allow_empty =
        dot.is_some() || matches!(ctx.as_deref(), Some("FROM" | "JOIN" | "INTO" | "UPDATE"));

    if prefix.is_empty() && !allow_empty {
        s.completion.visible = false;
        s.completion.items.clear();
        s.completion.selected = 0;
        return;
    }
    let ctx = context_before_cursor(&s.sql_text, start);
    let mut items: Vec<String> = Vec::new();

    // Column completion: "... table_alias_or_table.<prefix>"
    if let Some(table_like) = dot {
        if let Some(cols) = s.sql_columns.get(table_like) {
            push_prefix(items.as_mut(), cols.iter().map(String::as_str), prefix, 30);
            finalize(s, start, items);
            return;
        }
    }
    // Table completion after FROM/JOIN/INTO/UPDATE
    if matches!(ctx.as_deref(), Some("FROM" | "JOIN" | "INTO" | "UPDATE")) {
        push_prefix(
            items.as_mut(),
            s.sql_tables.iter().map(String::as_str),
            prefix,
            30,
        );
        finalize(s, start, items);
        return;
    }
    // General completion: keywords + tables
    push_prefix(items.as_mut(), KEYWORDS.iter().copied(), prefix, 30);
    push_prefix(
        items.as_mut(),
        s.sql_tables.iter().map(String::as_str),
        prefix,
        30,
    );

    items.sort_unstable();
    items.dedup();
    items.truncate(30);

    finalize(s, start, items);
}

fn finalize(s: &mut SessionState, start: usize, items: Vec<String>) {
    if items.is_empty() {
        s.completion.visible = false;
        s.completion.items.clear();
        s.completion.selected = 0;
        return;
    }
    s.completion.prefix_start = start;
    s.completion.items = items;
    s.completion.selected = 0;
    s.completion.visible = true;
}

// Detect context keyword immediately before current token
// Very simple logic for MVP: look at the last "word" before `start`
fn context_before_cursor(text: &str, start: usize) -> Option<String> {
    let before = &text[..start];
    let mut it = before.split_whitespace();
    let last = it.next_back()?;
    Some(last.to_ascii_uppercase())
}

// If the char before prefix is '.', return the identifier before that
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

fn push_prefix<'a, I>(out: &mut Vec<String>, iter: I, prefix: &str, limit: usize)
where
    I: Iterator<Item = &'a str>,
{
    if out.len() >= limit {
        return;
    }

    if prefix.is_empty() {
        for s in iter.take(limit - out.len()) {
            out.push(s.to_string());
        }
        return;
    }

    let p_up = prefix.to_ascii_uppercase();
    for s in iter {
        if s.to_ascii_uppercase().starts_with(&p_up) {
            out.push(s.to_string());
            if out.len() >= limit {
                return;
            }
        }
    }
}

fn current_prefix(text: &str, cursor: usize) -> (usize, &str) {
    let c = cursor.min(text.len());
    let bytes = text.as_bytes();

    let mut i = c;
    while i > 0 {
        let b = bytes[i - 1];
        let ok = (b as char).is_ascii_alphanumeric() || b == b'_';
        if !ok {
            break;
        }
        i -= 1;
    }
    (i, &text[i..c])
}

pub fn accept_completion(s: &mut SessionState) {
    if !s.completion.visible || s.completion.items.is_empty() {
        return;
    }
    let kw = s.completion.items[s.completion.selected].clone();
    let start = s.completion.prefix_start;
    let end = s.sql_cursor.min(s.sql_text.len());

    s.sql_text.replace_range(start..end, &kw);
    s.sql_cursor = start + kw.len();
    s.completion.visible = false;
}
