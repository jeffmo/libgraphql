use crate::ast::AstNode;
use crate::ast::EnumTypeDefinition;
use crate::ast::InputObjectTypeDefinition;
use crate::ast::InterfaceTypeDefinition;
use crate::ast::ObjectTypeDefinition;
use crate::ast::ScalarTypeDefinition;
use crate::ast::UnionTypeDefinition;
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
