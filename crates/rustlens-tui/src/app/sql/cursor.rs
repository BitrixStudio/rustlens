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
