use super::{
    cold_branch,
    search::{byte_search, safe_byte_match_table, SafeByteMatchTable},
    Kind, Lexer, Token,
};
use crate::diagnostics;

use memchr::{memchr, memchr2};
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start_unicode};

static ASCII_JSX_ID_START_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'$' | b'-'));

static NOT_ASCII_JSX_ID_CONTINUE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'-')));

impl<'a> Lexer<'a> {
    /// `JSXDoubleStringCharacters` ::
    ///   `JSXDoubleStringCharacter` `JSXDoubleStringCharactersopt`
    /// `JSXDoubleStringCharacter` ::
    ///   `JSXStringCharacter` but not "
    /// `JSXSingleStringCharacters` ::
    ///   `JSXSingleStringCharacter` `JSXSingleStringCharactersopt`
    /// `JSXSingleStringCharacter` ::
    ///   `JSXStringCharacter` but not '
    /// `JSXStringCharacter` ::
    ///   `SourceCharacter` but not one of `HTMLCharacterReference`

    /// Read JSX string literal.
    /// # SAFETY
    /// * `delimiter` must be an ASCII character.
    /// * Next char in `lexer.source` must be ASCII.
    pub(super) unsafe fn read_jsx_string_literal(&mut self, delimiter: u8) -> Kind {
        // Skip opening quote
        debug_assert!(delimiter.is_ascii());
        // SAFETY: Caller guarantees next byte is ASCII, so `.add(1)` is a UTF-8 char boundary
        let after_opening_quote = self.source.position().add(1);
        let remaining = self.source.str_from_pos_to_end(after_opening_quote);

        let len = memchr(delimiter, remaining.as_bytes());
        if let Some(len) = len {
            // SAFETY: `after_opening_quote` + `len` is position of delimiter.
            // Caller guarantees delimiter is ASCII, so 1 byte after it is a UTF-8 char boundary.
            let after_closing_quote = after_opening_quote.add(len + 1);
            self.source.set_position(after_closing_quote);
            Kind::Str
        } else {
            self.source.advance_to_end();
            self.error(diagnostics::UnterminatedString(self.unterminated_range()));
            Kind::Undetermined
        }
    }

    pub(crate) fn next_jsx_child(&mut self) -> Token {
        self.token.start = self.offset();
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    /// Expand the current token for `JSXIdentifier`
    pub(crate) fn next_jsx_identifier(&mut self, start_offset: u32) -> Token {
        let kind = self.read_jsx_identifier(start_offset);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// [`JSXChild`](https://facebook.github.io/jsx/#prod-JSXChild)
    /// `JSXChild` :
    /// `JSXText`
    /// `JSXElement`
    /// `JSXFragment`
    /// { `JSXChildExpressionopt` }
    fn read_jsx_child(&mut self) -> Kind {
        match self.peek() {
            Some('<') => {
                self.consume_char();
                Kind::LAngle
            }
            Some('{') => {
                self.consume_char();
                Kind::LCurly
            }
            Some(_) => {
                // The tokens `{`, `<`, `>` and `}` cannot appear in JSX text.
                // The TypeScript compiler raises the error "Unexpected token. Did you mean `{'>'}` or `&gt;`?".
                // Where as the Babel compiler does not raise any errors.
                // The following check omits `>` and `}` so that more Babel tests can be passed.
                let len = memchr2(b'{', b'<', self.remaining().as_bytes());
                if let Some(len) = len {
                    // SAFETY: `memchr2` guarantees `len` will be offset from current position
                    // of a `{` or `<` byte. So must be a valid UTF-8 boundary, and within bounds of source.
                    let end = unsafe { self.source.position().add(len) };
                    self.source.set_position(end);
                } else {
                    self.source.advance_to_end();
                }
                Kind::JSXText
            }
            None => Kind::Eof,
        }
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn read_jsx_identifier(&mut self, _start_offset: u32) -> Kind {
        // Handle start char
        let Some(start_byte) = self.source.peek_byte() else {
            return Kind::Ident;
        };

        if !start_byte.is_ascii() {
            // Unicode identifiers are rare, so cold branch
            return cold_branch(|| {
                if is_identifier_start_unicode(self.peek().unwrap()) {
                    self.consume_char();
                    self.read_jsx_identifier_tail_unicode()
                } else {
                    Kind::Ident
                }
            });
        }

        if !ASCII_JSX_ID_START_TABLE.matches(start_byte) {
            return Kind::Ident;
        }

        // Consume bytes which are part of identifier tail
        let next_byte = byte_search! {
            lexer: self,
            table: NOT_ASCII_JSX_ID_CONTINUE_TABLE,
            handle_eof: {
                return Kind::Ident;
            },
        };

        // Found a matching byte.
        // Either end of identifier found, or a Unicode char.
        if !next_byte.is_ascii() {
            // Unicode chars are rare in identifiers, so cold branch to keep common path for ASCII
            // as fast as possible
            return cold_branch(|| self.read_jsx_identifier_tail_unicode());
        }

        Kind::Ident
    }

    /// Consume rest of JSX identifier after Unicode character found.
    fn read_jsx_identifier_tail_unicode(&mut self) -> Kind {
        while let Some(c) = self.peek() {
            if !is_identifier_part(c) && c != '-' {
                break;
            }
            self.consume_char();
        }
        Kind::Ident
    }
}
