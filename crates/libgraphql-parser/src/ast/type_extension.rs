use crate::ast::AstNode;
use crate::ast::EnumTypeExtension;
use crate::ast::InputObjectTypeExtension;
use crate::ast::InterfaceTypeExtension;
use crate::ast::Name;
use crate::ast::ObjectTypeExtension;
use crate::ast::ScalarTypeExtension;
use crate::ast::UnionTypeExtension;
use crate::ByteSpan;
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
            Self::Enum(ext) => ext.span,
            Self::InputObject(ext) => ext.span,
            Self::Interface(ext) => ext.span,
            Self::Object(ext) => ext.span,
            Self::Scalar(ext) => ext.span,
            Self::Union(ext) => ext.span,
        }
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
}
