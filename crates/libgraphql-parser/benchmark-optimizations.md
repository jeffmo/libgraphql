# libgraphql-parser Benchmark Optimizations

Tracker for performance optimization opportunities in the lexer and parser.
Each entry documents the problem, fix, trade-offs, and (once implemented)
benchmark results.

Status legend: **Pending** | **Completed** | **Skipped**

---

## B1: `peek_char()` uses `remaining().chars().nth(0)` on every character [CRITICAL]

**Status:** Completed
**Priority:** 1 (highest bang-for-buck)
**File:** `src/token_source/str_to_graphql_token_source.rs`
**Date:** 2026-02-08

**Problem:** Every character peek constructs a `&str` slice via `remaining()`,
creates a `Chars` iterator, and walks to the nth element. Called millions of
times for large inputs (every `consume()`, `skip_whitespace()`, `lex_name()`,
`lex_comment()`, `lex_block_string()`, `next_token()`).

**Change made:** Replaced both `peek_char()` and `consume()` with ASCII fast
paths. `peek_char()` does direct byte indexing + `is_ascii()` check instead
of creating a `Chars` iterator. `consume()` skips `ch.len_utf8()` and
`ch.len_utf16()` calls for ASCII (known to be 1 byte / 1 code unit). Non-ASCII
falls back to full UTF-8 decoding.

**Trade-offs:** Adds ASCII vs non-ASCII branch; branch prediction strongly
favors ASCII. `peek_char_nth(n)` for n>0 still needs iterator approach.

**Benchmark results (clean run, both before/after on AC power):**

Lexer-only (isolated lexer performance, most reliable signal):

| Fixture               | Before   | After    | Change     |
|-----------------------|----------|----------|------------|
| lexer/github_schema   | 8.089ms  | 7.552ms  | **-6.6%**  |
| lexer/large_schema    | 6.483ms  | 6.234ms  | **-3.8%**  |
| lexer/starwars_schema | 40.60us  | 37.20us  | **-6.6%**  |
| lexer/medium_schema   | 1.381ms  | 1.335ms  | **-3.5%**  |
| lexer/small_schema    | 28.95us  | 28.15us  | **-2.9%**  |

Full schema parse (lexer + parser combined):

| Fixture              | Before   | After    | Change     |
|----------------------|----------|----------|------------|
| schema_parse/github  | 23.01ms  | 22.33ms  | **-3.0%**  |
| schema_parse/large   | 24.71ms  | 24.73ms  | ~0%        |
| schema_parse/medium  | 4.961ms  | 4.985ms  | ~0%        |
| schema_parse/starwars | 87.23us | 91.38us  | +4.8% (*)  |
| schema_parse/small   | 44.34us  | 43.83us  | -1.3%      |

Cross-parser comparison (libgraphql_parser only):

| Fixture                             | Before   | After    | Change     |
|-------------------------------------|----------|----------|------------|
| compare_schema_parse/.../github     | 22.89ms  | 22.47ms  | **-1.9%**  |
| compare_schema_parse/.../large      | 24.59ms  | 24.61ms  | ~0%        |
| compare_schema_parse/.../medium     | 4.997ms  | 4.974ms  | -0.5%      |
| compare_schema_parse/.../starwars   | 77.14us  | 85.22us  | +10.5% (*) |
| compare_schema_parse/.../small      | 83.99us  | 81.21us  | -2.6%      |

(*) The starwars parse regression is anomalous: the lexer for starwars
clearly improved by -6.6%, and control parsers (graphql_parser,
apollo_parser) showed 0-1.5% random drift. This appears to be
measurement noise on the small (~4KB) fixture where variance is high.

**Machine:** Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
**Rust:** rustc 1.90.0-nightly (0d9592026 2025-07-19)

**Verdict:** Consistent 3-7% lexer improvement across all fixture sizes.
Full parse shows ~2-3% improvement on the largest real-world input
(github). Keeping.

---

## B2: `consume()` does per-character position tracking [HIGH]

**Status:** Completed (all 4 sub-optimizations kept)
**Priority:** 6
**File:** `src/token_source/str_to_graphql_token_source.rs`
**Date:** 2026-02-09

**Problem:** Every character consumed updates 5-6 fields (peek_char, newline
check, curr_col_utf8, curr_col_utf16, last_char_was_cr, curr_byte_offset).
For a name like `PullRequestReviewCommentConnection` that's 36 chars x 6 ops.

**Change made:** Implemented byte-scanning fast paths for 4 hot lexer
methods. Each scans raw bytes in a tight loop (one branch per byte)
and batch-updates position tracking once at the end. Shared helper
`compute_columns_for_span()` handles ASCII fast path for column
computation (ASCII byte count = char count = UTF-16 unit count).

