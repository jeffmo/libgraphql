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
//! // TODO(Phase 3): Update this example once the parser
//! // produces `ast::Document` instead of legacy AST types.
//! // See custom-ast-plan.md, Phase 3: Parser Integration.
//! use libgraphql_parser::GraphQLParser;
//!
//! let source = "type Query { hello: String }";
//! let parser = GraphQLParser::new(source);
//! let result = parser.parse_schema_document();
//! let doc = result.valid_ast().unwrap();
//! ```
//!
//! [`Cow<'src, str>`]: std::borrow::Cow
//! [`GraphQLSourceSpan`]: crate::GraphQLSourceSpan

mod argument;
mod ast_node;
mod boolean_value;
mod delimiter_pair;
mod directive_annotation;
mod directive_definition;
mod directive_location;
mod document;
mod enum_type_definition;
mod enum_type_extension;
mod enum_value;
mod enum_value_definition;
mod field;
mod field_definition;
mod float_value;
mod fragment_definition;
mod fragment_spread;
mod inline_fragment;
mod input_object_type_definition;
mod input_object_type_extension;
mod input_value_definition;
mod int_value;
mod interface_type_definition;
mod interface_type_extension;
mod list_type_annotation;
mod list_value;
mod name;
mod named_type_annotation;
mod null_value;
mod nullability;
mod object_field;
mod object_type_definition;
mod object_type_extension;
mod object_value;
mod operation_definition;
mod operation_type;
mod root_operation_type_definition;
mod scalar_type_definition;
mod scalar_type_extension;
mod schema_definition;
mod schema_extension;
mod selection;
mod selection_set;
mod string_value;
mod type_annotation;
mod type_condition;
mod type_definition;
mod type_extension;
mod union_type_definition;
mod union_type_extension;
mod value;
mod variable_definition;
mod variable_value;

#[cfg(test)]
pub(crate) mod tests;

pub use argument::Argument;
pub use argument::ArgumentSyntax;
pub use ast_node::AstNode;
pub use boolean_value::BooleanValue;
pub use boolean_value::BooleanValueSyntax;
pub use delimiter_pair::DelimiterPair;
pub use directive_annotation::DirectiveAnnotation;
pub use directive_annotation::DirectiveAnnotationSyntax;
pub use directive_definition::DirectiveDefinition;
pub use directive_definition::DirectiveDefinitionSyntax;
pub use directive_location::DirectiveLocation;
pub use directive_location::DirectiveLocationKind;
pub use directive_location::DirectiveLocationSyntax;
pub use document::Definition;
pub use document::Document;
pub use document::DocumentSyntax;
pub use enum_type_definition::EnumTypeDefinition;
pub use enum_type_definition::EnumTypeDefinitionSyntax;
pub use enum_type_extension::EnumTypeExtension;
pub use enum_type_extension::EnumTypeExtensionSyntax;
pub use enum_value::EnumValue;
pub use enum_value::EnumValueSyntax;
pub use enum_value_definition::EnumValueDefinition;
pub use field::Field;
pub use field::FieldSyntax;
pub use field_definition::FieldDefinition;
pub use field_definition::FieldDefinitionSyntax;
pub use float_value::FloatValue;
pub use float_value::FloatValueSyntax;
pub use fragment_definition::FragmentDefinition;
pub use fragment_definition::FragmentDefinitionSyntax;
pub use fragment_spread::FragmentSpread;
pub use fragment_spread::FragmentSpreadSyntax;
pub use inline_fragment::InlineFragment;
pub use inline_fragment::InlineFragmentSyntax;
pub use input_object_type_definition::InputObjectTypeDefinition;
pub use input_object_type_definition::InputObjectTypeDefinitionSyntax;
pub use input_object_type_extension::InputObjectTypeExtension;
pub use input_object_type_extension::InputObjectTypeExtensionSyntax;
pub use input_value_definition::InputValueDefinition;
pub use input_value_definition::InputValueDefinitionSyntax;
pub use int_value::IntValue;
pub use int_value::IntValueSyntax;
pub use interface_type_definition::InterfaceTypeDefinition;
pub use interface_type_definition::InterfaceTypeDefinitionSyntax;
pub use interface_type_extension::InterfaceTypeExtension;
pub use interface_type_extension::InterfaceTypeExtensionSyntax;
pub use list_type_annotation::ListTypeAnnotation;
pub use list_type_annotation::ListTypeAnnotationSyntax;
pub use list_value::ListValue;
pub use list_value::ListValueSyntax;
pub use name::Name;
pub use name::NameSyntax;
pub use named_type_annotation::NamedTypeAnnotation;
pub use null_value::NullValue;
pub use null_value::NullValueSyntax;
pub use nullability::Nullability;
pub use object_field::ObjectField;
pub use object_field::ObjectFieldSyntax;
pub use object_type_definition::ObjectTypeDefinition;
pub use object_type_definition::ObjectTypeDefinitionSyntax;
pub use object_type_extension::ObjectTypeExtension;
pub use object_type_extension::ObjectTypeExtensionSyntax;
pub use object_value::ObjectValue;
pub use object_value::ObjectValueSyntax;
pub use operation_definition::OperationDefinition;
pub use operation_definition::OperationDefinitionSyntax;
pub use operation_type::OperationKind;
pub use root_operation_type_definition::RootOperationTypeDefinition;
pub use root_operation_type_definition::RootOperationTypeDefinitionSyntax;
pub use scalar_type_definition::ScalarTypeDefinition;
pub use scalar_type_definition::ScalarTypeDefinitionSyntax;
pub use scalar_type_extension::ScalarTypeExtension;
pub use scalar_type_extension::ScalarTypeExtensionSyntax;
pub use schema_definition::SchemaDefinition;
pub use schema_definition::SchemaDefinitionSyntax;
pub use schema_extension::SchemaExtension;
pub use schema_extension::SchemaExtensionSyntax;
pub use selection::Selection;
pub use selection_set::SelectionSet;
pub use selection_set::SelectionSetSyntax;
pub use string_value::StringValue;
pub use string_value::StringValueSyntax;
pub use type_annotation::TypeAnnotation;
pub use type_condition::TypeCondition;
pub use type_condition::TypeConditionSyntax;
pub use type_definition::TypeDefinition;
pub use type_extension::TypeExtension;
pub use union_type_definition::UnionTypeDefinition;
pub use union_type_definition::UnionTypeDefinitionSyntax;
pub use union_type_extension::UnionTypeExtension;
pub use union_type_extension::UnionTypeExtensionSyntax;
pub use value::Value;
pub use variable_definition::VariableDefinition;
pub use variable_definition::VariableDefinitionSyntax;
pub use variable_value::VariableValue;
pub use variable_value::VariableValueSyntax;
