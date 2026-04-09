use crate::error_note::ErrorNoteKind;
use crate::names::TypeName;
use crate::schema::SchemaBuildErrorKind;
use crate::schema::SchemaBuilder;
use crate::span::Span;
use crate::type_builders::ObjectTypeBuilder;
use crate::types::ScalarKind;

// Verifies that SchemaBuilder::new() pre-seeds the five built-in
// scalar types: Boolean, Float, ID, Int, String.
//
// See https://spec.graphql.org/September2025/#sec-Scalars.Built-in-Scalars
//
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_scalars_seeded() {
    let sb = SchemaBuilder::new();
    let types = sb.types();

    let expected = [
        ("Boolean", ScalarKind::Boolean),
        ("Float", ScalarKind::Float),
        ("ID", ScalarKind::ID),
        ("Int", ScalarKind::Int),
        ("String", ScalarKind::String),
    ];

    for (name, expected_kind) in expected {
        let t = types.get(name)
            .unwrap_or_else(|| {
                panic!("built-in scalar `{name}` not found")
            });
        let scalar = t.as_scalar()
            .unwrap_or_else(|| {
                panic!("`{name}` is not a ScalarType")
            });
        assert_eq!(scalar.kind(), expected_kind);
        assert!(scalar.is_builtin());
    }

    assert_eq!(types.len(), 5);
}

// Verifies that SchemaBuilder::new() pre-seeds the five built-in
// directives: @skip, @include, @deprecated, @specifiedBy, @oneOf.
//
// See https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives
//
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_directives_seeded() {
    let sb = SchemaBuilder::new();
    let defs = sb.directive_defs();

    assert_eq!(defs.len(), 5);

    let skip = defs.get("skip").expect("@skip not found");
    assert!(skip.is_builtin());
    assert!(!skip.is_repeatable());
    assert_eq!(skip.parameters().len(), 1);
    assert!(skip.parameters().contains_key("if"));

    let include = defs.get("include").expect("@include not found");
    assert!(include.is_builtin());
    assert_eq!(include.parameters().len(), 1);
    assert!(include.parameters().contains_key("if"));

    let deprecated = defs.get("deprecated")
        .expect("@deprecated not found");
    assert!(deprecated.is_builtin());
    assert_eq!(deprecated.parameters().len(), 1);
    assert!(deprecated.parameters().contains_key("reason"));
    // Verify default value and nullability.
    // The `reason` parameter must be non-nullable (String!) per
    // the September 2025 spec. A previous bug had it set to
    // nullable (true) instead of non-nullable (false).
    let reason_param = deprecated.parameters().get("reason")
        .expect("reason param not found");
    assert!(reason_param.default_value().is_some());
    assert!(
        !reason_param.type_annotation().nullable(),
        "@deprecated reason must be non-nullable (String!)",
    );

    let specified_by = defs.get("specifiedBy")
        .expect("@specifiedBy not found");
    assert!(specified_by.is_builtin());
    assert_eq!(specified_by.parameters().len(), 1);
    assert!(specified_by.parameters().contains_key("url"));

    let one_of = defs.get("oneOf").expect("@oneOf not found");
    assert!(one_of.is_builtin());
    assert!(one_of.parameters().is_empty());
}

// Verifies that load_str() can parse a simple type definition
// and register it in the schema builder.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_basic() {
    let mut sb = SchemaBuilder::new();
    sb.load_str("type Foo { bar: String }").unwrap();

    let types = sb.types();
    assert!(
        types.contains_key("Foo"),
        "type `Foo` should be registered",
    );
    let foo = types.get("Foo").unwrap();
    assert!(foo.as_object().is_some());
}

