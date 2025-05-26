use crate::ast;
use crate::loc;
use crate::types::ListTypeAnnotation;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;

/// Represents the annotated type for a [`Field`](crate::types::Field),
/// [`Variable`](crate::operation::Variable), or
/// [`Parameter`](crate::types::Parameter).
#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnnotation {
    List(ListTypeAnnotation),
    Named(NamedTypeAnnotation),
}
impl TypeAnnotation {
    pub fn as_list_annotation(&self) -> Option<&ListTypeAnnotation> {
        if let Self::List(annot) = self {
            Some(annot)
        } else {
            None
        }
    }

    pub fn as_named_annotation(&self) -> Option<&NamedTypeAnnotation> {
        if let Self::Named(annot) = self {
            Some(annot)
        } else {
            None
        }
    }

    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        match self {
            TypeAnnotation::List(ListTypeAnnotation { def_location, .. }) => def_location,
            TypeAnnotation::Named(NamedTypeAnnotation { type_ref, .. }) => type_ref.def_location(),
        }
    }

    pub(crate) fn from_ast_type(
        def_location: &loc::SchemaDefLocation,
        ast_type: &ast::operation::Type,
    ) -> Self {
        Self::from_ast_type_impl(def_location, ast_type, /* nullable = */ true)
    }

    fn from_ast_type_impl(
        def_location: &loc::SchemaDefLocation,
        ast_type: &ast::operation::Type,
        nullable: bool,
    ) -> Self {
        match ast_type {
            ast::operation::Type::ListType(inner) =>
                Self::List(ListTypeAnnotation {
                    inner_type_ref: Box::new(Self::from_ast_type_impl(
                        def_location,
                        inner,
                        true,
                    )),
                    nullable,
                    def_location: def_location.clone(),
                }),

            ast::operation::Type::NamedType(name) =>
                Self::Named(NamedTypeAnnotation {
                    nullable,
                    type_ref: NamedGraphQLTypeRef::new(
                        name,
                        def_location.clone(),
                    ),
                }),

            ast::operation::Type::NonNullType(inner) =>
                Self::from_ast_type_impl(def_location, inner, false),
        }
    }

    pub fn inner_named_type_annotation(&self) -> &NamedTypeAnnotation {
        match self {
            TypeAnnotation::List(ListTypeAnnotation { inner_type_ref, .. })
                => inner_type_ref.inner_named_type_annotation(),
            TypeAnnotation::Named(named_annot)
                => named_annot,
        }
    }

    pub(crate) fn inner_named_type_ref(&self) -> &NamedGraphQLTypeRef {
        match self {
            TypeAnnotation::List(ListTypeAnnotation { inner_type_ref, .. })
                => inner_type_ref.inner_named_type_ref(),
            TypeAnnotation::Named(NamedTypeAnnotation { type_ref, .. })
                => type_ref,
        }
    }

    pub fn nullable(&self) -> bool {
        match self {
            TypeAnnotation::List(ListTypeAnnotation { nullable, .. }) => *nullable,
            TypeAnnotation::Named(NamedTypeAnnotation { nullable, .. }) => *nullable,
        }
    }
}
impl std::convert::From<ListTypeAnnotation> for TypeAnnotation {
    fn from(value: ListTypeAnnotation) -> Self {
        Self::List(value)
    }
}
impl std::convert::From<NamedTypeAnnotation> for TypeAnnotation {
    fn from(value: NamedTypeAnnotation) -> Self {
        Self::Named(value)
    }
}
