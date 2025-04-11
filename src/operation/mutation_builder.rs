use crate::ast;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::Schema;
use crate::types::Directive;
use crate::types::DirectiveAnnotation;
use crate::types::GraphQLTypeRef;
use crate::operation::Mutation;
use crate::operation::OperationImpl;
use crate::operation::OperationBuilder;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::SelectionSetBuildError;
use crate::operation::Variable;
use crate::Value;
use inherent::inherent;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, MutationBuildError>;

#[derive(Debug)]
pub struct MutationBuilder<'schema> {
    annotations: Vec<DirectiveAnnotation>,
    def_location: Option<loc::FilePosition>,
    name: Option<String>,
    schema: &'schema Schema,
    selection_set: SelectionSet<'schema>,
    variables: BTreeMap<String, Variable>,
}
impl<'schema> MutationBuilder<'schema> {
}

#[inherent]
impl<'schema> OperationBuilder<
    'schema,
    ast::operation::Mutation,
    MutationBuildError,
    Mutation<'schema>,
> for MutationBuilder<'schema> {
    /// Adds a [DirectiveAnnotation] to the [Mutation].
    ///
    /// If other annotations are already present, this will add the new
    /// annotation after the others.
    pub fn add_annotation(
        mut self,
        annot: DirectiveAnnotation,
    ) -> Result<Self> {
        // TODO: Error if a non-repeatable annotation is added twice
        self.annotations.push(annot);
        Ok(self)
    }

    /// Adds a [Selection] to the [Mutation].
    ///
    /// If other selections are already present, this will add the new selection
    /// after the others.
    pub fn add_selection(
        mut self,
        selection: Selection<'schema>,
    ) -> Result<Self> {
        self.selection_set.selections.push(selection);
        Ok(self)
    }

    /// Adds a [Variable] to the [Mutation].
    pub fn add_variable(
        mut self,
        variable: Variable,
    ) -> Result<Self> {
        if self.variables.get(variable.name.as_str()).is_some() {
            return Err(MutationBuildError::DuplicateVariableName {
                file_pos1: None,
                file_pos2: None,
                variable_name: variable.name,
            })
        }
        self.variables.insert(variable.name.to_owned(), variable);
        Ok(self)
    }

    /// Consume the [MutationBuilder] to produce a [Mutation].
    pub fn build(self) -> Result<Mutation<'schema>> {
        Ok(Mutation(OperationImpl {
            annotations: self.annotations,
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

    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema>> {
        if schema.mutation_type.is_none() {
            return Err(MutationBuildError::NoMutationTypeDefinedInSchema);
        }

        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );

        let mut mutation_annotations = vec![];
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
                    return Err(MutationBuildError::DuplicateFieldArgument {
                        argument_name: arg_name.to_string(),
                        location1: directive_position.clone(),
                        location2: directive_position,
                    });
                }
            }

            mutation_annotations.push(DirectiveAnnotation {
                args: arguments,
                directive_ref: Directive::named_ref(
                    ast_directive.name.as_str(),
                    directive_position,
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
            let type_ref = GraphQLTypeRef::from_ast_type(
                &vardef_location,
                &ast_var_def.var_type,
            );

            if let Some(var_def) = variables.get(var_name.as_str()) {
                return Err(MutationBuildError::DuplicateVariableName {
                    file_pos1: Some(var_def.def_location.clone()),
                    file_pos2: Some(vardef_location),
                    variable_name: var_name,
                })
            }

            // Ensure the inner named type reference is a valid type within the
            // schema.
            type_ref.extract_inner_named_ref()
                .deref(schema)
                .map_err(|err| match err {
                    DerefByNameError::DanglingReference(var_name)
                        => MutationBuildError::UndefinedVariableType {
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
                type_: GraphQLTypeRef::from_ast_type(
                    &file_position,
                    &ast_var_def.var_type,
                ),
            });
        }

        Ok(Mutation(OperationImpl {
            annotations: mutation_annotations,
            def_location: Some(file_position.clone()),
            name: def.name,
            phantom_ast: PhantomData,
            phantom_error: PhantomData,
            phantom_op: PhantomData,
            phantom_builder: PhantomData,
            schema,
            selection_set: SelectionSet::from_ast(
                schema,
                file_path,
                &def.selection_set,
            )?,
            variables,
        }))
    }

    pub fn new(schema: &'schema Schema) -> Result<Self> {
        if schema.mutation_type.is_none() {
            return Err(MutationBuildError::NoMutationTypeDefinedInSchema);
        }

        Ok(MutationBuilder {
            annotations: vec![],
            def_location: None,
            name: None,
            schema,
            selection_set: SelectionSet {
                schema,
                selections: vec![],
            },
            variables: BTreeMap::new(),
        })
    }

    /// Sets the list of [DirectiveAnnotation]s.
    ///
    /// NOTE: If any previous annotations were added (either using this function
    /// or [MutationBuilder::add_annotation]), they will be fully replaced by the
    /// [Vec] passed here.
    pub fn set_annotations(
        mut self,
        annots: &[DirectiveAnnotation],
    ) -> Result<Self> {
        self.annotations = annots.into();
        Ok(self)
    }

    /// Sets the name of the [Mutation].
    pub fn set_name(mut self, name: Option<String>) -> Result<Self> {
        self.name = name;
        Ok(self)
    }

    /// Sets the [SelectionSet] for the [Mutation].
    ///
    /// NOTE: If any previous selections were added (either using this function
    /// or [MutationBuilder::add_selection]), they will be fully replaced by the
    /// selections in the [SelectionSet] passed here.
    pub fn set_selection_set(
        mut self,
        selection_set: SelectionSet<'schema>,
    ) -> Result<Self> {
        self.selection_set = selection_set;
        Ok(self)
    }

    /// Sets the collection of [Variable]s for the [Mutation].
    ///
    /// NOTE: If any previous variables were added (either using this function
    /// or [MutationBuilder::add_variable]), they will be fully replaced by the
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
pub enum MutationBuildError {
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

    #[error("No Mutation type defined on this schema")]
    NoMutationTypeDefinedInSchema,

    #[error("Error while building a SelectionSet within this query")]
    SelectionSetBuildError(Box<SelectionSetBuildError>),

    #[error("Named type is not defined in the schema for this query")]
    UndefinedVariableType {
        location: loc::FilePosition,
        variable_name: String,
    },
}
impl std::convert::From<SelectionSetBuildError> for MutationBuildError {
    fn from(err: SelectionSetBuildError) -> MutationBuildError {
        MutationBuildError::SelectionSetBuildError(Box::new(err))
    }
}
