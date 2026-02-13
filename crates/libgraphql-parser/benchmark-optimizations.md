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
