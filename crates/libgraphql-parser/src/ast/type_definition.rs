use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::EnumTypeDefinition;
use crate::ast::InputObjectTypeDefinition;
use crate::ast::InterfaceTypeDefinition;
use crate::ast::Name;
use crate::ast::ObjectTypeDefinition;
use crate::ast::ScalarTypeDefinition;
use crate::ast::StringValue;
use crate::ast::UnionTypeDefinition;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A type definition in a GraphQL schema.
///
/// See
/// [Types](https://spec.graphql.org/September2025/#sec-Types)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition<'src> {
    Enum(EnumTypeDefinition<'src>),
    InputObject(InputObjectTypeDefinition<'src>),
    Interface(InterfaceTypeDefinition<'src>),
    Object(ObjectTypeDefinition<'src>),
    Scalar(ScalarTypeDefinition<'src>),
    Union(UnionTypeDefinition<'src>),
}

impl<'src> TypeDefinition<'src> {
    /// Returns the description string for this type definition,
    /// if one is present.
    pub fn description(&self) -> Option<&StringValue<'src>> {
        match self {
            Self::Enum(def) => def.description.as_ref(),
            Self::InputObject(def) => def.description.as_ref(),
            Self::Interface(def) => def.description.as_ref(),
            Self::Object(def) => def.description.as_ref(),
            Self::Scalar(def) => def.description.as_ref(),
            Self::Union(def) => def.description.as_ref(),
        }
    }

    /// Returns the directives applied to this type definition.
    pub fn directives(
        &self,
    ) -> &[DirectiveAnnotation<'src>] {
        match self {
            Self::Enum(def) => &def.directives,
            Self::InputObject(def) => &def.directives,
            Self::Interface(def) => &def.directives,
            Self::Object(def) => &def.directives,
            Self::Scalar(def) => &def.directives,
            Self::Union(def) => &def.directives,
        }
    }

    /// Returns the [`Name`] of this type definition.
    pub fn name(&self) -> &Name<'src> {
        match self {
            Self::Enum(def) => &def.name,
            Self::InputObject(def) => &def.name,
            Self::Interface(def) => &def.name,
            Self::Object(def) => &def.name,
            Self::Scalar(def) => &def.name,
            Self::Union(def) => &def.name,
        }
    }

    /// Returns the name of this type definition as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name().value`.
    pub fn name_value(&self) -> &str {
        self.name().value.as_ref()
    }
}

#[inherent]
impl AstNode for TypeDefinition<'_> {
    /// See [`AstNode::append_source()`](crate::ast::AstNode::append_source).
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            TypeDefinition::Enum(d) => {
                d.append_source(sink, source)
            },
            TypeDefinition::InputObject(d) => {
                d.append_source(sink, source)
            },
            TypeDefinition::Interface(d) => {
                d.append_source(sink, source)
            },
            TypeDefinition::Object(d) => {
                d.append_source(sink, source)
            },
            TypeDefinition::Scalar(d) => {
                d.append_source(sink, source)
            },
            TypeDefinition::Union(d) => {
                d.append_source(sink, source)
            },
        }
    }

    /// Returns this type definition's byte-offset span within
    /// the source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::Enum(def) => def.span,
            Self::InputObject(def) => def.span,
            Self::Interface(def) => def.span,
            Self::Object(def) => def.span,
            Self::Scalar(def) => def.span,
            Self::Union(def) => def.span,
        }
    }

    /// Resolves this type definition's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
