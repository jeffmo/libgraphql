use crate::ast;
use crate::file_reader;
use crate::operation::ExecutableDocument;
use crate::operation::FragmentRegistry;
use crate::operation::Operation;
use crate::operation::OperationBuildError;
use crate::operation::OperationBuilder;
use crate::schema::Schema;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

type Result<T> = std::result::Result<T, ExecutableDocumentBuildError>;

pub struct ExecutableDocumentBuilder<'schema: 'fragreg, 'fragreg> {
    fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
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
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
    ) -> Self {
        Self {
            fragment_registry,
            operations: vec![],
            schema,
        }
    }

    pub fn from_ast(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
        ast: &ast::operation::Document,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let mut operation_build_errors = vec![];
        let mut operations = vec![];
        for def in &ast.definitions {
            use ast::operation::Definition as Def;
            match def {
                Def::Fragment(_frag_def) => {
                    // TODO(!!!)
                },

                Def::Operation(op_def) => {
                    // NOTE: We intentionally do not enforce the following rule
                    //       defined in the GraphQL spec stating that if a
                    //       document contains multiple operations, all
                    //       operations in that document must be named:
                    //
                    //       https://spec.graphql.org/October2021/#sel-EAFPTCAACCkEr-Y
                    //
                    //       This is easy to detect if/when it's relevant to a
                    //       use-case, but it's quite limiting to enforce at
                    //       this layer for many other use-cases
                    //       (e.g. batch processing tools, etc).
                    let mut maybe_op = OperationBuilder::from_ast(
                        schema,
                        fragment_registry,
                        op_def,
                        file_path,
                    ).and_then(|op_builder| op_builder.build());

                    if let Err(errs) = &mut maybe_op {
                        operation_build_errors.append(errs);
                        continue;
                    }
                    operations.push(maybe_op.unwrap())
                },
            }
        }

        if !operation_build_errors.is_empty() {
            return Err(ExecutableDocumentBuildError::OperationBuildErrors(
                operation_build_errors,
            ));
        }

        Ok(Self {
            fragment_registry,
            schema,
            operations,
        })
    }

    pub fn from_file(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
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
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let ast_doc = ast::operation::parse(content.as_ref())?;
        Self::from_ast(schema, fragment_registry, &ast_doc, file_path)
    }
}

#[derive(Clone, Debug, Error)]
pub enum ExecutableDocumentBuildError {
    #[error(
        "Failure while trying to read an executable document file from disk: $0"
    )]
    ExecutableDocumentFileReadError(Box<file_reader::ReadContentError>),

    #[error(
        "Encountered errors while building operations within this \
        executable document: {0:?}",
    )]
    OperationBuildErrors(Vec<OperationBuildError>),

    #[error("Error parsing executable document: $0")]
    ParseError(Arc<ast::operation::ParseError>),
}
impl std::convert::From<ast::operation::ParseError> for ExecutableDocumentBuildError {
    fn from(value: ast::operation::ParseError) -> Self {
        Self::ParseError(Arc::new(value))
    }
}
