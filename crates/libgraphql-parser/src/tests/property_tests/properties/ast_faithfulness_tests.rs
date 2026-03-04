//! Property tests verifying structural invariants on parsed ASTs.
//!
//! These tests go beyond "does it parse?" to check that the AST
//! produced by the parser has correct structural properties: valid
//! spans, non-empty required collections, valid names, and correct
//! document kinds.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::ast;
use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;
use crate::GraphQLParserConfig;

/// Checks whether a string is a valid GraphQL name:
/// `[_A-Za-z][_0-9A-Za-z]*`.
fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {},
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Collects all top-level definition names from a document as
/// owned strings. Consumes the document to avoid lifetime issues
/// with the proptest macro.
fn collect_definition_names(doc: ast::Document<'_>) -> Vec<String> {
    let mut names = Vec::new();
    for def in doc.definitions {
        match def {
            ast::Definition::OperationDefinition(op) => {
                if let Some(name) = op.name {
                    names.push(name.value.into_owned());
                }
            },
            ast::Definition::FragmentDefinition(frag) => {
                names.push(frag.name.value.into_owned());
            },
            ast::Definition::TypeDefinition(td) => {
                names.push(type_def_name_owned(td));
            },
            ast::Definition::TypeExtension(te) => {
                names.push(type_ext_name_owned(te));
            },
            ast::Definition::DirectiveDefinition(dd) => {
                names.push(dd.name.value.into_owned());
            },
            ast::Definition::SchemaDefinition(_)
            | ast::Definition::SchemaExtension(_) => {},
        }
    }
    names
}

/// Gets the name of a type definition as an owned String.
fn type_def_name_owned(td: ast::TypeDefinition<'_>) -> String {
    match td {
        ast::TypeDefinition::Enum(e) => e.name.value.into_owned(),
        ast::TypeDefinition::InputObject(io) => io.name.value.into_owned(),
        ast::TypeDefinition::Interface(i) => i.name.value.into_owned(),
        ast::TypeDefinition::Object(o) => o.name.value.into_owned(),
        ast::TypeDefinition::Scalar(s) => s.name.value.into_owned(),
        ast::TypeDefinition::Union(u) => u.name.value.into_owned(),
    }
}

/// Gets the name of a type extension as an owned String.
fn type_ext_name_owned(te: ast::TypeExtension<'_>) -> String {
    match te {
        ast::TypeExtension::Enum(e) => e.name.value.into_owned(),
        ast::TypeExtension::InputObject(io) => io.name.value.into_owned(),
        ast::TypeExtension::Interface(i) => i.name.value.into_owned(),
        ast::TypeExtension::Object(o) => o.name.value.into_owned(),
        ast::TypeExtension::Scalar(s) => s.name.value.into_owned(),
        ast::TypeExtension::Union(u) => u.name.value.into_owned(),
    }
}

/// Checks whether any definitions in the document are executable.
fn has_executable_defs(doc: &ast::Document<'_>) -> bool {
    doc.definitions.iter().any(|def| {
        matches!(
            def,
            ast::Definition::OperationDefinition(_)
                | ast::Definition::FragmentDefinition(_),
        )
    })
}

/// Checks whether any definitions in the document are type-system.
fn has_type_system_defs(doc: &ast::Document<'_>) -> bool {
    doc.definitions.iter().any(|def| {
        matches!(
            def,
            ast::Definition::TypeDefinition(_)
                | ast::Definition::TypeExtension(_)
                | ast::Definition::SchemaDefinition(_)
                | ast::Definition::SchemaExtension(_)
                | ast::Definition::DirectiveDefinition(_),
        )
    })
}

