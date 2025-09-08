use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::Parameter;
use crate::types::TypeAnnotation;
use indexmap::IndexMap;

/// Represents a [field](https://spec.graphql.org/October2021/#FieldDefinition)
/// defined on an [`ObjectType`](crate::types::ObjectType) or
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// *(Note that fields defined on
/// [`InputObjectType`](crate::types::InputObjectType)s are represented by
/// [`InputField`](crate::types::InputField).)*
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub(super) def_location: loc::SourceLocation,
    pub(super) description: Option<String>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) parameters: IndexMap<String, Parameter>,
    pub(super) type_annotation: TypeAnnotation,
}

impl Field {
    /// The [`SourceLocation`](loc::SourceLocation) indicating where this
    /// [`Field`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    /// The description of this [Field`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Field`].
    ///
    /// This list of [`DirectiveAnnotation`]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [`Field`] definition in
    /// the schema. Note that [`DirectiveAnnotation`]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// Indicates whether operations that select this [`Field`] must also
    /// specify a selection set for it.
    ///
    /// For example, in the following [`Query`](crate::operation::Query):
    ///
    ///   ```graphql
    ///   query ExampleQuery {
    ///
    ///     me {
    ///       firstName,
    ///       lastName,
    ///     },
    ///   }
    ///   ```
    ///
    /// The `me` field on the root `Query` type is defined as an
    /// [`ObjectType`](crate::types::ObjectType) which has at least 2 fields of
    /// its own (`firstName` and `lastName`). In GraphQL, an
    /// [operation must always specify a selection set for any object-,
    /// interface-, and union-typed selected fields](https://spec.graphql.org/October2021/#sec-Field-Selections).
    pub fn requires_selection_set(&self, schema: &Schema) -> bool {
        self.type_annotation()
            .innermost_named_type_annotation()
            .graphql_type(schema)
            .requires_selection_set()
    }

    /// The name of this [`Field`].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// A map from ParameterName -> [`Parameter`] for all parameters defined on
    /// this [`Field`].
    ///
    /// This returns an [`IndexMap`] to guarantee that map entries retain the same
    /// ordering as the order of parameters defined on the [`Field`] in the
    /// schema. Note that parameterss added from type extensions will appear in the
    /// order they were specified on the type extension, but there is no
    /// guarantee about where in this list a given type extension's fields will
    /// be added.
    pub fn parameters(&self) -> &IndexMap<String, Parameter> {
        &self.parameters
    }

    /// The [`TypeAnnotation`] specifying the schema-defined type of this [`Field`].
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
