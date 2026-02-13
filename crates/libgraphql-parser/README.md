<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-banner-dark.svg" />
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-banner-light.svg" />
    <img src="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-banner-light.svg" alt="libgraphql-parser — Blazing fast, error-resilient GraphQL parser" width="100%" />
  </picture>
</p>

<p align="center">
  <a href="https://crates.io/crates/libgraphql-parser"><img src="https://img.shields.io/crates/v/libgraphql-parser.svg?style=flat-square" alt="crates.io" /></a>
  <a href="https://docs.rs/libgraphql-parser/"><img src="https://img.shields.io/docsrs/libgraphql-parser?style=flat-square" alt="docs.rs" /></a>
  <a href="https://github.com/jeffmo/libgraphql/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/libgraphql-parser.svg?style=flat-square" alt="license" /></a>
  <img src="https://img.shields.io/badge/GraphQL_Spec-September_2025-e535ab?style=flat-square" alt="GraphQL Spec" />
</p>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-code-dark.svg" />
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-code-light.svg" />
  <img src="https://raw.githubusercontent.com/jeffmo/libgraphql/refs/heads/main/crates/libgraphql-parser/assets/readme-code-light.svg" alt="Two-column code preview showing GraphQL schema and query parsing with syntax-highlighted error diagnostics" width="100%" />
</picture>

<br />
<br />

> [!WARNING]
> `libgraphql-parser` is still under active development.
>
> All items listed in the "Features" section below are complete and heavily
> unit-tested, but `libgraphql-parser` still outputs the `graphql_parser` AST
> structure. This AST format is battle-tested, but it is not up to date with the
> `Sep 2025` spec and it discards a large amount of the information that
> `libgraphql_parser::GraphQLParser` collects while parsing.
> 
> Expect all `0.0.x` versions of `libgraphql-parser` to contain breaking
> changes.

## Features

- **Error-resilient parsing** — documents produce a partial AST alongside a list
  of errors, even when the input is malformed. Never panics.
- **Rust-inspired error output** — error messages with source snippets, span 
  highlighting, contextual notes, and fix suggestions.
