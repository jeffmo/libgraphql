# Snapshot Testing Framework

The snapshot testing framework validates libgraphql against vendored collections of "known good" and "known invalid" GraphQL schemas and operations from real-world sources.

## Quick Start

Snapshot tests run automatically with `cargo test`:

```bash
cargo test --package libgraphql-core
```

To run only snapshot tests:

```bash
cargo test --package libgraphql-core verify_operation_snapshot_tests verify_schema_snapshot_tests
```

## Directory Structure

```
fixtures/
├── valid_schemas/           # Schemas that should validate successfully
│   ├── <suite_name>/
│   │   ├── schema.graphql           # Single-file schema
│   │   ├── *.schema.graphql         # OR multi-file schema
│   │   ├── valid_operations/        # Operations that should validate
│   │   │   └── *.graphql
│   │   └── invalid_operations/      # Operations that should fail
│   │       └── *.graphql
└── invalid_schemas/         # Schemas that should fail validation
    ├── <test_name>.graphql          # Single-file invalid schema
    └── <suite_name>/                # OR multi-file invalid schema
        └── *.schema.graphql
```

## Adding New Test Cases

### Valid Schema Test

Create a directory in `fixtures/valid_schemas/` with either:
- A single `schema.graphql` file, OR
- Multiple `*.schema.graphql` files

**Example:**
```bash
# Create a new schema suite
mkdir -p fixtures/valid_schemas/my_api/

# Add the schema
cat > fixtures/valid_schemas/my_api/schema.graphql << 'EOF'
type Query {
  hello: String!
}
EOF
```

### Invalid Schema Test

Create a GraphQL file in `fixtures/invalid_schemas/` with a descriptive name:

**Example:**
```bash
cat > fixtures/invalid_schemas/missing_query_type.graphql << 'EOF'
# EXPECTED_ERROR_TYPE: NoQueryOperationTypeDefined
type Mutation {
  doSomething: Boolean
}
EOF
```

### Valid Operation Test

Add a `.graphql` file in `fixtures/valid_schemas/<suite>/valid_operations/`:

**Example:**
```bash
cat > fixtures/valid_schemas/my_api/valid_operations/hello_query.graphql << 'EOF'
query {
  hello
}
EOF
```

### Invalid Operation Test

Add a `.graphql` file in `fixtures/valid_schemas/<suite>/invalid_operations/` with an expected error comment:

**Example:**
```bash
cat > fixtures/valid_schemas/my_api/invalid_operations/bad_field.graphql << 'EOF'
# EXPECTED_ERROR_CONTAINS: UndefinedField
query {
  nonExistentField
}
EOF
```

## Expected Error Annotations

Use error annotation comments to specify expected validation errors:

```graphql
# EXPECTED_ERROR_TYPE: UndefinedFieldName
# EXPECTED_ERROR_CONTAINS: nonExistentField
query {
  user {
    nonExistentField
  }
}
```

**Syntax:**
- `# EXPECTED_ERROR_TYPE: <type_name>` - The error type must match `<type_name>` (checks if type appears in Debug output)
- `# EXPECTED_ERROR_CONTAINS: <text>` - The error message must contain `<text>` (substring match)
- Multiple error annotations - ALL patterns must match (order-independent)
- No error annotation comment - Any error is acceptable

**Pattern Matching:**

1. **`EXPECTED_ERROR_TYPE`** - Matches error type names in Debug format:
   - `DuplicateTypeDefinition` matches `DuplicateTypeDefinition { type_name: "User", ... }`
   - `SelectionSetBuildError` matches the error type
   - Case-sensitive

2. **`EXPECTED_ERROR_CONTAINS`** - Matches substrings anywhere in error output:
   - `"User" is defined twice` matches the exact error text
   - `nonExistentField` matches field name in error
   - Case-sensitive

## Fragment Support

Fragments are automatically extracted from **all valid operations** in a test suite and shared across all operations in that suite.

**Example:**
```graphql
fragment UserFields on User {
  id
  name
  email
}

query GetUser($id: ID!) {
  user(id: $id) {
    ...UserFields
  }
}
```

**Important:** Fragment names must be unique within each test suite. A single fragment registry is built from all `valid_operations/*.graphql` files and shared across all operations in the suite.

## Multi-File Schemas

For complex schemas split across multiple files, use the `*.schema.graphql` naming convention:

