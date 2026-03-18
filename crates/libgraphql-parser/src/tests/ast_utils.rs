//! Utilities for testing AST structure in parser tests.
//!
//! This module provides helper functions for extracting specific AST nodes
//! from parsed GraphQL documents, making tests more readable and reducing
//! boilerplate code.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_schema;

// =============================================================================
// Document-level Extraction - Executable Documents
// =============================================================================

/// Parse source and extract the first query operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a
/// non-shorthand query.
pub(super) fn extract_query(source: &str) -> ast::OperationDefinition<'_> {
    let (doc, _) = parse_executable(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::OperationDefinition(op))
            if op.operation_kind == ast::OperationKind::Query
                && !op.shorthand =>
        {
            op
        },
        other => panic!("Expected non-shorthand Query operation, got: {other:?}"),
    }
}

/// Parse source and extract the first mutation operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a mutation.
pub(super) fn extract_mutation(source: &str) -> ast::OperationDefinition<'_> {
    let (doc, _) = parse_executable(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::OperationDefinition(op))
            if op.operation_kind == ast::OperationKind::Mutation =>
        {
            op
        },
        other => panic!("Expected Mutation operation, got: {other:?}"),
    }
}

/// Parse source and extract the first subscription operation.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a subscription.
pub(super) fn extract_subscription(source: &str) -> ast::OperationDefinition<'_> {
    let (doc, _) = parse_executable(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::OperationDefinition(op))
            if op.operation_kind == ast::OperationKind::Subscription =>
        {
            op
        },
        other => panic!("Expected Subscription operation, got: {other:?}"),
    }
}

/// Parse source and extract the first fragment definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a fragment.
pub(super) fn extract_fragment(source: &str) -> ast::FragmentDefinition<'_> {
    let (doc, _) = parse_executable(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::FragmentDefinition(f)) => f,
        other => panic!("Expected FragmentDefinition, got: {other:?}"),
    }
}

/// Parse source and extract the first shorthand query (anonymous
/// selection set).
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a shorthand
/// query.
pub(super) fn extract_shorthand_query(source: &str) -> ast::OperationDefinition<'_> {
    let (doc, _) = parse_executable(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::OperationDefinition(op)) if op.shorthand => op,
        other => {
            panic!("Expected shorthand query (OperationDefinition with shorthand=true), got: {other:?}")
        },
    }
}

// =============================================================================
// Document-level Extraction - Schema Documents
// =============================================================================

