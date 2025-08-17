use crate::operation::ExecutableDocumentBuilder;
use crate::operation::FragmentRegistry;
use crate::operation::Operation;
use crate::schema::Schema;

/// Represents a GraphQL
/// ["exectuable document"](https://spec.graphql.org/October2021/#ExecutableDocument).
/// As [described in the GraphQL spec](https://spec.graphql.org/October2021/#sel-EAFPNCAACEB6la):
///
/// > `Document`s are only executable by a GraphQL service if they are
/// > [`ExecutableDocument`] and contain at least one `OperationDefinition`.
/// > A `Document` which contains `TypeSystemDefinitionOrExtension` must not be
/// > executed; GraphQL execution services which receive a `Document` containing
/// > these should return a descriptive error.
///
/// Generally you'll only want to work with [`ExecutableDocument`]s
/// only when you're working with a file that groups multiple operations and/or
/// fragments in one place. If you're only working with a single [`Operation`]
/// or [`NamedFragment`](crate::operation::NamedFragment) at a time, though,
/// you're better off working more directly with those types.
#[derive(Clone, Debug)]
pub struct ExecutableDocument<'schema: 'fragreg, 'fragreg> {
    pub(super) fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
    pub(super) operations: Vec<Operation<'schema, 'fragreg>>,
    pub(super) schema: &'schema Schema,
}

impl<'schema, 'fragreg> ExecutableDocument<'schema, 'fragreg> {
    /// Convenience wrapper around [`ExecutableDocumentBuilder::new()`].
    pub fn builder(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
    ) -> ExecutableDocumentBuilder<'schema, 'fragreg> {
        ExecutableDocumentBuilder::new(schema, fragment_registry)
    }

    pub fn fragment_registry(&self) -> Option<&'fragreg FragmentRegistry<'schema>> {
        self.fragment_registry.to_owned()
    }

    pub fn operations(&self) -> &Vec<Operation<'schema, 'fragreg>> {
        &self.operations
    }

    pub fn schema(&self) -> &'schema Schema {
        self.schema
    }
}