```
my_schema/
├── types.schema.graphql
├── queries.schema.graphql
├── mutations.schema.graphql
└── valid_operations/
    └── example.graphql
```

All `*.schema.graphql` files are automatically discovered and loaded in lexical order.

## Test Execution

Snapshot tests verify:

1. **Schema Validation** - Valid schemas build successfully, invalid schemas fail
2. **Operation Validation** - Valid operations validate against their schema
3. **Error Patterns** - Invalid operations fail with expected error patterns
4. **Fragment Support** - Fragments are correctly extracted and validated

## Error Reporting

When tests fail, you'll see detailed error messages with:

- File path to the failing test
- Expected vs actual behavior
- Code snippet showing the error location (if applicable)
- Error pattern mismatches (for expected errors)

**Example Output:**
```
❌ my_api/invalid_operations/bad_field.graphql
   File: /path/to/fixtures/valid_schemas/my_api/invalid_operations/bad_field.graphql
   Expected: Should fail validation
   Got: Operation validated successfully (false negative!)

   Expected errors based on comments:
     1 → # EXPECTED_ERROR_CONTAINS: UndefinedField ⚠️ (error not raised!)
     2 │ query {
     3 │   nonExistentField
     4 │ }
```

## Disabling Tests

To temporarily disable a test without deleting it, rename the file with a `.disabled` extension:

```bash
mv my_test.graphql my_test.graphql.disabled
```

Disabled tests are ignored by the test discovery process.

**Use Cases:**
- Tests for validation not yet implemented in libgraphql
- Temporarily broken tests during refactoring
- Tests that need investigation

## Best Practices

1. **Descriptive Names** - Use clear, descriptive names for test files (e.g., `duplicate_type_definition.graphql` not `test1.graphql`)

2. **One Concept Per Test** - Each test should validate one specific concept or error case

3. **Add Comments** - Include comments in GraphQL files explaining what the test validates:
   ```graphql
   # This test validates that duplicate type definitions are rejected
   # EXPECTED_ERROR_TYPE: DuplicateTypeDefinition
   type User { id: ID! }
   type User { name: String }
   ```

4. **Group Related Tests** - Use suite directories to group related schemas and operations

5. **Vendor Real Examples** - When possible, use real-world schemas from actual GraphQL APIs

## Troubleshooting

### Test Not Discovered

- Ensure files follow naming conventions (`*.graphql` or `*.schema.graphql`)
- Check file is in the correct directory
- Verify parent directory exists in `valid_schemas/` or `invalid_schemas/`

### Fragment Registry Errors

- One fragment registry is built per test suite from all valid operations
- Fragment names must be unique within each test suite
- Fragments can be defined in any `valid_operations/*.graphql` file and used in any operation in that suite
- Fragment cycles are not currently detected (validation gap)

### False Negatives

If a test that should fail validation actually passes:
1. Check if the validation is implemented in libgraphql
2. Consider disabling the test with `.disabled` extension
3. Add a comment explaining why it's disabled

### Expected Error Not Matching

- Error patterns are case-sensitive
- Patterns match anywhere in the error message
- Use more specific patterns if getting false matches
- Check actual error output in test failure message

## Contributing

When adding new snapshot tests:

1. Run tests to ensure they pass: `cargo test verify_snapshot`
2. Run clippy to check code quality: `cargo clippy --tests`
3. Add descriptive comments to GraphQL files
4. Update this README if adding new patterns or conventions

## Future Coverage Areas

The following areas are planned for future test coverage:

- [ ] **Schema extensions** - `extend type Query { ... }` and extension validation
- [ ] **Directive validation** - Custom directive errors and constraints
- [ ] **Input object cycles** - Circular input object reference detection (see `circular_input.graphql.disabled`)
- [ ] **Very large schemas** - Performance and memory testing with schemas >10,000 lines
- [ ] **Unicode in field names** - International character handling and validation
- [ ] **Fragment validation** - Fragment cycles, undefined fragments, type mismatches (see `fixtures/*/invalid_operations/*.disabled`)
- [ ] **Operation argument validation** - Missing required arguments, type mismatches (see `missing_required_arg.graphql.disabled`)
- [ ] **Field validation** - Undefined fields, incorrect field selections (see `undefined_field.graphql.disabled`)

To work on any of these areas, find the corresponding `.disabled` test file, implement the validation
in libgraphql, then rename the file to remove the `.disabled` extension.