// Verifies that loading two types with the same name produces
// a DuplicateTypeDefinition error (the second definition is
// rejected, the first remains). Also verifies the error
// carries a "first defined here" note with a non-None span.
//
// See https://spec.graphql.org/September2025/#sec-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_duplicate_type_rejected() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "type Foo { a: Int }\ntype Foo { b: Int }",
    ).unwrap();

    // The first Foo should still be registered
    assert!(sb.types().contains_key("Foo"));
    let foo = sb.types().get("Foo").unwrap().as_object().unwrap();
    assert!(
        foo.field("a").is_some(),
        "first definition should win",
    );

    // Verify the DuplicateTypeDefinition error
    let dup_errors: Vec<_> = sb.errors().iter().filter(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateTypeDefinition { .. },
        )
    }).collect();
    assert!(
        !dup_errors.is_empty(),
        "expected a DuplicateTypeDefinition error",
    );

    // Verify the "first defined here" note with a span
    let notes = dup_errors[0].notes();
    let first_defined_note = notes.iter().find(|n| {
        n.message == "first defined here"
    });
    assert!(
        first_defined_note.is_some(),
        "expected a 'first defined here' note",
    );
    let note = first_defined_note.unwrap();
    assert_eq!(note.kind, ErrorNoteKind::General);
    assert!(
        note.span.is_some(),
        "expected 'first defined here' note to have a span",
    );
}

// Verifies that a type with a `__` prefix is rejected per the
// GraphQL spec's reserved name rules. With eager error
// reporting, from_ast() returns Err and the type is NOT
// registered -- only the error is accumulated.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_dunder_prefix_rejected() {
    let mut sb = SchemaBuilder::new();
    sb.load_str("type __Bad { x: Int }").unwrap();

    // The type should NOT be registered because from_ast()
    // fails eagerly on the dunder prefix.
    let types = sb.types();
    assert!(
        !types.contains_key("__Bad"),
        "dunder type should not be registered",
    );

    // The error should be accumulated in SchemaBuilder
    let dunder_errors: Vec<_> = sb.errors().iter().filter(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                ..
            },
        )
    }).collect();
    assert!(
        !dunder_errors.is_empty(),
        "expected an InvalidDunderPrefixedTypeName error",
    );
}

// Verifies that a `schema { query: MyQuery }` definition
// correctly binds the query root operation type name.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_schema_definition() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "schema { query: MyQuery }\n\
         type MyQuery { x: Int }",
    ).unwrap();

    let query = sb.query_type_name()
        .expect("query_type_name should be set");
    assert_eq!(query.0.as_str(), "MyQuery");
    assert!(sb.types().contains_key("MyQuery"));
}

// Verifies that loading multiple types in a single load_str
// call registers all of them.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_multiple_types() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "type A { x: Int }\n\
         type B { y: String }\n\
         type C { z: Boolean }",
    ).unwrap();

    let types = sb.types();
    // 5 builtins + 3 user types
    assert_eq!(types.len(), 8);
    assert!(types.contains_key("A"));
    assert!(types.contains_key("B"));
    assert!(types.contains_key("C"));
}

// Verifies that absorb_type() works for programmatically
// constructed builders (not from parsed source).
//
// Written by Claude Code, reviewed by a human.
#[test]
fn absorb_type_programmatic() {
    let mut sb = SchemaBuilder::new();
    let builder = ObjectTypeBuilder::new(
        "MyType", Span::dummy(),
    ).unwrap();
    sb.absorb_type(builder).unwrap();

    assert!(sb.types().contains_key("MyType"));
}

// Verifies that attempting to redefine a built-in directive
// (e.g. @skip) produces a RedefinitionOfBuiltinDirective
// error.
//
// See https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives
//
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_directive_redefinition_rejected() {
    let mut sb = SchemaBuilder::new();
    let result = sb.load_str(
        "directive @skip on FIELD",
    );
    // load_str itself succeeds (errors go to self.errors)
    assert!(result.is_ok());
    // But the directive should not be re-registered --
    // the built-in @skip should remain
    let skip = sb.directive_defs().get("skip").unwrap();
    assert!(skip.is_builtin());
}

// Verifies that load_str() returns &mut Self for method
// chaining, allowing `sb.load_str(...)?.load_str(...)?`.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_chaining() {
    let mut sb = SchemaBuilder::new();
    let result: Result<(), Vec<_>> = (|| {
        sb.load_str("type A { x: Int }")?
            .load_str("type B { y: String }")?;
        Ok(())
    })();
    assert!(result.is_ok());
    assert!(sb.types().contains_key("A"));
    assert!(sb.types().contains_key("B"));
}

