Run pre-commit verification to ensure code is ready for commit. Execute these checks in order and report results:

1. **Compilation check**: `cargo check --tests`
2. **Lint**: `cargo clippy --tests -- -D warnings`
3. **Tests**: `cargo test`

After all checks:
- If all pass: report success and show which files have uncommitted changes (`sl status`)
- If any check fails: explain the failure clearly, provide the specific error message, suggest fixes, and do NOT attempt to commit
