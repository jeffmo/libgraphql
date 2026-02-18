use crate::ast::AstNode;
use crate::ast::EnumTypeExtension;
use crate::ast::InputObjectTypeExtension;
use crate::ast::InterfaceTypeExtension;
use crate::ast::ObjectTypeExtension;
use crate::ast::ScalarTypeExtension;
use crate::ast::UnionTypeExtension;
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
