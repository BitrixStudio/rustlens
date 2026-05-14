const CONFIRMATION_KEYWORDS: &[&str] = &[
    "ALTER", "CALL", "COMMENT", "COPY", "CREATE", "DELETE", "DO", "DROP", "GRANT", "INSERT",
    "MERGE", "REFRESH", "REINDEX", "REVOKE", "TRUNCATE", "UPDATE", "VACUUM",
];

pub fn requires_confirmation(sql: &str) -> bool {
    sql_words(sql)
        .iter()
        .any(|word| CONFIRMATION_KEYWORDS.contains(&word.as_str()))
}

fn sql_words(sql: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if chars.peek() == Some(&'\'') {
                    let _ = chars.next();
                } else {
                    in_single_quote = false;
                }
            }
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    let _ = chars.next();
                } else {
                    in_double_quote = false;
                }
            }
            continue;
        }

        if ch == '-' && chars.peek() == Some(&'-') {
            finish_word(&mut words, &mut word);
            let _ = chars.next();
            in_line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            finish_word(&mut words, &mut word);
            let _ = chars.next();
            in_block_comment = true;
            continue;
        }

        if ch == '\'' {
            finish_word(&mut words, &mut word);
            in_single_quote = true;
            continue;
        }

        if ch == '"' {
            finish_word(&mut words, &mut word);
            in_double_quote = true;
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            word.push(ch.to_ascii_uppercase());
        } else {
            finish_word(&mut words, &mut word);
        }
    }

    finish_word(&mut words, &mut word);
    words
}

fn finish_word(words: &mut Vec<String>, word: &mut String) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

#[cfg(test)]
mod tests {
    use super::requires_confirmation;

    #[test]
    fn allows_read_only_selects() {
        assert!(!requires_confirmation("select * from users"));
        assert!(!requires_confirmation(
            "with active as (select * from users) select * from active"
        ));
        assert!(!requires_confirmation("explain select * from users"));
    }

    #[test]
    fn requires_confirmation_for_mutating_statements() {
        assert!(requires_confirmation("delete from users"));
        assert!(requires_confirmation("update users set name = 'x'"));
        assert!(requires_confirmation(
            "insert into users(name) values ('x')"
        ));
        assert!(requires_confirmation("drop table users"));
        assert!(requires_confirmation("truncate users"));
        assert!(requires_confirmation(
            "alter table users add column active bool"
        ));
        assert!(requires_confirmation("grant select on users to readonly"));
    }

    #[test]
    fn ignores_keywords_in_strings_comments_and_quoted_identifiers() {
        assert!(!requires_confirmation("select 'drop table users'"));
        assert!(!requires_confirmation("select \"delete\" from audit"));
        assert!(!requires_confirmation("-- delete from users\nselect 1"));
        assert!(!requires_confirmation("/* update users */ select 1"));
    }
}
