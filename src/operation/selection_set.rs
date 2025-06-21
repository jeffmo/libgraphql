use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragment;
use crate::operation::NamedFragmentSelection;
use crate::operation::Selection;
use crate::schema::Schema;
use crate::Value;
use crate::types::Directive;
use crate::types::GraphQLType;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, SelectionSetBuildError>;

#[derive(Debug)]
pub struct SelectionSet<'schema> {
    pub(super) selections: Vec<Selection<'schema>>,
    pub(super) schema: &'schema Schema,
}
impl<'schema> SelectionSet<'schema> {
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        ast_sel_set: &ast::operation::SelectionSet,
    ) -> Result<SelectionSet<'schema>> {
        let mut selections = vec![];
        for ast_selection in &ast_sel_set.items {
            // TODO: Need to assert that all field selections are unambiguously unique.
            //
            //       E.g. There cannot be 2 fields with the same selection-name same set of
            //       argument names/argument types.
            selections.push(match ast_selection {
                ast::operation::Selection::Field(
                    ast::operation::Field {
                        alias,
                        arguments: ast_arguments,
                        directives: selected_field_ast_directives,
                        name,
                        position: selected_field_ast_position,
                        selection_set: ast_sub_selection_set,
                    }
                ) => {
                    let selected_field_position = loc::FilePosition::from_pos(
                        file_path,
                        *selected_field_ast_position,
                    );

                    let mut arguments =
                        HashMap::with_capacity(ast_arguments.len());
                    for (arg_name, ast_arg_value) in ast_arguments {
                        if arguments.insert(
                            arg_name.to_string(),
                            Value::from_ast(
                                ast_arg_value,
                                selected_field_position.clone(),
                            ),
                        ).is_some() {
                            return Err(SelectionSetBuildError::DuplicateFieldArgument {
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

                        let mut arguments = BTreeMap::new();
                        for (arg_name, ast_arg_value) in &ast_directive.arguments {
                            if arguments.insert(
                                arg_name.to_string(),
                                Value::from_ast(
                                    ast_arg_value,
                                    directive_position.clone(),
                                ),
                            ).is_some() {
                                return Err(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.clone(),
                                    location2: directive_position,
                                });
                            }
                        }

                        directives.push(DirectiveAnnotation {
                            args: arguments,
                            directive_ref: Directive::named_ref(
                                ast_directive.name.as_str(),
                                loc::SchemaDefLocation::Schema(directive_position),
                            ),
                        });
                    }

                    let selection_set = SelectionSet::from_ast(
                        schema,
                        file_path,
                        ast_sub_selection_set,
                    )?;

                    Selection::Field(FieldSelection {
                        directives,
                        alias: alias.clone(),
                        arguments,
                        name: name.to_string(),
                        position: selected_field_position,
                        selection_set,
                    })
                },

                ast::operation::Selection::FragmentSpread(
                    ast::operation::FragmentSpread {
                        directives: ast_directives,
                        fragment_name,
                        position: ast_fragspread_position,
                    }
                ) => {
                    let fragspread_position = loc::FilePosition::from_pos(
                        file_path,
                        *ast_fragspread_position,
                    );
                    let mut directives = vec![];
                    for ast_directive in ast_directives {
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
                                return Err(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.clone(),
                                    location2: directive_position,
                                });
                            }
                        }

                        directives.push(DirectiveAnnotation {
                            args: arguments,
                            directive_ref: Directive::named_ref(
                                ast_directive.name.as_str(),
                                loc::SchemaDefLocation::Schema(directive_position),
                            ),
                        });
                    }

                    Selection::NamedFragment(NamedFragmentSelection {
                        directives,
                        fragment: NamedFragment::named_ref(
                            fragment_name.as_str(),
                            loc::SchemaDefLocation::Schema(fragspread_position.clone()),
                        ),
                        position: fragspread_position,
                    })
                },

                ast::operation::Selection::InlineFragment(
                    ast::operation::InlineFragment {
                        directives: ast_inlinespread_directives,
                        position: ast_inlinespread_position,
                        selection_set: ast_sub_selection_set,
                        type_condition: ast_type_condition,
                    }
                ) => {
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

                        let mut arguments = BTreeMap::new();
                        for (arg_name, ast_arg_value) in &ast_directive.arguments {
                            if arguments.insert(
                                arg_name.to_string(),
                                Value::from_ast(
                                    ast_arg_value,
                                    directive_position.clone(),
                                ),
                            ).is_some() {
                                return Err(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.clone(),
                                    location2: directive_position,
                                });
                            }
                        }

                        directives.push(DirectiveAnnotation {
                            args: arguments,
                            directive_ref: Directive::named_ref(
                                ast_directive.name.as_str(),
                                loc::SchemaDefLocation::Schema(directive_position),
                            ),
                        });
                    }

                    let selection_set = SelectionSet::from_ast(
                        schema,
                        file_path,
                        ast_sub_selection_set,
                    )?;

                    Selection::InlineFragment(InlineFragmentSelection {
                        directives,
                        position: inlinespread_position.clone(),
                        selection_set,
                        type_condition: ast_type_condition.clone().map(
                            |ast_type_cond| match ast_type_cond {
                                ast::operation::TypeCondition::On(type_name) =>
                                    GraphQLType::named_ref(
                                        type_name.as_str(),
                                        loc::SchemaDefLocation::Schema(
                                            inlinespread_position,
                                        ),
                                    ),
                            }
                        ),
                    })
                },
            })
        }

        Ok(SelectionSet {
            schema,
            selections,
        })
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SelectionSetBuildError {
    #[error("Multiple fields selected with the same name")]
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },
}
