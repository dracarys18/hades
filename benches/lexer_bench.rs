use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hades::lexer::Lexer;

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

fn bench_lexer_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_simple");

    group.bench_function("simple_expr", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(SIMPLE_EXPR), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.bench_function("function_def", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(FUNCTION_DEF), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.bench_function("hello_world", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(HELLO_WORLD), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.finish();
}

fn bench_lexer_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_medium");

    group.bench_function("struct_with_methods", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(STRUCT_WITH_METHODS), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.bench_function("nested_structs", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(NESTED_STRUCTS), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.bench_function("control_flow", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(CONTROL_FLOW), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.bench_function("arrays", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(ARRAYS), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.finish();
}

fn bench_lexer_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_large");

    group.bench_function("large_program", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(LARGE_PROGRAM), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.finish();
}

fn bench_lexer_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_scaling");

    // Test how the lexer scales with input size
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
                let mut lexer = Lexer::new(black_box(code), "bench.hd".to_string());
                lexer.tokenize().unwrap();
                black_box(lexer.get_tokens());
            });
        });
    }

    group.finish();
}

fn bench_lexer_token_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_token_types");

    // Numbers
    group.bench_function("many_numbers", |b| {
        let code = (0..100)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&code), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    // Identifiers
    group.bench_function("many_identifiers", |b| {
        let code = (0..100)
            .map(|i| format!("identifier{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&code), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    // Keywords
    group.bench_function("many_keywords", |b| {
        let code = "let fn if else while for struct return break continue true false ".repeat(20);
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&code), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    // Strings
    group.bench_function("many_strings", |b| {
        let code = (0..100)
            .map(|i| format!(r#""string {}""#, i))
            .collect::<Vec<_>>()
            .join(" ");
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&code), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    // Operators
    group.bench_function("many_operators", |b| {
        let code = "+ - * / == != < > <= >= && || ".repeat(20);
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&code), "bench.hd".to_string());
            lexer.tokenize().unwrap();
            black_box(lexer.get_tokens());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_simple,
    bench_lexer_medium,
    bench_lexer_large,
    bench_lexer_scaling,
    bench_lexer_token_types
);
criterion_main!(benches);
