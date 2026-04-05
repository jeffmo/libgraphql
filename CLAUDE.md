# libgraphql - Project Conventions

## Project Overview
`libgraphql` is a comprehensive suite of GraphQL libraries and tools for building tools, clients, and servers that need to validate, interpret, execute, or manipulate GraphQL schemas and operations. The project follows the GraphQL specification (September 2025 edition).

## Repository Structure

### Workspace Organization
The project is organized as a Cargo workspace with four crates:

1. **libgraphql** (`/crates/libgraphql/`) - Main public API crate that re-exports core functionality
2. **libgraphql-core** (`/crates/libgraphql-core/`) - Core implementation with schema, operation, and type system logic
3. **libgraphql-parser** (`/crates/libgraphql-parser/`) - A highly performant GraphQL &str -> AST parser
4. **libgraphql-macros** (`/crates/libgraphql-macros/`) - Procedural macros for compile-time schema validation

## Coding Conventions

### Rust Style

When making updates or changes to existing code, if you observe violations of
any of these styling rules: Please fix them as part of your change unless
explicitly asked not to.

#### Line Length
- All lines of code should fit within 100 columns unless doing so is impossible or unreasonably less legible
- When 100 columns is not achievable, stay as close to 100 columns as possible
- Always put full `if` or `match` keywords + condition productions (up to and
  including the first `{`) on a single line UNLESS doing so would violate the
  100-col line rule. If the only thing pushing an `if` or `match` statement
  across multiple lines is the opening `{`: Keep the opening `{` on the same
  line anyway and consider this a rare exception to the 100-col rule. For example:
- Similarly if a function, its params, and its return type all fit on a single
  100-char line: Always put them all on a single line. If the opening `{` of the function is the only thing
  that pushes the function's "prelude" to 2 lines, just put the `{` at the end of the line anyway (even though it pushes
  over the 100-char column limit).

```rust
// Good
if some_object.should_do_something() && !override {
  do_the_thing();
} else {
  dont_do_the_thing();
}

// Good
if some_object.should_do_something() 
    && !override 
    && also_this_other_thing
    && also_this_other_thing_too {
  do_the_thing();
} else {
  dont_do_the_thing();
}

// Bad
if some_object.should_do_something() 
    && !override 
{
  do_the_thing();
} else {
  dont_do_the_thing();
}
```

#### Naming Patterns
- **Builders:** `TypeBuilder`, `SchemaBuilder`, `QueryBuilder` - Follow builder pattern
- **References:** `NamedXyzRef` - Named references to items
- **Validators:** `XyzValidator` - Validation logic (typically private)
- **File names:** Match primary struct/enum name (e.g., `schema_builder.rs` contains `SchemaBuilder`)

#### Import Statements
- **IMPORTANT:** Never use compound `use` statements with curly braces
- Always import one symbol per line for clarity and consistency
- **IMPORTANT:** Always sort all `use` statements alphabetically
- Never use `super` in an import statement. Always use a `crate`-rooted module path for importing local modules.
- Example (correct):
  ```rust
  use crate::ast;
  use crate::operation::ExecutableDocumentBuilder;
  use crate::operation::FragmentRegistryBuilder;
  use crate::schema::Schema;
  use crate::schema::SchemaBuilder;
  use std::fs;
  use std::path::Path;
  use std::path::PathBuf;
  ```
- Example (incorrect):
  ```rust
  use crate::operation::{ExecutableDocumentBuilder, FragmentRegistryBuilder};
  use std::path::Path;
  use crate::ast;
  use super::some_module;
  ```

#### Match Expressions
- All cases in a match-expression should end with a comma -- including the last
  case and cases wrapped in `{}`

#### Enum Definitions
- All variant definitions in an enum definition should be sorted alphabetically

#### Error Handling
- Use `thiserror::Error` derive macro for custom error types
- Define enum-based error types with detailed variants
- Create `Result<T>` type alias: `type Result<T> = std::result::Result<T, MyError>;`
- Track source location information in errors for better diagnostics
- Use `#[error(...)]` attributes for error messages

Common error types:
- `SchemaBuildError`
- `QueryBuildError`, `MutationBuildError`, `SubscriptionBuildError`
- `FragmentBuildError`
- `TypeValidationError`
- `SelectionSetBuildError`
- `ExecutableDocumentBuildError`

#### Code Style Rules
- When calling a function with a literal boolean argument (`true` or `false`), always prefix the boolean literal with an inline comment that clarifies the name of the parameter in the form of `foo(/* should_skip_lines = */ true)`
- Never place an opening `{` or `(` on its own line -- always place it at the
  end of the previous line (as a same-line continuation of the tokens it
  represents an "opening" for)

#### Module Organization
- Separate files for each major type/builder
- Private implementation modules with public API exports in `mod.rs`
- Use builder pattern for complex objects to ensure type safety
- Place tests in a `tests` submodule adjacent to the code being tested. Use `#[cfg(test)]` annotations in `mod.rs` to conditionally build/link `tests` submodules.

