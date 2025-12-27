# Golden Testing Framework

The golden testing framework validates libgraphql against vendored collections of "known good" and "known invalid" GraphQL schemas and operations from real-world sources.

## Quick Start

Golden tests run automatically with `cargo test`:

```bash
cargo test --package libgraphql-core
```

To run only golden tests:

```bash
cargo test --package libgraphql-core golden_test
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
# EXPECTED_ERROR: NoQueryOperationTypeDefined
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

Add a `.graphql` file in `fixtures/valid_schemas/<suite>/invalid_operations/` with an `EXPECTED_ERROR` comment:

**Example:**
```bash
cat > fixtures/valid_schemas/my_api/invalid_operations/bad_field.graphql << 'EOF'
# EXPECTED_ERROR: UndefinedField
query {
  nonExistentField
}
EOF
```

## Expected Error Annotations

Use `# EXPECTED_ERROR:` comments to specify expected validation errors:

```graphql
# EXPECTED_ERROR: UndefinedFieldName
# EXPECTED_ERROR: TypeMismatch
query {
  user {
    nonExistentField
  }
}
```

**Syntax:**
- `# EXPECTED_ERROR: <pattern>` - The error message must contain `<pattern>`
- Multiple `EXPECTED_ERROR` lines - ALL patterns must match (order-independent)
- No `EXPECTED_ERROR` comment - Any error is acceptable

**Pattern Matching:**
- Matches anywhere in the error message or type name
- Case-sensitive
- Examples:
  - `UndefinedField` matches `UndefinedFieldName { ... }`
  - `SelectionSetBuildError` matches error type
  - `missing_parameter` matches error message text

## Fragment Support

Fragments are automatically extracted from operation files and added to the fragment registry.

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

Each operation file has its own fragment registry built from fragments defined in that file.

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

Golden tests verify:

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
     1 → # EXPECTED_ERROR: UndefinedField ⚠️ (error not raised!)
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
   # EXPECTED_ERROR: DuplicateTypeDefinition
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

- Each operation file builds its own fragment registry
- Fragments must be defined in the same file where they're used
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

When adding new golden tests:

1. Run tests to ensure they pass: `cargo test golden_test`
2. Run clippy to check code quality: `cargo clippy --tests`
3. Add descriptive comments to GraphQL files
4. Update this README if adding new patterns or conventions
