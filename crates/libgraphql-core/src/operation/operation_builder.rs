use crate::ast;
use crate::ast::operation::OperationDefinition;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::file_reader;
use crate::loc;
use crate::named_ref::DerefByNameError;
use crate::operation::FragmentRegistry;
use crate::operation::Mutation;
use crate::operation::Operation;
use crate::operation::OperationBuilderTrait;
use crate::operation::OperationData;
use crate::operation::OperationKind;
use crate::operation::Query;
use crate::operation::Selection;
use crate::operation::SelectionSetBuilder;
use crate::operation::SelectionSetBuildError;
use crate::operation::Subscription;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::TypeAnnotation;
use crate::Value;
use indexmap::IndexMap;
use inherent::inherent;
use thiserror::Error;
use std::path::Path;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Vec<OperationBuildError>>;

struct LoadFromAstDetails<'ast, 'schema> {
    directives: &'ast Vec<ast::operation::Directive>,
    name: Option<&'ast String>,
    op_kind: OperationKind,
    op_type: &'schema GraphQLType,
    pos: &'ast ast::AstPos,
    selection_set: &'ast ast::operation::SelectionSet,
    variables: &'ast Vec<ast::operation::VariableDefinition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperationBuilder<'schema: 'fragreg, 'fragreg> {
    def_location: Option<loc::SourceLocation>,
    directives: Vec<DirectiveAnnotation>,
    fragment_registry: &'fragreg FragmentRegistry<'schema>,
    name: Option<String>,
    pub(super) operation_kind: Option<OperationKind>,
    schema: &'schema Schema,
    selection_set_builder: SelectionSetBuilder<'schema, 'fragreg>,
    variables: IndexMap<String, Variable>,
}

#[inherent]
impl<'schema: 'fragreg, 'fragreg> OperationBuilderTrait<
    'schema,
    'fragreg,
    OperationDefinition,
    Vec<OperationBuildError>,
    Operation<'schema, 'fragreg>,
> for OperationBuilder<'schema, 'fragreg> {
    /// Add a [`DirectiveAnnotation`] after any previously added
    /// `DirectiveAnnotation`s.
    pub fn add_directive(
        mut self,
        annot: DirectiveAnnotation,
    ) -> Result<Self> {
        // TODO: Error if a non-repeatable directive is added twice
        self.directives.push(annot);
        Ok(self)
    }

    /// Add a [`Selection`] after any previously added `Selection`s.
    pub fn add_selection(
        mut self,
        selection: Selection<'schema>,
    ) -> Result<Self> {
        self.selection_set_builder =
            self.selection_set_builder
                .add_selection(selection)
                .map_err(|e| vec![OperationBuildError::SelectionSetBuildErrors(e)])?;

        Ok(self)
    }

    /// Add a [`Variable`] after any previously added `Variable`s.
    pub fn add_variable(
        mut self,
        variable: Variable,
    ) -> Result<Self> {
        if let Some(existing_variable) = self.variables.get(variable.name()) {
            return Err(vec![
                OperationBuildError::DuplicateVariableName {
                    variable_definition1: existing_variable.def_location().to_owned(),
                    variable_definition2: variable.def_location().to_owned(),
                    variable_name: variable.name().to_string(),
                }
            ]);
        }
        self.variables.insert(variable.name().to_string(), variable);
        Ok(self)
    }

    /// Consume ths [`OperationBuilder`] to produce an [`Operation`].
    pub fn build(self) -> Result<Operation<'schema, 'fragreg>> {
        let selection_set =
            self.selection_set_builder
                .build()
                .map_err(|e| vec![OperationBuildError::SelectionSetBuildErrors(e)])?;

        let operation_data = OperationData {
            directives: self.directives,
            def_location: self.def_location.unwrap_or(
                loc::SourceLocation::ExecutableDocument
            ),
            fragment_registry: self.fragment_registry,
            name: self.name,
            schema: self.schema,
            selection_set,
            variables: self.variables,
        };

        Ok(match self.operation_kind {
            Some(OperationKind::Mutation) => Operation::Mutation(Box::new(
                Mutation(operation_data),
            )),

            Some(OperationKind::Query) => Operation::Query(Box::new(
                Query(operation_data),
            )),

            Some(OperationKind::Subscription) => Operation::Subscription(Box::new(
                Subscription(operation_data),
            )),

            None => return Err(vec![
                OperationBuildError::AmbiguousOperationKind {
                    operation_name: operation_data.name,
                }
            ]),
        })
    }

    /// Produce a [`OperationBuilder`] from a
    /// [`OperationDefinition`](ast::operation::OperationDefinition).
    pub fn from_ast(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        ast: &OperationDefinition,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let ast_details = match ast {
            OperationDefinition::SelectionSet(ss @ ast::operation::SelectionSet {
                span: (pos, _),
                ..
            }) => LoadFromAstDetails {
                directives: &vec![],
                name: None,
                op_kind: OperationKind::Query,
                op_type: schema.query_type(),
                pos,
                selection_set: ss,
                variables: &vec![],
            },

            OperationDefinition::Query(ast::operation::Query {
                directives,
                name,
                position,
                selection_set,
                variable_definitions,
                ..
            }) => LoadFromAstDetails {
                directives,
                name: name.as_ref(),
                op_kind: OperationKind::Query,
                op_type: schema.query_type(),
                pos: position,
                selection_set,
                variables: variable_definitions,
            },

            OperationDefinition::Mutation(ast::operation::Mutation {
                directives,
                name,
                position,
                selection_set,
                variable_definitions,
                ..
            }) => {
                let op_type = schema.mutation_type().ok_or_else(|| vec![
                    OperationBuildError::NoMutationTypeDefinedInSchema
                ])?;

                LoadFromAstDetails {
                    directives,
                    name: name.as_ref(),
                    op_kind: OperationKind::Mutation,
                    op_type,
                    pos: position,
                    selection_set,
                    variables: variable_definitions,
                }
            },

            OperationDefinition::Subscription(ast::operation::Subscription {
                directives,
                name,
                position,
                selection_set,
                variable_definitions,
                ..
            }) => {
                let op_type = schema.subscription_type().ok_or_else(|| vec![
                    OperationBuildError::NoSubscriptionTypeDefinedInSchema
                ])?;

                LoadFromAstDetails {
                    directives,
                    name: name.as_ref(),
                    op_kind: OperationKind::Subscription,
                    op_type,
                    pos: position,
                    selection_set,
                    variables: variable_definitions,
                }
            },
        };

        let opdef_srcloc = loc::SourceLocation::from_execdoc_ast_position(
            file_path,
            ast_details.pos,
        );

        let mut errors = vec![];

        let directives = DirectiveAnnotationBuilder::from_ast(
            &opdef_srcloc,
            ast_details.directives,
        );

        let mut variables = IndexMap::<String, Variable>::new();
        for ast_var_def in ast_details.variables {
            let var_name = ast_var_def.name.to_string();
            let vardef_srcloc =
                opdef_srcloc.with_ast_position(&ast_var_def.position);
            let type_ref = TypeAnnotation::from_ast_type(
                &vardef_srcloc.to_owned(),
                &ast_var_def.var_type,
            );

            if let Some(var_def) = variables.get(var_name.as_str()) {
                errors.push(OperationBuildError::DuplicateVariableName {
                    variable_definition1: var_def.def_location().to_owned(),
                    variable_definition2: vardef_srcloc,
                    variable_name: var_name,
                });
                continue
            }

            // Ensure the inner named type reference is a valid type within the
            // schema.
            let inner_named_type_is_valid =
                type_ref.inner_named_type_ref()
                    .deref(schema)
                    .map_err(|err| match err {
                        DerefByNameError::DanglingReference(var_name)
                            => OperationBuildError::UndefinedVariableType {
                                variable_name: var_name,
                                location: vardef_srcloc.to_owned(),
                            },
                    });
            if let Err(e) = inner_named_type_is_valid {
                errors.push(e);
                continue
            }

            let default_value =
                ast_var_def.default_value.as_ref().map(|val| {
                    Value::from_ast(val, &loc::SourceLocation::from_execdoc_ast_position(
                        file_path,
                        &ast_var_def.position,
                    ))
                });

            variables.insert(ast_var_def.name.to_string(), Variable {
                default_value,
                name: ast_var_def.name.to_string(),
                type_annotation: TypeAnnotation::from_ast_type(
                    &vardef_srcloc,
                    &ast_var_def.var_type,
                ),
                def_location: vardef_srcloc,
            });
        }

        let maybe_selection_set_builder =
            SelectionSetBuilder::from_ast(
                schema,
                fragment_registry,
                ast_details.op_type,
                ast_details.selection_set,
                file_path,
            );

        let selection_set_builder = match maybe_selection_set_builder {
            Ok(selection_set_builder) => selection_set_builder,
            Err(selection_set_build_errors) => {
                errors.push(selection_set_build_errors.into());
                return Err(errors);
            },
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            def_location: Some(opdef_srcloc),
            directives,
            fragment_registry,
            name: ast_details.name.map(|s| s.to_string()),
            operation_kind: Some(ast_details.op_kind),
            schema,
            selection_set_builder,
            variables,
        })
    }

