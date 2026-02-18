use crate::ast::ast_node::append_span_source_slice;
use crate::ast::Argument;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ast::TypeCondition;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

// =========================================================
// Operation definitions
// =========================================================

/// An operation definition (query, mutation, or
/// subscription).
///
/// See
/// [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub operation_kind: OperationKind,
    pub name: Option<Name<'src>>,
    pub variable_definitions:
        Vec<VariableDefinition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax:
        Option<OperationDefinitionSyntax<'src>>,
}

// =========================================================
// Fragment definitions
// =========================================================

/// A named fragment definition.
///
/// See
/// [Fragment Definitions](https://spec.graphql.org/September2025/#sec-Language.Fragments)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub type_condition: TypeCondition<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax:
        Option<FragmentDefinitionSyntax<'src>>,
}

// =========================================================
// Variable definitions
// =========================================================

/// A variable definition within an operation's
/// variable list (e.g. `$id: ID! = "default"`).
///
/// See
/// [Variable Definitions](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub variable: Name<'src>,
    pub var_type: TypeAnnotation<'src>,
    pub default_value: Option<Value<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax:
        Option<VariableDefinitionSyntax<'src>>,
}

// =========================================================
// Selection sets
// =========================================================

/// A selection set — the set of fields and fragments
/// selected within braces `{ ... }`.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'src> {
    pub span: GraphQLSourceSpan,
    pub selections: Vec<Selection<'src>>,
    pub syntax: Option<SelectionSetSyntax<'src>>,
}

/// A single selection within a selection set.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'src> {
    Field(Field<'src>),
    FragmentSpread(FragmentSpread<'src>),
    InlineFragment(InlineFragment<'src>),
}

// =========================================================
// Field
// =========================================================

/// A field selection within a selection set, optionally
/// aliased, with arguments, directives, and a nested
/// selection set.
///
/// See
/// [Fields](https://spec.graphql.org/September2025/#sec-Language.Fields)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Field<'src> {
    pub span: GraphQLSourceSpan,
    pub alias: Option<Name<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: Option<SelectionSet<'src>>,
    pub syntax: Option<FieldSyntax<'src>>,
}

// =========================================================
// Fragment spread
// =========================================================

/// A named fragment spread (`...FragmentName`).
///
/// See
/// [Fragment Spreads](https://spec.graphql.org/September2025/#FragmentSpread)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpread<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FragmentSpreadSyntax<'src>>,
}

// =========================================================
// Inline fragment
// =========================================================

/// An inline fragment (`... on Type { ... }` or
/// `... { ... }`).
///
/// See
/// [Inline Fragments](https://spec.graphql.org/September2025/#InlineFragment)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragment<'src> {
    pub span: GraphQLSourceSpan,
    pub type_condition: Option<TypeCondition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<InlineFragmentSyntax<'src>>,
}

// =========================================================
// Executable definition syntax structs
// =========================================================

/// Syntax detail for an [`OperationDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDefinitionSyntax<'src> {
    /// The operation keyword (`query`, `mutation`,
    /// `subscription`). `None` for shorthand queries.
    pub operation_keyword: Option<GraphQLToken<'src>>,
    pub variable_definition_parens:
        Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`FragmentDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentDefinitionSyntax<'src> {
    pub fragment_keyword: GraphQLToken<'src>,
    pub on_keyword: GraphQLToken<'src>,
}

/// Syntax detail for a [`VariableDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinitionSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

/// Syntax detail for a [`SelectionSet`].
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSetSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

/// Syntax detail for a [`Field`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldSyntax<'src> {
    /// The colon between alias and field name. `None`
    /// when no alias is present.
    pub alias_colon: Option<GraphQLToken<'src>>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`FragmentSpread`].
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpreadSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

/// Syntax detail for an [`InlineFragment`].
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragmentSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

#[inherent]
impl AstNode for OperationDefinition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for FragmentDefinition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for VariableDefinition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for SelectionSet<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for Selection<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Selection::Field(s) => {
                s.append_source(sink, source)
            },
            Selection::FragmentSpread(s) => {
                s.append_source(sink, source)
            },
            Selection::InlineFragment(s) => {
                s.append_source(sink, source)
            },
        }
    }
}

#[inherent]
impl AstNode for Field<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for FragmentSpread<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for InlineFragment<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}
