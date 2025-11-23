use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hades::ast::Program;
use hades::lexer::Lexer;
use hades::parser::Parser;

// Small code samples
const SIMPLE_EXPR: &str = "let x = 42 + 10;";

const FUNCTION_DEF: &str = r#"
fn main(): int {
    return 0;
}
"#;

const HELLO_WORLD: &str = r#"
fn main():int {
    printf("Hello world");
    return 0;
}
"#;

// Medium-sized code samples
const STRUCT_WITH_METHODS: &str = r#"
struct Point {
    x: int,
    y: int
}

fn main(): int {
    let p = Point {x: 10, y: 20};
    printf("Point: %d, %d\n", p.x, p.y);
    return 0;
}
"#;

const NESTED_STRUCTS: &str = r#"
struct Point {
    x: int,
    y: int
}

struct Line {
    start: Point,
    end: Point
}

fn main(): int {
    let p = Point {x: 10, y: 20};
    let line = Line {
        start: Point {x: 0, y: 0},
        end: Point {x: 100, y: 200}
    };
    printf("Basic field access: %d, %d\n", p.x, p.y);
    printf("Struct init field access: %d\n", Point {x: 42, y: 84}.y);
    printf("Nested field access: %d, %d\n", line.start.x, line.end.y);
    return 0;
}
"#;

const CONTROL_FLOW: &str = r#"
fn main(): int {
    let x = 10;
    let y = 20;

    if x < y {
        printf("x is less than y\n");
    } else {
        printf("x is greater than or equal to y\n");
    }

    let i = 0;
    while i < 10 {
        printf("Loop iteration: %d\n", i);
        i = i + 1;
    }

    return 0;
}
"#;

const ARRAYS: &str = r#"
fn main(): int {
    let arr = [1, 2, 3, 4, 5];
    let first = arr[0];
    let last = arr[4];

    printf("First: %d, Last: %d\n", first, last);

    let i = 0;
    while i < 5 {
        printf("arr[%d] = %d\n", i, arr[i]);
        i = i + 1;
    }

    return 0;
}
"#;

// Large code sample - complex program
const LARGE_PROGRAM: &str = r#"
struct Point {
    x: int,
    y: int
}

struct Rectangle {
    topLeft: Point,
    bottomRight: Point
}

fn createPoint(x: int, y: int): Point {
    return Point { x: x, y: y };
}

fn createRectangle(x1: int, y1: int, x2: int, y2: int): Rectangle {
    let p1 = createPoint(x1, y1);
    let p2 = createPoint(x2, y2);
    return Rectangle { topLeft: p1, bottomRight: p2 };
}

fn area(r: Rectangle): int {
    let width = r.bottomRight.x - r.topLeft.x;
    let height = r.bottomRight.y - r.topLeft.y;
    return width * height;
}

fn translate(p: Point, dx: int, dy: int): Point {
    let newX = p.x + dx;
    let newY = p.y + dy;
    return Point { x: newX, y: newY };
}

fn main(): int {
    let p1 = createPoint(0, 0);
    let p2 = createPoint(10, 10);
    let p3 = createPoint(20, 20);

    let r1 = createRectangle(0, 0, 10, 10);
    let r2 = createRectangle(5, 5, 15, 15);
    let r3 = createRectangle(10, 10, 20, 20);

    let a1 = area(r1);
    let a2 = area(r2);
    let a3 = area(r3);

    printf("Areas: %d, %d, %d\n", a1, a2, a3);

    let i = 0;
    while i < 10 {
        let pt = translate(p1, i, i);
        printf("Point at iteration %d: (%d, %d)\n", i, pt.x, pt.y);
        i = i + 1;
    }

    let j = 0;
    while j < 5 {
        let k = 0;
        while k < 5 {
            let rect = createRectangle(j, k, j + 10, k + 10);
            let a = area(rect);
            printf("Rectangle [%d][%d] area: %d\n", j, k, a);
            k = k + 1;
        }
        j = j + 1;
    }

    return 0;
}
"#;

