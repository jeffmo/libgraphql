use crate::ast;
use crate::DirectiveAnnotationBuilder;
use crate::Value;
use crate::loc;
use crate::loc::SourceLocation;
use crate::named_ref::DerefByName;
use crate::operation::FieldSelection;
use crate::operation::Fragment;
use crate::operation::FragmentRegistry;
use crate::operation::FragmentSpread;
use crate::operation::InlineFragment;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeKind;
use indexmap::IndexMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

type Result<T> = std::result::Result<T, Vec<SelectionSetBuildError>>;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSetBuilder<'schema: 'fragreg, 'fragreg> {
    fragment_registry: &'fragreg FragmentRegistry<'schema>,
    schema: &'schema Schema,
    selections: Vec<Selection<'schema>>,
}
impl<'schema: 'fragreg, 'fragreg> SelectionSetBuilder<'schema, 'fragreg> {
    pub fn add_selection(
        mut self,
        selection: Selection<'schema>,
    ) -> Result<Self> {
        self.selections.push(selection);
        Ok(self)
    }

    pub fn build(self) -> Result<SelectionSet<'schema>> {
        Ok(SelectionSet {
            selections: self.selections,
            schema: self.schema,
        })
    }

    pub fn from_ast(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        parent_type: &'schema GraphQLType,
        ast: &ast::SelectionSet<'_>,
        source_map: &ast::SourceMap<'_>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let parent_fields = match parent_type {
            GraphQLType::Interface(iface_t) => iface_t.fields(),
            GraphQLType::Object(obj_t) => obj_t.fields(),
            _ => return Err(vec![
                SelectionSetBuildError::UnselectableFieldType {
                    location: loc::SourceLocation::from_execdoc_span(
                        file_path,
                        ast.span,
                        source_map,
                    ),
                    parent_type_kind: parent_type.type_kind().to_owned(),
                    parent_type_name: parent_type.name().to_string(),
                }
            ]),
        };

        let mut errors = vec![];
        let mut selections = vec![];
        for ast_selection in &ast.selections {
            // TODO: Need to assert that all field selections are
            //       unambiguously unique.
            //
            //       E.g. There cannot be 2 fields with the same
            //       selection-name same set of argument names/argument
            //       types.
            selections.push(match ast_selection {
                ast::Selection::Field(ast_field) => {
                    let field_name = ast_field.name.value.as_ref();
                    let selected_field_srcloc =
                        loc::SourceLocation::from_execdoc_span(
                            file_path,
                            ast_field.span,
                            source_map,
                        );

                    let selected_field = match parent_fields.get(field_name) {
                        Some(field) => field,
                        None => {
                            errors.push(
                                SelectionSetBuildError::UndefinedFieldName {
                                    location: selected_field_srcloc,
                                    parent_type_name:
                                        parent_type.name().to_string(),
                                    undefined_field_name:
                                        field_name.to_string(),
                                }
                            );
                            continue
                        },
                    };

                    let mut arguments = IndexMap::new();
                    for ast_arg in &ast_field.arguments {
                        let arg_name =
                            ast_arg.name.value.as_ref().to_string();
                        if arguments.insert(
                            arg_name.clone(),
                            Value::from_ast(
                                &ast_arg.value,
                                &selected_field_srcloc,
                            ),
                        ).is_some() {
                            errors.push(
                                SelectionSetBuildError::DuplicateFieldArgument {
                                    argument_name: arg_name,
                                    location1:
                                        selected_field_srcloc.to_owned(),
                                    location2:
                                        selected_field_srcloc.to_owned(),
                                },
                            );
                            continue
                        }
                    }

                    let directives = DirectiveAnnotationBuilder::from_ast(
                        &selected_field_srcloc,
                        source_map,
                        &ast_field.directives,
                    );

                    let selection_set =
                        if let Some(ref ast_sub_ss) = ast_field.selection_set {
                            let maybe_selection_set = Self::from_ast(
                                schema,
                                fragment_registry,
                                selected_field.type_annotation()
                                    .innermost_named_type_annotation()
                                    .graphql_type(schema),
                                ast_sub_ss,
                                source_map,
                                file_path,
                            ).and_then(|builder| builder.build());

                            match maybe_selection_set {
                                Ok(selection_set) => Some(selection_set),
                                Err(mut ss_errors) => {
                                    errors.append(&mut ss_errors);
                                    continue;
                                },
                            }
                        } else {
                            None
                        };

                    Selection::Field(FieldSelection {
                        alias: ast_field.alias.as_ref()
                            .map(|a| a.value.to_string()),
                        arguments,
                        def_location: selected_field_srcloc,
                        directives,
                        field: selected_field,
                        schema,
                        selection_set,
                    })
                },

                ast::Selection::FragmentSpread(ast_frag_spread) => {
                    let fragspread_srcloc =
                        loc::SourceLocation::from_execdoc_span(
                            file_path,
                            ast_frag_spread.span,
                            source_map,
                        );

                    let directives = DirectiveAnnotationBuilder::from_ast(
                        &fragspread_srcloc,
                        source_map,
                        &ast_frag_spread.directives,
                    );

                    Selection::FragmentSpread(FragmentSpread {
                        def_location: fragspread_srcloc.to_owned(),
                        directives,
                        fragment_ref: Fragment::named_ref(
                            ast_frag_spread.name.value.as_ref(),
                            fragspread_srcloc,
                        ),
                    })
                },

                ast::Selection::InlineFragment(ast_inline) => {
                    let inlinespread_srcloc =
                        loc::SourceLocation::from_execdoc_span(
                            file_path,
                            ast_inline.span,
                            source_map,
                        );

                    let directives = DirectiveAnnotationBuilder::from_ast(
                        &inlinespread_srcloc,
                        source_map,
                        &ast_inline.directives,
                    );

                    let parent_type =
                        if let Some(ref type_cond) =
                            ast_inline.type_condition {
                            let typecond_type_name =
                                type_cond.named_type.value.as_ref();
                            let typecond_type =
                                schema.all_types().get(typecond_type_name);

                            let typecond_type =
                                if let Some(typecond_type) = typecond_type {
                                    typecond_type
                                } else {
                                    errors.push(
                                        SelectionSetBuildError::UndefinedTypeQualifierType {
                                            undefined_type_name:
                                                typecond_type_name
                                                    .to_string(),
                                            type_qualifier_location:
                                                inlinespread_srcloc,
                                        }
                                    );
                                    continue;
                                };

                            let is_valid_qualifying_type =
                                match parent_type {
                                    GraphQLType::Bool
                                        | GraphQLType::Enum(_)
                                        | GraphQLType::Float
                                        | GraphQLType::ID
                                        | GraphQLType::InputObject(_)
                                        | GraphQLType::Int
                                        | GraphQLType::Scalar(_)
                                        | GraphQLType::String
                                        => false,

                                    GraphQLType::Interface(parent_iface) =>
                                        typecond_type
                                            .implements_interface(
                                                schema,
                                                parent_iface,
                                            ),

                                    GraphQLType::Object(_) =>
                                        parent_type == typecond_type,

                                    GraphQLType::Union(parent_union) =>
                                        if let GraphQLType::Interface(
                                            typecond_iface
                                        ) = typecond_type {
                                            parent_union
                                                .implements_interface(
                                                    schema,
                                                    typecond_iface,
                                                )
                                        } else {
                                            parent_union.contains_member(
                                                typecond_type,
                                            )
                                        },
                                };

                            if !is_valid_qualifying_type {
                                errors.push(
                                    SelectionSetBuildError::InvalidQualifyingType {
                                        invalid_qualifying_type_name:
                                            typecond_type_name.to_string(),
                                        parent_type_name:
                                            parent_type.name().to_string(),
                                        qualifier_location:
                                            inlinespread_srcloc,
                                    }
                                );
                                continue;
                            }

                            typecond_type
                        } else {
                            parent_type
                        };

                    let maybe_selection_set =
                        SelectionSetBuilder::from_ast(
                            schema,
                            fragment_registry,
                            parent_type,
                            &ast_inline.selection_set,
                            source_map,
                            file_path,
                        ).and_then(|builder| builder.build());

                    let selection_set = match maybe_selection_set {
                        Ok(selection_set) => selection_set,
                        Err(mut ss_errors) => {
                            errors.append(&mut ss_errors);
                            continue;
                        },
                    };

                    Selection::InlineFragment(InlineFragment {
                        directives,
                        selection_set,
                        type_condition:
                            ast_inline.type_condition.as_ref().map(
                                |tc| GraphQLType::named_ref(
                                    tc.named_type.value.as_ref(),
                                    inlinespread_srcloc.with_span(
                                        tc.span,
                                        source_map,
                                    ),
                                ),
                            ),
                        def_location: inlinespread_srcloc,
                    })
                },
            })
        }

        Ok(SelectionSetBuilder {
            fragment_registry,
            schema,
            selections,
        })
    }

    pub fn from_str(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        parent_type: &'schema GraphQLType,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let parse_result = ast::parse_executable(content.as_ref());
        if parse_result.has_errors() {
            return Err(
                parse_result.errors().iter()
                    .map(|e| SelectionSetBuildError::ParseError(
                        Arc::new(e.clone()),
                    ))
                    .collect(),
            );
        }
        let (ast_doc, source_map) = parse_result.into_valid().unwrap();

        let num_defs = ast_doc.definitions.len();
        if num_defs != 1 {
            return Err(vec![
                SelectionSetBuildError::StringIsNotASelectionSet
            ]);
        }

        let selection_set_ast = match ast_doc.definitions.first() {
            Some(ast::Definition::OperationDefinition(op_def))
                if op_def.shorthand =>
            {
                &op_def.selection_set
            },

            _ => return Err(vec![
                SelectionSetBuildError::StringIsNotASelectionSet,
            ]),
        };

        Self::from_ast(
            schema,
            fragment_registry,
            parent_type,
            selection_set_ast,
            &source_map,
            file_path,
        )
    }

    pub fn new(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> Self {
        Self {
            fragment_registry,
            schema,
            selections: vec![],
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum SelectionSetBuildError {
    #[error("Multiple fields selected with the same name")]
    DuplicateFieldArgument {
        argument_name: String,
        location1: loc::SourceLocation,
        location2: loc::SourceLocation,
    },

    #[error(
        "Attempted to selected a type-qualified set of fields using an \
        invalid type. `{invalid_qualifying_type_name}` is not a subtype \
        of `{parent_type_name}`."
    )]
    InvalidQualifyingType {
        invalid_qualifying_type_name: String,
        parent_type_name: String,
        qualifier_location: SourceLocation,
    },

    #[error("Error parsing SelectionSet from string: {0}")]
    ParseError(Arc<ast::GraphQLParseError>),

    #[error("The string provided is not a selection set")]
    StringIsNotASelectionSet,

    #[error(
        "Attempted to select a field named `{undefined_field_name}` on the \
        `{parent_type_name}` type, but `{parent_type_name}` has no such field \
        defined."
    )]
    UndefinedFieldName {
        location: loc::SourceLocation,
        parent_type_name: String,
        undefined_field_name: String,
    },

    #[error(
        "Attempted to select sub-fields on the `{parent_type_name}` type, but \
        `{parent_type_name}` is neither an Object nor an Interface type."
    )]
    UnselectableFieldType {
        location: loc::SourceLocation,
        parent_type_kind: GraphQLTypeKind,
        parent_type_name: String,
    },

    #[error(
        "Attempted to specify `... {undefined_type_name}`, but \
        `{undefined_type_name}` is not a type defined in the schema."
    )]
    UndefinedTypeQualifierType {
        undefined_type_name: String,
        type_qualifier_location: SourceLocation,
    },
}
impl std::convert::From<ast::GraphQLParseError> for SelectionSetBuildError {
    fn from(value: ast::GraphQLParseError) -> Self {
        Self::ParseError(Arc::new(value))
    }
}
