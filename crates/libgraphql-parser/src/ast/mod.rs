//! Custom AST types for representing parsed GraphQL documents.
//!
//! This module provides a comprehensive, zero-copy AST for GraphQL
//! documents. All node types are parameterized over a `'src` lifetime
//! that borrows strings from the source text via [`Cow<'src, str>`].
//!
//! The AST has two conceptual layers:
//!
//! - **Semantic layer** (always present): Typed structs with names,
//!   values, directives, and all GraphQL semantics. Every node carries
//!   a [`GraphQLSourceSpan`] for source location tracking.
//!
//! - **Syntax layer** (optional): Each node has an
//!   `Option<XyzSyntax<'src>>` field that, when populated, contains
//!   keyword/punctuation tokens with their trivia (whitespace,
//!   comments, commas). This enables lossless source reconstruction
//!   for formatter and IDE tooling.
//!
//! # Example
//!
//! ```rust,ignore
//! use libgraphql_parser::GraphQLParser;
//!
//! let source = "type Query { hello: String }";
//! let parser = GraphQLParser::new(source);
//! let result = parser.parse_schema_document();
//! let doc = result.output;
//! ```
//!
//! [`Cow<'src, str>`]: std::borrow::Cow
//! [`GraphQLSourceSpan`]: crate::GraphQLSourceSpan

mod ast_node;
mod delimiter_pair;
mod directive_location;
mod document;
mod executable_defs;
mod name;
mod operation_type;
mod shared_nodes;
mod type_annotation;
mod type_extensions;
mod type_system_defs;
mod values;

#[cfg(test)]
mod tests;

pub use ast_node::AstNode;
pub use delimiter_pair::DelimiterPair;
pub use directive_location::DirectiveLocation;
pub use directive_location::DirectiveLocationKind;
pub use directive_location::DirectiveLocationSyntax;
pub use document::Definition;
pub use document::Document;
pub use document::DocumentSyntax;
pub use executable_defs::Field;
pub use executable_defs::FieldSyntax;
pub use executable_defs::FragmentDefinition;
pub use executable_defs::FragmentDefinitionSyntax;
pub use executable_defs::FragmentSpread;
pub use executable_defs::FragmentSpreadSyntax;
pub use executable_defs::InlineFragment;
pub use executable_defs::InlineFragmentSyntax;
pub use executable_defs::OperationDefinition;
pub use executable_defs::OperationDefinitionSyntax;
pub use executable_defs::Selection;
pub use executable_defs::SelectionSet;
pub use executable_defs::SelectionSetSyntax;
pub use executable_defs::VariableDefinition;
pub use executable_defs::VariableDefinitionSyntax;
pub use name::Name;
pub use name::NameSyntax;
pub use operation_type::OperationKind;
pub use shared_nodes::Argument;
pub use shared_nodes::ArgumentSyntax;
pub use shared_nodes::DirectiveAnnotation;
pub use shared_nodes::DirectiveAnnotationSyntax;
pub use shared_nodes::EnumValueDefinition;
pub use shared_nodes::FieldDefinition;
pub use shared_nodes::FieldDefinitionSyntax;
pub use shared_nodes::InputValueDefinition;
pub use shared_nodes::InputValueDefinitionSyntax;
pub use shared_nodes::TypeCondition;
pub use shared_nodes::TypeConditionSyntax;
pub use type_annotation::ListTypeAnnotation;
pub use type_annotation::ListTypeAnnotationSyntax;
pub use type_annotation::NamedTypeAnnotation;
pub use type_annotation::Nullability;
pub use type_annotation::TypeAnnotation;
pub use type_extensions::EnumTypeExtension;
pub use type_extensions::EnumTypeExtensionSyntax;
pub use type_extensions::InputObjectTypeExtension;
pub use type_extensions::InputObjectTypeExtensionSyntax;
pub use type_extensions::InterfaceTypeExtension;
pub use type_extensions::InterfaceTypeExtensionSyntax;
pub use type_extensions::ObjectTypeExtension;
pub use type_extensions::ObjectTypeExtensionSyntax;
pub use type_extensions::ScalarTypeExtension;
pub use type_extensions::ScalarTypeExtensionSyntax;
pub use type_extensions::SchemaExtension;
pub use type_extensions::SchemaExtensionSyntax;
pub use type_extensions::TypeExtension;
pub use type_extensions::UnionTypeExtension;
pub use type_extensions::UnionTypeExtensionSyntax;
pub use type_system_defs::DirectiveDefinition;
pub use type_system_defs::DirectiveDefinitionSyntax;
pub use type_system_defs::EnumTypeDefinition;
pub use type_system_defs::EnumTypeDefinitionSyntax;
pub use type_system_defs::InputObjectTypeDefinition;
pub use type_system_defs::InputObjectTypeDefinitionSyntax;
pub use type_system_defs::InterfaceTypeDefinition;
pub use type_system_defs::InterfaceTypeDefinitionSyntax;
pub use type_system_defs::ObjectTypeDefinition;
pub use type_system_defs::ObjectTypeDefinitionSyntax;
pub use type_system_defs::RootOperationTypeDefinition;
pub use type_system_defs::RootOperationTypeDefinitionSyntax;
pub use type_system_defs::ScalarTypeDefinition;
pub use type_system_defs::ScalarTypeDefinitionSyntax;
pub use type_system_defs::SchemaDefinition;
pub use type_system_defs::SchemaDefinitionSyntax;
pub use type_system_defs::TypeDefinition;
pub use type_system_defs::UnionTypeDefinition;
pub use type_system_defs::UnionTypeDefinitionSyntax;
pub use values::BooleanValue;
pub use values::BooleanValueSyntax;
pub use values::EnumValue;
pub use values::EnumValueSyntax;
pub use values::FloatValue;
pub use values::FloatValueSyntax;
pub use values::IntValue;
pub use values::IntValueSyntax;
pub use values::ListValue;
pub use values::ListValueSyntax;
pub use values::NullValue;
pub use values::NullValueSyntax;
pub use values::ObjectField;
pub use values::ObjectFieldSyntax;
pub use values::ObjectValue;
pub use values::ObjectValueSyntax;
pub use values::StringValue;
pub use values::StringValueSyntax;
pub use values::Value;
pub use values::VariableValue;
pub use values::VariableValueSyntax;