    /// Produce a [`OperationBuilder`] from a file on disk that whose contents
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
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let file_path = file_path.as_ref();
        let file_content = file_reader::read_content(file_path)
            .map_err(|e| vec![
                OperationBuildError::OperationFileReadError(Box::new(e)),
            ])?;
        Self::from_str(schema, fragment_registry, file_content, Some(file_path))
    }

    /// Produce a [`OperationBuilder`] from a string whose contents contain a
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
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let ast_doc =
            ast::operation::parse(content.as_ref())
                .map_err(|e| vec![e.into()])?;
        let op_def =
            if ast_doc.definitions.len() > 1 {
                let mut op_count = 0;
                let mut frag_count = 0;
                for def in ast_doc.definitions {
                    match def {
                        ast::operation::Definition::Operation(_) =>
                            op_count += 1,
                        ast::operation::Definition::Fragment(_) =>
                            frag_count += 1,
                    }
                }

                if frag_count > 0 {
                    return Err(vec![
                        OperationBuildError::SchemaDeclarationsFoundInExecutableDocument
                    ]);
                } else {
                    return Err(vec![
                        OperationBuildError::MultipleOperationsInExecutableDocument {
                            num_operations_found: op_count,
                        }
                    ]);
                }
            } else if let Some(op_def) = ast_doc.definitions.first() {
                match op_def {
                    ast::operation::Definition::Operation(op_def)
                        => op_def,
                    ast::operation::Definition::Fragment(_)
                        => return Err(vec![
                            OperationBuildError::SchemaDeclarationsFoundInExecutableDocument,
                        ]),
                }
            } else {
                return Err(vec![
                    OperationBuildError::NoOperationsFoundInExecutableDocument,
                ]);
            };

        Self::from_ast(schema, fragment_registry, op_def, file_path)
    }

    pub fn new(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> OperationBuilder<'schema, 'fragreg> {
        Self {
            def_location: None,
            directives: vec![],
            fragment_registry,
            name: None,
            operation_kind: None,
            schema,
            selection_set_builder: SelectionSetBuilder::new(
                schema,
                fragment_registry,
            ),
            variables: IndexMap::new(),
        }
    }

    /// Set the list of [`DirectiveAnnotation`]s.
    ///
    /// NOTE: If any previous directives were added (either using this function
    /// or [`OperationBuilder::add_directive()`]), they will be fully replaced by
    /// the [`DirectiveAnnotation`]s passed here.
    pub fn set_directives(
        mut self,
        directives: &[DirectiveAnnotation],
    ) -> Result<Self> {
        self.directives = directives.into();
        Ok(self)
    }

    /// Set the name of the [`Operation`].
    pub fn set_name(mut self, name: Option<String>) -> Result<Self> {
        self.name = name;
        Ok(self)
    }

    /// Set the list of [`Variable`]s.
    ///
    /// NOTE: If any previous variables were added (either using this function
    /// or [`OperationBuilder::add_variable`]), they will be fully replaced by the
    /// collection of variables passed here.
    pub fn set_variables(mut self, variables: Vec<Variable>) -> Result<Self> {
        self.variables =
            variables.into_iter()
                .map(|var| (var.name.to_owned(), var))
                .collect();
        Ok(self)
    }
}