/// Parse schema source and extract the first type definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a type
/// definition.
pub(super) fn extract_first_type_def(source: &str) -> ast::TypeDefinition<'_> {
    let (doc, _) = parse_schema(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::TypeDefinition(td)) => td,
        other => panic!("Expected TypeDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first object type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an object type.
pub(super) fn extract_first_object_type(source: &str) -> ast::ObjectTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::Object(obj) => obj,
        other => panic!("Expected ObjectType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first interface type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an interface
/// type.
pub(super) fn extract_first_interface_type(
    source: &str,
) -> ast::InterfaceTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::Interface(iface) => iface,
        other => panic!("Expected InterfaceType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first enum type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an enum type.
pub(super) fn extract_first_enum_type(source: &str) -> ast::EnumTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::Enum(e) => e,
        other => panic!("Expected EnumType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first union type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a union type.
pub(super) fn extract_first_union_type(source: &str) -> ast::UnionTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::Union(u) => u,
        other => panic!("Expected UnionType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first input object type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not an input type.
pub(super) fn extract_first_input_object_type(
    source: &str,
) -> ast::InputObjectTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::InputObject(io) => io,
        other => panic!("Expected InputObjectType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first scalar type.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a scalar type.
pub(super) fn extract_first_scalar_type(source: &str) -> ast::ScalarTypeDefinition<'_> {
    match extract_first_type_def(source) {
        ast::TypeDefinition::Scalar(s) => s,
        other => panic!("Expected ScalarType, got: {other:?}"),
    }
}

/// Parse schema source and extract the first directive definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a directive
/// def.
pub(super) fn extract_first_directive_def(
    source: &str,
) -> ast::DirectiveDefinition<'_> {
    let (doc, _) = parse_schema(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::DirectiveDefinition(dd)) => dd,
        other => panic!("Expected DirectiveDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first schema definition.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a schema def.
pub(super) fn extract_schema_def(source: &str) -> ast::SchemaDefinition<'_> {
    let (doc, _) = parse_schema(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::SchemaDefinition(sd)) => sd,
        other => panic!("Expected SchemaDefinition, got: {other:?}"),
    }
}

/// Parse schema source and extract the first type extension.
///
/// # Panics
/// Panics if parsing fails or if the first definition is not a type
/// extension.
pub(super) fn extract_first_type_extension(source: &str) -> ast::TypeExtension<'_> {
    let (doc, _) = parse_schema(source).into_valid().unwrap();
    match doc.definitions.into_iter().next() {
        Some(ast::Definition::TypeExtension(te)) => te,
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
pub(super) fn first_field<'a, 'src>(ss: &'a ast::SelectionSet<'src>) -> &'a ast::FieldSelection<'src> {
    match ss.selections.first() {
        Some(ast::Selection::Field(f)) => f,
        other => panic!("Expected first selection to be Field, got: {other:?}"),
    }
}

/// Extract field at index from SelectionSet.
///
/// # Panics
/// Panics if index is out of bounds or item at index is not a Field.
pub(super) fn field_at<'a, 'src>(
    ss: &'a ast::SelectionSet<'src>,
    idx: usize,
) -> &'a ast::FieldSelection<'src> {
    match ss.selections.get(idx) {
        Some(ast::Selection::Field(f)) => f,
        other => panic!(
            "Expected selection at index {idx} to be Field, got: {other:?}"
        ),
    }
}

/// Extract the first FragmentSpread from SelectionSet.
///
/// # Panics
/// Panics if no FragmentSpread is found in the selection set.
pub(super) fn first_fragment_spread<'a, 'src>(
    ss: &'a ast::SelectionSet<'src>,
) -> &'a ast::FragmentSpread<'src> {
    for item in &ss.selections {
        if let ast::Selection::FragmentSpread(fs) = item {
            return fs;
        }
    }
    panic!("No FragmentSpread found in selection set");
}

/// Extract the first InlineFragment from SelectionSet.
///
/// # Panics
/// Panics if no InlineFragment is found in the selection set.
pub(super) fn first_inline_fragment<'a, 'src>(
    ss: &'a ast::SelectionSet<'src>,
) -> &'a ast::InlineFragment<'src> {
    for item in &ss.selections {
        if let ast::Selection::InlineFragment(inf) = item {
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
pub(super) fn first_arg_value<'a, 'src>(field: &'a ast::FieldSelection<'src>) -> &'a ast::Value<'src> {
    match field.arguments.first() {
        Some(arg) => &arg.value,
        None => panic!("Field has no arguments"),
    }
}

// =============================================================================
// Type Helpers
// =============================================================================

/// Get the inner type name from a TypeAnnotation, stripping
/// List wrappers.
pub(super) fn inner_type_name<'a, 'src>(ty: &'a ast::TypeAnnotation<'src>) -> &'a str {
    match ty {
        ast::TypeAnnotation::Named(n) => &n.name.value,
        ast::TypeAnnotation::List(l) => inner_type_name(&l.element_type),
    }
}

// =============================================================================
// Schema Definition Helpers
// =============================================================================

/// Find a root operation type name in a SchemaDefinition by
/// operation kind.
pub(super) fn find_root_op<'a, 'src>(
    sd: &'a ast::SchemaDefinition<'src>,
    kind: ast::OperationKind,
) -> Option<&'a str> {
    sd.root_operations
        .iter()
        .find(|r| r.operation_kind == kind)
        .map(|r| &*r.named_type.value)
}

// =============================================================================
// Object Value Helpers
// =============================================================================

/// Find a field value in an ObjectValue by name.
pub(super) fn object_field_value<'a, 'src>(
    obj: &'a ast::ObjectValue<'src>,
    name: &str,
) -> Option<&'a ast::Value<'src>> {
    obj.fields
        .iter()
        .find(|f| f.name.value == name)
        .map(|f| &f.value)
}
