use crate::loc;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct NamedTypeAnnotation {
    pub(crate) nullable: bool,
    pub(crate) type_ref: NamedGraphQLTypeRef,
}

impl NamedTypeAnnotation {
    pub fn graphql_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema GraphQLType {
        self.type_ref.deref(schema).unwrap()
    }

    pub fn graphql_type_name(&self) -> &str {
        self.type_ref.name()
    }

    /// Check if two named type annotations are definitionally equal.
    ///
    /// Two named type annotations are equivalent if they have:
    /// - Same type name
    /// - Same nullability
    ///
    /// Source location is intentionally ignored for semantic comparison.
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.nullable == other.nullable
            && self.type_ref.name() == other.type_ref.name()
    }

    pub fn is_subtype_of(
        &self,
        schema: &Schema,
        other: &Self,
    ) -> bool {
        self.is_subtype_of_impl(&schema.types, other)
    }

    pub(super) fn is_subtype_of_impl(
        &self,
        types_map: &HashMap<String, GraphQLType>,
        other: &Self,
    ) -> bool {
        let self_graphql_type =
            if let Some(type_) = types_map.get(self.type_ref.name()) {
                type_
            } else {
                return false;
            };
        let other_graphql_type =
            if let Some(type_) = types_map.get(other.type_ref.name()) {
                type_
            } else {
                return false;
            };

        match (self_graphql_type, other_graphql_type) {
            (self_t, other_t) if self_t == other_t
                => true,
            (GraphQLType::Interface(self_iface),
             GraphQLType::Interface(other_iface))
                => self_iface.interface_names().contains(&other_iface.name()),
            (GraphQLType::Object(self_obj),
             GraphQLType::Interface(other_iface))
                => self_obj.interface_names().contains(&other_iface.name()),
            (GraphQLType::Object(self_obj),
             GraphQLType::Union(other_union))
                => other_union.member_type_names().contains(&self_obj.name()),
            (_, _) => false,
        }
    }

    pub fn nullable(&self) -> bool {
        self.nullable
    }

    pub fn ref_location(&self) -> &loc::SourceLocation {
        self.type_ref.ref_location()
    }

    pub fn to_graphql_string(&self) -> String {
        format!(
            "{}{}",
            self.type_ref.name(),
            if self.nullable { "?" } else { "" },
        )
    }
}
