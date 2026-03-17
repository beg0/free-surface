//! # Parse helpers
//!
//! Helper functions to parse config files

/// Remove surrounding single quotes, and unescape '' -> '
pub fn unquote_single(s: &str) -> String {
    let inner = if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner.replace("''", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- No quotes ---

    #[test]
    fn test_unquoted_empty_string() {
        assert_eq!(unquote_single(""), "");
    }

    #[test]
    fn test_unquoted_with_spaces() {
        assert_eq!(unquote_single("hello world"), "hello world");
    }

    // --- Surrounding quotes removed ---

    #[test]
    fn test_quoted_string() {
        assert_eq!(unquote_single("'hello'"), "hello");
    }

    #[test]
    fn test_quoted_empty_string() {
        assert_eq!(unquote_single("''"), "");
    }

    #[test]
    fn test_quoted_string_with_spaces() {
        assert_eq!(unquote_single("'hello world'"), "hello world");
    }

    #[test]
    fn test_quoted_single_character() {
        assert_eq!(unquote_single("'x'"), "x");
    }

    // --- Escape sequence '' -> ' ---

    #[test]
    fn test_escaped_quote_in_quoted_string() {
        assert_eq!(unquote_single("'it''s'"), "it's");
    }

    #[test]
    fn test_escaped_quote_at_start_of_quoted_string() {
        assert_eq!(unquote_single("'''hello'"), "'hello");
    }

    #[test]
    fn test_escaped_quote_at_end_of_quoted_string() {
        assert_eq!(unquote_single("'hello'''"), "hello'");
    }

    #[test]
    fn test_multiple_escaped_quotes_in_quoted_string() {
        assert_eq!(unquote_single("'it''s a l''amour'"), "it's a l'amour");
    }

    #[test]
    fn test_escaped_quote_in_unquoted_string() {
        // No surrounding quotes: '' is still unescaped to '
        assert_eq!(unquote_single("it''s"), "it's");
    }

    // --- Edge cases ---

    #[test]
    fn test_single_quote_alone_is_not_unquoted() {
        // A lone "'" has len < 2 for the starts+ends check to be meaningful,
        // but starts_with and ends_with both match the same character -
        // len >= 2 guard prevents stripping, so it stays as-is.
        assert_eq!(unquote_single("'"), "'");
    }

    #[test]
    fn test_only_opening_quote_not_stripped() {
        assert_eq!(unquote_single("'hello"), "'hello");
    }

    #[test]
    fn test_only_closing_quote_not_stripped() {
        assert_eq!(unquote_single("hello'"), "hello'");
    }

    #[test]
    fn test_double_quoted_string_not_stripped() {
        // Double quotes are not handled by this function
        assert_eq!(unquote_single("\"hello\""), "\"hello\"");
    }

    #[test]
    fn test_quoted_whitespace_only() {
        assert_eq!(unquote_single("'   '"), "   ");
    }
}
