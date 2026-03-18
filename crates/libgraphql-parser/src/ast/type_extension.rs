use crate::ast::AstNode;
use crate::ast::DirectiveAnnotation;
use crate::ast::EnumTypeExtension;
use crate::ast::InputObjectTypeExtension;
use crate::ast::InterfaceTypeExtension;
use crate::ast::Name;
use crate::ast::ObjectTypeExtension;
use crate::ast::ScalarTypeExtension;
use crate::ast::UnionTypeExtension;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A type extension in a GraphQL schema.
///
/// See
/// [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeExtension<'src> {
    Enum(EnumTypeExtension<'src>),
    InputObject(InputObjectTypeExtension<'src>),
    Interface(InterfaceTypeExtension<'src>),
    Object(ObjectTypeExtension<'src>),
    Scalar(ScalarTypeExtension<'src>),
    Union(UnionTypeExtension<'src>),
}

impl<'src> TypeExtension<'src> {
    /// Returns the directives applied to this type extension.
    pub fn directives(
        &self,
    ) -> &[DirectiveAnnotation<'src>] {
        match self {
            Self::Enum(ext) => &ext.directives,
            Self::InputObject(ext) => &ext.directives,
            Self::Interface(ext) => &ext.directives,
            Self::Object(ext) => &ext.directives,
            Self::Scalar(ext) => &ext.directives,
            Self::Union(ext) => &ext.directives,
        }
    }

    /// Returns the [`Name`] of this type extension.
    pub fn name(&self) -> &Name<'src> {
        match self {
            Self::Enum(ext) => &ext.name,
            Self::InputObject(ext) => &ext.name,
            Self::Interface(ext) => &ext.name,
            Self::Object(ext) => &ext.name,
            Self::Scalar(ext) => &ext.name,
            Self::Union(ext) => &ext.name,
        }
    }

    /// Returns the name of this type extension as a string
    /// slice.
    ///
    /// Convenience accessor for `self.name().value`.
    pub fn name_value(&self) -> &str {
        self.name().value.as_ref()
    }
}

#[inherent]
impl AstNode for TypeExtension<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            TypeExtension::Enum(d) => {
                d.append_source(sink, source)
            },
            TypeExtension::InputObject(d) => {
                d.append_source(sink, source)
            },
            TypeExtension::Interface(d) => {
                d.append_source(sink, source)
            },
            TypeExtension::Object(d) => {
                d.append_source(sink, source)
            },
            TypeExtension::Scalar(d) => {
                d.append_source(sink, source)
            },
            TypeExtension::Union(d) => {
                d.append_source(sink, source)
            },
        }
    }

    /// Returns this type extension's byte-offset span within
    /// the source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::Enum(ext) => ext.span,
            Self::InputObject(ext) => ext.span,
            Self::Interface(ext) => ext.span,
            Self::Object(ext) => ext.span,
            Self::Scalar(ext) => ext.span,
            Self::Union(ext) => ext.span,
        }
    }

    /// Resolves this type extension's position to line/column
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
