type LineCol = (usize, usize);

pub struct LocatedChars<'a> {
    input: &'a str,
    offset: usize,
    line_num: usize,
    col_num: usize,
    peeked: Option<((usize, char), LineCol)>,
    pos: LineCol,
}

#[allow(dead_code)]
impl<'a> LocatedChars<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            line_num: 1,
            col_num: 1,
            peeked: None,
            pos: (1, 1),
        }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.pos.0, self.pos.1)
    }

    pub fn peek(&mut self) -> Option<(usize, char)> {
        if self.peeked.is_none() {
            self.peeked = self.next_inner();
        }
        self.peeked.map(|p| p.0)
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.peek().map(|(_, c)| c)
    }

    pub fn next_if(&mut self, predicate: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        self.peek();
        match self.peeked {
            Some((item, pos)) if predicate(item.1) => {
                self.peeked = None;
                self.pos = pos;
                Some(item)
            }
            _ => None,
        }
    }

    pub fn next_if_eq(&mut self, expected: char) -> Option<(usize, char)> {
        self.next_if(|c| c == expected)
    }

    /// Returns the remaining unprocessed input slice.
    fn remaining(&self) -> &'a str {
        &self.input[self.offset..]
    }

    fn next_inner(&mut self) -> Option<((usize, char), LineCol)> {
        let c = self.remaining().chars().next()?;
        let offset = self.offset;
        let pos = (self.line_num, self.col_num);

        match c {
            '\r' => {
                self.offset += '\r'.len_utf8();
                // Consume a following '\n' if present (CRLF)
                if self.remaining().starts_with('\n') {
                    self.offset += '\n'.len_utf8();
                }

                // self.last_line = self.line_num;
                // self.last_col = self.col_num;
                self.line_num += 1;
                self.col_num = 1;
            }
            '\n' => {
                self.offset += '\n'.len_utf8();
                // self.last_line = self.line_num;
                // self.last_col = self.col_num;
                self.line_num += 1;
                self.col_num = 1;
            }
            _ => {
                self.offset += c.len_utf8();
                // self.last_line = self.line_num;
                // self.last_col = self.col_num;
                self.col_num += 1;
            }
        }

        Some(((offset, c), pos))
    }
}

