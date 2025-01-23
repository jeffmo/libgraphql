use crate::ast;
use crate::loc;
use crate::schema_builder::SchemaBuildError;
use crate::schema_builder::TypesMapBuilder;
use crate::types::GraphQLDirectiveAnnotation;
use crate::types::GraphQLTypeRef;
use crate::types::InputFieldDef;
use crate::types::NamedDirectiveRef;
use crate::types::GraphQLFieldDef;
use std::collections::HashMap;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
use crate::types::GraphQLType;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[cfg(test)]
pub(super) struct TestBuildFromAst<TType, TExt> {
    pub ast_def: Vec<TType>,
    pub ast_ext: Vec<TExt>,
    pub file_path: PathBuf,
}

pub(super) trait TypeBuilder: Sized {
    type AstTypeDef;
    type AstTypeExtension;

    /// Helper used in tests to quickly run a type definitions AST through a
    /// TypeBuilder types and produce a TypeMap.
    #[cfg(test)]
    fn build_from_ast(
        mut self,
        args: TestBuildFromAst<Self::AstTypeDef, Self::AstTypeExtension>,
    ) -> Result<HashMap<String, GraphQLType>> {
        let mut types_builder = TypesMapBuilder::new();

        for typedef_ast in args.ast_def.into_iter() {
            self.visit_type_def(
                &mut types_builder,
                args.file_path.as_path(),
                typedef_ast,
            )?;
        }

        for typedef_ext in args.ast_ext.into_iter() {
            self.visit_type_extension(
                &mut types_builder,
                args.file_path.as_path(),
                typedef_ext,
            )?;
        }

        types_builder.into_types_map()
    }

    fn finalize(self, types_map_builder: &mut TypesMapBuilder) -> Result<()>;

    fn visit_type_def(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: Self::AstTypeDef,
    ) -> Result<()>;

    fn visit_type_extension(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: Self::AstTypeExtension,
    ) -> Result<()>;
}

pub(super) struct TypeBuilderHelpers;
impl TypeBuilderHelpers {
    pub fn directive_refs_from_ast(
        file_path: &Path,
        directives: &[ast::operation::Directive],
    ) -> Vec<GraphQLDirectiveAnnotation> {
        directives.iter().map(|d| {
            GraphQLDirectiveAnnotation {
                args: d.arguments.clone().into_iter().collect(),
                directive_ref: NamedDirectiveRef::new(
                    &d.name,
                    loc::FilePosition::from_pos(
                        file_path,
                        d.position,
                    ),
                ),
            }
        }).collect()
    }

    pub fn inputobject_fields_from_ast(
        schema_def_location: &loc::SchemaDefLocation,
        input_fields: &[ast::schema::InputValue],
    ) -> Result<HashMap<String, InputFieldDef>> {
        Ok(input_fields.iter().map(|input_field| {
            (input_field.name.to_string(), InputFieldDef {
                def_location: schema_def_location.clone(),
            })
        }).collect())
    }

    pub fn object_fielddefs_from_ast(
        ref_location: &loc::FilePosition,
        fields: &[ast::schema::Field],
    ) -> HashMap<String, GraphQLFieldDef> {
        fields.iter().map(|field| {
            let field_def_position = loc::FilePosition::from_pos(
                ref_location.file.clone(),
                field.position,
            );

            (field.name.to_string(), GraphQLFieldDef {
                type_ref: GraphQLTypeRef::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
                    &field_def_position,
                    &field.field_type,
                ),
                def_location: loc::SchemaDefLocation::Schema(
                    field_def_position,
                ),
            })
        }).collect()
    }
}
