mod fixtures;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use libgraphql_parser::GraphQLParser;
use libgraphql_parser::GraphQLParserConfig;
use libgraphql_parser::token::StrGraphQLTokenSource;

// ─── Group 1: Schema Parsing ─────────────────────────────

fn schema_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_parse");
    let shopify_admin =
        fixtures::load_shopify_admin_schema();

    group.bench_function("small (synthetic)", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::SMALL_SCHEMA);
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("medium (synthetic)", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::MEDIUM_SCHEMA);
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("large (synthetic)", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::LARGE_SCHEMA);
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("starwars", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::STARWARS_SCHEMA);
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("github", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::GITHUB_SCHEMA);
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("shopify_admin", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(&shopify_admin);
            black_box(parser.parse_schema_document())
        })
    });

    group.finish();
}

// ─── Group 2: Executable Document Parsing ─────────────────

fn executable_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("executable_parse");

    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::SIMPLE_QUERY);
            black_box(parser.parse_executable_document())
        })
    });

    group.bench_function("complex_query", |b| {
        b.iter(|| {
            let parser =
                GraphQLParser::new(fixtures::COMPLEX_QUERY);
            black_box(parser.parse_executable_document())
        })
    });

    let nested_10 =
        fixtures::operations::deeply_nested_query(10);
    group.bench_function("nested_depth_10", |b| {
        b.iter(|| {
            let parser = GraphQLParser::new(&nested_10);
            black_box(parser.parse_executable_document())
        })
    });

    let nested_30 =
        fixtures::operations::deeply_nested_query(30);
    group.bench_function("nested_depth_30", |b| {
        b.iter(|| {
            let parser = GraphQLParser::new(&nested_30);
            black_box(parser.parse_executable_document())
        })
    });

    let many_ops =
        fixtures::operations::many_operations(50);
    group.bench_function("many_operations_50", |b| {
        b.iter(|| {
            let parser = GraphQLParser::new(&many_ops);
            black_box(parser.parse_executable_document())
        })
    });

    group.finish();
}

// ─── Group 3: Schema Parsing (Lean Mode) ─────────────────

fn schema_parse_lean(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_parse_lean");
    let shopify_admin =
        fixtures::load_shopify_admin_schema();
    let lean = GraphQLParserConfig::lean();

    group.bench_function("small (synthetic)", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::SMALL_SCHEMA,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("medium (synthetic)", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::MEDIUM_SCHEMA,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("large (synthetic)", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::LARGE_SCHEMA,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("starwars", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::STARWARS_SCHEMA,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("github", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::GITHUB_SCHEMA,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.bench_function("shopify_admin", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                &shopify_admin,
                lean.clone(),
            );
            black_box(parser.parse_schema_document())
        })
    });

    group.finish();
}

// ─── Group 4: Executable Document Parsing (Lean Mode) ────

fn executable_parse_lean(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("executable_parse_lean");
    let lean = GraphQLParserConfig::lean();

    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::SIMPLE_QUERY,
                lean.clone(),
            );
            black_box(parser.parse_executable_document())
        })
    });

    group.bench_function("complex_query", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                fixtures::COMPLEX_QUERY,
                lean.clone(),
            );
            black_box(parser.parse_executable_document())
        })
    });

    let nested_10 =
        fixtures::operations::deeply_nested_query(10);
    group.bench_function("nested_depth_10", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                &nested_10,
                lean.clone(),
            );
            black_box(parser.parse_executable_document())
        })
    });

    let nested_30 =
        fixtures::operations::deeply_nested_query(30);
    group.bench_function("nested_depth_30", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                &nested_30,
                lean.clone(),
            );
            black_box(parser.parse_executable_document())
        })
    });

    let many_ops =
        fixtures::operations::many_operations(50);
    group.bench_function("many_operations_50", |b| {
        b.iter(|| {
            let parser = GraphQLParser::with_config(
                &many_ops,
                lean.clone(),
            );
            black_box(parser.parse_executable_document())
        })
    });

    group.finish();
}