const DEEPLY_NESTED: &str = r#"
fn main(): int {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        printf("Deep nesting\n");
                    }
                }
            }
        }
    }
    return 0;
}
"#;

const MANY_PARAMETERS: &str = r#"
fn many_params(a: int, b: int, c: int, d: int, e: int, f: int, g: int, h: int, i: int, j: int): int {
    return a + b + c + d + e + f + g + h + i + j;
}

fn main(): int {
    let result = many_params(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    return result;
}
"#;

const COMPLEX_EXPRESSIONS: &str = r#"
fn main(): int {
    let a = 1 + 2 * 3 - 4 / 2;
    let b = (a + 5) * (a - 3);
    let c = a > b && b < 100 || a == 0;
    let d = !c && (a != b);
    return a + b;
}
"#;

const CRAZY_NESTED_BLOCKS: &str = r#"
fn main(): int {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        if true {
                                            let x = 42;
                                            printf("Deep in the nesting\n");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return 0;
}
"#;

const CRAZY_NESTED_IFS: &str = r#"
fn main(): int {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        if true {
                                            printf("10 levels deep\n");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return 0;
}
"#;

const CRAZY_NESTED_WHILE: &str = r#"
fn main(): int {
    let a = 0;
    while a < 1 {
        let b = 0;
        while b < 1 {
            let c = 0;
            while c < 1 {
                let d = 0;
                while d < 1 {
                    let e = 0;
                    while e < 1 {
                        printf("Nested while loops\n");
                        e = e + 1;
                    }
                    d = d + 1;
                }
                c = c + 1;
            }
            b = b + 1;
        }
        a = a + 1;
    }
    return 0;
}
"#;

const CRAZY_NESTED_STRUCTS: &str = r#"
struct Level1 {
    value: int
}

struct Level2 {
    l1: Level1
}

struct Level3 {
    l2: Level2
}

struct Level4 {
    l3: Level3
}

struct Level5 {
    l4: Level4
}

struct Level6 {
    l5: Level5
}

fn main(): int {
    let deep = Level6 {
        l5: Level5 {
            l4: Level4 {
                l3: Level3 {
                    l2: Level2 {
                        l1: Level1 {
                            value: 42
                        }
                    }
                }
            }
        }
    };

    let val = deep.l5.l4.l3.l2.l1.value;
    printf("Deep value: %d\n", val);
    return 0;
}
"#;

const CRAZY_NESTED_FUNCTION_CALLS: &str = r#"
fn f1(x: int): int {
    return x + 1;
}

fn f2(x: int): int {
    return f1(x) + 1;
}

fn f3(x: int): int {
    return f2(x) + 1;
}

fn f4(x: int): int {
    return f3(x) + 1;
}

fn f5(x: int): int {
    return f4(x) + 1;
}

fn f6(x: int): int {
    return f5(x) + 1;
}

fn f7(x: int): int {
    return f6(x) + 1;
}

fn f8(x: int): int {
    return f7(x) + 1;
}

fn main(): int {
    let result = f8(f7(f6(f5(f4(f3(f2(f1(0))))))));
    return result;
}
"#;

const CRAZY_NESTED_EXPRESSIONS: &str = r#"
fn main(): int {
    let x = ((((((((((1 + 2) * 3) - 4) / 2) + 5) * 6) - 7) / 2) + 8) * 9);
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let y = (((((a > b) && (c < d)) || (e == f)) && (g != h)) || ((i >= j) && (k <= l)));
    let z = !!!!!true;
    return x;
}
"#;

const CRAZY_MIXED_NESTING: &str = r#"
struct Container {
    value: int
}

fn process(c: Container): int {
    if c.value > 0 {
        let i = 0;
        while i < c.value {
            if i > 0 {
                if i < 10 {
                    if i > 5 {
                        let temp = Container { value: i };
                        if temp.value > 0 {
                            printf("Value: %d\n", temp.value);
                        }
                    }
                }
            }
            i = i + 1;
        }
    }
    return c.value;
}

fn main(): int {
    let outer = 0;
    while outer < 3 {
        if outer > 0 {
            let inner = 0;
            while inner < 2 {
                if inner >= 0 {
                    let c = Container { value: outer + inner };
                    let result = process(c);
                    if result > 0 {
                        printf("Result: %d\n", result);
                    }
                }
                inner = inner + 1;
            }
        }
        outer = outer + 1;
    }
    return 0;
}
"#;

fn lex_and_parse(source: &str) -> Program {
    let mut lexer = Lexer::new(source.as_bytes(), "bench.hd".to_string());
    lexer.tokenize().unwrap();
    let tokens = lexer.into_tokens();
    let mut parser = Parser::new(tokens, "bench.hd".to_string());
    match parser.parse() {
        Ok(program) => program,
        Err(_) => panic!("Parse should succeed in benchmark"),
    }
}

fn bench_parser_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_simple");

    group.bench_function("simple_expr", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(SIMPLE_EXPR)));
        });
    });

    group.bench_function("function_def", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(FUNCTION_DEF)));
        });
    });

    group.bench_function("hello_world", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(HELLO_WORLD)));
        });
    });

    group.finish();
}

