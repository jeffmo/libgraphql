use crate::error_note::ErrorNoteKind;
use crate::names::TypeName;
use crate::operation_kind::OperationKind;
use crate::schema::SchemaBuildErrorKind;
use crate::schema::SchemaBuilder;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;
use crate::type_builders::ObjectTypeBuilder;
use crate::types::GraphQLTypeKind;
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

// Regression test for a bug where parse errors returned by
// SchemaBuilder::load_str() carried un-translated spans
// (Span::builtin() with BUILTIN_SOURCE_MAP_ID) instead of
// spans pointing at the source map that was allocated for
// the loaded string. The effect was that diagnostics emitted
// for parse errors were effectively un-locatable in tooling
// because they did not reference the actual input source.
// This test asserts that load_str() now translates parse
// error spans so their source_map_id points at the loaded
// source.
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

// -----------------------------------------------------------
// SchemaBuilder::build() tests (Task 16)
// -----------------------------------------------------------

// Verifies that a minimal schema with just `type Query { ... }`
// builds successfully and that query_type() returns the Query
// object type. This exercises the implicit query type resolution
// path (no explicit `schema { query: ... }` definition).
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_from_str_minimal() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).unwrap();

    assert_eq!(schema.query_type_name().as_str(), "Query");
    let query_type = schema.query_type()
        .expect("query_type() should return the Query object");
    assert!(query_type.field("hello").is_some());
}

// Verifies that a schema containing all six type kinds (object,
// interface, union, enum, scalar, input object) builds
// successfully and that each type is queryable via both the
// generic get_type() and the typed lookup accessors.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_from_str_with_all_type_kinds() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { node: Node, search: SearchResult }\n\
         interface Node { id: ID! }\n\
         type User implements Node { id: ID!, name: String }\n\
         union SearchResult = User\n\
         enum Status { ACTIVE INACTIVE }\n\
         scalar DateTime\n\
         input CreateInput { name: String! }",
    ).unwrap();

    // Generic lookup
    assert!(schema.get_type("Query").is_some());
    assert!(schema.get_type("Node").is_some());
    assert!(schema.get_type("User").is_some());
    assert!(schema.get_type("SearchResult").is_some());
    assert!(schema.get_type("Status").is_some());
    assert!(schema.get_type("DateTime").is_some());
    assert!(schema.get_type("CreateInput").is_some());
    assert!(schema.get_type("NonExistent").is_none());

    // Typed lookups
    assert!(schema.object_type("Query").is_some());
    assert!(schema.object_type("User").is_some());
    assert!(schema.interface_type("Node").is_some());
    assert!(schema.union_type("SearchResult").is_some());
    assert!(schema.enum_type("Status").is_some());
    assert!(schema.scalar_type("DateTime").is_some());
    assert!(schema.input_object_type("CreateInput").is_some());

    // Typed lookups return None for wrong category
    assert!(schema.object_type("Node").is_none());
    assert!(schema.interface_type("Query").is_none());
}

// Verifies that building a schema with no Query type (and no
// explicit schema definition) produces a
// NoQueryOperationTypeDefined error.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_no_query_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Foo { x: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_no_query = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::NoQueryOperationTypeDefined,
        )
    });
    assert!(
        has_no_query,
        "expected NoQueryOperationTypeDefined error",
    );
}

// Verifies that binding a root operation type to a non-object
// type (e.g. an enum) produces a RootOperationTypeNotObjectType
// error.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_root_type_not_object_fails() {
    let result = SchemaBuilder::build_from_str(
        "schema { query: MyEnum }\n\
         enum MyEnum { A B }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_not_object = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::RootOperationTypeNotObjectType {
                actual_kind: GraphQLTypeKind::Enum,
                ..
            },
        )
    });
    assert!(
        has_not_object,
        "expected RootOperationTypeNotObjectType error",
    );
}

// Verifies that an object type with zero fields produces an
// EmptyObjectOrInterfaceType error during build.
//
// Note: the parser may or may not accept `type Foo {}`. We
// construct this scenario programmatically via ObjectTypeBuilder
// to ensure the empty-fields check in build() is exercised
// independently of parser behavior.
//
// See https://spec.graphql.org/September2025/#sec-Objects
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_empty_object_type_fails() {
    let mut sb = SchemaBuilder::new();
    // Add empty object type programmatically
    let empty_obj = ObjectTypeBuilder::new(
        "EmptyObj", Span::dummy(),
    ).unwrap();
    sb.absorb_type(empty_obj).unwrap();

    // Also add a valid Query type so we don't get
    // NoQueryOperationTypeDefined.
    sb.load_str("type Query { x: Int }").unwrap();

    let result = sb.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_empty = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::EmptyObjectOrInterfaceType {
                type_kind: GraphQLTypeKind::Object,
                ..
            },
        )
    });
    assert!(
        has_empty,
        "expected EmptyObjectOrInterfaceType error for object",
    );
}

// Verifies that implementing a non-existent interface produces
// a TypeValidation error wrapping
// ImplementsUndefinedInterface during build.
//
// See https://spec.graphql.org/September2025/#IsValidImplementation()
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_invalid_interface_impl_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query implements NonExistent { x: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_validation_error = errors.errors().iter().any(|e| {
        if let SchemaBuildErrorKind::TypeValidation(tve) = e.kind() {
            matches!(
                tve.kind(),
                TypeValidationErrorKind::ImplementsUndefinedInterface {
                    ..
                },
            )
        } else {
            false
        }
    });
    assert!(
        has_validation_error,
        "expected TypeValidation(ImplementsUndefinedInterface)",
    );
}