// ─── Group 5: Lexer (Tokenization Only) ──────────────────

fn lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    let shopify_admin =
        fixtures::load_shopify_admin_schema();

    group.throughput(Throughput::Bytes(
        fixtures::SMALL_SCHEMA.len() as u64,
    ));
    group.bench_function("small_schema (synthetic)", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                fixtures::SMALL_SCHEMA,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.throughput(Throughput::Bytes(
        fixtures::MEDIUM_SCHEMA.len() as u64,
    ));
    group.bench_function("medium_schema (synthetic)", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                fixtures::MEDIUM_SCHEMA,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.throughput(Throughput::Bytes(
        fixtures::LARGE_SCHEMA.len() as u64,
    ));
    group.bench_function("large_schema (synthetic)", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                fixtures::LARGE_SCHEMA,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.throughput(Throughput::Bytes(
        fixtures::STARWARS_SCHEMA.len() as u64,
    ));
    group.bench_function("starwars_schema", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                fixtures::STARWARS_SCHEMA,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.throughput(Throughput::Bytes(
        fixtures::GITHUB_SCHEMA.len() as u64,
    ));
    group.bench_function("github_schema", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                fixtures::GITHUB_SCHEMA,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.throughput(Throughput::Bytes(
        shopify_admin.len() as u64,
    ));
    group.bench_function("shopify_admin_schema", |b| {
        b.iter(|| {
            let source = StrGraphQLTokenSource::new(
                &shopify_admin,
            );
            for token in source {
                black_box(token);
            }
        })
    });

    group.finish();
}

// ─── Group 6: Cross-Parser Comparisons ───────────────────

fn compare_schema_parse(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("compare_schema_parse");
    let shopify_admin =
        fixtures::load_shopify_admin_schema();

    let inputs: Vec<(&str, &str)> = vec![
        ("small", fixtures::SMALL_SCHEMA),
        ("medium", fixtures::MEDIUM_SCHEMA),
        ("large", fixtures::LARGE_SCHEMA),
        ("starwars", fixtures::STARWARS_SCHEMA),
        ("github", fixtures::GITHUB_SCHEMA),
        ("shopify_admin", &shopify_admin),
    ];

    for (label, input) in &inputs {
        group.bench_with_input(
            BenchmarkId::new("libgraphql_parser", label),
            input,
            |b, input| {
                b.iter(|| {
                    let parser = GraphQLParser::new(input);
                    black_box(
                        parser.parse_schema_document(),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("graphql_parser", label),
            input,
            |b, input| {
                b.iter(|| {
                    black_box(
                        graphql_parser::schema
                            ::parse_schema::<String>(
                                input,
                            ),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("apollo_parser", label),
            input,
            |b, input| {
                b.iter(|| {
                    let parser =
                        apollo_parser::Parser::new(input);
                    black_box(parser.parse())
                })
            },
        );
    }

    group.finish();
}

fn compare_executable_parse(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("compare_executable_parse");

    let inputs: &[(&str, &str)] = &[
        ("simple", fixtures::SIMPLE_QUERY),
        ("complex", fixtures::COMPLEX_QUERY),
    ];

    for &(label, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("libgraphql_parser", label),
            &input,
            |b, input| {
                b.iter(|| {
                    let parser = GraphQLParser::new(input);
                    black_box(
                        parser.parse_executable_document(),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("graphql_parser", label),
            &input,
            |b, input| {
                b.iter(|| {
                    black_box(
                        graphql_parser::query
                            ::parse_query::<String>(input),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("apollo_parser", label),
            &input,
            |b, input| {
                b.iter(|| {
                    let parser =
                        apollo_parser::Parser::new(input);
                    black_box(parser.parse())
                })
            },
        );
    }

    group.finish();
}

// ─── Criterion Entrypoint ────────────────────────────────

criterion_group!(
    benches,
    schema_parse,
    executable_parse,
    schema_parse_lean,
    executable_parse_lean,
    lexer,
    compare_schema_parse,
    compare_executable_parse,
);
criterion_main!(benches);