The approach is safe for multi-byte UTF-8 because the sentinel bytes
(`"`, `\`, `\n`, `\r`) are all ASCII (<0x80) and can never appear as
continuation bytes in multi-byte UTF-8 sequences (which are >=0x80).

Sub-optimizations (each a separate commit):
1. **`lex_name()`** — Byte-scan `[_0-9A-Za-z]` pattern. Names are
   ASCII-only by spec, no newlines, so column = byte count.
2. **`skip_whitespace()`** — Byte-scan ` `, `\t`, `\n`, `\r`, BOM.
   Tracks newline positions and BOM count for column computation.
3. **`lex_comment()`** — Byte-scan to `\n`/`\r`/EOF. Comments are
   single-line, so only column advances. Uses
   `compute_columns_for_span()` for potential non-ASCII content.
4. **`lex_block_string()`** — Byte-scan for `"`, `\`, `\n`, `\r`
   sentinels, skip everything else with `i += 1`. Tracks newlines
   for position reconstruction via `compute_columns_for_span()`.

**Trade-offs:** More complex position tracking logic (batch vs
per-char). `skip_whitespace()` tracks BOM count for correct column
math. `lex_block_string()` uses `compute_columns_for_span()` which
iterates after-last-newline range (but has ASCII fast path).

**Benchmark results (back-to-back, both on AC power):**

Machine: Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
Rust: rustc 1.90.0-nightly (0d9592026 2025-07-19)

Controls: graphql_parser ±0-1.6%, apollo_parser ±0-1.9% — clean
measurement, all changes attributable to our code.

### Net results (all 4 sub-optimizations combined vs B5 baseline)

Schema parse:

| Fixture  | Before    | After     | Change       |
|----------|-----------|-----------|--------------|
| small    | 43.0 µs   | 37.8 µs   | **-12.1%**   |
| medium   | 2.07 ms   | 1.81 ms   | **-12.6%**   |
| large    | 9.65 ms   | 8.40 ms   | **-12.9%**   |
| starwars | 53.4 µs   | 42.8 µs   | **-19.5%**   |
| github   | 12.6 ms   | 10.5 ms   | **-16.9%**   |

Executable parse:

| Fixture          | Before    | After     | Change       |
|------------------|-----------|-----------|--------------|
| simple_query     | 1.94 µs   | 1.73 µs   | **-10.9%**   |
| complex_query    | 35.8 µs   | 31.7 µs   | **-11.2%**   |
| nested_depth_10  | 7.72 µs   | 6.2 µs    | **-19.8%**   |
| nested_depth_30  | 28.1 µs   | 18.5 µs   | **-34.2%**   |
| many_ops_50      | 141 µs    | 131 µs    | **-7.2%**    |

Lexer-only (isolates lexer changes):

| Fixture         | Before    | After     | Change       |
|-----------------|-----------|-----------|--------------|
| small_schema    | 28.5 µs   | 25.8 µs   | **-6.3%**    |
| medium_schema   | 1.35 ms   | 1.22 ms   | **-9.3%**    |
| large_schema    | 6.30 ms   | 5.69 ms   | **-9.6%**    |
| starwars_schema | 37.6 µs   | 29.9 µs   | **-20.2%**   |
| github_schema   | 7.68 ms   | 5.85 ms   | **-23.8%**   |

Cross-parser comparison (schema parse, after B2):

| Fixture  | libgraphql   | graphql_parser | apollo_parser |
|----------|--------------|----------------|---------------|
| small    | **37.9 µs**  | 47.1 µs        | 48.8 µs       |
| medium   | **1.82 ms**  | 2.09 ms        | 2.24 ms       |
| large    | **8.41 ms**  | 9.63 ms        | 10.7 ms       |
| starwars | **42.8 µs**  | 52.9 µs        | 58.4 µs       |
| github   | 10.5 ms      | **9.46 ms**    | 14.1 ms       |

Cross-parser comparison (executable parse, after B2):

| Fixture  | libgraphql   | graphql_parser | apollo_parser |
|----------|--------------|----------------|---------------|
| simple   | **1.74 µs**  | 3.02 µs        | 3.17 µs       |
| complex  | **31.7 µs**  | 41.9 µs        | 41.0 µs       |

### Bisection — marginal contribution of each sub-optimization

Marginal % = this commit's incremental effect (cumulative minus
previous cumulative). Values within ±3% are within measurement
noise and marked with ~.

schema_parse marginals:

| Fixture  | lex_name   | skip_ws  | lex_comment  | block_string |
|----------|------------|----------|--------------|--------------|
| small    | **-5.9%**  | ~        | ~            | **-7.0%**    |
| medium   | **-7.9%**  | ~        | ~            | **-3.1%**    |
| large    | **-10.5%** | ~        | ~            | **-4.6%**    |
| starwars | **-4.1%**  | ~        | **-12.9%**   | ~            |
| github   | ~          | ~        | ~            | **-10.5%**   |

executable_parse marginals:

| Fixture          | lex_name   | skip_ws     | lex_comment | block_string |
|------------------|------------|-------------|-------------|--------------|
| simple_query     | **-6.8%**  | ~           | ~           | ~            |
| complex_query    | **-8.9%**  | ~           | ~           | ~            |
| nested_depth_10  | **-7.9%**  | **-10.1%**  | ~           | ~            |
| nested_depth_30  | **-3.2%**  | **-29.3%**  | ~           | ~            |
| many_ops_50      | **-5.9%**  | ~           | ~           | ~            |

lexer marginals:

| Fixture         | lex_name   | skip_ws  | lex_comment  | block_string  |
|-----------------|------------|----------|--------------|---------------|
| small_schema    | ~          | ~        | ~            | ~             |
| medium_schema   | **-4.1%**  | ~        | ~            | **-3.3%**     |
| large_schema    | **-3.6%**  | ~        | ~            | ~             |
| starwars_schema | ~          | ~        | **-20.8%**   | ~             |
| github_schema   | **-4.0%**  | ~        | ~            | **-18.8%**    |

### Per sub-optimization assessment

**lex_name:** Broad, consistent 4-11% improvement across schema and
executable parsing. Names are the most frequent token type — every
identifier, type name, field name, keyword benefits.

**skip_whitespace:** Dramatic 10-29% improvement on deeply-nested
executable parsing (depth_10, depth_30) where whitespace-heavy
indentation dominates. Negligible on other fixtures. The nested
fixtures have proportionally more whitespace due to deep indentation.

**lex_comment:** 13-21% improvement on starwars fixture (comment-
heavy). Negligible on other fixtures which have few `#` comments.
The starwars schema has extensive `#`-style comments throughout.

**lex_block_string:** 3-19% improvement on schema fixtures with
block string descriptions (github has 3,246 descriptions). The
github lexer improvement (-18.8%) is particularly striking. No
effect on executable parsing (queries don't typically contain block
strings).

**Verdict:** All 4 sub-optimizations kept. Each targets a different
token type and shows clear signal above noise on fixtures where that
token type is prevalent. No regressions detected on any fixture.
libgraphql-parser now leads graphql_parser and apollo_parser on all
schema fixtures except github (where graphql_parser still leads by
~10%). On executable parsing, libgraphql-parser leads by ~1.7-1.8x.

---

## B3: Block string parsing allocates heavily [HIGH]

**Status:** Completed
**Priority:** 2
**File:** `src/token/graphql_token_kind.rs`
**Date:** 2026-02-08

**Problem:** `parse_block_string()` is called for every description. For the
GitHub schema (~3,246 block strings), each call does:
1. `content.replace("\\\"\"\"", "\"\"\"")` — always allocates even when no
   escaped triple quotes exist (common case)