impl<'a> Iterator for LocatedChars<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((item, pos)) = self.peeked.take() {
            self.pos = pos;
            return Some(item);
        }

        let (item, pos) = self.next_inner()?;
        self.pos = pos;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================
    // Offset
    // =========================================================

    #[test]
    fn test_offset_starts_at_zero() {
        let mut iter = LocatedChars::new("abc");
        assert_eq!(iter.next(), Some((0, 'a')));
    }

    #[test]
    fn test_offset_increments_by_one_for_ascii() {
        let mut iter = LocatedChars::new("abc");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, 'b')));
        assert_eq!(iter.next(), Some((2, 'c')));
    }

    #[test]
    fn test_offset_increments_by_utf8_byte_width() {
        let mut iter = LocatedChars::new("aéb");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, 'é')));
        assert_eq!(iter.next(), Some((3, 'b')));
    }

    #[test]
    fn test_offset_across_lf() {
        let mut iter = LocatedChars::new("a\nb");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, '\n')));
        assert_eq!(iter.next(), Some((2, 'b')));
    }

    #[test]
    fn test_offset_across_cr() {
        let mut iter = LocatedChars::new("a\rb");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, '\r')));
        assert_eq!(iter.next(), Some((2, 'b')));
    }

    #[test]
    fn test_offset_across_crlf() {
        let mut iter = LocatedChars::new("a\r\nb");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, '\r')));
        assert_eq!(iter.next(), Some((3, 'b')));
    }

    #[test]
    fn test_offset_emoji() {
        let mut iter = LocatedChars::new("a😀b");
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, '😀')));
        assert_eq!(iter.next(), Some((5, 'b')));
    }

    // =========================================================
    // pos()
    // =========================================================

    #[test]
    fn test_pos_before_any_next_is_one_one() {
        let iter = LocatedChars::new("abc");
        assert_eq!(iter.pos(), (1, 1));
    }

    #[test]
    fn test_pos_after_first_char() {
        let mut iter = LocatedChars::new("abc");
        iter.next();
        assert_eq!(iter.pos(), (1, 1));
    }

    #[test]
    fn test_pos_after_second_char() {
        let mut iter = LocatedChars::new("abc");
        iter.next();
        iter.next();
        assert_eq!(iter.pos(), (1, 2));
    }

    #[test]
    fn test_pos_after_lf() {
        let mut iter = LocatedChars::new("a\nb");
        iter.next();
        iter.next();
        assert_eq!(iter.pos(), (1, 2));
        iter.next();
        assert_eq!(iter.pos(), (2, 1));
    }

    #[test]
    fn test_pos_after_cr() {
        let mut iter = LocatedChars::new("a\rb");
        iter.next();
        iter.next();
        assert_eq!(iter.pos(), (1, 2));
        iter.next();
        assert_eq!(iter.pos(), (2, 1));
    }

    #[test]
    fn test_pos_after_crlf() {
        let mut iter = LocatedChars::new("a\r\nb");
        iter.next();
        iter.next();
        assert_eq!(iter.pos(), (1, 2));
        iter.next();
        assert_eq!(iter.pos(), (2, 1));
    }

    #[test]
    fn test_pos_not_affected_by_peek() {
        let mut iter = LocatedChars::new("abc");
        iter.next();
        iter.peek();
        assert_eq!(iter.pos(), (1, 1));
    }

    #[test]
    fn test_pos_updates_after_consuming_peeked() {
        let mut iter = LocatedChars::new("abc");
        iter.next();
        iter.peek();
        iter.next();
        assert_eq!(iter.pos(), (1, 2));
    }

    // =========================================================
    // Internal offset field directly
    // =========================================================

    #[test]
    fn test_internal_offset_zero_at_start() {
        let iter = LocatedChars::new("hello");
        assert_eq!(iter.offset, 0);
    }

    #[test]
    fn test_internal_offset_advances_correctly() {
        let mut iter = LocatedChars::new("abc");
        iter.next();
        assert_eq!(iter.offset, 1);
        iter.next();
        assert_eq!(iter.offset, 2);
    }

    #[test]
    fn test_internal_offset_skips_lf_in_crlf() {
        let mut iter = LocatedChars::new("\r\n");
        iter.next(); // consumes '\r' and '\n'
        assert_eq!(iter.offset, 2); // both bytes consumed
    }

    #[test]
    fn test_remaining_correct_after_advance() {
        let mut iter = LocatedChars::new("abcd");
        iter.next();
        iter.next();
        assert_eq!(iter.remaining(), "cd");
    }

    #[test]
    fn test_remaining_empty_at_end() {
        let mut iter = LocatedChars::new("a");
        iter.next();
        assert_eq!(iter.remaining(), "");
    }

    // =========================================================
    // peek / next_if / next_if_eq
    // =========================================================

    #[test]
    fn test_peek_returns_correct_offset() {
        let mut iter = LocatedChars::new("abc");
        assert_eq!(iter.peek(), Some((0, 'a')));
    }

    #[test]
    fn test_peek_does_not_change_internal_offset_visibly() {
        let mut iter = LocatedChars::new("abc");
        iter.peek();
        // next() must return the same item as peek()
        assert_eq!(iter.next(), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, 'b')));
    }

    #[test]
    fn test_next_if_returns_correct_offset() {
        let mut iter = LocatedChars::new("abc");
        assert_eq!(iter.next_if(|c| c == 'a'), Some((0, 'a')));
        assert_eq!(iter.next(), Some((1, 'b')));
    }

    #[test]
    fn test_next_if_eq_returns_correct_offset() {
        let mut iter = LocatedChars::new("abc");
        assert_eq!(iter.next_if_eq('a'), Some((0, 'a')));
    }

    // =========================================================
    // Round-trip
    // =========================================================

    #[test]
    fn test_offsets_are_strictly_increasing() {
        let input = "hello\nworld";
        let offsets: Vec<usize> = LocatedChars::new(input).map(|(offset, _)| offset).collect();
        for w in offsets.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn test_offset_indexes_into_origin() {
        let input = "hello\nworld";
        for (offset, c) in LocatedChars::new(input) {
            if c != '\r' {
                assert_eq!(input[offset..].chars().next(), Some(c));
            }
        }
    }
}
