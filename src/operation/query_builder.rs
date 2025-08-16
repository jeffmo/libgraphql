use crate::ast;
use crate::operation::operation_builder::OperationBuildError;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::DirectiveAnnotation;
use crate::operation::FragmentSet;
use crate::operation::OperationBuilderTrait;
use crate::operation::Query;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use inherent::inherent;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, QueryBuildError>;

#[derive(Clone, Debug, PartialEq)]
pub struct QueryBuilder<'schema, 'fragset>(
    OperationBuilder<'schema, 'fragset>,
);

#[inherent]
impl<'schema, 'fragset> OperationBuilderTrait<
    'schema,
    'fragset,
    ast::operation::Query,
    QueryBuildError,
    Query<'schema, 'fragset>,
> for QueryBuilder<'schema, 'fragset> {
    /// Add a [`DirectiveAnnotation`] after any previously added
    /// `DirectiveAnnotation`s.
    pub fn add_directive(self, annot: DirectiveAnnotation) -> Result<Self> {
        Ok(Self(self.0.add_directive(annot)?))
    }

    /// Add a [`Selection`] after any previously added `Selection`s.
    pub fn add_selection(self, selection: Selection<'fragset>) -> Result<Self> {
        Ok(Self(self.0.add_selection(selection)?))
    }

    /// Add a [`Variable`] after any previously added `Variable`s.
    pub fn add_variable(self, variable: Variable) -> Result<Self> {
        Ok(Self(self.0.add_variable(variable)?))
    }

    /// Consume this [`QueryBuilder`] to produce a [`Query`].
    pub fn build(self) -> Result<Query<'schema, 'fragset>> {
        let operation_kind = self.0.operation_kind.to_owned();
        match self.0.build()? {
            Operation::Query(query) => Ok(*query),
            _ => panic!("Unexpected OperationKind: `{operation_kind:#?}`"),
        }
    }

    /// Produce a [`QueryBuilder`] from a [`Query`](ast::operation::Query).
    pub fn from_ast(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
        ast: &ast::operation::Query,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        Ok(Self(OperationBuilder::from_ast(
            schema,
            fragset,
            &ast::operation::OperationDefinition::Query(ast.to_owned()),
            file_path
        )?))
    }

    /// Produce a [`QueryBuilder`] from a file on disk that whose contents
    /// contain an
    /// [executable document](https://spec.graphql.org/October2021/#ExecutableDocument)
    /// with only a single query defined in it.
    ///
    /// If multiple operations are defined in the document, an error will be
    /// returned. For cases where multiple operations may be defined in a single
    /// document, use
    /// [`ExecutableDocumentBuilder`](crate::operation::ExecutableDocumentBuilder).
    ///
    /// If the document contents include any fragment definitions, an error will
    /// be returned. For cases where operations and fragments may be defined
    /// together in a single document, use
    /// ['ExecutableDocumentBuilder`](crate::operation::ExecutableDocumentBuilder).
    pub fn from_file(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        Ok(Self(OperationBuilder::from_file(schema, fragset, file_path)?))
    }

    /// Produce a [`QueryBuilder`] from a string whose contents contain a
    /// [document](https://spec.graphql.org/October2021/#sec-Document) with only
    /// a single query defined in it.
    ///
    /// If multiple operations are defined in the document, an error will be
    /// returned. For cases where multiple operations may be defined in a single
    /// document, use
    /// [`ExecutableDocumentBuilder`](crate::operation::ExecutableDocumentBuilder).
    ///
    /// If the document contents include any fragment definitions, an error will
    /// be returned. For cases where operations and fragments may be defined
    /// together in a single document, use
    /// ['ExecutableDocumentBuilder`](crate::operation::ExecutableDocumentBuilder).
    pub fn from_str(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        Ok(Self(OperationBuilder::from_str(schema, fragset, content, file_path)?))
    }

    pub fn new(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
    ) -> Self {
        Self(OperationBuilder::new(schema, fragset))
    }

    /// Set the list of [`DirectiveAnnotation`]s.
    ///
    /// NOTE: If any previous directives were added (either using this function
    /// or [`QueryBuilder::add_directive()`]), they will be fully replaced by
    /// the [`DirectiveAnnotation`]s passed here.
    pub fn set_directives(
        self,
        directives: &[DirectiveAnnotation],
    ) -> Result<Self> {
        Ok(Self(self.0.set_directives(directives)?))
    }

    /// Set the name of the [`Query`].
    pub fn set_name(self, name: Option<String>) -> Result<Self> {
        Ok(Self(self.0.set_name(name)?))
    }

    /// Set the [`SelectionSet`].
    ///
    /// NOTE: If any previous selections were added (either using this function
    /// or [`QueryBuilder::add_selection()`]), they will be fully replaced by the
    /// selections in the [`SelectionSet`] passed here.
    pub fn set_selection_set(
        self,
        selection_set: SelectionSet<'fragset>,
    ) -> Result<Self> {
        Ok(Self(self.0.set_selection_set(selection_set)?))
    }

    /// Set the list of [`Variable`]s.
    ///
    /// NOTE: If any previous variables were added (either using this function
    /// or [`QueryBuilder::add_variable()`]), they will be fully replaced by the
    /// collection of variables passed here.
    pub fn set_variables(self, variables: Vec<Variable>) -> Result<Self> {
        Ok(Self(self.0.set_variables(variables)?))
    }
}

#[derive(Clone, Debug, Error)]
pub enum QueryBuildError {
    #[error("Error building Query operation: $0")]
    OperationBuildError(Box<OperationBuildError>),
}
impl std::convert::From<OperationBuildError> for QueryBuildError {
    fn from(value: OperationBuildError) -> Self {
        Self::OperationBuildError(Box::new(value))
    }
}
