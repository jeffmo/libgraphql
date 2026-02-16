//! Utilities for testing AST structure in parser tests.
//!
//! This module provides helper functions for extracting specific AST nodes
//! from parsed GraphQL documents, making tests more readable and reducing
//! boilerplate code.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

// =============================================================================
// Document-level Extraction - Executable Documents
// =============================================================================

/// Parse source and extract the first query operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a query.
pub(super) fn extract_query(source: &str) -> legacy_ast::operation::Query {
    let doc = parse_executable(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::operation::Definition::Operation(
            legacy_ast::operation::OperationDefinition::Query(q),
        )) => q,
        other => panic!("Expected Query operation, got: {other:?}"),
    }
}

/// Parse source and extract the first mutation operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a mutation.
pub(super) fn extract_mutation(source: &str) -> legacy_ast::operation::Mutation {
    let doc = parse_executable(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::operation::Definition::Operation(
            legacy_ast::operation::OperationDefinition::Mutation(m),
        )) => m,
        other => panic!("Expected Mutation operation, got: {other:?}"),
    }
}

/// Parse source and extract the first subscription operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a subscription.
pub(super) fn extract_subscription(source: &str) -> legacy_ast::operation::Subscription {
    let doc = parse_executable(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::operation::Definition::Operation(
            legacy_ast::operation::OperationDefinition::Subscription(s),
        )) => s,
        other => panic!("Expected Subscription operation, got: {other:?}"),
    }
}

/// Parse source and extract the first fragment definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a fragment.
pub(super) fn extract_fragment(source: &str) -> legacy_ast::operation::FragmentDefinition {
    let doc = parse_executable(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::operation::Definition::Fragment(f)) => f,
        other => panic!("Expected FragmentDefinition, got: {other:?}"),
    }
}

/// Parse source and extract the shorthand selection set (anonymous query).
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a selection set.
pub(super) fn extract_selection_set(source: &str) -> legacy_ast::operation::SelectionSet {
    let doc = parse_executable(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::operation::Definition::Operation(
            legacy_ast::operation::OperationDefinition::SelectionSet(ss),
        )) => ss,
        other => panic!("Expected SelectionSet (shorthand query), got: {other:?}"),
    }
}

// =============================================================================
// Document-level Extraction - Schema Documents
// =============================================================================

/// Parse schema source and extract the first type definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a type definition.
pub(super) fn extract_first_type_def(source: &str) -> legacy_ast::schema::TypeDefinition {
    let doc = parse_schema(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::schema::Definition::TypeDefinition(td)) => td,
        other => panic!("Expected TypeDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first object type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an object type.
pub(super) fn extract_first_object_type(source: &str) -> legacy_ast::schema::ObjectType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::Object(obj) => obj,
        other => panic!("Expected ObjectType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first interface type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an interface type.
pub(super) fn extract_first_interface_type(source: &str) -> legacy_ast::schema::InterfaceType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::Interface(iface) => iface,
        other => panic!("Expected InterfaceType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first enum type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an enum type.
pub(super) fn extract_first_enum_type(source: &str) -> legacy_ast::schema::EnumType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::Enum(e) => e,
        other => panic!("Expected EnumType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first union type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a union type.
pub(super) fn extract_first_union_type(source: &str) -> legacy_ast::schema::UnionType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::Union(u) => u,
        other => panic!("Expected UnionType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first input object type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an input type.
pub(super) fn extract_first_input_object_type(source: &str) -> legacy_ast::schema::InputObjectType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::InputObject(io) => io,
        other => panic!("Expected InputObjectType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first scalar type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a scalar type.
pub(super) fn extract_first_scalar_type(source: &str) -> legacy_ast::schema::ScalarType {
    match extract_first_type_def(source) {
        legacy_ast::schema::TypeDefinition::Scalar(s) => s,
        other => panic!("Expected ScalarType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first directive definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a directive def.
pub(super) fn extract_first_directive_def(source: &str) -> legacy_ast::schema::DirectiveDefinition {
    let doc = parse_schema(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::schema::Definition::DirectiveDefinition(dd)) => dd,
        other => panic!("Expected DirectiveDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first schema definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a schema def.
pub(super) fn extract_schema_def(source: &str) -> legacy_ast::schema::SchemaDefinition {
    let doc = parse_schema(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::schema::Definition::SchemaDefinition(sd)) => sd,
        other => panic!("Expected SchemaDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first type extension.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a type extension.
pub(super) fn extract_first_type_extension(source: &str) -> legacy_ast::schema::TypeExtension {
    let doc = parse_schema(source).into_valid_ast().unwrap();
    match doc.definitions.into_iter().next() {
        Some(legacy_ast::schema::Definition::TypeExtension(te)) => te,
        other => panic!("Expected TypeExtension, got: {other:?}"),
    }
}

// =============================================================================
// Selection Set Helpers
// =============================================================================

/// Extract the first Field from a SelectionSet.
///
/// # Panics
/// Panics if the selection set is empty or the first item is not a Field.
pub(super) fn first_field(ss: &legacy_ast::operation::SelectionSet) -> &legacy_ast::operation::Field {
    match ss.items.first() {
        Some(legacy_ast::operation::Selection::Field(f)) => f,
        other => panic!("Expected first selection to be Field, got: {other:?}"),
    }
}

/// Extract field at index from SelectionSet.
///
/// # Panics
/// Panics if index is out of bounds or item at index is not a Field.
pub(super) fn field_at(ss: &legacy_ast::operation::SelectionSet, idx: usize) -> &legacy_ast::operation::Field {
    match ss.items.get(idx) {
        Some(legacy_ast::operation::Selection::Field(f)) => f,
        other => panic!(
            "Expected selection at index {idx} to be Field, got: {other:?}"
        ),
    }
}

/// Extract the first FragmentSpread from SelectionSet.
///
/// # Panics
/// Panics if no FragmentSpread is found in the selection set.
pub(super) fn first_fragment_spread(
    ss: &legacy_ast::operation::SelectionSet,
) -> &legacy_ast::operation::FragmentSpread {
    for item in &ss.items {
        if let legacy_ast::operation::Selection::FragmentSpread(fs) = item {
            return fs;
        }
    }
    panic!("No FragmentSpread found in selection set");
}

/// Extract the first InlineFragment from SelectionSet.
///
/// # Panics
/// Panics if no InlineFragment is found in the selection set.
pub(super) fn first_inline_fragment(
    ss: &legacy_ast::operation::SelectionSet,
) -> &legacy_ast::operation::InlineFragment {
    for item in &ss.items {
        if let legacy_ast::operation::Selection::InlineFragment(inf) = item {
            return inf;
        }
    }
    panic!("No InlineFragment found in selection set");
}

// =============================================================================
// Value Helpers
// =============================================================================

/// Extract the first argument value from a Field.
///
/// # Panics
/// Panics if the field has no arguments.
pub(super) fn first_arg_value(field: &legacy_ast::operation::Field) -> &legacy_ast::Value {
    match field.arguments.first() {
        Some((_, value)) => value,
        None => panic!("Field has no arguments"),
    }
}

// =============================================================================
// Type Helpers
// =============================================================================

/// Get the inner type name from a Type, stripping NonNull/List wrappers.
pub(super) fn inner_type_name(ty: &legacy_ast::operation::Type) -> &str {
    match ty {
        legacy_ast::operation::Type::NamedType(name) => name.as_str(),
        legacy_ast::operation::Type::NonNullType(inner) => inner_type_name(inner),
        legacy_ast::operation::Type::ListType(inner) => inner_type_name(inner),
    }
}

