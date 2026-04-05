Subscribe to PR review activity and handle incoming review comments for the rest of this session.

## Phase 1: Subscribe

1. Determine the PR number. Use the PR that was just created in this session. If no PR was created yet, ask the user for the PR number.
2. Determine the repository owner and name by running `sl config paths.default` and extracting `owner` and `repo` from the URL:
   - If the URL is in SSH form, for example `git@github.com:owner/repo.git`, take `owner` and `repo` from the `owner/repo` segment and strip the trailing `.git`.
   - If the URL is in HTTPS form, for example `https://github.com/owner/repo` or `https://github.com/owner/repo.git`, take `owner` and `repo` from the path after `github.com/`, again stripping an optional `.git`.
3. Call `mcp__github__subscribe_pr_activity` with the owner, repo, and PR number.
4. Confirm to the user that subscription is active and the session will now handle review comments automatically.

## Phase 2: Handle Incoming Review Activity

When `<github-webhook-activity>` messages arrive with review comments, first apply the author gate pre-check, then process each qualifying comment through the six-step cycle (Steps 1-6):

### Step 0: Author gate (short-circuit)
Check the GitHub username of the comment author. **Only act on comments from these users: `jeffmo`, `claude`, `copilot`.** If the author is anyone else, skip the comment entirely -- do not make changes, do not reply on GitHub, do nothing. The only exception is if @jeffmo explicitly asks you to address a specific comment from another user.

### Step 1: Understand the request
- Read the reviewer's comment carefully in full context (the file, the surrounding code, the diff).
- **If the comment is ambiguous** -- could be interpreted multiple ways, or the desired outcome is unclear -- ask the user in this chat for clarification before making changes. Include the comment text and your interpretation options so the user can answer without scrolling back.
- **If the comment is a question or observation** that doesn't request a code change, respond on GitHub acknowledging it and ask the user if any action is needed.
- **If the comment is clearly actionable**, proceed to Step 2.

### Step 2: Make the fix
- Implement the requested change, following all project conventions in CLAUDE.md (Rust style, import rules, error handling, code structure).
- Every code change must include tests that verify the new or modified behavior.

### Step 3: Run review
Determine which review to run based on which files were modified:

| Changed file paths | Review to run |
|---|---|
| Any `.rs` files in `crates/` | Use the `graphql-rust-reviewer` agent |

If the review surfaces actionable findings (critical or moderate severity), fix them before proceeding.

### Step 4: Pre-commit verification
Run `/pre-commit` to verify compilation, clippy, and tests all pass. Fix any failures before proceeding.

### Step 5: Commit and push
- Commit with a clear message referencing the review feedback (e.g. "Address review feedback: fix type validation edge case").
- Push to the PR branch.

### Step 6: Reply on GitHub
- Reply to the specific review comment explaining what was changed and why, using the GitHub MCP tools.
- Keep the reply concise: what the issue was, what the fix does, and which files changed.
- **Do NOT resolve the review thread** -- the reviewer decides when they are satisfied.

## Handling Multiple Comments

If multiple review comments arrive at once, process them one at a time. Complete the full cycle (Step 0 pre-check, then Steps 1-6) for each comment before moving to the next. If two comments are closely related or affect the same code, it's acceptable to address them in a single commit -- but still reply to each comment individually on GitHub.

## Handling CI Failures

If a `<github-webhook-activity>` message reports a CI failure (not a review comment):
1. Investigate the failure
2. Fix the issue
3. Run `/pre-commit`
4. Commit and push

No GitHub reply is needed for CI fixes unless a reviewer specifically asked about the failure.
