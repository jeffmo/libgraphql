//! Deterministic, virtualized-CPU benchmarks for libgraphql-parser.
//!
//! These benchmarks run under Valgrind's Callgrind (via
//! `iai-callgrind`), which executes the benchmark on a simulated CPU
//! and counts the exact number of instructions executed along with
//! simulated cache hits/misses. Because the measurements come from a
//! virtualized CPU rather than wall-clock time, they are fully
//! deterministic: repeated runs of the same binary produce identical
//! counts regardless of machine load, CPU frequency scaling, or
//! shared-runner noise. This makes them suitable for CI environments
//! where wall-clock benchmarks (see `parse_benchmarks.rs`) would be
//! hopelessly noisy.
//!
//! The trade-off is that instruction counts and "estimated cycles"
//! (derived from the cache simulation) are a proxy for real-world
//! latency, not a replacement: they do not model superscalar
//! execution, branch prediction, or memory-level parallelism. Use
//! these benchmarks for regression tracking and cross-parser
//! comparison trends; use the Criterion benchmarks on quiet bare
//! metal for headline wall-clock numbers.
//!
//! Valgrind only supports Linux (notably: not macOS on Apple
//! Silicon), so this benchmark target compiles to a stub that prints
//! an explanation on other platforms.
//!
//! Usage (requires `valgrind` and a matching-version
//! `iai-callgrind-runner` on `$PATH`):
//!
//! ```sh
//! cargo install iai-callgrind-runner --version 0.16.1
//! cargo bench -p libgraphql-parser --bench callgrind_benchmarks
//! ```
//!
//! Or use the wrapper script which also formats results as markdown:
//!
//! ```sh
//! ./crates/libgraphql-parser/scripts/run-callgrind-benchmarks.sh
//! ```

#[cfg(target_os = "linux")]
mod fixtures;

#[cfg(target_os = "linux")]
mod callgrind_benches {
    use crate::fixtures;
    use iai_callgrind::Callgrind;
    use iai_callgrind::LibraryBenchmarkConfig;
    use iai_callgrind::library_benchmark;
    use iai_callgrind::library_benchmark_group;
    use iai_callgrind::main;
    use libgraphql_parser::GraphQLParser;
    use libgraphql_parser::GraphQLParserConfig;
    use libgraphql_parser::token::StrGraphQLTokenSource;
    use std::hint::black_box;

    /// Resolves a schema-document fixture name to its GraphQL source
    /// text. Runs as an iai-callgrind `setup` function, so none of
    /// the work done here (file I/O included) is attributed to the
    /// measured benchmark.
    fn schema_source(name: &str) -> String {
        match name {
            "small" => fixtures::SMALL_SCHEMA.to_string(),
            "medium" => fixtures::MEDIUM_SCHEMA.to_string(),
            "large" => fixtures::LARGE_SCHEMA.to_string(),
            "starwars" => fixtures::STARWARS_SCHEMA.to_string(),
            "github" => fixtures::GITHUB_SCHEMA.to_string(),
            "shopify_admin" => fixtures::load_shopify_admin_schema(),
            _ => panic!("Unknown schema fixture: {name}"),
        }
    }

    /// Resolves an executable-document fixture name to its GraphQL
    /// source text. Runs as an iai-callgrind `setup` function (see
    /// `schema_source()`).
    fn executable_source(name: &str) -> String {
        match name {
            "simple" => fixtures::SIMPLE_QUERY.to_string(),
            "complex" => fixtures::COMPLEX_QUERY.to_string(),
            "nested_10" => fixtures::operations::deeply_nested_query(10),
            "nested_30" => fixtures::operations::deeply_nested_query(30),
            "many_ops_50" => fixtures::operations::many_operations(50),
            _ => panic!("Unknown executable fixture: {name}"),
        }
    }

    // ─── Schema Document Parsing ─────────────────────────────

    #[library_benchmark(setup = schema_source)]
    #[bench::small("small")]
    #[bench::medium("medium")]
    #[bench::large("large")]
    #[bench::starwars("starwars")]
    #[bench::github("github")]
    #[bench::shopify_admin("shopify_admin")]
    fn schema_libgraphql(schema: String) {
        let parser = GraphQLParser::new(&schema);
        black_box(parser.parse_schema_document());
    }

    #[library_benchmark(setup = schema_source)]
    #[bench::small("small")]
    #[bench::medium("medium")]
    #[bench::large("large")]
    #[bench::starwars("starwars")]
    #[bench::github("github")]
    #[bench::shopify_admin("shopify_admin")]
    fn schema_libgraphql_lean(schema: String) {
        let parser = GraphQLParser::with_config(&schema, GraphQLParserConfig::lean());
        black_box(parser.parse_schema_document());
    }