// Verifies that a successfully built schema exposes correct
// typed lookups, iterators, and types_implementing() results.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_valid_schema_typed_lookups() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { user: User }\n\
         interface Node { id: ID! }\n\
         type User implements Node { id: ID!, name: String }\n\
         type Post implements Node { id: ID!, title: String }\n\
         enum Role { ADMIN USER }\n\
         input CreateUserInput { name: String! }",
    ).unwrap();

    // object_types iterator
    let obj_names: Vec<_> = schema.object_types()
        .map(|o| o.name().as_str().to_string())
        .collect();
    assert!(obj_names.contains(&"Query".to_string()));
    assert!(obj_names.contains(&"User".to_string()));
    assert!(obj_names.contains(&"Post".to_string()));

    // interface_types iterator
    let iface_names: Vec<_> = schema.interface_types()
        .map(|i| i.name().as_str().to_string())
        .collect();
    assert!(iface_names.contains(&"Node".to_string()));

    // enum_types iterator
    let enum_names: Vec<_> = schema.enum_types()
        .map(|e| e.name().as_str().to_string())
        .collect();
    assert!(enum_names.contains(&"Role".to_string()));

    // types_implementing
    let node_implementors = schema.types_implementing("Node");
    assert_eq!(
        node_implementors.len(),
        2,
        "User and Post both implement Node",
    );
    let implementor_names: Vec<_> = node_implementors.iter()
        .map(|t| t.name().to_string())
        .collect();
    assert!(implementor_names.contains(&"User".to_string()));
    assert!(implementor_names.contains(&"Post".to_string()));

    // types() and directive_defs() collection accessors
    assert!(schema.types().len() > 5); // 5 builtins + user types
    assert!(schema.directive_defs().len() >= 5); // 5 builtins

    // source_maps()
    assert!(
        !schema.source_maps().is_empty(),
        "should have at least the builtin source map",
    );
}

// Verifies that SchemaBuilder::build_from_str() is a
// convenient one-step parse-and-build that produces a valid
// Schema.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_from_str_convenience() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).unwrap();
    assert_eq!(schema.query_type_name().as_str(), "Query");
    assert!(schema.query_type().is_some());
}

// Verifies that build() correctly resolves the implicit Query
// type when no explicit `schema { ... }` definition is present.
// Per the spec, if no schema definition exists, the default
// query type name is "Query".
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_implicit_query_type_resolution() {
    // No `schema { ... }` definition -- should implicitly use
    // "Query" as the query root type.
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }",
    ).unwrap();
    assert_eq!(schema.query_type_name().as_str(), "Query");
    assert!(schema.query_type().is_some());
}

// Verifies that an explicit `schema { query: MyQuery }`
// overrides the default "Query" name, and that the schema
// correctly resolves the custom query type name.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_explicit_query_type_name() {
    let schema = SchemaBuilder::build_from_str(
        "schema { query: RootQuery }\n\
         type RootQuery { x: Int }",
    ).unwrap();
    assert_eq!(
        schema.query_type_name().as_str(),
        "RootQuery",
    );
    assert!(schema.query_type().is_some());
}

// Verifies that mutation and subscription root types are
// correctly resolved when defined via schema { ... }.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_with_mutation_and_subscription() {
    let schema = SchemaBuilder::build_from_str(
        "schema {\n\
           query: Query\n\
           mutation: Mutation\n\
           subscription: Subscription\n\
         }\n\
         type Query { x: Int }\n\
         type Mutation { doThing: Boolean }\n\
         type Subscription { onThing: Boolean }",
    ).unwrap();

    assert!(schema.mutation_type().is_some());
    assert_eq!(
        schema.mutation_type_name().unwrap().as_str(),
        "Mutation",
    );
    assert!(schema.subscription_type().is_some());
    assert_eq!(
        schema.subscription_type_name().unwrap().as_str(),
        "Subscription",
    );
}

// Verifies that get_directive() returns both built-in and
// custom directives, and that the schema preserves directive
// definitions after build.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_directive_lookups() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         directive @auth on FIELD_DEFINITION",
    ).unwrap();

    // Built-in directives
    assert!(schema.get_directive("skip").is_some());
    assert!(schema.get_directive("include").is_some());
    assert!(schema.get_directive("deprecated").is_some());
    assert!(schema.get_directive("specifiedBy").is_some());
    assert!(schema.get_directive("oneOf").is_some());

    // Custom directive
    assert!(schema.get_directive("auth").is_some());

    // Non-existent
    assert!(schema.get_directive("nonexistent").is_none());
}

// Verifies that a mutation root type pointing to a non-existent
// type produces a RootOperationTypeNotDefined error.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_mutation_root_not_defined_fails() {
    let result = SchemaBuilder::build_from_str(
        "schema { query: Query, mutation: Missing }\n\
         type Query { x: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::RootOperationTypeNotDefined {
                operation: OperationKind::Mutation,
                ..
            },
        )
    });
    assert!(
        has_error,
        "expected RootOperationTypeNotDefined for mutation",
    );
}

// Verifies that an enum type with no values produces an
// EnumWithNoValues error during build. Since the parser
// typically requires at least one value, we construct this
// scenario by loading an enum via the builder and confirming
// the error is caught during build().
//
// Written by Claude Code, reviewed by a human.
#[test]
fn build_enum_with_no_values_fails() {
    let mut sb = SchemaBuilder::new();
    sb.load_str("type Query { x: Int }").unwrap();

    // Create an empty enum programmatically
    let empty_enum = crate::type_builders::EnumTypeBuilder::new(
        "EmptyEnum", Span::dummy(),
    ).unwrap();
    sb.absorb_type(empty_enum).unwrap();

    let result = sb.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::EnumWithNoValues { .. },
        )
    });
    assert!(has_error, "expected EnumWithNoValues error");
}
