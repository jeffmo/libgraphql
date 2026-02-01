# libgraphql-parser

A GraphQL parser crate with error-resilient
parsing of schema, executable, and mixed GraphQL documents.

## Core Components

- **`GraphQLParser<S>`** — Generic recursive-descent parser over any
  `GraphQLTokenSource`. Produces `ParseResult<T>` containing a partial
  AST and accumulated errors for IDE-friendly diagnostics.
- **`ParseResult<T>`** — Holds both the best-effort AST and a list of
  `GraphQLParseError` values, enabling callers to display diagnostics
  even when the input is malformed.
- **`StrGraphQLTokenSource`** — Zero-copy lexer producing
  `GraphQLToken` streams from `&str` input. Uses `Cow<'src, str>` to
  avoid allocations for tokens that match the source verbatim.

## Usage

```rust
use libgraphql_parser::GraphQLParser;
use libgraphql_parser::StrGraphQLTokenSource;

let source = StrGraphQLTokenSource::new("{ field }");
let result = GraphQLParser::new(source).parse_executable_document();
```

## Running Tests

```bash
cargo test --package libgraphql-parser
```

## Fuzz Testing

The crate includes a [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz)
setup under `fuzz/` with four targets:

| Target                    | Entry point                                    |
|---------------------------|------------------------------------------------|
| `fuzz_lexer`              | `StrGraphQLTokenSource` (full token iteration) |
| `fuzz_parse_schema`       | `GraphQLParser::parse_schema_document()`       |
| `fuzz_parse_executable`   | `GraphQLParser::parse_executable_document()`   |
| `fuzz_parse_mixed`        | `GraphQLParser::parse_mixed_document()`        |

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

**Date:** 2026-01-29
**Duration:** 15 minutes per target (4 targets in parallel)
**Platform:** macOS (aarch64), nightly Rust

| Target                  | Executions  | Exec/s  | Corpus Entries | Crashes |
|-------------------------|-------------|---------|----------------|---------|
| `fuzz_lexer`            | 16,773,894  | ~18,600 | 25,819         | 0       |
| `fuzz_parse_schema`     | 2,699,717   | ~3,000  | 31,720         | 0       |
| `fuzz_parse_executable` | 3,149,679   | ~3,500  | 28,362         | 0       |
| `fuzz_parse_mixed`      | 2,852,045   | ~3,165  | 33,435         | 0       |

**Total:** 25,475,335 executions across all targets, zero crashes.