// Verifies that load_str() returns an Err containing a
// ParseError when the input is not valid GraphQL syntax.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_parse_error() {
    let mut sb = SchemaBuilder::new();
    let result = sb.load_str("type { broken }");
    assert!(result.is_err());
    let errors = match result {
        Err(errs) => errs,
        Ok(_) => panic!("expected parse error"),
    };
    assert!(matches!(
        errors[0].kind(),
        SchemaBuildErrorKind::ParseError { .. },
    ));
}

// Verifies that two `schema { query: ... }` definitions in
// the same load produce a DuplicateOperationDefinition error
// for the second one while still retaining the first binding.
// Also verifies the error carries a "first defined here" note
// with a non-None span.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_duplicate_root_operation_rejected() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "schema { query: Q1 }\n\
         schema { query: Q2 }\n\
         type Q1 { x: Int }\n\
         type Q2 { x: Int }",
    ).unwrap();

    // The first binding should win
    let query = sb.query_type_name()
        .expect("query_type_name should be set");
    assert_eq!(query.0.as_str(), "Q1");

    // A DuplicateOperationDefinition error should be
    // accumulated
    let dup_errors: Vec<_> = sb.errors().iter().filter(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateOperationDefinition {
                ..
            },
        )
    }).collect();
    assert!(
        !dup_errors.is_empty(),
        "expected a DuplicateOperationDefinition error",
    );

    // Verify the "first defined here" note with a span
    let notes = dup_errors[0].notes();
    let first_defined_note = notes.iter().find(|n| {
        n.message == "first defined here"
    });
    assert!(
        first_defined_note.is_some(),
        "expected a 'first defined here' note",
    );
    let note = first_defined_note.unwrap();
    assert_eq!(note.kind, ErrorNoteKind::General);
    assert!(
        note.span.is_some(),
        "expected 'first defined here' note to have a span",
    );
}

// Verifies duplicate custom directive definitions are rejected.
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_duplicate_custom_directive_rejected() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "directive @auth on FIELD_DEFINITION\n\
         directive @auth on OBJECT",
    ).unwrap();
    let dup_errors: Vec<_> = sb.errors().iter().filter(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateDirectiveDefinition { .. },
        )
    }).collect();
    assert!(!dup_errors.is_empty());
}

// Verifies load_str dispatches all 6 type definition kinds
// correctly through the builder pipeline.
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_all_type_kinds() {
    let mut sb = SchemaBuilder::new();
    sb.load_str(
        "type Query { x: Int }\n\
         interface Node { id: ID! }\n\
         union SearchResult = Query\n\
         enum Status { ACTIVE }\n\
         scalar DateTime\n\
         input CreateInput { name: String! }",
    ).unwrap();
    assert!(sb.types().contains_key(&TypeName::new("Query")));
    assert!(sb.types().contains_key(&TypeName::new("Node")));
    assert!(sb.types().contains_key(&TypeName::new("SearchResult")));
    assert!(sb.types().contains_key(&TypeName::new("Status")));
    assert!(sb.types().contains_key(&TypeName::new("DateTime")));
    assert!(sb.types().contains_key(&TypeName::new("CreateInput")));
}

// Regression test for parse error span translation. Parse
// errors returned by load_str() must carry properly translated
// spans (with a non-zero SourceMapId pointing at the loaded
// source), not Span::builtin() which would make them
// un-locatable in diagnostic output.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn load_str_parse_error_has_proper_span() {
    let mut sb = SchemaBuilder::new();
    let result = sb.load_str("type { broken }");
    assert!(result.is_err());
    let errors = match result {
        Err(errs) => errs,
        Ok(_) => panic!("expected parse error"),
    };
    assert!(!errors.is_empty());
    assert!(matches!(
        errors[0].kind(),
        SchemaBuildErrorKind::ParseError { .. },
    ));
    // The span's source_map_id must NOT be the built-in id
    // (SourceMapId(0)). It should point to the source map
    // created for the loaded string.
    let span = errors[0].span();
    assert_ne!(
        span.source_map_id,
        crate::span::BUILTIN_SOURCE_MAP_ID,
        "parse error span should reference the loaded source, \
        not Span::builtin()",
    );
}