2. `content.lines().collect::<Vec<&str>>()` — allocates a Vec
3. `Vec::with_capacity(lines.len())` of `String` — allocates Vec of Strings
4. `.to_string()` per line — heap allocation per line
5. `result_lines.remove(0)` — O(n) shift
6. `result_lines.join("\n")` — final allocation

~6 allocations per description x 3,246 = ~19,000+ heap allocations.

**Change made:** Rewrote `parse_block_string()` as a two-pass, low-allocation
algorithm:
- `Cow::Borrowed` fast path skips `replace()` when no `\"""` present
- Pass 1 iterates `str::lines()` lazily to compute common indent and
  first/last non-blank line indices (no Vec allocation)
- Pass 2 iterates `str::lines()` again, writing stripped lines directly
  into a single pre-allocated `String`
- No `Vec<String>`, no `remove(0)`, no `join()` — just one `String`
  allocation for the entire result

**Trade-offs:** More complex two-pass logic (two `str::lines()` iterations
instead of one collect). Must preserve exact spec semantics for edge cases.

**Benchmark results (clean back-to-back runs):**

Schema parse (full parse, lexer + parser):

| Fixture               | Before   | After    | Change    |
|-----------------------|----------|----------|-----------|
| schema_parse/github   | 22.35ms  | 21.55ms  | **-3.6%** |
| schema_parse/large    | 24.61ms  | 23.96ms  | **-2.6%** |
| schema_parse/medium   | 4.917ms  | 4.816ms  | **-2.0%** |
| schema_parse/small    | 43.70us  | 41.84us  | **-4.0%** |
| schema_parse/starwars | 90.61us  | 93.14us  | +2.6% (*) |

Cross-parser comparison (libgraphql_parser only):

| Fixture                            | Before   | After    | Change    |
|------------------------------------|----------|----------|-----------|
| compare_.../libgraphql_.../github  | 22.72ms  | 21.35ms  | **-3.0%** |
| compare_.../libgraphql_.../large   | 24.49ms  | 24.28ms  | ~0%       |
| compare_.../libgraphql_.../medium  | 4.916ms  | 4.834ms  | ~0%       |
| compare_.../libgraphql_.../small   | 79.67us  | 75.22us  | **-6.0%** |

Executable parse (B3 benefits string-heavy queries):

| Fixture                  | Before   | After    | Change       |
|--------------------------|----------|----------|--------------|
| executable_parse/complex | 72.30us  | 54.46us  | **-25.4%**   |
| compare_.../complex      | 72.93us  | 68.41us  | **-4.3%**    |

Lexer (expected: no change — B3 is parser-level):

| Fixture             | Before   | After    | Change |
|---------------------|----------|----------|--------|
| lexer/github_schema | 7.550ms  | 7.526ms  | ~0%    |
| lexer/large_schema  | 6.212ms  | 6.220ms  | ~0%    |

(*) The starwars regression is anomalous: control parsers showed 0%
drift, and the starwars schema has few descriptions (B3 should be
irrelevant there). This appears to be measurement noise.

**Machine:** Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
**Rust:** rustc 1.90.0-nightly (0d9592026 2025-07-19)

**Verdict:** Clear improvement on description-heavy inputs (github
-3%, complex query -25%). Lexer unaffected as expected. Keeping.

---

## B4: `name.into_owned()` forces heap allocation for every identifier [DEFERRED]

**Status:** Pending (deferred — requires significant architectural changes)
**Priority:** — (future work)
**File:** `src/graphql_parser.rs`, `src/ast.rs`

**Problem:** AST types use `String` for identifiers. Every name is converted
from `Cow::Borrowed(&str)` to owned `String` via `into_owned()`. For the
GitHub schema with ~70,000+ identifiers, that's ~70,000 heap allocations.

**Suggested fix (long-term):** Define native AST types with `Cow<'src, str>`.
**Suggested fix (short-term):** String interning / arena allocation.

**Trade-offs:** Major architectural refactor. Lifetime `'src` propagates
through all AST consumers.

**Est. impact:** HIGHEST — but deferred due to scope

---

## B5: Token clone in `expect()` [CRITICAL]

**Status:** Completed
**Priority:** 5
**File:** `src/graphql_parser.rs`, `src/graphql_token_stream.rs`
**Date:** 2026-02-09

**Problem:** `expect()` cloned the peeked token before consuming.
Clone included `GraphQLTokenKind` enum (with `Cow<str>`),
`SmallVec<[GraphQLTriviaToken; 2]>` trivia, `GraphQLSourceSpan`.
Called for every punctuator — tens of thousands of times per schema.

**Change made:** Replaced `Vec`+index buffer in `GraphQLTokenStream`
with `VecDeque` ring buffer. `consume()` now returns
`Option<GraphQLToken>` (owned) via O(1) `pop_front()`. Eliminated:

- Token clone in `expect()` (cloned full GraphQLToken per punctuator)
- `Cow<str>` clone in `expect_name_only()` (now moves from owned token)
- Span clone in `expect_keyword()` success path
- Span clone in `parse_description()` error path

Removed `compact_buffer()` (VecDeque naturally discards consumed
tokens) and `current_token()` (callers use `consume_token()` return
directly). Parser tracks `last_end_position: Option<SourcePosition>`
for EOF error anchoring.

**Trade-offs:** Changed `GraphQLTokenStream` API — removed
`current_token()` and `compact_buffer()`. All ~50
`self.token_stream.consume()` call sites updated to
`self.consume_token()` wrapper that tracks end position.

**Benchmark results (clean back-to-back, both on AC power):**

Machine: Apple M-series arm64, macOS
Rust: rustc 1.90.0-nightly (0d9592026 2025-07-19)

Controls confirm clean measurement: lexer benchmarks all within
±0.3–1.8% (no parser changes expected). Competitor parsers
(graphql_parser, apollo_parser) within ±0–3% noise.

Standalone schema_parse (libgraphql only):

| Fixture  | Before    | After    | Change       |
|----------|-----------|----------|--------------|
| small    | 42.1 µs   | 42.1 µs  | ~0%          |
| medium   | 5.25 ms   | 2.04 ms  | **-61.1%**   |
| large    | 20.3 ms   | 9.50 ms  | **-53.2%**   |
| starwars | 92.7 µs   | 52.1 µs  | **-42.8%**   |
| github   | 21.7 ms   | 12.4 ms  | **-42.7%**   |

