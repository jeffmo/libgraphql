use crate::ast;
use crate::file_reader;
use crate::operation::FragmentSet;
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
use crate::operation::SubscriptionBuilder;
use crate::operation::SubscriptionBuildError;
use crate::schema::Schema;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

type Result<'schema, 'fragset, T> = std::result::Result<
    T,
    OperationSetBuildError<'schema, 'fragset>,
>;

#[derive(Debug)]
pub struct OperationSetBuilder<'schema, 'fragset> {
    fragments: HashMap<String, NamedFragment<'schema>>,
    loaded_str_id_counter: u16,
    named_mutations: HashMap<String, Mutation<'schema, 'fragset>>,
    named_queries: HashMap<String, Query<'schema, 'fragset>>,
    named_subscriptions: HashMap<String, Subscription<'schema, 'fragset>>,
    schema: &'schema Schema,
}
impl<'schema, 'fragset: 'schema> OperationSetBuilder<'schema, 'fragset> {
    pub fn add_query_from_ast(
        mut self,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<'schema, 'fragset, Self> {
        let query = QueryBuilder::from_ast(
            self.schema,
            file_path,
            def,
        )?;

        match query.name() {
            Some(query_name) => {
                if let Some(existing_query) = self.queries.get(query_name) {
                    return Err(OperationSetBuildError::QueryNameAlreadyExists {
                        name: query_name.to_string(),
                        existing_query: existing_query.to_owned(),
                    });
                }
                self.queries.insert(query_name.to_string(), query);
            },

            None => {
                //if let Some(existing_operation) = self.lone_anonymous_operation {
                //    return Err(OperationSetBuildError::
                //}
            },
        }
        //if query.name().is_none() && self.lone_anonymous_operation.is_some() {
        //    return Err(OperationSetBuildError::
        //}


        // Every Query operation must have a unique name.
        /*
        if let Some(query_name) = query.name() {
            if self.queries.contains_key(query_name) {
                return Err(OperationSetBuildError::QueryNameAlreadyExists {
                    name: query.name().to_string(),
                    existing_query: self.queries.get(query.name()).to_owned(),
                });
            }
        }
        */

        Ok(self)
    }

    pub fn add_mutation_from_ast(
        mut self,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<'schema, 'fragset, Self> {
        self.mutations.push(MutationBuilder::from_ast(
            self.schema,
            file_path,
            def,
        )?);
        Ok(self)
    }

    pub fn build(self) -> OperationSet<'schema, 'fragset> {
        let fragment_set =
            if self.fragments.is_empty() {
                None
            } else {
                Some(FragmentSet(self.fragments))
            };

        OperationSet {
            fragment_set,
            named_mutations: self.named_mutations,
            named_queries: self.named_queries,
            named_subscriptions: self.named_subscriptions,
            schema: self.schema,
        }
    }

    pub fn load_files<P: AsRef<Path>>(
        mut self,
        file_paths: Vec<P>,
    ) -> Result<'schema, 'fragset, Self> {
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
    ) -> Result<'schema, 'fragset, Self> {
        let file_path = file_path.unwrap_or_else(|| {
            let str_id = self.loaded_str_id_counter;
            self.loaded_str_id_counter += 1;
            PathBuf::from(format!("str://{str_id}"))
        });

        let ast_doc = graphql_parser::query::parse_query::<String>(content)
            .map_err(|err| OperationSetBuildError::Parse {
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
            mutations: HashMap::new(),
            queries: HashMap::new(),
            schema,
            subscriptions: HashMap::new(),
        }
    }

    fn visit_ast_def(
        &mut self,
        file_path: &Path,
        def: ast::operation::Definition,
    ) -> Result<'schema, 'fragset, ()> {
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
                self.subscriptions.push(SubscriptionBuilder::from_ast(
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
pub enum OperationSetBuildError<'schema, 'fragset: 'schema> {
    #[error("Failure to build a named fragment")]
    NamedFragmentBuild(Box<NamedFragmentBuildError>),

    #[error("Failure to build mutation operation")]
    MutationBuild(Box<MutationBuildError>),

    #[error("There is already a mutation defined with name `{name:?}`")]
    MutationNameAlreadyExists {
        name: String,
        existing_mutation: Mutation<'schema, 'fragset>,
    },

    #[error("Failure while trying to read an operation file from disk")]
    OperationFileReadError(Box<file_reader::ReadContentError>),

    #[error("Error parsing operation string")]
    Parse {
        file_path: PathBuf,
        err: String,
    },

    #[error("Failure to build query operation")]
    QueryBuild(Box<QueryBuildError>),

    #[error("There is already a query defined with name `{name:?}`")]
    QueryNameAlreadyExists {
        name: String,
        existing_query: Query<'schema, 'fragset>,
    },

    #[error("Failure to build subscription operation")]
    SubscriptionBuild(Box<SubscriptionBuildError>),

    #[error("There is already a subscription defined with name `{name:?}`")]
    SubscriptionNameAlreadyExists {
        name: String,
        existing_query: Subscription<'schema, 'fragset>,
    },

}
impl<'schema, 'fragset: 'schema> std::convert::From<NamedFragmentBuildError>
        for OperationSetBuildError<'schema, 'fragset> {
    fn from(err: NamedFragmentBuildError) -> OperationSetBuildError<'schema, 'fragset> {
        OperationSetBuildError::NamedFragmentBuild(Box::new(err))
    }
}
impl<'schema, 'fragset: 'schema> std::convert::From<QueryBuildError>
        for OperationSetBuildError<'schema, 'fragset> {
    fn from(err: QueryBuildError) -> OperationSetBuildError<'schema, 'fragset> {
        OperationSetBuildError::QueryBuild(Box::new(err))
    }
}
impl<'schema, 'fragset: 'schema> std::convert::From<MutationBuildError>
        for OperationSetBuildError<'schema, 'fragset> {
    fn from(err: MutationBuildError) -> OperationSetBuildError<'schema, 'fragset> {
        OperationSetBuildError::MutationBuild(Box::new(err))
    }
}
impl<'schema, 'fragset: 'schema> std::convert::From<SubscriptionBuildError>
        for OperationSetBuildError<'schema, 'fragset> {
    fn from(err: SubscriptionBuildError) -> OperationSetBuildError<'schema, 'fragset> {
        OperationSetBuildError::SubscriptionBuild(Box::new(err))
    }
}