proptest! {
    #![proptest_config(proptest_config())]

    /// Verifies that all names in parsed schema documents are valid
    /// GraphQL identifiers matching `[_A-Za-z][_0-9A-Za-z]*`.
    ///
    /// This tests that the parser correctly identifies name tokens
    /// and doesn't accidentally include adjacent punctuation or
    /// whitespace in name values.
    ///
    /// See [Names](https://spec.graphql.org/September2025/#Name).
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_doc_names_are_valid(source in arb_schema_document(4)) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assert!(
            !result.has_errors(),
            "Generated schema document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let names = collect_definition_names(result.into_ast());
        for name in &names {
            prop_assert!(
                is_valid_name(name),
                "Invalid name '{}' in schema document.\nSource:\n{}",
                name,
                source,
            );
        }
    }

    /// Verifies that all names in parsed executable documents are
    /// valid GraphQL identifiers.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_doc_names_are_valid(source in arb_executable_document(4)) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assert!(
            !result.has_errors(),
            "Generated executable document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let names = collect_definition_names(result.into_ast());
        for name in &names {
            prop_assert!(
                is_valid_name(name),
                "Invalid name '{}' in executable document.\nSource:\n{}",
                name,
                source,
            );
        }
    }

    /// Verifies that all operation definitions in executable documents
    /// have non-empty selection sets.
    ///
    /// Per the spec, every operation must have at least one selection.
    /// See [SelectionSet](https://spec.graphql.org/September2025/#SelectionSet).
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn operations_have_non_empty_selection_sets(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assume!(!result.has_errors());
        let has_empty = result.into_ast().definitions.iter().any(|def| {
            if let ast::Definition::OperationDefinition(op) = def {
                op.selection_set.selections.is_empty()
            } else {
                false
            }
        });
        prop_assert!(
            !has_empty,
            "Operation has empty selection set.\nSource:\n{}",
            source,
        );
    }

    /// Verifies that schema documents contain no executable definitions
    /// (operations or fragments).
    ///
    /// When parsed as a schema document, the parser should only produce
    /// type-system definitions.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_docs_have_no_executable_definitions(
        source in arb_schema_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assume!(!result.has_errors());
        prop_assert!(
            !has_executable_defs(&result.into_ast()),
            "Schema doc should not contain executable defs.\n\
             Source:\n{}",
            source,
        );
    }

    /// Verifies that executable documents contain no type-system
    /// definitions.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_docs_have_no_type_system_definitions(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assume!(!result.has_errors());
        prop_assert!(
            !has_type_system_defs(&result.into_ast()),
            "Executable doc should not contain type-system defs.\n\
             Source:\n{}",
            source,
        );
    }

    /// Verifies that syntax fields are populated in default mode
    /// (retain_syntax = true).
    ///
    /// In default mode, all `*Syntax` fields should be `Some`,
    /// preserving whitespace and formatting information for lossless
    /// source reconstruction.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn syntax_populated_in_default_mode(source in arb_schema_document(3)) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assume!(!result.has_errors());
        prop_assert!(
            result.into_ast().syntax.is_some(),
            "Document syntax should be populated in default mode.\n\
             Source:\n{}",
            source,
        );
    }

    /// Verifies that syntax fields are NOT populated in lean mode
    /// (retain_syntax = false).
    ///
    /// In lean mode, `*Syntax` fields should be `None` for
    /// performance.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn syntax_not_populated_in_lean_mode(source in arb_schema_document(3)) {
        let result = GraphQLParser::with_config(
            &source,
            GraphQLParserConfig::lean(),
        ).parse_schema_document();
        prop_assume!(!result.has_errors());
        prop_assert!(
            result.into_ast().syntax.is_none(),
            "Document syntax should be None in lean mode.\n\
             Source:\n{}",
            source,
        );
    }

    /// Verifies that enum value definitions do not use reserved names
    /// (`true`, `false`, `null`).
    ///
    /// See [EnumValue](https://spec.graphql.org/September2025/#EnumValue).
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn enum_values_not_reserved_names(source in arb_schema_document(4)) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assume!(!result.has_errors());
        let has_reserved = result.into_ast().definitions.iter().any(|def| {
            if let ast::Definition::TypeDefinition(
                ast::TypeDefinition::Enum(enum_def),
            ) = def {
                enum_def.values.iter().any(|val| {
                    let name = val.name.value.as_ref();
                    name == "true" || name == "false" || name == "null"
                })
            } else {
                false
            }
        });
        prop_assert!(
            !has_reserved,
            "Enum value has reserved name in:\n{}",
            source,
        );
    }
}