Standalone executable_parse (libgraphql only):

| Fixture          | Before    | After     | Change       |
|------------------|-----------|-----------|--------------|
| simple_query     | 1.93 µs   | 1.91 µs   | -1.6%        |
| complex_query    | 70.8 µs   | 34.9 µs   | **-51.1%**   |
| nested_depth_10  | 8.25 µs   | 7.54 µs   | **-9.0%**    |
| nested_depth_30  | 61.1 µs   | 27.6 µs   | **-54.8%**   |
| many_ops_50      | 198.7 µs  | 138.3 µs  | **-30.1%**   |

Cross-parser comparison (schema parse, after B5):

| Fixture  | libgraphql  | graphql_parser | apollo_parser |
|----------|-------------|----------------|---------------|
| small    | **42.0 µs** | 46.6 µs        | 48.3 µs       |
| medium   | **2.05 ms** | 2.06 ms        | 2.20 ms       |
| large    | **9.49 ms** | 9.48 ms        | 10.6 ms       |
| starwars | 52.1 µs     | **52.6 µs**    | 57.6 µs       |
| github   | 12.5 ms     | **9.35 ms**    | 13.9 ms       |

Cross-parser comparison (executable parse, after B5):

| Fixture | libgraphql  | graphql_parser | apollo_parser |
|---------|-------------|----------------|---------------|
| simple  | **1.91 µs** | 3.03 µs        | 3.13 µs       |
| complex | **34.9 µs** | 41.3 µs        | 40.6 µs       |

**Verdict:** Massive improvement — original "MODERATE" estimate was
dramatically wrong. Token cloning was the dominant parser bottleneck.
libgraphql-parser is now competitive with or faster than both
graphql_parser and apollo_parser on most benchmarks. The original
2–2.5x gap is closed to 1.0–1.3x across all fixtures.

---

## B6: `starts_with()` in block string lexing [MODERATE]

**Status:** Skipped (no measurable improvement — reverted)
**Priority:** 4
**File:** `src/token_source/str_to_graphql_token_source.rs`
**Date:** 2026-02-09

**Problem:** Inside the block string lexer, every character checks:
```rust
self.remaining().starts_with("\\\"\"\"")
self.remaining().starts_with("\"\"\"")
```
`remaining()` creates a new slice each time. Adds up for long block strings.
Also replaced in `lex_string()` for the block string detection check.

**Change attempted:** Added `next_is_triple_quote()` and
`next_is_escaped_triple_quote()` helper methods that use direct byte
indexing into `self.source.as_bytes()`. Also removed unnecessary
`#[inline]` from `peek_char()` (inlining is already handled by LLVM
for crate-local calls).

**Benchmark results (two independent back-to-back runs):**

Lexer-only (B6 targets lexer — most direct signal):

| Fixture               | Before (r1) | After (r1) | Before (r2) | After (r2) |
|-----------------------|-------------|------------|-------------|------------|
| lexer/github_schema   | 7.650ms     | 7.741ms    | 7.543ms     | 7.562ms    |
| lexer/large_schema    | 6.287ms     | 6.327ms    | 6.277ms     | 6.230ms    |
| lexer/medium_schema   | 1.328ms     | 1.345ms    | 1.321ms     | 1.349ms    |
| lexer/small_schema    | 28.73us     | 28.13us    | 28.06us     | 27.83us    |
| lexer/starwars_schema | 37.60us     | 37.83us    | 37.10us     | 37.20us    |

Cross-parser comparison (controlled — all parsers in same run):

| Fixture                           | Before (r1) | After (r1) | Before (r2) | After (r2) |
|-----------------------------------|-------------|------------|-------------|------------|
| compare_.../libgraphql_.../github | 21.48ms     | 21.35ms    | 21.30ms     | 21.33ms    |
| compare_.../libgraphql_.../large  | 24.50ms     | 24.36ms    | 24.08ms     | 24.11ms    |
| compare_.../libgraphql_.../medium | 4.823ms     | 4.746ms    | 4.813ms     | 4.878ms    |
| compare_.../graphql_parser/github | 9.600ms     | 9.434ms    | 9.463ms     | 9.392ms    |
| compare_.../graphql_parser/large  | 9.661ms     | 9.587ms    | 9.533ms     | 9.541ms    |
| compare_.../apollo_parser/github  | 14.93ms     | 14.01ms    | 15.01ms     | 14.04ms    |
| compare_.../apollo_parser/large   | 11.05ms     | 10.74ms    | 10.70ms     | 10.74ms    |

Control parsers show the same magnitude of drift as libgraphql_parser
across both runs, confirming B6 has no effect above the noise floor.

**Machine:** Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
**Rust:** rustc 1.90.0-nightly (0d9592026 2025-07-19)

**Verdict:** No measurable performance change across two independent
back-to-back runs. `starts_with()` for short literal patterns is
already well-optimized by the compiler. Code changes reverted.

---

## B7: `shrink_to_fit()` in `compact_buffer()` [LOW-MODERATE]

**Status:** Completed
**Priority:** 3
**File:** `src/graphql_token_stream.rs`
**Date:** 2026-02-08

**Problem:** After every buffer compaction (once per top-level definition),
`shrink_to_fit()` may trigger a reallocation to shrink Vec capacity, only for
the buffer to grow again for the next definition. For 1000+ definitions,
that's 1000+ potential realloc cycles.

**Change made:** Removed `shrink_to_fit()` call. Buffer retains capacity
between definitions.

**Trade-offs:** Slightly higher peak memory (~few KB retained). Negligible.

**Benchmark results (clean back-to-back runs):**

Schema parse (full parse, lexer + parser):

| Fixture               | Before   | After    | Change      |
|-----------------------|----------|----------|-------------|
| schema_parse/github   | 21.55ms  | 19.43ms  | **-9.9%**   |
| schema_parse/large    | 24.34ms  | 21.91ms  | **-10.0%**  |
| schema_parse/medium   | 4.769ms  | 4.155ms  | **-12.9%**  |
| schema_parse/small    | 42.29us  | 42.41us  | ~0%         |
| schema_parse/starwars | 91.03us  | 89.64us  | -1.3%       |

