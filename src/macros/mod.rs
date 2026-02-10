#[macro_export]
macro_rules! tok {
    ($source:expr,$kind:expr, $start:expr, $end:expr) => {
        Token::new($kind, Span::new(PathBuf::from($source), $start, $end))
    };

    ($kind:path, $arg:expr, $start:expr, $end:expr) => {
        Token::new($kind($arg), Span::new($start, $end))
    };
}

#[macro_export]
macro_rules! impl_span {
    ($t:ty) => {
        impl $t {
            pub fn span(&self) -> &Span {
                &self.span
            }
        }
    };
}

#[macro_export]
macro_rules! token_matches {
    ($token:expr, $($pattern:pat_param)|+) => {
        matches!($token.kind(), $($pattern)|+)
    };
}
