use crate::ast;
use crate::file_reader;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::operation::DirectiveAnnotation;
use crate::operation::Query;
use crate::operation::NamedFragment;
use crate::operation::OperationArgValue;
use crate::operation::OperationSelection;
use crate::operation::OperationVarDef;
use crate::schema::Schema;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, OperationBuildError>;

#[derive(Debug)]
pub struct OperationsBuilder<'schema> {
    //queries: HashMap<QueryId, Query>,
    queries: Vec<Query>,
    schema: &'schema Schema,
}
impl<'schema> OperationsBuilder<'schema> {
    pub fn new(schema: &'schema Schema) -> Self {
        Self {
            queries: vec![],
            schema,
        }
    }

    pub fn load_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> Result<()> {
        self.load_files(vec![file_path])
    }

    pub fn load_files<P: AsRef<Path>>(
        &mut self,
        file_paths: Vec<P>,
    ) -> Result<()> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let file_content = file_reader::read_content(file_path)
                .map_err(|err| OperationBuildError::FileReadError(
                    Box::new(err),
                ))?;

            let doc = ast::operation::parse(file_content.as_str())
                    .map_err(|err| OperationBuildError::ParseError {
                        file_path: Some(file_path.to_path_buf()),
                        err,
                    })?;

            for def in doc.definitions {
                self.visit_definition(file_path.to_path_buf(), def)?;
            }
        }
        todo!()
    }

    fn visit_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::operation::Definition,
    ) -> Result<()> {
        use ast::operation::Definition;
        use ast::operation::OperationDefinition as OpDef;
        match def {
            Definition::Fragment(frag_def)
                => self.visit_fragment_definition(file_path, frag_def),

            // Represents the no-name shorthand for OpDef::Query()
            Definition::Operation(OpDef::SelectionSet(sel_set_op))
                => todo!(),

            Definition::Operation(OpDef::Query(query_op))
                => self.visit_query_op_definition(file_path, query_op),

            Definition::Operation(OpDef::Mutation(mut_op))
                => self.visit_mutation_op_definition(file_path, mut_op),

            Definition::Operation(OpDef::Subscription(mut_sub))
                => self.visit_subscription_op_definition(file_path, mut_sub),
        }
    }

    fn visit_fragment_definition(
        &mut self,
        _file_path: PathBuf,
        _def: ast::operation::FragmentDefinition,
    ) -> Result<()> {
        todo!()
    }

    fn visit_query_op_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::operation::Query,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            def.position,
        );

        let mut directives = vec![];
        for ast_directive in def.directives {
            let directive_position = loc::FilePosition::from_pos(
                &file_path,
                ast_directive.position.clone(),
            );

            let mut arguments =
                HashMap::with_capacity(ast_directive.arguments.len());
            for (arg_name, ast_arg_value) in &ast_directive.arguments {
                if arguments.insert(
                    arg_name.to_string(),
                    OperationArgValue::from_ast_value(
                        &ast_arg_value,
                        directive_position.clone(),
                    ),
                ).is_some() {
                    return Err(OperationBuildError::DuplicateFieldArgument {
                        argument_name: arg_name.to_string(),
                        location1: directive_position.clone(),
                        location2: directive_position,
                    });
                }
            }

            directives.push(DirectiveAnnotation {
                arguments,
                directive_ref: Directive::named_ref(
                    ast_directive.name.as_str(),
                    directive_position.clone(),
                ),
                position: directive_position,
            });
        }


        let mut var_defs = HashMap::<String, OperationVarDef>::with_capacity(
            def.variable_definitions.len(),
        );
        for ast_var_def in def.variable_definitions {
            let var_name = ast_var_def.name.to_string();
            let vardef_location = loc::FilePosition::from_pos(
                file_path.to_path_buf(),
                ast_var_def.position,
            );
            let type_ref = GraphQLTypeRef::from_ast_type(
                &vardef_location,
                &ast_var_def.var_type,
            );

            if let Some(var_def) = var_defs.get(var_name.as_str()) {
                return Err(OperationBuildError::DuplicateVariableName {
                    location1: var_def.def_location.clone(),
                    location2: vardef_location,
                    variable_name: var_name,
                })
            }

            // Ensure the inner named type reference is a valid type within the
            // schema.
            type_ref.extract_inner_named_ref()
                .try_deref(&self.schema)
                .map_err(|err| match err {
                    DerefByNameError::DanglingReference(var_name)
                        => OperationBuildError::UndefinedVariableType {
                            variable_name: var_name,
                            location: vardef_location.clone(),
                        },
                })?;

            var_defs.insert(ast_var_def.name.to_string(), OperationVarDef {
                def_location: file_position.clone(),
                default_value: ast_var_def.default_value.clone(),
                name: ast_var_def.name.to_string(),
                type_: GraphQLTypeRef::from_ast_type(
                    &file_position,
                    &ast_var_def.var_type,
                ),
            });
        }

        self.queries.push(Query {
            directives,
            name: def.name,
            selections: load_ast_selection_set(
                &def.selection_set,
                &file_path,
            )?,
            def_location: Some(file_position.clone()),
            var_defs,
        });

        Ok(())
    }

    fn visit_mutation_op_definition(
        &mut self,
        _file_path: PathBuf,
        _def: ast::operation::Mutation,
    ) -> Result<()> {
        todo!()
    }

    fn visit_subscription_op_definition(
        &mut self,
        _file_path: PathBuf,
        _def: ast::operation::Subscription,
    ) -> Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub enum OperationBuildError {
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },
    DuplicateVariableName {
        location1: loc::FilePosition,
        location2: loc::FilePosition,
        variable_name: String,
    },
    FileReadError(Box<file_reader::ReadContentError>),
    ParseError {
        file_path: Option<PathBuf>,
        err: ast::operation::ParseError,
    },
    UndefinedVariableType {
        location: loc::FilePosition,
        variable_name: String,
    },
}