fn bench_parser_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_medium");

    group.bench_function("struct_with_methods", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(STRUCT_WITH_METHODS)));
        });
    });

    group.bench_function("nested_structs", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(NESTED_STRUCTS)));
        });
    });

    group.bench_function("control_flow", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CONTROL_FLOW)));
        });
    });

    group.bench_function("arrays", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(ARRAYS)));
        });
    });

    group.finish();
}

fn bench_parser_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_large");

    group.bench_function("large_program", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(LARGE_PROGRAM)));
        });
    });

    group.finish();
}

fn bench_parser_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_scaling");

    // Test how the parser scales with input size
    for size in [10, 50, 100, 500, 1000].iter() {
        let code = format!(
            "fn main(): int {{\n{}\n    return 0;\n}}",
            (0..*size)
                .map(|i| format!("    let x{} = {} + {};", i, i, i + 1))
                .collect::<Vec<_>>()
                .join("\n")
        );

        group.bench_with_input(BenchmarkId::from_parameter(size), &code, |b, code| {
            b.iter(|| {
                black_box(lex_and_parse(black_box(code)));
            });
        });
    }

    group.finish();
}

fn bench_parser_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_patterns");

    group.bench_function("deeply_nested", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(DEEPLY_NESTED)));
        });
    });

    group.bench_function("many_parameters", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(MANY_PARAMETERS)));
        });
    });

    group.bench_function("complex_expressions", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(COMPLEX_EXPRESSIONS)));
        });
    });

    group.finish();
}

fn bench_parser_crazy_nesting(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_crazy_nesting");

    group.bench_function("crazy_nested_blocks", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_BLOCKS)));
        });
    });

    group.bench_function("crazy_nested_ifs", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_IFS)));
        });
    });

    group.bench_function("crazy_nested_while", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_WHILE)));
        });
    });

    group.bench_function("crazy_nested_structs", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_STRUCTS)));
        });
    });

    group.bench_function("crazy_nested_function_calls", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_FUNCTION_CALLS)));
        });
    });

    group.bench_function("crazy_nested_expressions", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_NESTED_EXPRESSIONS)));
        });
    });

    group.bench_function("crazy_mixed_nesting", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(CRAZY_MIXED_NESTING)));
        });
    });

    group.finish();
}

