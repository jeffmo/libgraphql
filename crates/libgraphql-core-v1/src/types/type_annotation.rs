use crate::names::TypeName;
use crate::span::Span;
use crate::types::graphql_type::GraphQLType;
use crate::types::list_type_annotation::ListTypeAnnotation;
use crate::types::named_type_annotation::NamedTypeAnnotation;
use indexmap::IndexMap;

/// A GraphQL
/// [type reference](https://spec.graphql.org/September2025/#sec-Type-References)
/// (type annotation).
///
/// Represents the type of a field, argument, variable, or input
/// field — including nullability and list wrapping. Recursive:
/// `[String!]!` is `List(non-null, Named(non-null, "String"))`.
///
/// # Subtype and equivalence checks
///
/// [`is_equivalent_to()`](Self::is_equivalent_to) checks structural
/// identity (used for parameter type validation per
/// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())).
///
/// [`is_subtype_of()`](Self::is_subtype_of) checks covariant subtyping
/// (used for field return type validation per
/// [IsSubType](https://spec.graphql.org/September2025/#IsSubType())).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum TypeAnnotation {
    List(ListTypeAnnotation),
    Named(NamedTypeAnnotation),
}

impl TypeAnnotation {
    pub fn named(
        type_name: impl Into<TypeName>,
        nullable: bool,
    ) -> Self {
        Self::Named(NamedTypeAnnotation {
            nullable,
            span: Span::builtin(),
            type_name: type_name.into(),
        })
    }

    pub fn list(inner: TypeAnnotation, nullable: bool) -> Self {
        Self::List(ListTypeAnnotation {
            inner: Box::new(inner),
            nullable,
            span: Span::builtin(),
        })
    }

    #[inline]
    pub fn nullable(&self) -> bool {
        match self {
            Self::List(l) => l.nullable(),
            Self::Named(n) => n.nullable(),
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        match self {
            Self::List(l) => l.span(),
            Self::Named(n) => n.span(),
        }
    }

    /// Recursively unwrap list layers and return the innermost
    /// named type annotation.
    pub fn innermost_named(&self) -> &NamedTypeAnnotation {
        match self {
            Self::List(l) => l.inner().innermost_named(),
            Self::Named(n) => n,
        }
    }

    /// The name of the innermost type (convenience for
    /// `self.innermost_named().type_name()`).
    pub fn innermost_type_name(&self) -> &TypeName {
        self.innermost_named().type_name()
    }

    /// Structural equivalence check. Two annotations are
    /// equivalent if they have the same structure, nullability
    /// at every level, and the same innermost type name.
    ///
    /// Source locations are intentionally ignored.
    ///
    /// Useful for things like parameter type validation where
    /// exact type matching is required per
    /// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                a.nullable() == b.nullable()
                    && a.type_name() == b.type_name()
            },
            (Self::List(a), Self::List(b)) => {
                a.nullable() == b.nullable()
                    && a.inner().is_equivalent_to(b.inner())
            },
            _ => false,
        }
    }

    /// Covariant subtype check per
    /// [IsSubType](https://spec.graphql.org/September2025/#IsSubType()).
    ///
    /// `self` is a valid subtype of `other` if it has equal or
    /// stricter nullability and the innermost type is the same
    /// or a subtype (union member, interface implementor).
    pub fn is_subtype_of(
        &self,
        types_map: &IndexMap<TypeName, GraphQLType>,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                if a.type_name() == b.type_name() {
                    return !a.nullable() || b.nullable();
                }
                (!a.nullable() || b.nullable())
                    && Self::is_type_subtype_of(
                        types_map,
                        a.type_name(),
                        b.type_name(),
                    )
            },
            (Self::List(a), Self::List(b)) => {
                (!a.nullable() || b.nullable())
                    && a.inner().is_subtype_of(types_map, b.inner())
            },
            _ => false,
        }
    }

    /// Check if `sub` is a subtype of `super_` in the type
    /// hierarchy. A type is a subtype of itself, or of an
    /// interface it implements, or of a union it is a member of.
    ///
    /// Per [IsSubType](https://spec.graphql.org/September2025/#IsSubType()):
    /// - Step 2 (union membership) requires `sub` to be an Object
    /// - Step 3 (interface implementation) requires `sub` to be
    ///   Object or Interface
    fn is_type_subtype_of(
        types_map: &IndexMap<TypeName, GraphQLType>,
        sub: &TypeName,
        super_: &TypeName,
    ) -> bool {
        if sub == super_ {
            return true;
        }
        let Some(super_type) = types_map.get(super_) else {
            return false;
        };
        match super_type {
            GraphQLType::Interface(_) => {
                let Some(sub_type) = types_map.get(sub) else {
                    return false;
                };
                match sub_type {
                    GraphQLType::Object(obj) => {
                        obj.interfaces().iter().any(|l| &l.value == super_)
                    },
                    GraphQLType::Interface(iface) => {
                        iface.interfaces().iter().any(|l| &l.value == super_)
                    },
                    _ => false,
                }
            },
            GraphQLType::Union(union_type) => {
                matches!(types_map.get(sub), Some(GraphQLType::Object(_)))
                    && union_type.members().iter().any(|m| &m.value == sub)
            },
            _ => false,
        }
    }
}

impl std::fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(n) => write!(
                f, "{}{}",
                n.type_name(),
                if n.nullable() { "" } else { "!" },
            ),
            Self::List(l) => write!(
                f, "[{}]{}",
                l.inner(),
                if l.nullable() { "" } else { "!" },
            ),
        }
    }
}
