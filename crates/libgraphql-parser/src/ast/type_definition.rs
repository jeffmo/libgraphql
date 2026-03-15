use crate::ast::AstNode;
use crate::ast::EnumTypeDefinition;
use crate::ast::InputObjectTypeDefinition;
use crate::ast::InterfaceTypeDefinition;
use crate::ast::Name;
use crate::ast::ObjectTypeDefinition;
use crate::ast::ScalarTypeDefinition;
use crate::ast::StringValue;
use crate::ast::UnionTypeDefinition;
use crate::ByteSpan;
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

    pub fn name_value(&self) -> &str {
        self.name().value.as_ref()
    }

    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Enum(def) => def.span,
            Self::InputObject(def) => def.span,
            Self::Interface(def) => def.span,
            Self::Object(def) => def.span,
            Self::Scalar(def) => def.span,
            Self::Union(def) => def.span,
        }
    }
}

#[inherent]
impl AstNode for TypeDefinition<'_> {
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
}
