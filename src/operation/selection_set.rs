use crate::ast;
use crate::loc::FilePosition;
use crate::types::GraphQLTypeKind;
use crate::types::TypeAnnotation;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragment;
use crate::operation::NamedFragmentSelection;
use crate::operation::Selection;
use crate::schema::Schema;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::Value;
use indexmap::IndexMap;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, Vec<SelectionSetBuildError>>;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'schema> {
    pub(super) selections: Vec<Selection<'schema>>,
}
impl<'schema> SelectionSet<'schema> {
    // TODO: Move this to a `SelectionSetBuilder` to be more consistent with
    //       other builder-focused API patterns.
    pub fn from_ast<'parent>(
        schema: &'schema Schema,
        parent_type_annotation: &'parent TypeAnnotation,
        ast_sel_set: &ast::operation::SelectionSet,
        file_path: &Path,
    ) -> Result<SelectionSet<'schema>> {
        let parent_type =
            parent_type_annotation.innermost_named_type_annotation()
                .graphql_type(schema);

        let parent_fields = match parent_type {
            GraphQLType::Interface(iface_t) => iface_t.fields(),
            GraphQLType::Object(obj_t) => obj_t.fields(),
            _ => return Err(vec![
                SelectionSetBuildError::UnselectableFieldType {
                    loc: loc::FilePosition::from_pos(
                        file_path,
                        ast_sel_set.span.0,
                    ),
                    parent_type_kind: parent_type.type_kind().to_owned(),
                    parent_type_name: parent_type.name().to_string(),
                }
            ]),
        };

        let mut errors = vec![];
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
                        name: field_name,
                        position: selected_field_ast_position,
                        selection_set: ast_sub_selection_set,
                    }
                ) => {
                    let selected_field_position = loc::FilePosition::from_pos(
                        file_path,
                        *selected_field_ast_position,
                    );

                    let selected_field = match parent_fields.get(field_name) {
                        Some(field) => field,
                        None => {
                            errors.push(
                                SelectionSetBuildError::UndefinedFieldName {
                                    loc: selected_field_position,
                                    parent_type_name: parent_type.name().to_string(),
                                    undefined_field_name: field_name.to_string(),
                                }
                            );
                            continue
                        }
                    };

                    let mut arguments = IndexMap::new();
                    for (arg_name, ast_arg_value) in ast_arguments {
                        if arguments.insert(
                            arg_name.to_string(),
                            Value::from_ast(
                                ast_arg_value,
                                selected_field_position.clone(),
                            ),
                        ).is_some() {
                            errors.push(SelectionSetBuildError::DuplicateFieldArgument {
                                argument_name: arg_name.to_string(),
                                location1: selected_field_position.to_owned(),
                                location2: selected_field_position.to_owned(),
                            });
                            continue
                        }
                    }

                    let mut directives = vec![];
                    for ast_directive in selected_field_ast_directives {
                        let directive_position = loc::FilePosition::from_pos(
                            file_path,
                            ast_directive.position,
                        );

                        let mut arguments = IndexMap::new();
                        for (arg_name, ast_arg_value) in &ast_directive.arguments {
                            if arguments.insert(
                                arg_name.to_string(),
                                Value::from_ast(
                                    ast_arg_value,
                                    directive_position.clone(),
                                ),
                            ).is_some() {
                                errors.push(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.to_owned(),
                                    location2: directive_position.to_owned(),
                                });
                                continue
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
                        selected_field.type_annotation(),
                        ast_sub_selection_set,
                        file_path,
                    )?;

                    Selection::Field(FieldSelection {
                        alias: alias.clone(),
                        arguments,
                        def_location: selected_field_position.into(),
                        directives,
                        field: selected_field,
                        schema,
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

                        let mut arguments = IndexMap::new();
                        for (arg_name, ast_arg_value) in &ast_directive.arguments {
                            if arguments.insert(
                                arg_name.to_string(),
                                Value::from_ast(
                                    ast_arg_value,
                                    directive_position.clone(),
                                ),
                            ).is_some() {
                                errors.push(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.to_owned(),
                                    location2: directive_position.to_owned(),
                                });
                                continue
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
                        def_location: fragspread_position.to_owned().into(),
                        directives,
                        fragment: NamedFragment::named_ref(
                            fragment_name.as_str(),
                            fragspread_position.clone().into(),
                        ),
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

                        let mut arguments = IndexMap::new();
                        for (arg_name, ast_arg_value) in &ast_directive.arguments {
                            if arguments.insert(
                                arg_name.to_string(),
                                Value::from_ast(
                                    ast_arg_value,
                                    directive_position.clone(),
                                ),
                            ).is_some() {
                                errors.push(SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name.to_string(),
                                    location1: directive_position.to_owned(),
                                    location2: directive_position.to_owned(),
                                });
                                continue
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
                        &parent_type.as_type_annotation(
                            /* nullable = */ false,
                        ),
                        ast_sub_selection_set,
                        file_path,
                    )?;

                    Selection::InlineFragment(InlineFragmentSelection {
                        def_location: inlinespread_position.clone().into(),
                        directives,
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
            selections,
        })
    }

    pub fn selections(&self) -> &Vec<Selection<'schema>> {
        &self.selections
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SelectionSetBuildError {
    #[error("Multiple fields selected with the same name")]
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },

    #[error(
        "Attempted to select a field named `{undefined_field_name}` on the \
        `{parent_type_name}` type, but `{parent_type_name}` has no such field \
        defined."
    )]
    UndefinedFieldName {
        loc: FilePosition,
        parent_type_name: String,
        undefined_field_name: String,
    },

    #[error(
        "Attempted to select sub-fields on the `{parent_type_name}` type, but \
        `{parent_type_name}` is neither an Object nor an Interface type."
    )]
    UnselectableFieldType {
        loc: FilePosition,
        parent_type_kind: GraphQLTypeKind,
        parent_type_name: String
    }
}