Lexer (expected: no change — B7 is parser-level):

| Fixture             | Before   | After    | Change |
|---------------------|----------|----------|--------|
| lexer/github_schema | 7.679ms  | 7.646ms  | ~0%    |
| lexer/large_schema  | 6.430ms  | 6.318ms  | -1.7%  |

Impact scales with number of top-level definitions: medium (~200
types) and large (~1000 types) showed the biggest gains. Small
schemas with few definitions showed no change, confirming that the
improvement comes from reduced realloc churn in the compaction loop.

**Machine:** Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
**Rust:** rustc 1.90.0-nightly (0d9592026 2025-07-19)

**Verdict:** Unexpectedly large improvement (10-13% on medium/large
schemas). Much bigger than the estimated LOW-MODERATE. Keeping.

---

## B8: No `[profile.bench]` in workspace Cargo.toml [LOW — MEASUREMENT ONLY]

**Status:** Pending
**Priority:** 7
**File:** `Cargo.toml` (workspace root)

**Problem:** No benchmark-specific profile. Adding LTO and single codegen unit
helps cross-crate inlining.

**Suggested fix:**
```toml
[profile.bench]
lto = "thin"
codegen-units = 1
```

**Trade-offs:** Slower benchmark compilation. Affects all parsers equally in
comparative benchmarks.

**Est. impact:** LOW for relative comparisons, potentially MODERATE for absolute

---

## B9: Dual UTF-16 column tracking on every character [LOW]

**Status:** Pending (subsumed by B2 if adopted)
**Priority:** 9
**File:** `src/token_source/str_to_graphql_token_source.rs:202`

**Problem:** Every non-newline char updates `curr_col_utf16 += ch.len_utf16()`.
For ASCII `len_utf16()` always returns 1. Small per-char overhead.

**Suggested fix:** Make UTF-16 tracking opt-in via constructor flag, or defer
to B2's lazy position computation.

**Trade-offs:** API change; consumers needing UTF-16 must opt in.

**Est. impact:** LOW standalone — subsumed by B2

---

## B11: Remove Box from syntax structs (inline them) [REVERTED]

**Status:** Reverted — catastrophic regression
**Date:** 2026-03-04

**Hypothesis:** Eliminating ~48 `Box::new()` heap allocations per AST node
construction would reduce allocation overhead, especially for large schemas
(~100K+ nodes for github).

**Change:** `Option<Box<XyzSyntax<'src>>>` → `Option<XyzSyntax<'src>>` across all
42 AST struct files + parser. Removed all `Box::new()` calls except the recursive
`element_type` in `ListTypeAnnotation`.

**Result:** Massive regression across the board.

| Benchmark        | B11     | A.1 Baseline | Change |
|------------------|---------|--------------|--------|
| medium (default) | 5.87 ms | 3.86 ms      | +52%   |
| large (default)  | 31.4 ms | 21.0 ms      | +50%   |
| github (default) | 23.5 ms | 19.5 ms      | +20%   |
| medium (lean)    | 5.49 ms | ~1.92 ms     | +186%  |
| github (lean)    | 22.0 ms | ~13.0 ms     | +70%   |

**Root cause:** `Option<Box<T>>` with `None` = 8 bytes (null pointer).
`Option<T>` with `None` = full `size_of::<T>()`. Inlining syntax structs bloated
every AST node — even in lean mode where syntax is always `None`. The larger
structs destroyed cache locality, overwhelming any savings from fewer heap
allocations. Lean mode was hit hardest (+150-400%) because every `None` field
now carries the full struct weight instead of a null pointer.

**Lesson:** `Option<Box<T>>` is the correct pattern for "expensive when present,
free when absent" optional data. The per-allocation cost of `Box::new()` (~25ns)
is far less than the cache miss penalty from bloated structs.

---

## B14: #[inline] on hot parser functions [REVERTED]

**Status:** Reverted — no statistically significant improvement
**Date:** 2026-03-04

**Change:** Added `#[inline]` to `peek_is_keyword()`, `peek_is()`,
`consume_token()`, `make_span()`, `make_span_ref()`, `token_kinds_match()`.

**Result:** No measurable improvement. LLVM already makes good inlining decisions
for intra-crate functions.

---

## B16: Vec::with_capacity() hints [REVERTED]

**Status:** Reverted — regression (7-14% on schema, 13-19% on lean)
**Date:** 2026-03-04

**Change:** Replaced ~38 `Vec::new()` calls with `Vec::with_capacity(N)` using
typical-size hints.

**Root cause:** `Vec::new()` is zero-allocation until first `push()`.
`Vec::with_capacity(N)` allocates immediately. For frequently-empty Vecs (like
`directives` on most nodes), pre-allocation wastes heap allocations that
`Vec::new()` avoids entirely.

---

## B10: SmallVec for commonly-small collection fields [SKIPPED]

**Status:** Skipped — not viable based on struct size analysis
**Date:** 2026-03-04

**Hypothesis:** Replace `Vec<T>` with `SmallVec<[T; N]>` for fields like
`directives`, `arguments`, and `implements` to avoid heap allocation for the
common 0-1 element case.

**Why we skipped it:** Struct size measurements revealed the core problem is
struct bloat, not allocation count. SmallVec inlines elements into the parent
struct, which would increase node sizes for the same cache-locality reasons that
killed B11. Key measurements:

| Type                   | Size (bytes) |
|------------------------|--------------|
| `Vec<T>` (any T)      |           24 |
| `DirectiveAnnotation`  |          192 |
| `Argument`             |          336 |
| `Name`                 |           96 |
| `GraphQLToken`         |          504 |
| `GraphQLSourceSpan`    |           64 |

`SmallVec<[DirectiveAnnotation; 1]>` would add ~168 bytes per node vs `Vec`'s
constant 24 bytes — for a field that is usually empty. Same cache-locality
destruction as B11, just smaller scale.

**Root cause (shared with B11, B16):** The real bottleneck was that
`GraphQLToken` was 504 bytes and syntax structs were 504-1,512 bytes. Allocation
count is not the problem; struct size is. B19 subsequently addressed the token
size issue by boxing the Error variant, shrinking `GraphQLToken` from 504 to
304 bytes. Further gains may come from shrinking spans/tokens (B12).

