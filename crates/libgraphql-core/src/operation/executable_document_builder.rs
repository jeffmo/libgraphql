use crate::ast;
use crate::file_reader;
use crate::operation::ExecutableDocument;
use crate::operation::FragmentBuilder;
use crate::operation::FragmentBuildError;
use crate::operation::FragmentRegistry;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::operation::OperationBuildError;
use crate::schema::Schema;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, Vec<ExecutableDocumentBuildError>>;

pub struct ExecutableDocumentBuilder<'schema: 'fragreg, 'fragreg> {
    fragment_registry: &'fragreg FragmentRegistry<'schema>,
    operations: Vec<Operation<'schema, 'fragreg>>,
    schema: &'schema Schema,
}

impl<'schema, 'fragreg> ExecutableDocumentBuilder<'schema, 'fragreg> {
    pub fn build(self) -> Result<ExecutableDocument<'schema, 'fragreg>> {
        Ok(ExecutableDocument {
            fragment_registry: self.fragment_registry,
            operations: self.operations,
            schema: self.schema,
        })
    }

    pub fn new(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> Self {
        Self {
            fragment_registry,
            operations: vec![],
            schema,
        }
    }

    pub fn from_ast(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        ast: &ast::Document<'_>,
        source_map: &ast::SourceMap<'_>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let mut frag_errors = vec![];
        let mut op_build_errors = vec![];
        let mut errors = vec![];
        let mut operations = vec![];
        for def in &ast.definitions {
            match def {
                ast::Definition::FragmentDefinition(frag_def) => {
                    // Validate that the fragment in the document matches
                    // the one in the registry
                    let fragment_name =
                        frag_def.name.value.as_ref();

                    // Build the fragment from the document
                    let fragment = FragmentBuilder::from_ast(
                        schema,
                        fragment_registry,
                        frag_def,
                        source_map,
                        file_path,
                    ).and_then(|builder| builder.build());
                    let fragment = match fragment {
                        Ok(fragment) => fragment,
                        Err(err) => {
                            frag_errors.push(err);
                            continue;
                        }
                    };

                    // Check if fragment exists in registry
                    let registry_frag =
                        fragment_registry
                            .fragments()
                            .get(fragment_name);

                    if let Some(registry_frag) = registry_frag
                        && &fragment != registry_frag {
                        // Validate that fragments match exactly
                        // For now, we do a simple comparison - in a more complete implementation,
                        // we would compare type condition, selection set, and directives
                        errors.push(
                            ExecutableDocumentBuildError::FragmentDefinitionMismatch {
                                fragment_name: fragment_name.to_string(),
                                document_location: fragment.def_location.clone(),
                                registry_location: registry_frag.def_location.clone(),
                            }
                        );
                    } else if registry_frag.is_none() {
                        errors.push(
                            ExecutableDocumentBuildError::FragmentNotInRegistry {
                                fragment_name: fragment_name.to_string(),
                                document_location: fragment.def_location.clone(),
                            }
                        );
                    }
                },

                ast::Definition::OperationDefinition(op_def) => {
                    // NOTE: We intentionally do not enforce the following
                    //       rule defined in the GraphQL spec stating that
                    //       if a document contains multiple operations, all
                    //       operations in that document must be named:
                    //
                    //       https://spec.graphql.org/October2021/#sel-EAFPTCAACCkEr-Y
                    //
                    //       This is easy to detect if/when it's relevant to
                    //       a use-case, but it's quite limiting to enforce
                    //       at this layer for many other use-cases
                    //       (e.g. batch processing tools, etc).
                    let mut maybe_op = OperationBuilder::from_ast(
                        schema,
                        fragment_registry,
                        op_def,
                        source_map,
                        file_path,
                    ).and_then(|op_builder| op_builder.build());

                    if let Err(errs) = &mut maybe_op {
                        op_build_errors.append(errs);
                        continue;
                    }
                    operations.push(maybe_op.unwrap())
                },

                _ => {
                    // Schema-level definitions (type definitions,
                    // directive definitions, etc.) are ignored in
                    // executable documents.
                },
            }
        }

        if !frag_errors.is_empty() {
            errors.push(
                ExecutableDocumentBuildError::FragmentValidationErrors(
                    frag_errors
                )
            )
        }

        if !op_build_errors.is_empty() {
            errors.push(
                ExecutableDocumentBuildError::OperationBuildErrors(
                    op_build_errors
                )
            );
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            fragment_registry,
            schema,
            operations,
        })
    }

    pub fn from_file(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let file_path = file_path.as_ref();
        let file_content = file_reader::read_content(file_path)
            .map_err(|e| ExecutableDocumentBuildError::ExecutableDocumentFileReadError(
                Box::new(e),
            ))?;
        Self::from_str(schema, fragment_registry, file_content, Some(file_path))
    }

    pub fn from_str(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let parse_result = ast::parse_executable(content.as_ref());
        if parse_result.has_errors() {
            return Err(vec![
                ExecutableDocumentBuildError::ParseErrors(
                    parse_result.errors().to_vec(),
                ),
            ]);
        }
        let (ast_doc, source_map) = parse_result.into_valid().unwrap();
        Self::from_ast(
            schema,
            fragment_registry,
            &ast_doc,
            &source_map,
            file_path,
        )
    }
}

#[derive(Clone, Debug, Error)]
pub enum ExecutableDocumentBuildError {
    #[error(
        "Failure while trying to read an executable document file from disk: $0"
    )]
    ExecutableDocumentFileReadError(Box<file_reader::ReadContentError>),

    #[error(
        "Fragment '{fragment_name}' is defined in the document but does not match \
        the fragment in the registry"
    )]
    FragmentDefinitionMismatch {
        fragment_name: String,
        document_location: crate::loc::SourceLocation,
        registry_location: crate::loc::SourceLocation,
    },

    #[error(
        "Fragment '{fragment_name}' is defined in the document but does not exist \
        in the provided FragmentRegistry"
    )]
    FragmentNotInRegistry {
        fragment_name: String,
        document_location: crate::loc::SourceLocation,
    },

    #[error("Some fragments have validation errors: {0:?}")]
    FragmentValidationErrors(Vec<FragmentBuildError>),

    #[error(
        "Encountered errors while building operations within this \
        executable document: {0:?}",
    )]
    OperationBuildErrors(Vec<OperationBuildError>),

    #[error("Error parsing executable document: {0:?}")]
    ParseErrors(Vec<ast::GraphQLParseError>),
}
impl std::convert::From<Vec<ast::GraphQLParseError>> for ExecutableDocumentBuildError {
    fn from(value: Vec<ast::GraphQLParseError>) -> Self {
        Self::ParseErrors(value)
    }
}
impl std::convert::From<ExecutableDocumentBuildError> for Vec<ExecutableDocumentBuildError> {
    fn from(value: ExecutableDocumentBuildError) -> Self {
        vec![value]
    }
}
