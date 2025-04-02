use crate::ast;
use crate::loc;
use crate::SchemaBuildError;
use crate::types::TypesMapBuilder;
use crate::types::DirectiveAnnotation;
use crate::types::GraphQLTypeRef;
use crate::types::InputField;
use crate::types::NamedDirectiveRef;
use crate::types::Field;
use crate::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[cfg(test)] use std::collections::HashMap;
#[cfg(test)] use std::path::PathBuf;
#[cfg(test)] use crate::types::GraphQLType;

#[cfg(test)] pub struct TestBuildFromAst<TType, TExt> {
    pub ast_def: Vec<TType>,
    pub ast_ext_after: Vec<TExt>,
    pub ast_ext_before: Vec<TExt>,
    pub file_path: PathBuf,
    pub types_after: Vec<GraphQLType>,
    pub types_before: Vec<GraphQLType>,
}

type Result<T> = std::result::Result<T, SchemaBuildError>;

pub trait TypeBuilder: Sized {
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

        for type_ in args.types_before.into_iter() {
            let file_pos = match type_.get_def_location() {
                loc::SchemaDefLocation::GraphQLBuiltIn => continue,
                loc::SchemaDefLocation::Schema(def_loc) => def_loc.clone(),
            };
            types_builder.add_new_type(
                file_pos,
                type_.clone().get_name().unwrap(),
                type_,
            )?;
        }

        for typedef_ext in args.ast_ext_before.into_iter() {
            self.visit_type_extension(
                &mut types_builder,
                args.file_path.as_path(),
                typedef_ext,
            )?;
        }

        for typedef_ast in args.ast_def.into_iter() {
            self.visit_type_def(
                &mut types_builder,
                args.file_path.as_path(),
                typedef_ast,
            )?;
        }

        for typedef_ext in args.ast_ext_after.into_iter() {
            self.visit_type_extension(
                &mut types_builder,
                args.file_path.as_path(),
                typedef_ext,
            )?;
        }

        for type_ in args.types_after.into_iter() {
            let file_pos = match type_.get_def_location() {
                loc::SchemaDefLocation::GraphQLBuiltIn => continue,
                loc::SchemaDefLocation::Schema(def_loc) => def_loc.clone(),
            };
            types_builder.add_new_type(
                file_pos,
                type_.clone().get_name().unwrap(),
                type_,
            )?;
        }

        self.finalize(&mut types_builder)?;
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

pub struct TypeBuilderHelpers;
impl TypeBuilderHelpers {
    pub fn directive_refs_from_ast(
        file_path: &Path,
        directives: &[ast::operation::Directive],
    ) -> Vec<DirectiveAnnotation> {
        directives.iter().map(|ast_annot| {
            let annot_file_pos = loc::FilePosition::from_pos(
                file_path,
                ast_annot.position,
            );
            let mut args = BTreeMap::new();
            for (arg_name, ast_arg) in ast_annot.arguments.iter() {
                args.insert(
                    arg_name.to_string(),
                    Value::from_ast(ast_arg, annot_file_pos.clone()),
                );
            }
            DirectiveAnnotation {
                args,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    annot_file_pos,
                ),
            }
        }).collect()
    }

    pub fn inputobject_fields_from_ast(
        schema_def_location: &loc::SchemaDefLocation,
        input_fields: &[ast::schema::InputValue],
    ) -> Result<BTreeMap<String, InputField>> {
        Ok(input_fields.iter().map(|input_field| {
            (input_field.name.to_string(), InputField {
                def_location: schema_def_location.clone(),
            })
        }).collect())
    }

    pub fn object_fielddefs_from_ast(
        ref_location: &loc::FilePosition,
        fields: &[ast::schema::Field],
    ) -> BTreeMap<String, Field> {
        fields.iter().map(|field| {
            let field_def_position = loc::FilePosition::from_pos(
                ref_location.file.clone(),
                field.position,
            );

            (field.name.to_string(), Field {
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
