use crate::ast;
use crate::file_reader;
use crate::operation::NamedFragment;
use crate::operation::NamedFragmentBuildError;
use crate::operation::Mutation;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::OperationSet;
use crate::operation::Query;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::operation::Subscription;
use crate::operation::SubscriptionBuildError;
use crate::schema::Schema;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

type Result<T> = std::result::Result<T, OperationSetBuildError>;

#[derive(Debug)]
pub struct OperationSetBuilder<'schema> {
    fragments: HashMap<String, NamedFragment<'schema>>,
    loaded_str_id_counter: u16,
    mutations: Vec<Mutation<'schema>>,
    queries: Vec<Query<'schema>>,
    schema: &'schema Schema,
    subscriptions: Vec<Subscription<'schema>>,
}
impl<'schema> OperationSetBuilder<'schema> {
    pub fn add_query_from_ast(
        mut self,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<Self> {
        self.queries.push(QueryBuilder::from_ast(
            self.schema,
            file_path,
            def,
        )?);
        Ok(self)
    }

    pub fn add_mutation_from_ast(
        mut self,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Self> {
        self.mutations.push(MutationBuilder::from_ast(
            self.schema,
            file_path,
            def,
        )?);
        Ok(self)
    }

    pub fn build(self) -> OperationSet<'schema> {
        OperationSet {
            fragments: self.fragments,
            mutations: self.mutations,
            queries: self.queries,
            subscriptions: self.subscriptions,
        }
    }

    pub fn load_files<P: AsRef<Path>>(
        mut self,
        file_paths: Vec<P>,
    ) -> Result<Self> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let file_content = file_reader::read_content(file_path)
                .map_err(|err| OperationSetBuildError::OperationFileReadError(
                    Box::new(err),
                ))?;

            self = self.load_str(
                Some(file_path.to_path_buf()),
                file_content.as_str(),
            )?;
        }
        Ok(self)
    }

    pub fn load_str(
        mut self,
        file_path: Option<PathBuf>,
        content: &str,
    ) -> Result<Self> {
        let file_path = file_path.unwrap_or_else(|| {
            let str_id = self.loaded_str_id_counter;
            self.loaded_str_id_counter += 1;
            PathBuf::from(format!("str://{str_id}"))
        });

        let ast_doc = graphql_parser::query::parse_query::<String>(content)
            .map_err(|err| OperationSetBuildError::ParseError {
                file_path: file_path.to_owned(),
                err: err.to_string(),
            })?.into_static();

        for def in ast_doc.definitions {
            self.visit_ast_def(file_path.as_path(), def)?;
        }

        Ok(self)
    }

    pub fn new(schema: &'schema Schema) -> Self {
        Self {
            fragments: HashMap::new(),
            loaded_str_id_counter: 0,
            mutations: vec![],
            queries: vec![],
            schema,
            subscriptions: vec![],
        }
    }

    fn visit_ast_def(
        &mut self,
        file_path: &Path,
        def: ast::operation::Definition,
    ) -> Result<()> {
        use ast::operation::Definition;
        use ast::operation::OperationDefinition as OpDef;
        match def {
            Definition::Fragment(frag_def) => {
                self.fragments.insert(
                    frag_def.name.to_string(),
                    NamedFragment::from_ast(self.schema, file_path, frag_def)?,
                );
            },

            Definition::Operation(OpDef::Mutation(mut_op)) => {
                self.mutations.push(Mutation::from_ast(
                    self.schema,
                    file_path,
                    mut_op,
                )?);
            },

            Definition::Operation(OpDef::Query(query_op)) => {
                self.queries.push(Query::from_ast(
                        self.schema,
                        file_path,
                        query_op,
                )?);
            },

            Definition::Operation(OpDef::SelectionSet(sel_set)) => {
                self.queries.push(
                    Query::from_ast(self.schema, file_path, ast::operation::Query {
                        position: sel_set.span.0,
                        name: None,
                        variable_definitions: vec![],
                        directives: vec![],
                        selection_set: sel_set,
                    })?
                );
            },

            Definition::Operation(OpDef::Subscription(sub_op)) => {
                self.subscriptions.push(Subscription::from_ast(
                    self.schema,
                    file_path,
                    sub_op,
                )?);
            },
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum OperationSetBuildError {
    #[error("Failure to build a named fragment")]
    NamedFragmentBuildError(Box<NamedFragmentBuildError>),

    #[error("Failure to build mutation operation")]
    MutationBuildError(Box<MutationBuildError>),

    #[error("Failure while trying to read an operation file from disk")]
    OperationFileReadError(Box<file_reader::ReadContentError>),

    #[error("Error parsing operation string")]
    ParseError {
        file_path: PathBuf,
        err: String,
    },

    #[error("Failure to build query operation")]
    QueryBuildError(Box<QueryBuildError>),

    #[error("Failure to build subscription operation")]
    SubscriptionBuildError(Box<SubscriptionBuildError>),
}
impl std::convert::From<NamedFragmentBuildError> for OperationSetBuildError {
    fn from(err: NamedFragmentBuildError) -> OperationSetBuildError {
        OperationSetBuildError::NamedFragmentBuildError(Box::new(err))
    }
}
impl std::convert::From<QueryBuildError> for OperationSetBuildError {
    fn from(err: QueryBuildError) -> OperationSetBuildError {
        OperationSetBuildError::QueryBuildError(Box::new(err))
    }
}
impl std::convert::From<MutationBuildError> for OperationSetBuildError {
    fn from(err: MutationBuildError) -> OperationSetBuildError {
        OperationSetBuildError::MutationBuildError(Box::new(err))
    }
}
impl std::convert::From<SubscriptionBuildError> for OperationSetBuildError {
    fn from(err: SubscriptionBuildError) -> OperationSetBuildError {
        OperationSetBuildError::SubscriptionBuildError(Box::new(err))
    }
}