---

## B19: Box the Error variant of GraphQLTokenKind [COMPLETED]

**Status:** Completed
**Date:** 2026-03-04
**Files:** `src/token/graphql_token_kind.rs`, `src/token/mod.rs`,
`src/graphql_parser.rs`, `src/token_source/str_to_graphql_token_source.rs`
(+ corresponding files in `libgraphql-macros`)

**Problem:** `GraphQLTokenKind` was 232 bytes because its `Error` variant
contained `GraphQLErrorNotes` (`SmallVec<[GraphQLErrorNote; 2]>` = 208 bytes)
plus a `String` (24 bytes). Since Rust enums are sized by their largest
variant, every token — including simple `Name`, `IntValue`, and punctuator
tokens — paid the 232-byte cost of the `Error` variant. This bloated
`GraphQLToken` to 504 bytes (span 64B + kind 232B + preceding_trivia 208B).

**Change made:** Extracted the Error variant's payload into a separate
`GraphQLTokenError` struct and boxed it:

```rust
// Before: Error { message: String, error_notes: GraphQLErrorNotes }
// After:
Error(Box<GraphQLTokenError>)

pub struct GraphQLTokenError {
    pub message: String,
    pub error_notes: GraphQLErrorNotes,
}
```

The existing `GraphQLTokenKind::error()` constructor abstracts the boxing,
so most call sites use the constructor without needing to know about the
`Box`. Pattern match sites were updated from `Error { message, .. }` to
`Error(err)` with `err.message` access.

**Size reduction:**

| Type               | Before (bytes) | After (bytes) | Reduction |
|--------------------|----------------|---------------|-----------|
| `GraphQLTokenKind` |            232 |            32 | **7.25x** |
| `GraphQLToken`     |            504 |           304 | **1.66x** |
| `GraphQLTokenError`|            232 |           232 | (no change; now heap-allocated only on error) |

**Why this works:** Errors are rare during parsing — most tokens are names,
punctuators, and keywords. By boxing the error payload, the `Error` variant
shrinks from 232 bytes to a single pointer-sized value, and the enum's overall size
drops to 32 bytes (determined by the next-largest variant, `StringValue`
with `Cow<str>`). The `Box` allocation only occurs when an actual error is
emitted, which is negligible. Every non-error token benefits from the
smaller struct: faster VecDeque moves, better cache locality, smaller AST
nodes (which embed tokens in syntax structs).

**Trade-offs:** Error construction now requires a heap allocation. Since
errors are rare and always terminate parsing soon after, this is
negligible. Pattern matching on `Error` is slightly less ergonomic
(`Error(err)` instead of `Error { message, .. }`).

**Benchmark results (full run, 300 samples, 20s measurement, 0.99 confidence):**

Machine: Apple M2 Max, 12 cores, 64 GB RAM, macOS (Darwin 23.6.0, arm64)
Rust: rustc 1.90.0-nightly (0d9592026 2025-07-19)

All CI widths under 2%, indicating highly reproducible measurements.

### Schema parse (standalone, default config)

Comparison against A.1 baseline (post-AST-regression, pre-B19) where
available. A.1 baseline values from the B11 entry.

| Fixture       | A.1 Baseline | After B19 | Change       |
|---------------|--------------|-----------|--------------|
| small         | —            | 37.7 µs   | —            |
| medium        | 3.86 ms      | 1.81 ms   | **-53.1%**   |
| large         | 21.0 ms      | 8.40 ms   | **-60.0%**   |
| starwars      | —            | 59.2 µs   | —            |
| github        | 19.5 ms      | 14.9 ms   | **-23.6%**   |
| shopify_admin | —            | 29.6 ms   | —            |

### Schema parse (standalone, lean mode)

| Fixture       | After B19 |
|---------------|-----------|
| starwars      | 38.2 µs   |
| github        | 8.58 ms   |
| shopify_admin | 17.4 ms   |

### Executable parse (standalone, default config)

| Fixture       | After B19 |
|---------------|-----------|
| simple_query  | 2.51 µs   |
| complex_query | 45.3 µs   |
| nested_10     | 9.00 µs   |
| nested_30     | 29.0 µs   |

### Executable parse (standalone, lean mode)

| Fixture       | After B19 |
|---------------|-----------|
| simple_query  | 1.39 µs   |
| complex_query | 25.3 µs   |
| nested_10     | 5.09 µs   |
| nested_30     | 16.3 µs   |

### Cross-parser comparison (schema parse, after B19)

| Fixture       | libgraphql    | graphql_parser | apollo_parser |
|---------------|---------------|----------------|---------------|
| small         | 47.4 µs       | **44.0 µs**    | 46.0 µs       |
| medium        | 2.97 ms       | **1.99 ms**    | 2.08 ms       |
| large         | 15.3 ms       | **9.15 ms**    | 9.95 ms       |
| starwars      | 59.3 µs       | **50.6 µs**    | 55.1 µs       |
| github        | 15.6 ms       | **8.96 ms**    | 12.9 ms       |
| shopify_admin | 29.0 ms       | **17.7 ms**    | 27.3 ms       |

### Cross-parser comparison (executable parse, after B19)

| Fixture       | libgraphql    | graphql_parser | apollo_parser |
|---------------|---------------|----------------|---------------|
| simple        | **2.60 µs**   | 2.89 µs        | 3.02 µs       |
| complex       | 45.5 µs       | 40.1 µs        | **38.9 µs**   |

### Lexer throughput (after B19)

| Fixture       | Time     | Throughput   |
|---------------|----------|--------------|
| small         | 13.9 µs  | ~162 MiB/s   |
| medium        | 633 µs   | ~159 MiB/s   |
| large         | 2.94 ms  | ~162 MiB/s   |
| starwars      | 20.0 µs  | ~199 MiB/s   |
| github        | 3.66 ms  | ~319 MiB/s   |
| shopify_admin | 7.76 ms  | ~399 MiB/s   |

