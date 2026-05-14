use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use rustlens_core::sql::completion::CompletionKind;

use crate::theme;

const KEYWORDS: &[&str] = &[
    "select",
    "from",
    "where",
    "join",
    "left",
    "right",
    "inner",
    "outer",
    "on",
    "group",
    "by",
    "order",
    "limit",
    "offset",
    "insert",
    "into",
    "values",
    "update",
    "set",
    "delete",
    "create",
    "table",
    "view",
    "index",
    "alter",
    "drop",
    "and",
    "or",
    "not",
    "null",
    "true",
    "false",
    "distinct",
    "as",
    "union",
    "all",
    "case",
    "when",
    "then",
    "else",
    "end",
    "returning",
    "ilike",
    "with",
    "recursive",
];

pub fn highlight_job(src: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut chars = src.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        if ch == '-' && chars.peek().map(|(_, c)| *c) == Some('-') {
            let mut end = start + ch.len_utf8();
            for (idx, c) in chars.by_ref() {
                end = idx + c.len_utf8();
                if c == '\n' {
                    break;
                }
            }
            append(&mut job, &src[start..end], theme::MUTED, true);
        } else if ch == '\'' {
            let mut end = start + ch.len_utf8();
            while let Some((idx, c)) = chars.next() {
                end = idx + c.len_utf8();
                if c == '\'' {
                    if chars.peek().map(|(_, next)| *next) == Some('\'') {
                        let _ = chars.next();
                    } else {
                        break;
                    }
                }
            }
            append(
                &mut job,
                &src[start..end],
                egui::Color32::from_rgb(134, 239, 172),
                false,
            );
        } else if ch.is_ascii_digit() {
            let mut end = start + ch.len_utf8();
            while let Some((idx, c)) = chars.peek().copied() {
                if c.is_ascii_digit() || c == '.' {
                    let _ = chars.next();
                    end = idx + c.len_utf8();
                } else {
                    break;
                }
            }
            append(&mut job, &src[start..end], theme::WARNING, false);
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let mut end = start + ch.len_utf8();
            while let Some((idx, c)) = chars.peek().copied() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    let _ = chars.next();
                    end = idx + c.len_utf8();
                } else {
                    break;
                }
            }
            let word = &src[start..end];
            let color = if KEYWORDS.contains(&word.to_ascii_lowercase().as_str()) {
                theme::PRIMARY
            } else {
                theme::TEXT
            };
            append(&mut job, word, color, false);
        } else {
            append(
                &mut job,
                &src[start..start + ch.len_utf8()],
                theme::TEXT,
                false,
            );
        }
    }

    job
}

pub fn completion_kind_label(kind: CompletionKind) -> &'static str {
    match kind {
        CompletionKind::Keyword => "keyword",
        CompletionKind::Table => "table",
        CompletionKind::Column => "column",
        CompletionKind::Snippet => "snippet",
    }
}

fn append(job: &mut LayoutJob, text: &str, color: egui::Color32, italics: bool) {
    let mut format = TextFormat {
        font_id: egui::FontId::monospace(14.0),
        color,
        ..Default::default()
    };
    format.italics = italics;
    job.append(text, 0.0, format);
}

#[cfg(test)]
mod tests {
    use super::highlight_job;

    #[test]
    fn highlights_without_losing_text() {
        let src = "select 'x' -- comment";
        let job = highlight_job(src);
        let text: String = job
            .sections
            .iter()
            .map(|section| &src[section.byte_range.clone()])
            .collect();
        assert_eq!(text, src);
    }
}
