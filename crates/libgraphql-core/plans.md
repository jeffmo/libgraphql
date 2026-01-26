# libgraphql-core — Consolidated Plans & Remaining Work

**Last Updated:** 2026-01-22

This document consolidates all remaining work for the `libgraphql-core` crate.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 167 tests passing, 1 doc-test passing (2 ignored)

**Core Implementation: ✅ Functional**
- Schema building and validation (SchemaBuilder, Schema)
- Operation building (QueryBuilder, MutationBuilder, SubscriptionBuilder)
- Fragment system (FragmentBuilder, FragmentRegistry)
- Type system (object, interface, union, enum, scalar, input types)
- Directive annotations and validation

**Remaining Work Categories:**
1. Directive Validation (Section 1)
2. Operation Validation (Section 2)
3. Code Organization (Section 3)

---

## Section 1: Directive Validation

### 1.1 Non-Repeatable Directive Enforcement

**Purpose:** GraphQL spec requires non-repeatable directives appear at most once per location. Currently not enforced in builders.

**Current Progress:** TODOs exist but validation not implemented.

**Priority:** MEDIUM

#### Tasks

1. **FragmentBuilder directive validation**
   - `fragment_builder.rs:28`: Error if non-repeatable directive added twice

2. **OperationBuilder directive validation**
   - `operation_builder.rs:68`: Error if non-repeatable directive added twice

3. **ScalarTypeBuilder directive validation**
   - `scalar_type_builder.rs:37`: Non-repeatable directives must not be repeated

### Definition of Done
- [ ] All builders reject duplicate non-repeatable directives
- [ ] Clear error messages with source locations
- [ ] Tests verify rejection behavior

---

## Section 2: Operation Validation

### 2.1 Field Selection Uniqueness

**Purpose:** Ensure field selections within a selection set are unambiguously unique per spec.

**Current Progress:** TODO exists at `selection_set_builder.rs:72`.

**Priority:** MEDIUM

#### Tasks

1. **Implement uniqueness check in SelectionSetBuilder**
   - Assert all field selections are unambiguously unique
   - Handle alias vs field name combinations

### Definition of Done
- [ ] SelectionSetBuilder validates field uniqueness
- [ ] Proper error for ambiguous selections

---

### 2.2 OperationBuilder Field Validation

**Purpose:** Validate fields exist on types and root operations use correct types.

**Current Progress:** TODOs in test files indicate validation not yet implemented.

**Priority:** HIGH

#### Tasks

1. **Field existence validation**
   - `executable_document_builder_tests.rs:792`: OperationBuilder needs field validation

2. **Root field validation**
   - `executable_document_builder_tests.rs:823`: OperationBuilder needs root field validation

### Definition of Done
- [ ] Fields validated against schema types
- [ ] Root operations validated against schema root types
- [ ] Clear error messages for invalid fields

---

### 2.3 Fragment Spread Cycle Detection

**Purpose:** Prevent fragment spreads that create cycles.

**Current Progress:** TODO at `fragment_builder.rs:45` mentions verifying no cycles.

**Priority:** MEDIUM

#### Tasks

1. **Verify no fragment-spreads create cycles**
   - Detect direct and indirect cycles
   - Provide helpful error messages showing cycle path

### Definition of Done
- [ ] Cycle detection implemented
- [ ] Tests for direct and indirect cycles

---

### 2.4 Operation Builder Test Coverage

**Purpose:** Add tests asserting certain invalid states are impossible to construct.

**Current Progress:** TODOs exist for mutation and subscription builders.

**Priority:** LOW

#### Tasks

1. **Mutation builder test**
   - `mutation.rs:57`: Test impossible to build invalid mutation

2. **Subscription builder test**
   - `subscription.rs:56`: Test impossible to build invalid subscription

### Definition of Done
- [ ] Tests verify type-safe construction prevents invalid states

---

## Section 3: Code Organization

### 3.1 Value Module Refactoring

**Purpose:** Move value-related functionality to appropriate location.

**Current Progress:** TODO at `value.rs:32`.

**Priority:** LOW

#### Tasks

1. **Move function to OperationsBuilder**
   - Refactor private function placement

### Definition of Done
- [ ] Code relocated appropriately
- [ ] No public API changes

---

## Priority Summary

**HIGH Priority:**
- OperationBuilder field validation (Section 2.2) — core validation gap

**MEDIUM Priority:**
- Non-repeatable directive enforcement (Section 1.1)
- Field selection uniqueness (Section 2.1)
- Fragment spread cycle detection (Section 2.3)

**LOW Priority:**
- Operation builder test coverage (Section 2.4)
- Value module refactoring (Section 3.1)

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

*(No items yet)*

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated 2026-01-22):

| File                                        | Line | TODO                                                |
|---------------------------------------------|------|-----------------------------------------------------|
| `operation/fragment_builder.rs`             |   28 | Error if non-repeatable directive added twice       |
| `operation/fragment_builder.rs`             |   45 | Verify no fragment-spread cycles                    |
| `operation/fragment_builder.rs`             |  150 | Handle def_location changes                         |
| `operation/mutation.rs`                     |   57 | Test asserting impossible to build invalid mutation |
| `operation/operation_builder.rs`            |   68 | Error if non-repeatable directive added twice       |
| `operation/selection_set_builder.rs`        |   72 | Assert field selections unambiguously unique        |
| `operation/subscription.rs`                 |   56 | Test asserting impossible to build invalid sub      |
| `tests/executable_document_builder_tests.rs`|  792 | OperationBuilder needs field validation             |
| `tests/executable_document_builder_tests.rs`|  823 | OperationBuilder needs root field validation        |
| `types/input_object_type_validator.rs`      |   98 | Reduce duplicate error iteration                    |
| `types/scalar_type_builder.rs`              |   37 | Non-repeatable directives must not be repeated      |
| `value.rs`                                  |   32 | Move to private function on OperationsBuilder       |
