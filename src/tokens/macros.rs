#[macro_export]
macro_rules! tok {
    // Case 1: Just a TokenKind, with start..end
    ($kind:expr, $start:expr, $end:expr) => {
        Token::new($kind, Span::new($start, $end))
    };

    // Case 2: TokenKind constructor with arguments, like Ident("foo")
    ($kind:path, $arg:expr, $start:expr, $end:expr) => {
        Token::new($kind($arg), Span::new($start, $end))
    };
}

#[macro_export]
macro_rules! token_matches {
    ($token:expr, $($pattern:pat_param)|+) => {
        matches!($token.kind(), $($pattern)|+)
    };
}