    #[library_benchmark(setup = schema_source)]
    #[bench::small("small")]
    #[bench::medium("medium")]
    #[bench::large("large")]
    #[bench::starwars("starwars")]
    #[bench::github("github")]
    #[bench::shopify_admin("shopify_admin")]
    fn schema_graphql_parser(schema: String) {
        let _ = black_box(graphql_parser::schema::parse_schema::<String>(&schema));
    }

    #[library_benchmark(setup = schema_source)]
    #[bench::small("small")]
    #[bench::medium("medium")]
    #[bench::large("large")]
    #[bench::starwars("starwars")]
    #[bench::github("github")]
    #[bench::shopify_admin("shopify_admin")]
    fn schema_apollo_parser(schema: String) {
        let parser = apollo_parser::Parser::new(&schema);
        black_box(parser.parse());
    }

    // ─── Executable Document Parsing ─────────────────────────

    #[library_benchmark(setup = executable_source)]
    #[bench::simple("simple")]
    #[bench::complex("complex")]
    #[bench::nested_10("nested_10")]
    #[bench::nested_30("nested_30")]
    #[bench::many_ops_50("many_ops_50")]
    fn executable_libgraphql(query: String) {
        let parser = GraphQLParser::new(&query);
        black_box(parser.parse_executable_document());
    }

    #[library_benchmark(setup = executable_source)]
    #[bench::simple("simple")]
    #[bench::complex("complex")]
    #[bench::nested_10("nested_10")]
    #[bench::nested_30("nested_30")]
    #[bench::many_ops_50("many_ops_50")]
    fn executable_libgraphql_lean(query: String) {
        let parser = GraphQLParser::with_config(&query, GraphQLParserConfig::lean());
        black_box(parser.parse_executable_document());
    }

    #[library_benchmark(setup = executable_source)]
    #[bench::simple("simple")]
    #[bench::complex("complex")]
    #[bench::nested_10("nested_10")]
    #[bench::nested_30("nested_30")]
    #[bench::many_ops_50("many_ops_50")]
    fn executable_graphql_parser(query: String) {
        let _ = black_box(graphql_parser::query::parse_query::<String>(&query));
    }

    #[library_benchmark(setup = executable_source)]
    #[bench::simple("simple")]
    #[bench::complex("complex")]
    #[bench::nested_10("nested_10")]
    #[bench::nested_30("nested_30")]
    #[bench::many_ops_50("many_ops_50")]
    fn executable_apollo_parser(query: String) {
        let parser = apollo_parser::Parser::new(&query);
        black_box(parser.parse());
    }

    // ─── Lexer (Tokenization Only) ───────────────────────────

    #[library_benchmark(setup = schema_source)]
    #[bench::small("small")]
    #[bench::medium("medium")]
    #[bench::large("large")]
    #[bench::starwars("starwars")]
    #[bench::github("github")]
    #[bench::shopify_admin("shopify_admin")]
    fn lexer_libgraphql(schema: String) {
        let source = StrGraphQLTokenSource::new(&schema);
        for token in source {
            black_box(token);
        }
    }

    // ─── Groups & Entrypoint ─────────────────────────────────

    library_benchmark_group!(
        name = schema_parse;
        benchmarks =
            schema_libgraphql,
            schema_libgraphql_lean,
            schema_graphql_parser,
            schema_apollo_parser,
    );

    library_benchmark_group!(
        name = executable_parse;
        benchmarks =
            executable_libgraphql,
            executable_libgraphql_lean,
            executable_graphql_parser,
            executable_apollo_parser,
    );

    library_benchmark_group!(
        name = lexer;
        benchmarks = lexer_libgraphql,
    );

    main!(
        // `--cache-sim=yes` enables Callgrind's cache simulation,
        // which adds L1/LL hit/miss counts and a derived "estimated
        // cycles" metric on top of raw instruction counts. This is
        // explicit (rather than relying on defaults) so results stay
        // comparable across iai-callgrind versions.
        config = LibraryBenchmarkConfig::default()
            .tool(Callgrind::with_args(["--cache-sim=yes"]));
        library_benchmark_groups = schema_parse, executable_parse, lexer
    );

    pub fn run() {
        main();
    }
}

#[cfg(target_os = "linux")]
fn main() {
    callgrind_benches::run();
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!(
        "The callgrind_benchmarks bench target requires Valgrind, which only \
         supports Linux. Use the Criterion wall-clock benchmarks instead:\n\n  \
         cargo bench -p libgraphql-parser --bench parse_benchmarks\n",
    );
}