#[derive(Clone, Debug, Error)]
pub enum OperationBuildError {
    #[error("No operation type specified.")]
    AmbiguousOperationKind {
        operation_name: Option<String>,
    },

    #[error("Found multiple directive arguments with the same name.")]
    DuplicateDirectiveArgument {
        argument_name: String,
        loc1: loc::FilePosition,
        loc2: loc::FilePosition,
    },

    #[error("Found multiple variables defined with the same name on this operation")]
    DuplicateVariableName {
        variable_definition1: loc::SourceLocation,
        variable_definition2: loc::SourceLocation,
        variable_name: String,
    },

    #[error("Found multiple arguments for the same parameter on a field in this query")]
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },

    #[error(
        "Found multiple operations in document. If this was expected, consider \
        using ExecutableDocumentBuilder instead.",
    )]
    MultipleOperationsInExecutableDocument {
        num_operations_found: i16,
    },

    #[error("No operations found in document.")]
    NoOperationsFoundInExecutableDocument,

    #[error("No Mutation type defined on this schema")]
    NoMutationTypeDefinedInSchema,

    #[error("No Subscription type defined on this schema")]
    NoSubscriptionTypeDefinedInSchema,

    #[error("Failure while trying to read a schema file from disk")]
    OperationFileReadError(Box<file_reader::ReadContentError>),

    #[error("Error parsing operation document: $0")]
    ParseError(Arc<ast::operation::ParseError>),

    #[error("Non-operations found in document.")]
    SchemaDeclarationsFoundInExecutableDocument,

    #[error("Failure to build the selection set for this operation: $0")]
    SelectionSetBuildErrors(Vec<SelectionSetBuildError>),

    #[error("Named type is not defined in the schema for this query")]
    UndefinedVariableType {
        location: loc::SourceLocation,
        variable_name: String,
    },
}
impl std::convert::From<Vec<SelectionSetBuildError>> for OperationBuildError {
    fn from(value: Vec<SelectionSetBuildError>) -> Self {
        Self::SelectionSetBuildErrors(value)
    }
}
impl std::convert::From<ast::operation::ParseError> for OperationBuildError {
    fn from(value: ast::operation::ParseError) -> Self {
        Self::ParseError(Arc::new(value))
    }
}
