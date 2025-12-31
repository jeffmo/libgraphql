use crate::ast;
use crate::loc;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::ListTypeAnnotation;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;
use std::collections::HashMap;

/// Represents the annotated type for a [`Field`](crate::types::Field),
/// [`Variable`](crate::operation::Variable), or
/// [`Parameter`](crate::types::Parameter).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TypeAnnotation {
    List(ListTypeAnnotation),
    Named(NamedTypeAnnotation),
}
impl TypeAnnotation {
    /// Unwrap the [`ListTypeAnnotation`] if this annotation is one.
    pub fn as_list_annotation(&self) -> Option<&ListTypeAnnotation> {
        if let Self::List(annot) = self {
            Some(annot)
        } else {
            None
        }
    }

    /// Unwrap the [`NamedTypeAnnotation`] if this annotation is one.
    pub fn as_named_annotation(&self) -> Option<&NamedTypeAnnotation> {
        if let Self::Named(annot) = self {
            Some(annot)
        } else {
            None
        }
    }

    /// The [`SourceLocation`](loc::SourceLocation) indicating where this
    /// [`TypeAnnotation`] was defined within the schema.
    pub fn ref_location(&self) -> &loc::SourceLocation {
        match self {
            Self::List(annot) => annot.ref_location(),
            Self::Named(annot) => annot.ref_location(),
        }
    }

    pub fn to_graphql_string(&self) -> String {
        match self {
            Self::Named(named_annot) => named_annot.to_graphql_string(),
            Self::List(list_annot) => list_annot.to_graphql_string(),
        }
    }

    pub(crate) fn from_ast_type(
        src_loc: &loc::SourceLocation,
        ast_type: &ast::operation::Type,
    ) -> Self {
        Self::from_ast_type_impl(src_loc, ast_type, /* nullable = */ true)
    }

    fn from_ast_type_impl(
        location: &loc::SourceLocation,
        ast_type: &ast::operation::Type,
        nullable: bool,
    ) -> Self {
        match ast_type {
            ast::operation::Type::ListType(inner) =>
                Self::List(ListTypeAnnotation {
                    inner_type_ref: Box::new(Self::from_ast_type_impl(
                        location,
                        inner,
                        true,
                    )),
                    nullable,
                    ref_location: location.to_owned(),
                }),

            ast::operation::Type::NamedType(name) =>
                Self::Named(NamedTypeAnnotation {
                    nullable,
                    type_ref: NamedGraphQLTypeRef::new(
                        name,
                        location.clone(),
                    ),
                }),

            ast::operation::Type::NonNullType(inner) =>
                Self::from_ast_type_impl(location, inner, false),
        }
    }

    /// Recursively unwrap this [`TypeAnnotation`] and return the inner-most
    /// [`NamedTypeAnnotation`] from it.
    pub fn innermost_named_type_annotation(&self) -> &NamedTypeAnnotation {
        match self {
            TypeAnnotation::List(ListTypeAnnotation { inner_type_ref, .. })
                => inner_type_ref.innermost_named_type_annotation(),
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

    /// Check if two type annotations are definitionally equal.
    ///
    /// Two type annotations are equivalent if they have:
    /// - Same type structure (Named vs List)
    /// - Same nullability at each level
    /// - Same innermost type name
    ///
    /// Source location is intentionally ignored for semantic comparison.
    ///
    /// This is used for parameter type validation where exact type matching
    /// is required per GraphQL spec section 3.1.2.1 (Type Validation).
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(self_list), Self::List(other_list))
                => self_list.is_equivalent_to(other_list),
            (Self::Named(self_named), Self::Named(other_named))
                => self_named.is_equivalent_to(other_named),
            _ => false, // List vs Named mismatch
        }
    }

    pub fn is_subtype_of(&self, schema: &Schema, other: &Self) -> bool {
        self.is_subtype_of_impl(&schema.types, other)
    }

    pub(super) fn is_subtype_of_impl(
        &self,
        types_map: &HashMap<String, GraphQLType>,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Self::List(self_inner), Self::List(other_inner))
                => self_inner.is_subtype_of_impl(types_map, other_inner),
            (Self::List(_), Self::Named(_))
                => false,
            (Self::Named(self_named), Self::Named(other_named))
                => self_named.is_subtype_of_impl(types_map, other_named),
            (Self::Named(_), Self::List(_))
                => false,
        }
    }

    /// Indicates if this [`TypeAnnotation`] is [nullable or
    /// non-nullable](https://spec.graphql.org/October2021/#sec-Non-Null).
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
impl std::fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(list_annot) => write!(
                f,
                "[{}]{}",
                list_annot.inner_type_annotation(),
                if list_annot.nullable() { "" } else { "!" },
            ),

            Self::Named(named_annot) => write!(
                f,
                "{}{}",
                named_annot.graphql_type_name(),
                if named_annot.nullable() { "" } else { "!" },
            ),
        }
    }
}
