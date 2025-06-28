use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::operation::OperationImpl;
use crate::operation::OperationBuilder;
use crate::operation::Query;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::SelectionSetBuildError;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::Directive;
use crate::types::TypeAnnotation;
use crate::Value;
use inherent::inherent;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, QueryBuildError>;

#[derive(Debug)]
pub struct QueryBuilder<'schema, 'fragset> {
    directives: Vec<DirectiveAnnotation>,
    name: Option<String>,
    schema: &'schema Schema,
    selection_set: SelectionSet<'fragset>,
    variables: BTreeMap<String, Variable>,
}

#[inherent]
impl<'schema, 'fragset> OperationBuilder<
    'schema,
    'fragset,
    ast::operation::Query,
    QueryBuildError,
    Query<'schema, 'fragset>,
> for QueryBuilder<'schema, 'fragset> {
    /// Adds a [DirectiveAnnotation] to the [Query].
    ///
    /// If other directives are already present, this will add the new
    /// directive after the others.
    pub fn add_directive(
        mut self,
        annot: DirectiveAnnotation,
    ) -> Result<Self> {
        // TODO: Error if a non-repeatable directive is added twice
        self.directives.push(annot);
        Ok(self)
    }

    /// Adds a [Selection] to the [Query].
    ///
    /// If other selections are already present, this will add the new selection
    /// after the others.
    pub fn add_selection(
        mut self,
        selection: Selection<'fragset>,
    ) -> Result<Self> {
        self.selection_set.selections.push(selection);
        Ok(self)
    }

    /// Adds a [Variable] to the [Query].
    pub fn add_variable(
        mut self,
        variable: Variable,
    ) -> Result<Self> {
        if self.variables.contains_key(variable.name.as_str()) {
            return Err(QueryBuildError::DuplicateVariableName {
                file_pos1: None,
                file_pos2: None,
                variable_name: variable.name,
            })
        }
        self.variables.insert(variable.name.to_owned(), variable);
        Ok(self)
    }

    /// Consume the [QueryBuilder] to produce a [Query].
    pub fn build(self) -> Result<Query<'schema, 'fragset>> {
        Ok(Query(OperationImpl {
            directives: self.directives,
            def_location: None,
            name: self.name,
            phantom_ast: PhantomData,
            phantom_error: PhantomData,
            phantom_op: PhantomData,
            phantom_builder: PhantomData,
            schema: self.schema,
            selection_set: self.selection_set,
            variables: self.variables,
        }))
    }

    /// Produce a [Query] object given a [Schema] and a fully parsed
    /// [ast::operation::Query].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<Query<'schema, 'fragset>> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );

        let mut query_directives = vec![];
        for ast_directive in def.directives {
            let directive_position = loc::FilePosition::from_pos(
                file_path,
                ast_directive.position,
            );

            let mut arguments = BTreeMap::new();
            for (arg_name, ast_arg_value) in &ast_directive.arguments {
                if arguments.insert(
                    arg_name.to_string(),
                    Value::from_ast(
                        ast_arg_value,
                        directive_position.clone(),
                    ),
                ).is_some() {
                    return Err(QueryBuildError::DuplicateFieldArgument {
                        argument_name: arg_name.to_string(),
                        location1: directive_position.clone(),
                        location2: directive_position,
                    });
                }
            }

            query_directives.push(DirectiveAnnotation {
                args: arguments,
                directive_ref: Directive::named_ref(
                    ast_directive.name.as_str(),
                    loc::SchemaDefLocation::Schema(directive_position),
                ),
            });
        }

        let mut variables = BTreeMap::<String, Variable>::new();
        for ast_var_def in def.variable_definitions {
            let var_name = ast_var_def.name.to_string();
            let vardef_location = loc::FilePosition::from_pos(
                file_path,
                ast_var_def.position,
            );
            let type_ref = TypeAnnotation::from_ast_type(
                &vardef_location.clone().into(),
                &ast_var_def.var_type,
            );

            if let Some(var_def) = variables.get(var_name.as_str()) {
                return Err(QueryBuildError::DuplicateVariableName {
                    file_pos1: Some(var_def.def_location.clone()),
                    file_pos2: Some(vardef_location),
                    variable_name: var_name,
                })
            }

            // Ensure the inner named type reference is a valid type within the
            // schema.
            type_ref.inner_named_type_ref()
                .deref(schema)
                .map_err(|err| match err {
                    DerefByNameError::DanglingReference(var_name)
                        => QueryBuildError::UndefinedVariableType {
                            variable_name: var_name,
                            location: vardef_location.clone(),
                        },
                })?;

            let default_value =
                ast_var_def.default_value.map(|val| {
                    Value::from_ast(&val, file_position.clone())
                });

            variables.insert(ast_var_def.name.to_string(), Variable {
                def_location: file_position.clone(),
                default_value,
                name: ast_var_def.name.to_string(),
                type_: TypeAnnotation::from_ast_type(
                    &file_position.clone().into(),
                    &ast_var_def.var_type,
                ),
            });
        }

        Ok(Query(OperationImpl {
            directives: query_directives,
            def_location: Some(file_position.clone()),
            name: def.name,
            phantom_ast: PhantomData,
            phantom_error: PhantomData,
            phantom_op: PhantomData,
            phantom_builder: PhantomData,
            schema,
            selection_set: SelectionSet::from_ast(
                file_path,
                &def.selection_set,
            )?,
            variables,
        }))
    }

    pub fn new(schema: &'schema Schema) -> Result<Self> {
        Ok(QueryBuilder {
            directives: vec![],
            name: None,
            schema,
            selection_set: SelectionSet {
                selections: vec![],
            },
            variables: BTreeMap::new(),
        })
    }

    /// Sets the list of [DirectiveAnnotation]s.
    ///
    /// NOTE: If any previous directives were added (either using this function
    /// or [QueryBuilder::add_directive]), they will be fully replaced by the
    /// [Vec] passed here.
    pub fn set_directives(
        mut self,
        directives: &[DirectiveAnnotation],
    ) -> Result<Self> {
        self.directives = directives.into();
        Ok(self)
    }

    /// Sets the name of the [Query].
    pub fn set_name(mut self, name: Option<String>) -> Result<Self> {
        self.name = name;
        Ok(self)
    }

    /// Sets the [SelectionSet] for the [Query].
    ///
    /// NOTE: If any previous selections were added (either using this function
    /// or [QueryBuilder::add_selection]), they will be fully replaced by the
    /// selections in the [SelectionSet] passed here.
    pub fn set_selection_set(
        mut self,
        selection_set: SelectionSet<'fragset>,
    ) -> Result<Self> {
        self.selection_set = selection_set;
        Ok(self)
    }

    /// Sets the collection of [Variable]s for the [Query].
    ///
    /// NOTE: If any previous variables were added (either using this function
    /// or [QueryBuilder::add_variable]), they will be fully replaced by the
    /// collection of variables passed here.
    pub fn set_variables(mut self, variables: Vec<Variable>) -> Result<Self> {
        self.variables =
            variables.into_iter()
                .map(|var| (var.name.to_owned(), var))
                .collect();
        Ok(self)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum QueryBuildError {
    #[error("Multiple query operations encountered while attempting to build a single query operation")]
    DuplicateOperationDefinition(loc::FilePosition),

    #[error("Found multiple arguments for the same parameter on a field in this query")]
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },

    #[error("Found multiple variables defined with the same name on this query")]
    DuplicateVariableName {
        file_pos1: Option<loc::FilePosition>,
        file_pos2: Option<loc::FilePosition>,
        variable_name: String,
    },

    #[error("Error while building a SelectionSet within this query")]
    SelectionSetBuildError(Box<SelectionSetBuildError>),

    #[error("The Query type name defined by the provided schema does not have an associated type definition")]
    UndefinedQueryType {
        type_name: String,
    },

    #[error("Named type is not defined in the schema for this query")]
    UndefinedVariableType {
        location: loc::FilePosition,
        variable_name: String,
    },
}
impl std::convert::From<SelectionSetBuildError> for QueryBuildError {
    fn from(err: SelectionSetBuildError) -> QueryBuildError {
        QueryBuildError::SelectionSetBuildError(Box::new(err))
    }
}