fn load_ast_selection_set(
    ast_selection_set: &ast::operation::SelectionSet,
    file_path: &Path,
) -> Result<Vec<OperationSelection>> {
    let mut selections = vec![];
    for ast_selection in &ast_selection_set.items {
        use ast::operation::Selection;
        selections.push(match ast_selection {
            Selection::Field(ast::operation::Field {
                alias,
                arguments: ast_arguments,
                directives: selected_field_ast_directives,
                name,
                position: selected_field_ast_position,
                selection_set: ast_sub_selection_set,
            }) => {
                let selected_field_position = loc::FilePosition::from_pos(
                    file_path,
                    selected_field_ast_position.clone(),
                );

                let mut arguments =
                    HashMap::with_capacity(ast_arguments.len());
                for (arg_name, ast_arg_value) in ast_arguments {
                    if arguments.insert(
                        arg_name.to_string(),
                        OperationArgValue::from_ast_value(
                            &ast_arg_value,
                            selected_field_position.clone(),
                        ),
                    ).is_some() {
                        return Err(OperationBuildError::DuplicateFieldArgument {
                            argument_name: arg_name.to_string(),
                            location1: selected_field_position.clone(),
                            location2: selected_field_position,
                        });
                    }
                }

                let mut directives = vec![];
                for ast_directive in selected_field_ast_directives {
                    let directive_position = loc::FilePosition::from_pos(
                        file_path,
                        ast_directive.position,
                    );

                    let mut arguments =
                        HashMap::with_capacity(ast_directive.arguments.len());
                    for (arg_name, ast_arg_value) in &ast_directive.arguments {
                        if arguments.insert(
                            arg_name.to_string(),
                            OperationArgValue::from_ast_value(
                                &ast_arg_value,
                                directive_position.clone(),
                            ),
                        ).is_some() {
                            return Err(OperationBuildError::DuplicateFieldArgument {
                                argument_name: arg_name.to_string(),
                                location1: directive_position.clone(),
                                location2: directive_position,
                            });
                        }
                    }

                    directives.push(DirectiveAnnotation {
                        arguments,
                        directive_ref: Directive::named_ref(
                            ast_directive.name.as_str(),
                            directive_position.clone(),
                        ),
                        position: directive_position,
                    });
                }

                let selections = load_ast_selection_set(
                    &ast_sub_selection_set,
                    file_path,
                )?;

                OperationSelection::Field {
                    directives,
                    alias: alias.clone(),
                    arguments,
                    name: name.to_string(),
                    position: selected_field_position,
                    selections,
                }
            },

            Selection::FragmentSpread(ast::operation::FragmentSpread {
                directives: ast_directives,
                fragment_name,
                position: ast_fragspread_position,
            }) => {
                let fragspread_position = loc::FilePosition::from_pos(
                    file_path,
                    *ast_fragspread_position,
                );
                let mut directives = vec![];
                for ast_directive in ast_directives {
                    let directive_position = loc::FilePosition::from_pos(
                        file_path,
                        ast_directive.position.clone(),
                    );

                    let mut arguments =
                        HashMap::with_capacity(ast_directive.arguments.len());
                    for (arg_name, ast_arg_value) in &ast_directive.arguments {
                        if arguments.insert(
                            arg_name.to_string(),
                            OperationArgValue::from_ast_value(
                                &ast_arg_value,
                                directive_position.clone(),
                            ),
                        ).is_some() {
                            return Err(OperationBuildError::DuplicateFieldArgument {
                                argument_name: arg_name.to_string(),
                                location1: directive_position.clone(),
                                location2: directive_position,
                            });
                        }
                    }

                    directives.push(DirectiveAnnotation {
                        arguments,
                        directive_ref: Directive::named_ref(
                            ast_directive.name.as_str(),
                            directive_position.clone(),
                        ),
                        position: directive_position,
                    });
                }
                OperationSelection::NamedFragmentSpread {
                    directives,
                    fragment: NamedFragment::named_ref(
                        fragment_name.as_str(),
                        fragspread_position.clone(),
                    ),
                    position: fragspread_position,
                }
            },

            Selection::InlineFragment(ast::operation::InlineFragment {
                directives: ast_inlinespread_directives,
                position: ast_inlinespread_position,
                selection_set: ast_sub_selection_set,
                type_condition: ast_type_condition,
            }) => {
                let inlinespread_position = loc::FilePosition::from_pos(
                    file_path,
                    *ast_inlinespread_position,
                );

                // TODO: This is copypaste from NamedFragmentSpread and Field above...
                let mut directives = vec![];
                for ast_directive in ast_inlinespread_directives {
                    let directive_position = loc::FilePosition::from_pos(
                        file_path,
                        ast_directive.position,
                    );

                    let mut arguments =
                        HashMap::with_capacity(ast_directive.arguments.len());
                    for (arg_name, ast_arg_value) in &ast_directive.arguments {
                        if arguments.insert(
                            arg_name.to_string(),
                            OperationArgValue::from_ast_value(
                                &ast_arg_value,
                                directive_position.clone(),
                            ),
                        ).is_some() {
                            return Err(OperationBuildError::DuplicateFieldArgument {
                                argument_name: arg_name.to_string(),
                                location1: directive_position.clone(),
                                location2: directive_position,
                            });
                        }
                    }

                    directives.push(DirectiveAnnotation {
                        arguments,
                        directive_ref: Directive::named_ref(
                            ast_directive.name.as_str(),
                            directive_position.clone(),
                        ),
                        position: directive_position,
                    });
                }

                let selections = load_ast_selection_set(
                    &ast_sub_selection_set,
                    file_path,
                )?;

                OperationSelection::InlineFragmentSpread {
                    directives,
                    position: inlinespread_position.clone(),
                    selections,
                    type_condition: ast_type_condition.clone().map(
                        |ast_type_cond| match ast_type_cond {
                            ast::operation::TypeCondition::On(type_name) =>
                                GraphQLType::named_ref(
                                    type_name.as_str(),
                                    inlinespread_position,
                                ),
                        }
                    ),
                }
            },
        })
    }
    Ok(selections)
}

