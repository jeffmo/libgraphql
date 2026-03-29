use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::operation::Fragment;
use crate::operation::FragmentRegistry;
use crate::operation::Selection;
use crate::operation::SelectionSetBuildError;
use crate::operation::SelectionSetBuilder;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeKind;
use crate::types::NamedGraphQLTypeRef;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, FragmentBuildError>;

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentBuilder<'schema: 'fragreg, 'fragreg> {
    def_location: loc::SourceLocation,
    directives: Vec<DirectiveAnnotation>,
    name: Option<String>,
    fragment_registry: &'fragreg FragmentRegistry<'schema>,
    schema: &'schema Schema,
    selection_set_builder: SelectionSetBuilder<'schema, 'fragreg>,
    type_condition_ref: Option<NamedGraphQLTypeRef>,
}

impl<'schema: 'fragreg, 'fragreg> FragmentBuilder<'schema, 'fragreg> {
    /// Add a [`DirectiveAnnotation`] after any previously added
    /// `DirectiveAnnotation`s.
    pub fn add_directive(
        mut self,
        annot: DirectiveAnnotation,
    ) -> Result<Self> {
        // TODO: Error if a non-repeatable directive is added twice
        self.directives.push(annot);
        Ok(self)
    }

    pub fn add_selection(
        mut self,
        selection: Selection<'schema>,
    ) -> Result<Self> {
        self.selection_set_builder =
            self.selection_set_builder
                .add_selection(selection)?;

        Ok(self)
    }

    pub fn build(self) -> Result<Fragment<'schema>> {
        // TODO: Verify that no fragment-spreads within this Fragment's
        //       SelectionSetBuilder form any cycles.
        //
        //       https://spec.graphql.org/September2025/#sec-Fragment-Spreads-Must-Not-Form-Cycles

        let fragment_name = self.name.ok_or(
            FragmentBuildError::NoFragmentNameSpecified {
                fragment_def_src_location: self.def_location.to_owned(),
            }
        )?;

        let type_condition_ref = self.type_condition_ref.ok_or(
            FragmentBuildError::NoTypeConditionSpecified {
                fragment_name: fragment_name.to_owned(),
                fragment_src_location: self.def_location.to_owned(),
            }
        )?;

        Ok(Fragment {
            directives: self.directives,
            def_location: self.def_location,
            name: fragment_name,
            schema: self.schema,
            selection_set: self.selection_set_builder.build()?,
            type_condition_ref,
        })
    }

    pub fn from_ast(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
        ast_frag: &ast::FragmentDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
        file_path: Option<&Path>,
    ) -> Result<Self> {
        let fragdef_srcloc = loc::SourceLocation::from_execdoc_span(
            file_path,
            ast_frag.span,
            source_map,
        );

        let directives = DirectiveAnnotationBuilder::from_ast(
            &fragdef_srcloc,
            source_map,
            &ast_frag.directives,
        );

        let type_condition_type_name =
            ast_frag.type_condition.named_type.value.as_ref();
        let type_condition_type =
            schema.all_types()
                .get(type_condition_type_name)
                .ok_or_else(||
                    FragmentBuildError::TypeConditionTypeDoesNotExistInSchema {
                        fragment_name:
                            ast_frag.name.value.to_string(),
                        fragment_src_location: fragdef_srcloc.to_owned(),
                        type_condition_type_name:
                            type_condition_type_name.to_owned(),
                    }
                )?;

        let selection_set_builder = SelectionSetBuilder::from_ast(
            schema,
            fragment_registry,
            type_condition_type,
            &ast_frag.selection_set,
            source_map,
            file_path,
        )?;

        Ok(Self {
            def_location: fragdef_srcloc.to_owned(),
            directives,
            fragment_registry,
            name: Some(ast_frag.name.value.to_string()),
            schema,
            selection_set_builder,
            type_condition_ref: Some(NamedGraphQLTypeRef::new(
                type_condition_type_name,
                fragdef_srcloc.to_owned(),
            )),
        })
    }

    pub fn set_name(mut self, name: impl Into<String>) -> Result<Self> {
        let _ = self.name.insert(name.into());
        Ok(self)
    }

    pub fn set_type_condition(
        mut self,
        graphql_type: &'schema GraphQLType,
    ) -> Result<Self> {
        match graphql_type {
            GraphQLType::Interface(_)
                | GraphQLType::Object(_)
                | GraphQLType::Union(_)
                => (),

            _ => return Err(
                FragmentBuildError::InvalidFragmentTypeConditionTypeKind {
                    fragment_def_src_location: self.def_location,
                    invalid_type_name: graphql_type.name().to_string(),
                    invalid_type_kind: graphql_type.into(),
                },
            ),
        };

        let _ = self.type_condition_ref.insert(NamedGraphQLTypeRef::new(
            graphql_type.name(),
            // TODO: Hmm... What if self.def_location is changed?
            self.def_location.to_owned(),
        ));

        Ok(self)
    }
}

#[derive(Clone, Debug, Error)]
pub enum FragmentBuildError {
    #[error("Duplicate fragment definition: '{fragment_name}'")]
    DuplicateFragmentDefinition {
        fragment_name: String,
        first_def_location: crate::loc::SourceLocation,
        second_def_location: crate::loc::SourceLocation,
    },

    #[error("Failure while trying to read a fragment document file from disk: $0")]
    FileReadError(Box<crate::file_reader::ReadContentError>),

    #[error("Invalid fragment type condition type: `{invalid_type_kind:?}`")]
    InvalidFragmentTypeConditionTypeKind {
        fragment_def_src_location: loc::SourceLocation,
        invalid_type_name: String,
        invalid_type_kind: GraphQLTypeKind,
    },

    #[error("All fragment definitions must include a name")]
    NoFragmentNameSpecified {
        fragment_def_src_location: loc::SourceLocation,
    },

    #[error(
        "Fragments must specify the type for which they apply to, but none \
        was specified for the `{fragment_name}` fragment."
    )]
    NoTypeConditionSpecified {
        fragment_name: String,
        fragment_src_location: loc::SourceLocation,
    },

    #[error("Error parsing fragment document: {0:?}")]
    ParseErrors(Vec<ast::GraphQLParseError>),

    #[error("Failure to build the selection set for this fragment: $0")]
    SelectionSetBuildErrors(Vec<SelectionSetBuildError>),

    #[error(
        "The `{fragment_name}` fragment declares its type condition as \
        `{type_condition_type_name}`, but this type is not defined in \
        the schema.
    ")]
    TypeConditionTypeDoesNotExistInSchema {
        fragment_name: String,
        fragment_src_location: loc::SourceLocation,
        type_condition_type_name: String,
    },
}
impl std::convert::From<Vec<SelectionSetBuildError>> for FragmentBuildError {
    fn from(value: Vec<SelectionSetBuildError>) -> Self {
        Self::SelectionSetBuildErrors(value)
    }
}
impl std::convert::From<Vec<ast::GraphQLParseError>> for FragmentBuildError {
    fn from(value: Vec<ast::GraphQLParseError>) -> Self {
        Self::ParseErrors(value)
    }
}
