pub fn cursor_line_col(text: &str, cursor: usize) -> (usize, usize) {
    let c = cursor.min(text.len());
    let before = &text[..c];

    let mut line = 0usize;
    let mut col = 0usize;

    for ch in before.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::cursor_line_col;

    #[test]
    fn reports_line_and_column() {
        assert_eq!(cursor_line_col("select", 0), (0, 0));
        assert_eq!(cursor_line_col("select", 3), (0, 3));
        assert_eq!(cursor_line_col("one\ntwo", 5), (1, 1));
        assert_eq!(cursor_line_col("one\ntwo", usize::MAX), (1, 3));
    }

    #[test]
    fn counts_unicode_characters_not_bytes() {
        assert_eq!(cursor_line_col("åβ\nç", "åβ\n".len()), (1, 0));
    }
}
