#[macro_export]
macro_rules! tok {
    ($kind:expr, $start:expr, $end:expr) => {
        Token::new($kind, Span::new($start, $end))
    };

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
