# plans.md Format Reference

Template and conventions for `plans.md` files in the libgraphql project.

## Document Structure

```markdown
# [crate-name] — Consolidated Plans & Remaining Work

**Last Updated:** YYYY-MM-DD

This document consolidates all remaining work for the `[crate-name]` crate.
[Optional: note about superseding other docs]

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** [X tests passing, Y doc-tests passing]

**Core Implementation: [STATUS]**
- [Brief bullet points of what's implemented]

**Remaining Work Categories:**
1. [Category Name] (Section 1)
2. [Category Name] (Section 2)
...

---

## Section 1: [Category Name]

### 1.1 [Item Title]

**Purpose:** [Why this matters]

**Current Progress:** [What's done so far]

**Priority:** HIGH | MEDIUM | LOW

[Optional: **Depends on:** Section X.Y]

#### Tasks

1. **[Task name]**
   - Detail
   - Detail

2. **[Task name]**
   - Detail

#### Code References (if applicable)
- `file.rs:123`: Brief note

### Definition of Done
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

---

## Section 2: [Next Category]

### 2.1 [Item Title]
...

---

## Priority Summary

**HIGH Priority:**
- [Item title] (Section X.Y) — [brief reason]

**MEDIUM Priority:**
- [Item title] (Section X.Y)

**LOW Priority:**
- [Item title] (Section X.Y)

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

### [Item Title] (YYYY-MM-DD)
[One-line description of what was completed]

### [Item Title] (YYYY-MM-DD)
[One-line description]

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated):

| File | Line | TODO |
|------|------|------|
| `file.rs` | 123 | Brief description |
| `file.rs` | 456 | Brief description |
```

## Section Numbering Rules

- Sections are numbered 1, 2, 3, etc.
- Items within sections are numbered X.1, X.2, X.3, etc.
- **NEVER renumber** — if 2.3 is completed, the next new item is 2.7 (or the next unused number)
- This ensures external references to "Section 2.3" remain valid

## Priority Levels

- **HIGH:** Security-critical, blocking other work, or core functionality gaps
- **MEDIUM:** Important for completeness, enables downstream work
- **LOW:** Nice-to-have, optimization, polish

## Completion Workflow

When wholly completing an item:

1. Check all "Definition of Done" boxes
2. Move to "Past Completed Work" section
3. Format: `### [Title] (YYYY-MM-DD)` + one-line description
4. Update "Last Updated" date at top

When partially completing:

1. Update "Current Progress" section
2. Revise remaining tasks
3. Check completed "Definition of Done" boxes
4. Update "Last Updated" date

## Code TODOs Appendix

The appendix table should be regenerated whenever updating the plans.md file. Format:

```markdown
| File | Line | TODO |
|------|------|------|
| `graphql_parser.rs` | 407 | Reduce clone overhead |
```

- File paths are relative to the crate root
- Line numbers should be current (regenerate when updating)
- TODO text should be concise — truncate if needed
