use crate::ast::AstNode;
use crate::ast::DirectiveDefinition;
use crate::ast::FragmentDefinition;
use crate::ast::Name;
use crate::ast::OperationDefinition;
use crate::ast::SchemaDefinition;
use crate::ast::SchemaExtension;
use crate::ast::StringValue;
use crate::ast::TypeDefinition;
use crate::ast::TypeExtension;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
use inherent::inherent;

/// A top-level definition in a GraphQL document.
///
/// Covers both type-system definitions (schema, types,
/// directives, extensions) and executable definitions
/// (operations, fragments).
///
/// See
/// [Document](https://spec.graphql.org/September2025/#sec-Document)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum Definition<'src> {
    DirectiveDefinition(DirectiveDefinition<'src>),
    FragmentDefinition(FragmentDefinition<'src>),
    OperationDefinition(OperationDefinition<'src>),
    SchemaDefinition(SchemaDefinition<'src>),
    SchemaExtension(SchemaExtension<'src>),
    TypeDefinition(TypeDefinition<'src>),
    TypeExtension(TypeExtension<'src>),
}

impl<'src> Definition<'src> {
    pub fn as_directive(&self) -> Option<&DirectiveDefinition<'src>> {
        if let Self::DirectiveDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_fragment(&self) -> Option<&FragmentDefinition<'src>> {
        if let Self::FragmentDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_operation(&self) -> Option<&OperationDefinition<'src>> {
        if let Self::OperationDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_schema(&self) -> Option<&SchemaDefinition<'src>> {
        if let Self::SchemaDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_schema_extension(&self) -> Option<&SchemaExtension<'src>> {
        if let Self::SchemaExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_type(&self) -> Option<&TypeDefinition<'src>> {
        if let Self::TypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_type_extension(&self) -> Option<&TypeExtension<'src>> {
        if let Self::TypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    /// Returns the description string for this definition,
    /// if one is present.
    #[inline]
    pub fn description(&self) -> Option<&StringValue<'src>> {
        match self {
            Self::DirectiveDefinition(def) => def.description.as_ref(),
            Self::FragmentDefinition(def) => def.description.as_ref(),
            Self::OperationDefinition(def) => def.description.as_ref(),
            Self::SchemaDefinition(def) => def.description.as_ref(),
            Self::SchemaExtension(_) => None,
            Self::TypeDefinition(def) => def.description(),
            Self::TypeExtension(_) => None,
        }
    }

    /// Returns the directives applied to this definition.
    #[inline]
    pub fn directives(&self) -> &[DirectiveDefinition<'src>] {
        todo!()
    }

    /// Returns the [`Name`] of this definition, or [`None`]
    /// for schema definitions/extensions (which have no name).
    #[inline]
    pub fn name(&self) -> Option<&Name<'src>> {
        match self {
            Self::DirectiveDefinition(def) => Some(&def.name),
            Self::FragmentDefinition(def) => Some(&def.name),
            Self::OperationDefinition(def) => def.name.as_ref(),
            Self::SchemaDefinition(_) => None,
            Self::SchemaExtension(_) => None,
            Self::TypeDefinition(def) => Some(def.name()),
            Self::TypeExtension(def) => Some(def.name()),
        }
    }

    /// Returns the name of this definition as a string slice,
    /// or [`None`] for unnamed definitions (schema
    /// definitions/extensions, anonymous operations).
    ///
    /// Convenience accessor for `self.name().value`.
    #[inline]
    pub fn name_value(&self) -> Option<&str> {
        self.name().map(|n| n.value.as_ref())
    }
}

#[inherent]
impl AstNode for Definition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Definition::DirectiveDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::FragmentDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::OperationDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::SchemaDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::SchemaExtension(d) => {
                d.append_source(sink, source)
            },
            Definition::TypeDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::TypeExtension(d) => {
                d.append_source(sink, source)
            },
        }
    }

    /// Returns this definition's byte-offset span within the
    /// source text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    #[inline]
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::DirectiveDefinition(def) => def.span,
            Self::FragmentDefinition(def) => def.span,
            Self::OperationDefinition(def) => def.span,
            Self::SchemaDefinition(def) => def.span,
            Self::SchemaExtension(def) => def.span,
            Self::TypeDefinition(def) => def.byte_span(),
            Self::TypeExtension(def) => def.byte_span(),
        }
    }

    /// Resolves this definition's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    #[inline]
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