**Verdict:** Massive improvement — the single largest optimization since B5.
Schema parsing improved 24-60% vs the A.1 baseline. `GraphQLToken` shrank
from 504 to 304 bytes (1.66x smaller), dramatically improving cache locality
and VecDeque throughput. libgraphql-parser now **wins on simple executable
queries** (2.60 µs vs 2.89 µs graphql-parser, 3.02 µs apollo-parser). Schema
parsing remains 1.5-1.7x behind graphql-parser on large schemas, but the gap
is significantly narrowed from the post-AST regression. The improvement even
surpasses the pre-AST performance for default-mode schema parsing (medium:
1.81 ms vs pre-AST 2.05 ms), confirming that the Error variant bloat was a
pre-existing bottleneck that was never addressed before.

---

## B20: `lex_string()` byte-scanning + memchr3 [HIGH]

**Status:** Completed
**Priority:** HIGH
**File:** `src/token_source/str_to_graphql_token_source.rs`

**Problem:** `lex_string()` uses `peek_char()`/`consume()` per character to scan single-line string bodies. These calls do bounds checks and ASCII tests on every byte. The function looks for sentinel bytes (`"`, `\`, `\n`, `\r`) but scans byte-by-byte instead of using SIMD-accelerated search.

**Suggested fix:** Replace the `loop { match self.peek_char() { ... } }` body with byte-scanning using `memchr::memchr3(b'"', b'\\', b'\n', &bytes[i..])` to jump directly to the next interesting byte, skipping all regular string content at 16–32 bytes/cycle. Bare `\r` is checked in the gap between matches (extremely rare in practice).

**Trade-offs:** Must handle escape sequences carefully — after `\`, the next byte must also be skipped (it could be `"` or `\` itself). Must preserve the existing error reporting behavior for unterminated strings and unescaped newlines. Used `memchr3` (3 needles) instead of `memchr4` since the crate only supports up to 3.

**Est. impact:** HIGH — `lex_string` is the only lexer scanning function not yet using byte-scanning.

### Benchmark results (B20)

#### Schema parsing

| Fixture       | Before     | After      | Delta   |
|---------------|------------|------------|---------|
| small         | 27.67 µs   | 27.75 µs   | ~0%     |
| medium        | 1.591 ms   | 1.546 ms   | -2.8%   |
| large         | 8.233 ms   | 7.917 ms   | -3.8%   |
| starwars      | 35.07 µs   | 33.89 µs   | -3.3%   |
| github        | 8.614 ms   | 8.191 ms   | -4.9%   |
| shopify_admin | 15.86 ms   | 15.30 ms   | -3.6%   |

#### Executable parsing

| Fixture            | Before     | After      | Delta   |
|--------------------|------------|------------|---------|
| simple_query       | 1.820 µs   | 1.786 µs   | -1.7%   |
| complex_query      | 29.39 µs   | 29.21 µs   | -0.6%   |
| nested_depth_10    | 6.193 µs   | 6.080 µs   | -2.0%   |
| nested_depth_30    | 18.73 µs   | 18.53 µs   | -1.1%   |
| many_operations_50 | 115.5 µs   | 112.2 µs   | -2.8%   |

#### Lexer throughput

| Fixture       | Before     | After      | Throughput   |
|---------------|------------|------------|--------------|
| small         | 7.168 µs   | 7.147 µs   | ~315 MiB/s   |
| medium        | 336.2 µs   | 338.8 µs   | ~298 MiB/s   |
| large         | 1.558 ms   | 1.590 ms   | ~300 MiB/s   |
| starwars      | 9.526 µs   | 9.511 µs   | ~417 MiB/s   |
| github        | 2.216 ms   | 2.172 ms   | ~537 MiB/s   |
| shopify_admin | 3.996 ms   | 3.959 ms   | ~782 MiB/s   |

#### Cross-parser comparison

| Fixture       | libgraphql | graphql-parser | apollo-parser |
|---------------|------------|----------------|---------------|
| small         | **29.6 µs** | 44.2 µs       | 46.1 µs       |
| medium        | **1.68 ms** | 1.97 ms        | 2.09 ms       |
| large         | **8.75 ms** | 9.11 ms        | 9.83 ms       |
| starwars      | **34.7 µs** | 49.2 µs       | 54.3 µs       |
| github        | **8.51 ms** | 8.56 ms        | 12.1 ms       |
| shopify_admin | **15.0 ms** | 16.8 ms        | 24.8 ms       |
| simple query  | **1.77 µs** | 2.86 µs       | 2.96 µs       |
| complex query | **28.3 µs** | 39.0 µs       | 38.1 µs       |

**Verdict:** Clear improvement. Schema parsing improved 2.8-4.9% on description-heavy schemas (medium through shopify_admin). Executable parsing improved 1.7-2.8%. libgraphql-parser now leads graphql-parser on the github schema (8.51ms vs 8.56ms), closing the last remaining competitive gap.

---

## B21: `is_name_continue_byte()` lookup table [MEDIUM-HIGH]

**Status:** Completed
**Priority:** MEDIUM-HIGH
**File:** `src/token_source/str_to_graphql_token_source.rs`

**Problem:** `is_name_continue_byte(b: u8) -> bool` uses `b == b'_' || b.is_ascii_alphanumeric()` which expands to multiple range checks. Called on every byte of every name in `lex_name()`'s tight loop. Names are the most frequent token type.

**Suggested fix:** Replace with a 256-byte `const` lookup table for O(1) branchless classification.

**Trade-offs:** 256 bytes of static data in the binary. Trivial cost — fits in a single L1 cache line pair and stays hot across the entire parse.

**Est. impact:** MEDIUM-HIGH — `lex_name()` is one of the lexer's hottest paths.

### Benchmark results (B21)

#### Lexer throughput (primary impact)

| Fixture       | Before (B20) | After (B21) | Delta   |
|---------------|--------------|-------------|---------|
| small         | 7.147 µs     | 7.042 µs    | -1.4%   |
| medium        | 338.8 µs     | 326.8 µs    | -3.2%   |
| large         | 1.590 ms     | 1.504 ms    | -5.4%   |
| starwars      | 9.511 µs     | 9.185 µs    | -3.5%   |
| github        | 2.172 ms     | 2.057 ms    | -5.3%   |
| shopify_admin | 3.959 ms     | 3.727 ms    | -5.9%   |

Throughput: small ~320 MiB/s, medium ~309 MiB/s, large ~317 MiB/s, starwars ~432 MiB/s, github ~568 MiB/s, shopify_admin ~831 MiB/s.

#### Schema parsing (lean mode — most sensitive to lexer perf)

| Fixture       | Before (B20) | After (B21) | Delta   |
|---------------|--------------|-------------|---------|
| small         | 18.74 µs     | 18.40 µs    | -1.8%   |
| medium        | 897.7 µs     | 873.0 µs    | -2.7%   |
| large         | 4.209 ms     | 4.076 ms    | -3.2%   |
| starwars      | 22.51 µs     | 22.07 µs    | -2.0%   |
| github        | 5.592 ms     | 5.491 ms    | -1.8%   |
| shopify_admin | 10.71 ms     | 10.22 ms    | -4.6%   |

#### Cross-parser comparison (after B21)

| Fixture       | libgraphql | graphql-parser | apollo-parser |
|---------------|------------|----------------|---------------|
| small         | **29.3 µs** | 43.5 µs       | 45.4 µs       |
| medium        | **1.63 ms** | 1.91 ms        | 2.01 ms       |
| large         | **8.38 ms** | 8.76 ms        | 9.52 ms       |
| starwars      | **35.2 µs** | 49.0 µs       | 54.0 µs       |
| github        | **8.50 ms** | 8.60 ms        | 12.2 ms       |
| shopify_admin | **14.9 ms** | 16.8 ms        | 24.9 ms       |
| simple query  | **1.79 µs** | 2.85 µs       | 2.94 µs       |
| complex query | **28.8 µs** | 39.0 µs       | 38.1 µs       |

**Verdict:** Excellent improvement. Lexer throughput improved 3-6% across all benchmarks, with shopify_admin reaching 831 MiB/s (+6.2%). Lean schema parsing improved 2-5%. The lookup table eliminates multiple branch instructions per byte in `lex_name()`'s hot loop, replacing them with a single array-indexed load.

---

## B22: `parse_single_line_string()` fast path for no-escape strings [MEDIUM-HIGH]

**Status:** Reverted — no measurable improvement
**Priority:** MEDIUM-HIGH
**File:** `src/token/graphql_token_kind.rs`

**Problem:** `parse_single_line_string()` iterates every character via `chars().peekable()`, pushing each into a `String`. The vast majority of GraphQL strings contain no escape sequences, so a single `memchr` check + `memcpy` would suffice for the common case.

**Suggested fix:** Before the character loop, use `memchr::memchr(b'\\', content.as_bytes())` to check for backslashes. If none found, return `String::from(content)` directly — one allocation, one memcpy, done. When escapes are present, bulk-copy everything before the first backslash via `push_str(&content[..first_escape])`, then start the char-by-char loop only from the first escape onward.

**Trade-offs:** The `memchr` scan does useful work in both paths: in the fast path it confirms no escapes exist; in the slow path its result drives the bulk prefix copy. No wasted work in either case.

**Est. impact:** MEDIUM-HIGH — affects schema parsing benchmarks for description-heavy schemas.

**Benchmark results (vs post-B21 baseline):**

Two variants tested:

1. **Unconditional memchr** (memchr on all strings): Showed 1.5–4.8% regressions across lean parsers and lexer. The memchr setup cost exceeded savings for typical short GraphQL strings. Lexer regressions (which don't call this function) indicated code layout / I-cache effects from the larger function body.

2. **Length-guarded memchr** (`content.len() > 32`): Extracted long-string path into separate function to reduce code layout impact. Results were mixed/neutral vs baseline — schema_parse showed small improvements on some inputs (-1.0% to -1.8%) but schema_parse_lean showed +1.2% to +3.0% regressions. Deltas were within run-to-run variance (the same benchmark duplicated via compare_schema_parse disagreed with schema_parse on direction of change).

**Verdict:** Reverted. The optimization's theoretical benefit doesn't materialize in practice because: (a) most GraphQL strings in benchmark fixtures are short (<32 bytes), and (b) the compiler already optimizes the simple char-by-char loop effectively. The memchr setup overhead dominates for short strings, and for long strings the improvement is lost in noise.

---

## B23: `skip_whitespace()` lookup table [LOW-MEDIUM]

**Status:** Reverted — regression
**Priority:** LOW-MEDIUM
**File:** `src/token_source/str_to_graphql_token_source.rs`

**Problem:** `skip_whitespace()` uses a 4-way `match` plus a BOM check on every byte. Called at the start of every lexer loop iteration. Already byte-scanning (from B2), but the match could be replaced with a lookup table to reduce branching.

**Suggested fix:** Use a 256-byte `const WHITESPACE_TABLE` for the main whitespace bytes. BOM handling stays as a special case (0xEF leadbyte is rare).

**Trade-offs:** The compiler may already optimize the current match into a similar form. This optimization may show no measurable improvement.

**Est. impact:** LOW-MEDIUM — called very frequently but processes few bytes per call.

**Benchmark results (vs post-B21 baseline, benchmark stopped early — regressions clear):**

| Category | Benchmark | Delta vs B21 |
|----------|-----------|-------------|
| schema_parse | large | -1.3% |
| schema_parse | github | -1.7% |
| executable_parse | simple | **+3.4%** ⚠️ |
| executable_parse | complex | **+1.6%** |
| executable_parse_lean | simple | **+5.0%** ⚠️ |
| executable_parse_lean | complex | **+3.5%** ⚠️ |
| executable_parse_lean | nested_10 | **+3.2%** ⚠️ |
| executable_parse_lean | many_ops | **+2.0%** |
| lexer | github | **+1.5%** |
| lexer | shopify_admin | **+1.5%** |

**Verdict:** Reverted. The compiler already optimizes the 4-way `match` on ASCII byte values into efficient code (likely a comparison chain or small jump table). Replacing it with a 256-byte lookup table added an extra memory indirection that hurt performance. The regression pattern — worse on executable documents (which have more whitespace-delimited tokens relative to their size) and on large lexer inputs — confirms the lookup table is slower than the compiler's native match optimization for this small set of 4 byte values.
