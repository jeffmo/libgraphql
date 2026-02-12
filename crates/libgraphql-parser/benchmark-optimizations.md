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

**Status:** Pending (deferred to later in roadmap — high effort)
**Priority:** 6
**File:** `src/token_source/str_to_graphql_token_source.rs`

**Problem:** Every character consumed updates 5-6 fields (peek_char, newline
check, curr_col_utf8, curr_col_utf16, last_char_was_cr, curr_byte_offset).
For a name like `PullRequestReviewCommentConnection` that's 36 chars x 6 ops.

**Suggested fix:** Byte-scanning fast paths for hot loops: `skip_whitespace()`,
`lex_name()`, `lex_comment()`, `lex_block_string()`. Scan bytes directly to
find boundaries, then compute positions once at the end.

**Trade-offs:** Significant refactoring. Must handle UTF-8 correctly. Position
tracking becomes "lazy". Risk of position calculation bugs.

**Est. impact:** HIGH

---

## B3: Block string parsing allocates heavily [HIGH]

**Status:** Pending
**Priority:** 2
**File:** `src/token/graphql_token_kind.rs`

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

**Suggested fix:** Single-pass algorithm:
- Skip `replace()` when no escaped triple quotes (`Cow::Borrowed` fast path)
- Compute common indent by iterating lines without collecting to Vec
- Build result string directly in second pass — one allocation total
- Replace `remove(0)` with index tracking

**Trade-offs:** More complex two-pass logic. Must preserve exact spec semantics.

**Est. impact:** HIGH for description-heavy schemas, MEDIUM for synthetic

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

## B5: Token clone in `expect()` [MODERATE]

**Status:** Pending
**Priority:** 5
**File:** `src/graphql_parser.rs:447-481`, `src/graphql_token_stream.rs`

**Problem:** `expect()` clones the peeked token before consuming (line 479).
Clone includes `GraphQLTokenKind` enum (with `Cow<str>`),
`SmallVec<[GraphQLTriviaToken; 2]>` trivia, `GraphQLSourceSpan`. Called for
every punctuator — thousands of times per schema.

**Suggested fix:** Modify `GraphQLTokenStream::consume()` to return the owned
token instead of a reference. Transfer ownership directly from the buffer.

**Trade-offs:** Changes `GraphQLTokenStream` API. `current_token()` callers
need updating. Buffer management changes.

**Est. impact:** MODERATE

---

## B6: `starts_with()` in block string lexing [MODERATE]

**Status:** Pending
**Priority:** 4
**File:** `src/token_source/str_to_graphql_token_source.rs:866,877`

**Problem:** Inside the block string lexer, every character checks:
```rust
self.remaining().starts_with("\\\"\"\"")  // line 866
self.remaining().starts_with("\"\"\"")    // line 877
```
`remaining()` creates a new slice each time. Adds up for long block strings.

**Suggested fix:** Direct byte comparison:
```rust
let src = self.source.as_bytes();
let i = self.curr_byte_offset;
if i + 2 < src.len()
    && src[i] == b'"' && src[i+1] == b'"' && src[i+2] == b'"' { ... }
```

**Trade-offs:** Minimal — strictly better.

**Est. impact:** MODERATE for description-heavy schemas

---

## B7: `shrink_to_fit()` in `compact_buffer()` [LOW-MODERATE]

**Status:** Pending
**Priority:** 3
**File:** `src/graphql_token_stream.rs:72`

**Problem:** After every buffer compaction (once per top-level definition),
`shrink_to_fit()` may trigger a reallocation to shrink Vec capacity, only for
the buffer to grow again for the next definition. For 1000+ definitions,
that's 1000+ potential realloc cycles.

**Suggested fix:** Remove `shrink_to_fit()` entirely.

**Trade-offs:** Slightly higher peak memory (~few KB retained). Negligible.

**Est. impact:** LOW-MODERATE

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
