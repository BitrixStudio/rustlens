use ratatui::text::{Line, Span};
use std::fmt;
use tui_syntax::{themes, Highlighter};

pub struct SqlSyntax {
    hl: Highlighter,
    last: String,
    cached: Vec<Line<'static>>,
}

impl fmt::Debug for SqlSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlSyntax")
            .field("hl", &"<Highlighter>")
            .field("last", &self.last)
            .field("cached_len", &self.cached.len())
            .finish()
    }
}

impl SqlSyntax {
    pub fn new() -> Self {
        let mut hl = Highlighter::new(themes::one_dark());
        hl.register_language(tui_syntax::sql())
            .expect("register sql");
        Self {
            hl,
            last: String::new(),
            cached: Vec::new(),
        }
    }

    pub fn highlight(&mut self, src: &str) -> &[Line<'static>] {
        if src != self.last {
            self.last.clear();
            self.last.push_str(src);

            let lines = self
                .hl
                .highlight("sql", src)
                .unwrap_or_else(|_| vec![Line::raw(src.to_string())]);

            self.cached = lines
                .into_iter()
                .map(|line| {
                    let spans: Vec<Span<'static>> = line
                        .spans
                        .into_iter()
                        .map(|sp| Span::styled(sp.content.to_string(), sp.style))
                        .collect();
                    Line::from(spans)
                })
                .collect();
        }

        &self.cached
    }
}
