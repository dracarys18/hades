use hades_ast::{CompilerContext, ModulePath as AstModulePath, WalkAst};
use hades_error::Span;
use hades_lexer::Lexer;
use hades_module::make_typed_module;
use hades_parser::Parser;

fn parse_and_lower(src: &str) -> hades_mir::mir::module::MirModule {
    let mut lexer = Lexer::new(src.as_bytes(), "test".to_string());
    lexer.tokenize().expect("lex error");
    let mut parser = Parser::new(lexer.into_tokens(), "test".to_string());
    let ast = parser.parse().unwrap_or_else(|_| panic!("parse error"));
    let mut ctx = CompilerContext::new();
    ctx.set_module_path(AstModulePath::Local("test".to_string()));
    let program = ast.walk(&mut ctx, Span::default()).expect("type error");
    let typed = make_typed_module(
        ctx,
        AstModulePath::Local("test".to_string()),
        program,
        vec![],
    );
    hades_mir::lower(typed)
}

#[test]
fn test_simple_return() {
    let mir = parse_and_lower("fn answer(): int { return 42; }");
    let out = mir.to_string();
    println!("{out}");
    let expected = "\
fn test__answer() -> int {
    let _0: int;

    bb0: {
        _0 = const 42;
        return;
    }
}
";
    assert_eq!(out, expected);
}

#[test]
fn test_binary_op() {
    let mir = parse_and_lower("fn add(a: int, b: int): int { return a + b; }");
    let out = mir.to_string();
    println!("{out}");
    let expected = "\
fn test__add(_1: int, _2: int) -> int {
    let _0: int;
    let _1: int;
    let _2: int;
    let _3: int;
    let _4: int;

    bb0: {
        _3 = copy _1;
        _4 = copy _2;
        _0 = copy _3 + copy _4;
        return;
    }
}
";
    assert_eq!(out, expected);
}

#[test]
fn test_if_else() {
    let mir =
        parse_and_lower("fn sign(x: int): int { if (x > 0) { return 1; } else { return -1; } }");
    let out = mir.to_string();
    println!("{out}");
    let expected = "\
fn test__sign(_1: int) -> int {
    let _0: int;
    let _1: int;
    let _2: int;
    let _3: int;
    let _4: bool;
    let _5: int;

    bb0: {
        _2 = copy _1;
        _3 = const 0;
        _4 = copy _2 > copy _3;
        switchInt(copy _4) {
            1 => bb1,
            otherwise => bb2,
        };
    }
    bb1: {
        _0 = const 1;
        return;
    }
    bb2: {
        _5 = const 1;
        _0 = -copy _5;
        return;
    }
    bb3: {
        return;
    }
}
";
    assert_eq!(out, expected);
}

#[test]
fn test_while_loop() {
    let mir = parse_and_lower(
        "fn count(n: int): int { let i: int = 0; while (i < n) { i = i + 1; } return i; }",
    );
    let out = mir.to_string();
    println!("{out}");
    let expected = "\
fn test__count(_1: int) -> int {
    let _0: int;
    let _1: int;
    let _2: int;
    let _3: int;
    let _4: int;
    let _5: bool;
    let _6: int;
    let _7: int;

    bb0: {
        _2 = const 0;
        goto -> bb1;
    }
    bb1: {
        _3 = copy _2;
        _4 = copy _1;
        _5 = copy _3 < copy _4;
        switchInt(copy _5) {
            1 => bb2,
            otherwise => bb3,
        };
    }
    bb2: {
        _6 = copy _2;
        _7 = const 1;
        _2 = copy _6 + copy _7;
        goto -> bb1;
    }
    bb3: {
        return;
    }
}
";
    assert_eq!(out, expected);
}
