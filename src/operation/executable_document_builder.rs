use crate::ast;
use crate::file_reader;
use crate::operation::ExecutableDocument;
use crate::operation::FragmentSet;
use crate::operation::Operation;
use crate::operation::OperationBuildError;
use crate::operation::OperationBuilder;
use crate::schema::Schema;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

type Result<T> = std::result::Result<T, ExecutableDocumentBuildError>;

pub struct ExecutableDocumentBuilder<'schema: 'fragset, 'fragset> {
    fragset: Option<&'fragset FragmentSet<'schema>>,
    operations: Vec<Operation<'schema, 'fragset>>,
    schema: &'schema Schema,
}

impl<'schema, 'fragset> ExecutableDocumentBuilder<'schema, 'fragset> {
    pub fn build(self) -> Result<ExecutableDocument<'schema, 'fragset>> {
        Ok(ExecutableDocument {
            fragset: self.fragset,
            operations: self.operations,
            schema: self.schema,
        })
    }

    pub fn new(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
    ) -> Self {
        Self {
            fragset,
            operations: vec![],
            schema,
        }
    }

    pub fn from_ast(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
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
                    let maybe_op = OperationBuilder::from_ast(
                        schema,
                        fragset,
                        op_def,
                        file_path,
                    ).and_then(|op_builder| op_builder.build());

                    match maybe_op {
                        Err(err) => operation_build_errors.push(err),
                        Ok(op) => operations.push(op),
                    };
                },
            }
        }

        if !operation_build_errors.is_empty() {
            return Err(ExecutableDocumentBuildError::OperationBuildErrors(
                operation_build_errors,
            ));
        }

        Ok(Self {
            fragset,
            schema,
            operations,
        })
    }

    pub fn from_file(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let file_path = file_path.as_ref();
        let file_content = file_reader::read_content(file_path)
            .map_err(|e| ExecutableDocumentBuildError::ExecutableDocumentFileReadError(
                Box::new(e),
            ))?;
        Self::from_str(schema, fragset, file_content, Some(file_path))
    }

    pub fn from_str(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let ast_doc = ast::operation::parse(content.as_ref())?;
        Self::from_ast(schema, fragset, &ast_doc, file_path)
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