fn bench_parser_struct_definitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_struct_definitions");

    for count in [5, 10, 20, 50].iter() {
        let code = (0..*count)
            .map(|i| {
                format!(
                    "struct Struct{} {{\n    field1: int,\n    field2: float,\n    field3: bool\n}}",
                    i
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
            + "\n\nfn main(): int { return 0; }";

        group.bench_with_input(BenchmarkId::from_parameter(count), &code, |b, code| {
            b.iter(|| {
                black_box(lex_and_parse(black_box(code)));
            });
        });
    }

    group.finish();
}

fn bench_parser_function_definitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_function_definitions");

    for count in [5, 10, 20, 50].iter() {
        let code = (0..*count)
            .map(|i| {
                format!(
                    "fn func{}(x: int, y: int): int {{\n    return x + y + {};\n}}",
                    i, i
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
            + "\n\nfn main(): int { return 0; }";

        group.bench_with_input(BenchmarkId::from_parameter(count), &code, |b, code| {
            b.iter(|| {
                black_box(lex_and_parse(black_box(code)));
            });
        });
    }

    group.finish();
}

fn bench_parser_expression_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_expression_complexity");

    // Binary expression chains
    for depth in [5, 10, 20, 50].iter() {
        let expr = (0..*depth)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(" + ");
        let code = format!("fn main(): int {{ let result = {}; return result; }}", expr);

        group.bench_with_input(BenchmarkId::new("binary_chain", depth), &code, |b, code| {
            b.iter(|| {
                black_box(lex_and_parse(black_box(code)));
            });
        });
    }

    // Nested parentheses
    for depth in [5, 10, 15, 20].iter() {
        let open = "(".repeat(*depth);
        let close = ")".repeat(*depth);
        let code = format!(
            "fn main(): int {{ let result = {}x{}; return result; }}",
            open, close
        );

        group.bench_with_input(
            BenchmarkId::new("nested_parens", depth),
            &code,
            |b, code| {
                b.iter(|| {
                    black_box(lex_and_parse(black_box(code)));
                });
            },
        );
    }

    group.finish();
}

fn bench_lexer_plus_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_plus_parser");

    group.bench_function("small_program", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(HELLO_WORLD)));
        });
    });

    group.bench_function("medium_program", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(NESTED_STRUCTS)));
        });
    });

    group.bench_function("large_program", |b| {
        b.iter(|| {
            black_box(lex_and_parse(black_box(LARGE_PROGRAM)));
        });
    });

    group.finish();
}

fn bench_parser_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_only");

    // Pre-lex the code samples to benchmark parser only
    let mut lexer = Lexer::new(HELLO_WORLD.as_bytes(), "bench.hd".to_string());
    lexer.tokenize().unwrap();
    let hello_tokens = lexer.into_tokens();

    let mut lexer = Lexer::new(NESTED_STRUCTS.as_bytes(), "bench.hd".to_string());
    lexer.tokenize().unwrap();
    let nested_tokens = lexer.into_tokens();

    let mut lexer = Lexer::new(LARGE_PROGRAM.as_bytes(), "bench.hd".to_string());
    lexer.tokenize().unwrap();
    let large_tokens = lexer.into_tokens();

    group.bench_function("small_program_tokens", |b| {
        b.iter(|| {
            let tokens = black_box(hello_tokens.clone());
            let mut parser = Parser::new(tokens, "bench.hd".to_string());
            let result = match parser.parse() {
                Ok(program) => program,
                Err(_) => panic!("Parse should succeed in benchmark"),
            };
            black_box(result);
        });
    });

    group.bench_function("medium_program_tokens", |b| {
        b.iter(|| {
            let tokens = black_box(nested_tokens.clone());
            let mut parser = Parser::new(tokens, "bench.hd".to_string());
            let result = match parser.parse() {
                Ok(program) => program,
                Err(_) => panic!("Parse should succeed in benchmark"),
            };
            black_box(result);
        });
    });

    group.bench_function("large_program_tokens", |b| {
        b.iter(|| {
            let tokens = black_box(large_tokens.clone());
            let mut parser = Parser::new(tokens, "bench.hd".to_string());
            let result = match parser.parse() {
                Ok(program) => program,
                Err(_) => panic!("Parse should succeed in benchmark"),
            };
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_simple,
    bench_parser_medium,
    bench_parser_large,
    bench_parser_scaling,
    bench_parser_patterns,
    bench_parser_struct_definitions,
    bench_parser_function_definitions,
    bench_parser_expression_complexity,
    bench_parser_crazy_nesting,
    bench_lexer_plus_parser,
    bench_parser_only
);
criterion_main!(benches);