- **Blazing fast** - [Perf metrics](#performance) exceeding
  [`apollo_parser`](https://crates.io/crates/apollo-parser) (v0.8.4) on all
  fixtures and [`graphql_parser`](https://crates.io/crates/graphql-parser) 
  (v0.4.1) on all but one fixture.
- **Zero-copy lexing** — uses `Cow<'src, str>` to avoid allocations for
  tokens that match the source verbatim.
- **Parse schema, executable, and mixed documents** — parses type definitions,
  operations/fragments, or documents containing both interleaved together.
- **[September 2025](https://spec.graphql.org/September2025/) GraphQL
  specification** compliance.
- **Dual column tracking** — reports both UTF-8 character positions (for
  display) and UTF-16 code unit positions (for LSP integration).
- **Comment/trivia preservation** — captures comments and other trivia as
  "preceding trivia" attached to tokens.
- **Generic over token sources** — the parser works with any
  `GraphQLTokenSource` (string input, Rust proc-macro token stream input, etc.).
- **Configurable AST access** — `valid_ast()` API for strict consumers that
  require error-free input, `ast()` for best-effort tooling that needs
  error-recovery (IDEs, linters, formatters, etc).
- **Fuzz-tested at scale** — [70M+ `libfuzzer` executions](#fuzz-testing) across
  4 fuzz targets, zero crashes.

_Coming soon:_

- **100% lossless syntax tree** - Lossless AST structure enabling full-fidelity,
  slice-based reproduction of the original source text.
- **Drop-in compat with `apollo-parser` and `graphql-parser` AST structures** - 
  [feature-flagged] translation utils to make it easy to integrate with tools that 
  already depend on the
  [`apollo_parser::cst`](https://docs.rs/apollo-parser/0.8.4/apollo_parser/cst/index.html)
  ,
  [`graphql_parser::query`](https://docs.rs/graphql-parser/0.4.1/graphql_parser/query/index.html)
  , and 
  [`graphql_parser::schema`](https://docs.rs/graphql-parser/0.4.1/graphql_parser/schema/index.html)
  AST structures.
- 

## Getting Started

```bash
cargo add libgraphql-parser
```

Or add this to your `Cargo.toml`:
```toml
[dependencies]
libgraphql-parser = "0.0.1"
```

## Usage

Parse a GraphQL schema document:

```rust
use libgraphql_parser::GraphQLParser;

let result = GraphQLParser::new("type Query { hello: String }")
    .parse_schema_document();

assert!(!result.has_errors());
let doc = result.valid_ast().unwrap();
```

Parse an executable document (queries, mutations, subscriptions):

```rust
use libgraphql_parser::GraphQLParser;

let result = GraphQLParser::new("{ user { name email } }")
    .parse_executable_document();

assert!(!result.has_errors());
```

### Error Recovery

Unlike many GraphQL parsers that stop at the first error,
`libgraphql-parser` collects multiple errors and still produces a
best-effort AST. This is essential for IDE integration, linters, and
formatters:

```rust
use libgraphql_parser::GraphQLParser;

// This schema has errors: missing `:` on field type, unclosed brace
let source = "type Query { hello String";
let result = GraphQLParser::new(source).parse_schema_document();

// Errors are collected — inspect them all at once
assert!(result.has_errors());
for error in &result.errors {
    eprintln!("{}", error.format_detailed(Some(source)));
}

// A partial AST is still available for best-effort tooling
if let Some(doc) = result.ast() {
    // IDE completions, formatting, linting can still work
    // on the partially-parsed document
    println!("Parsed {} definitions", doc.definitions.len());
}
```

### Diagnostic Output

Error messages include source spans, contextual notes, and fix
suggestions — inspired by the Rust compiler's diagnostic style:

```text
error: unknown directive location `FIELD_DEFINTION`
  --> <input>:1:42
   |
 1 | directive @deprecated(reason: String) on FIELD_DEFINTION | ENUM_VALUE
   |                                          ^^^^^^^^^^^^^^^
   = help: did you mean `FIELD_DEFINITION`?
```

Unclosed delimiters point back to the opening location:

```text
error: unclosed `{`
  --> <input>:9:2
   |
 9 | }
   |  ^
   = note: opening `{` in selection set here
      1 | query {
        |       -
```

### Strict vs. Best-Effort AST Access

`ParseResult` offers two modes for accessing the AST:

```rust
use libgraphql_parser::GraphQLParser;

let source = "type Query { hello: String }";
let result = GraphQLParser::new(source).parse_schema_document();

// Strict mode: returns AST only if there were zero errors.
// Use this when compiling schemas or executing queries.
if let Some(doc) = result.valid_ast() {
    // Guaranteed: no parse errors
}

// Best-effort mode: returns AST if present, even with errors.
// Use this for IDE features, formatters, and linters.
if let Some(doc) = result.ast() {
    // May be a partial/recovered AST — check result.has_errors()
}
```

## Design Goals

- **Performance** — zero-copy lexing via `Cow<'src, str>`, minimal
  allocations (e.g., `SmallVec` for trivia), and hand-written recursive
  descent parsing.
- **Error resilience** — always produce as much AST as possible, collecting
  all errors in a single pass rather than stopping at the first failure.
- **Spec correctness** — targets the
  [September 2025](https://spec.graphql.org/September2025/) GraphQL
  specification.
- **Extensible architecture** — the parser is generic over
  `GraphQLTokenSource`, enabling the same parsing logic to work across
  string input, proc-macro token streams, and other token sources.
- **Tooling-ready** — designed for IDE integration, linters, formatters,
  and compiler frontends with dual UTF-8/UTF-16 position tracking and
  configurable AST access.

## Comparison to Alternatives

|                          | `libgraphql-parser`     | `graphql-parser` | `apollo-parser`        | `cynic-parser`     | `async-graphql-parser`              |
|--------------------------|-------------------------|------------------|------------------------|--------------------|-------------------------------------|
| **Spec version**         | Sep 2025                | Oct 2016         | Oct 2021               | Oct 2021           | Oct 2021                            |
| **Error recovery**       | ✅ Partial AST + errors | ❌ Fail-fast     | ✅ Full CST + errors   | ✅ Multiple errors | ❌ Fail-fast                        |
| **Zero-copy lexing**     | ✅ `Cow<'src, str>`     | ❌               | ✅ `&'str`             | ✅                 | ❌                                  |
| **Output type**          | Lossless AST            | Lossy AST        | Lossless CST           | Lossy AST (arena)  | Lossy AST                           |
| **Mixed documents**      | ✅                      | ❌               | ✅                     | ✅                 | ❌                                  |
| **Trivia preserved**     | ✅ Comments             | ❌               | ✅ All whitespace      | ❌                 | ❌                                  |
| **GitHub schema parse**  | 9.67 ms                 | 8.73 ms          | 12.6 ms                | ??                 | ??                                  |

## Performance

Performance is a first-class design goal. The lexer avoids allocations via
`Cow<'src, str>` (borrowing directly from the source string for tokens that
don't need transformation) and uses `SmallVec` for trivia storage and
token-buffering. The parser itself is a hand-written recursive descent
parser, avoiding the overhead of parser generator runtimes and allowing for
more helpful and structured error information, notes, and possible-fix
suggestions.

Benchmarks run via [Criterion](https://github.com/bheisler/criterion.rs)
on synthetic schemas (small ~1.5KB, medium ~106KB, large ~500KB),
vendored real-world schemas (Star Wars ~4KB, GitHub ~1.2MB), and
executable documents. Run them yourself with
`cargo bench --package libgraphql-parser`, or use the high-confidence
script: `./crates/libgraphql-parser/scripts/run-benchmarks.sh`.

> **Measured:** 2026-02-11 on Apple M2 Max (arm64), 64 GB RAM, macOS,
> rustc 1.90.0-nightly (0d9592026 2025-07-19), `--release` profile.
> Comparison parsers: `graphql-parser` 0.4.1, `apollo-parser` 0.8.4.
> All values are Criterion point estimates at a 99% confidence level.

### Schema Parsing

| Input               | `libgraphql-parser` | `graphql-parser` | `apollo-parser` |
|---------------------|---------------------|------------------|-----------------|
| small (~1.5 KB)     | **35.8 µs**         | 43.7 µs          | 45.8 µs         |
| medium (~106 KB)    | **1.70 ms**         | 1.93 ms          | 2.03 ms         |
| large (~500 KB)     | **7.89 ms**         | 8.92 ms          | 9.62 ms         |
| starwars (~4 KB)    | **40.6 µs**         | 49.6 µs          | 54.7 µs         |
| github (~1.2 MB)    | 9.67 ms             | **8.73 ms**      | 12.6 ms         |

### Executable Document Parsing

| Input             | `libgraphql-parser` | `graphql-parser` | `apollo-parser` |
|-------------------|---------------------|------------------|-----------------|
| simple query      | **1.66 µs**         | 2.94 µs          | 3.02 µs         |
| complex query     | **30.1 µs**         | 39.6 µs          | 38.8 µs         |

### Lexer Throughput

| Input               | Time     | Throughput  |
|---------------------|----------|-------------|
| small (~1.5 KB)     | 21.3 µs  | ~106 MiB/s  |
| medium (~106 KB)    | 1.00 ms  | ~101 MiB/s  |
| large (~500 KB)     | 4.67 ms  | ~102 MiB/s  |
| starwars (~4 KB)    | 25.2 µs  | ~157 MiB/s  |
| github (~1.2 MB)    | 5.00 ms  | ~233 MiB/s  |

## Core Types

| Type                      | Description                                                                                                                         |
|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------|
| [`GraphQLParser<S>`]      | Generic recursive-descent parser. Entry points: `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()`. |
| [`ParseResult<T>`]        | Result type holding both a (possibly partial) AST and accumulated errors.                                                           |
| [`StrGraphQLTokenSource`] | Zero-copy lexer producing `GraphQLToken` streams from `&str` input.                                                                 |
| [`GraphQLParseError`]     | Parse error with message, source span, categorized kind, and contextual notes.                                                      |
| [`GraphQLTokenSource`]    | Trait for pluggable token sources (string input, proc-macro tokens, etc.).                                                          |

[`GraphQLParser<S>`]: https://docs.rs/libgraphql-parser/latest/libgraphql_parser/struct.GraphQLParser.html
[`ParseResult<T>`]: https://docs.rs/libgraphql-parser/latest/libgraphql_parser/struct.ParseResult.html
[`StrGraphQLTokenSource`]: https://docs.rs/libgraphql-parser/latest/libgraphql_parser/token_source/struct.StrGraphQLTokenSource.html
[`GraphQLParseError`]: https://docs.rs/libgraphql-parser/latest/libgraphql_parser/struct.GraphQLParseError.html
[`GraphQLTokenSource`]: https://docs.rs/libgraphql-parser/latest/libgraphql_parser/token_source/trait.GraphQLTokenSource.html

## Part of the `libgraphql` Ecosystem

`libgraphql-parser` is the parsing foundation of the
[`libgraphql`](https://crates.io/crates/libgraphql) project — a
comprehensive GraphQL engine library for building tools, clients, and
servers in Rust. It is used by `libgraphql-core` for schema building,
operation validation, and type system logic.

## Running Tests

```bash
cargo test --package libgraphql-parser
```

## Fuzz Testing

The crate includes a [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz)
setup under `fuzz/` with four targets:

| Target                  | Entry point                                      |
|-------------------------|--------------------------------------------------|
| `fuzz_lexer`            | `StrGraphQLTokenSource` (full token iteration)   |
| `fuzz_parse_schema`     | `GraphQLParser::parse_schema_document()`         |
| `fuzz_parse_executable` | `GraphQLParser::parse_executable_document()`     |
| `fuzz_parse_mixed`      | `GraphQLParser::parse_mixed_document()`          |

### Prerequisites

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

### Running Fuzz Tests

**Quick smoke test (1 minute per target, parallel):**
```bash
./crates/libgraphql-parser/scripts/run-fuzz-tests.sh
```

**Sustained run (15 minutes per target, parallel):**
```bash
./crates/libgraphql-parser/scripts/run-fuzz-tests.sh 15
```

**Single target:**
```bash
./crates/libgraphql-parser/scripts/run-fuzz-tests.sh 5 fuzz_lexer
```

**Raw `cargo fuzz` (from the fuzz directory):**
```bash
cd crates/libgraphql-parser/fuzz
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=60
```

### Latest Fuzz Testing Results

**Date:** 2026-02-10
**Duration:** 60 minutes per target (4 targets in parallel)
**Platform:** macOS (aarch64), nightly Rust

| Target                  | Executions | Exec/s  | Corpus Entries | Crashes |
|-------------------------|------------|---------|----------------|---------|
| `fuzz_lexer`            | 39,990,040 | ~11,105 | 31,875         | 0       |
| `fuzz_parse_schema`     | 9,220,187  | ~2,560  | 43,160         | 0       |
| `fuzz_parse_executable` | 11,261,274 | ~3,127  | 43,633         | 0       |
| `fuzz_parse_mixed`      | 10,173,478 | ~2,825  | 48,957         | 0       |

**Total:** 70,644,979 executions across all targets, zero crashes.

## License

Licensed under the [MIT license](https://github.com/jeffmo/libgraphql/blob/main/LICENSE).