#### Documentation
- Write comprehensive rustdoc comments with usage examples
- Link to the latest GraphQL specification when referencing spec rules (e.g. a version of the spec no older than https://spec.graphql.org/September2025/)
- Include doctests for public API examples
- Document error cases and validation rules

### Shell Script Conventions

- **IMPORTANT:** All functions defined within `scripts/_include.sh` must be ordered alphabetically
- Use snake_case for function names
- Write error messages to stderr with Unicode symbols for clarity
- Return 0 for success, 1 for failure
- Support cross-platform package managers (brew, apt, dnf, pacman, zypper)
- Include platform detection (macOS, Linux, Windows)

## Testing Conventions
- All newly-added or updated unit tests must include a clear, well-structured, English description of what the test aims to verify/validate and, if/when applicable, a link to the relevant portion(s) of the latest version of the GraphQL specification related to what the test is verifying/validating.
- All tests written by Claude should indicate in the aforementioned comment that the test was "Written by Claude Code, reviewed by a human."

### Test Organization
- Place tests in a `tests` submodule adjacent to the code being tested. Use `#[cfg(test)]` annotations in `mod.rs` to conditionally build/link `tests` submodules.
- Organize test modules in `tests/` subdirectory per module
- Use naming convention: `*_tests.rs` for test files
- Use `#[test]` attribute for test functions

### Test Locations
- `/crates/libgraphql-core/src/types/tests/` - Type system tests
- `/crates/libgraphql-core/src/schema/tests/` - Schema validation tests
- `/crates/libgraphql-macros/src/tests/` - Procedural macro tests
- `/crates/libgraphql/src/tests/` - Integration tests

### Running Tests
```bash
cargo test                           # Run all tests
cargo test --package libgraphql-core # Test specific package
cargo clippy --tests                 # Run linter
cargo check --tests                  # Quick compilation check
```

## Development Workflows

### Building and Documentation
```bash
cargo build                          # Build all crates
cargo doc --open                     # Generate and view documentation
cargo build --release                # Release build
```

### Test Coverage
```bash
./scripts/generate-test-coverage-report.sh
```

### Crate Version Management
```bash
./scripts/bump-version.sh            # Bump crate version numbers
```

### Project Tracking Docs
- Make project tracker docs extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each project tracker doc, write a list of unresolved questions to answer (if any)

### Commit Hygiene

- **Clean builds per commit**: Every commit should pass `cargo check --tests` (compilation), `cargo clippy --tests` (lint), and `cargo test` (tests) cleanly. When this isn't achievable (e.g. mid-refactor introducing a new module that doesn't exist yet), note the reason in the commit message body.
- **Frequent, reasonably-sized commits**: Commit after completing each logical unit of work — a new type builder, a set of related validation rules, a complete test module. Don't batch an entire phase into one commit. The commit history should tell the story of how the project was built.
- **Push periodically**: Push to the PR branch after each commit or small group of related commits so progress is visible in the PR.
- **Tests required for all changes**: Every code change and bug fix must be accompanied by unit tests and/or regression tests that verify the new or modified behavior. This is non-negotiable — untested code is incomplete code. New features need tests proving they work; bug fixes need tests that would have caught the bug. If existing test coverage is insufficient for the area you're modifying, add tests for the existing behavior before changing it.

### Commit Message Format

- **Summary line**: Clear, descriptive, imperative mood (e.g. "Add directive validation for repeatable directives"). Under 72 characters.
- **Body**: Thorough and well-articulated. Explain:
  - **Why** this change was made (the motivation, not just "added file X")
  - **Architectural decisions** — what approach was chosen and why
  - **GraphQL specification reasoning** where applicable (e.g. why a particular validation rule exists, which spec section it implements)
  - **Trade-offs** considered
  - NOT superficial details like file names or line numbers — the diff shows those

### Pre-commit Verification

Before each commit, run `/pre-commit` (or manually run these steps):
1. `cargo check --tests` — compilation passes
2. `cargo clippy --tests -- -D warnings` — no clippy warnings
3. `cargo test` — all tests pass

### PR Review Subscription

After creating a pull request, always run `/github-pr-autosubscribe` to subscribe to review activity on the PR. This enables the session to automatically receive and address reviewer comments — understanding feedback, making fixes, running the graphql-rust-reviewer agent, verifying with `/pre-commit`, and responding on GitHub.

### Code Review Cycle (Every ~3 Commits + Final)

After approximately every 3 commits, pause implementation and run a review. Also run a final review after all commits are complete (if one was not already run right after the last commit):
1. Use the `graphql-rust-reviewer` agent for changes touching any `.rs` files in `crates/`
2. Identify actionable findings — bugs, style violations, missing edge cases, spec compliance issues
3. Fix issues in a separate commit (e.g. "Address code review: fix directive validation edge case") before continuing

### Session Planning Docs (Optional)

For plan-driven sessions, you may offer to save the planning document as a date-prefixed `.md` file under `docs/`. **Always ask the user before creating a planning doc** — not every session needs one.

If the user agrees:
- Use the naming pattern `docs/YYYY-MM-DD.TOPIC-PLAN.md` (e.g. `docs/2026-04-04.DIRECTIVE-VALIDATION-PLAN.md`)
- Specify the associated PR number at the top (e.g. `> **PR:** #42 — title`)
- Link to the doc from the PR summary
- **Update the plan doc as you go** with any deviations from the original plan

### Guidance Capture

When the user provides direction, feedback, or establishes a pattern during a session, consider whether it represents a **reusable convention** that should be encoded into the project's Claude configuration (CLAUDE.md, `.claude/commands/`, etc.) so future sessions inherit it automatically. If so, ask the user: "This seems like guidance that could apply to future sessions too — should I add it to CLAUDE.md / a command definition?"

Examples of guidance worth capturing:
- Workflow rules ("always run a final review", "update the plan doc as you go")
- Code conventions not already in CLAUDE.md
- Review or testing requirements
- Documentation patterns
