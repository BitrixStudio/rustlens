use crate::app::state::SessionState;

const KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON", "GROUP", "BY",
    "ORDER", "LIMIT", "OFFSET", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE",
    "TABLE", "VIEW", "INDEX", "ALTER", "DROP", "AND", "OR", "NOT", "NULL", "TRUE", "FALSE",
    "DISTINCT", "AS", "UNION", "ALL", "CASE", "WHEN", "THEN", "ELSE", "END",
];

pub fn refresh_completion(s: &mut SessionState) {
    let (start, prefix) = current_prefix(&s.sql_text, s.sql_cursor);
    if prefix.is_empty() {
        s.completion.visible = false;
        s.completion.items.clear();
        s.completion.selected = 0;
        return;
    }

    let upper = prefix.to_ascii_uppercase();
    let mut items: Vec<&'static str> = KEYWORDS
        .iter()
        .copied()
        .filter(|k| k.starts_with(&upper))
        .take(30)
        .collect();

    if items.is_empty() {
        s.completion.visible = false;
        s.completion.items.clear();
        s.completion.selected = 0;
        return;
    }

    items.sort_unstable();

    s.completion.prefix_start = start;
    s.completion.items = items;
    s.completion.selected = 0;
    s.completion.visible = true;
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
    let kw = s.completion.items[s.completion.selected];
    let start = s.completion.prefix_start;
    let end = s.sql_cursor.min(s.sql_text.len());

    s.sql_text.replace_range(start..end, kw);
    s.sql_cursor = start + kw.len();
    s.completion.visible = false;
}
