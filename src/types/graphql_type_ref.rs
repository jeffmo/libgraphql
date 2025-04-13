use crate::ast;
use crate::loc;
use crate::types::NamedGraphQLTypeRef;

/// Represents a reference to a type (e.g. a "type annotation"). These are used
/// to describe the type of a [crate::types::Field],
/// [crate::operation::Variable], field parameter, or directive parameter.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTypeRef {
    List {
        inner_type_ref: Box<GraphQLTypeRef>,
        nullable: bool,
        ref_location: loc::FilePosition,
    },
    Named {
        nullable: bool,
        type_ref: NamedGraphQLTypeRef,
    }
}
impl GraphQLTypeRef {
    pub(crate) fn extract_inner_named_ref(&self) -> &NamedGraphQLTypeRef {
        match self {
            GraphQLTypeRef::List { inner_type_ref, .. }
                => inner_type_ref.extract_inner_named_ref(),
            GraphQLTypeRef::Named { type_ref, .. }
                => type_ref,
        }
    }

    pub(crate) fn from_ast_type(
        ref_location: &loc::FilePosition,
        ast_type: &ast::operation::Type,
    ) -> Self {
        Self::from_ast_type_impl(ref_location, ast_type, /* nullable = */ true)
    }

    fn from_ast_type_impl(
        ref_location: &loc::FilePosition,
        ast_type: &ast::operation::Type,
        nullable: bool,
    ) -> Self {
        match ast_type {
            ast::operation::Type::ListType(inner) =>
                Self::List {
                    inner_type_ref: Box::new(Self::from_ast_type_impl(
                        ref_location,
                        inner,
                        true,
                    )),
                    nullable,
                    ref_location: ref_location.clone(),
                },

            ast::operation::Type::NamedType(name) =>
                Self::Named {
                    nullable,
                    type_ref: NamedGraphQLTypeRef::new(
                        name,
                        ref_location.clone(),
                    ),
                },

            ast::operation::Type::NonNullType(inner) =>
                Self::from_ast_type_impl(ref_location, inner, false),
        }
    }

    pub fn extract_named_type_ref(&self) -> &NamedGraphQLTypeRef {
        match self {
            GraphQLTypeRef::List { inner_type_ref, .. } =>
                inner_type_ref.extract_named_type_ref(),

            GraphQLTypeRef::Named { type_ref, .. } =>
                type_ref,
        }
    }

    pub fn get_ref_location(&self) -> &loc::FilePosition {
        match self {
            GraphQLTypeRef::List { ref_location, .. } => ref_location,
            GraphQLTypeRef::Named { type_ref, .. } => type_ref.get_ref_location(),
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            GraphQLTypeRef::List { nullable, .. } => *nullable,
            GraphQLTypeRef::Named { nullable, .. } => *nullable,
        }
    }
}
