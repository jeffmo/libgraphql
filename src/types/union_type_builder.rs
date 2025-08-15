use crate::ast;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use crate::types::UnionType;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use inherent::inherent;
use indexmap::IndexMap;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(crate) struct UnionTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::UnionTypeExtension)>,
}

impl UnionTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        type_: &mut UnionType,
        ext_file_path: &Path,
        ext: ast::schema::UnionTypeExtension,
    ) -> Result<()> {
        type_.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            ext_file_path,
            &ext.directives,
        ));

        for ext_type_name in ext.types.iter() {
            let ext_type_loc = loc::FilePosition::from_pos(
                ext_file_path,
                ext.position,
            );

            // Error if this type is already present in this union defined.
            if let Some(existing_value) = type_.members.get(ext_type_name.as_str()) {
                return Err(SchemaBuildError::DuplicatedUnionMember {
                    type_name: ext_type_name.to_string(),
                    member1: existing_value.def_location().clone(),
                    member2: ext_type_loc.into(),
                });
            }
            type_.members.insert(ext_type_name.to_string(), NamedGraphQLTypeRef::new(
                ext_type_name,
                ext_type_loc.into(),
            ));
        }

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for UnionTypeBuilder {
    type AstTypeDef = ast::schema::UnionType;
    type AstTypeExtension = ast::schema::UnionTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Union(union_type)) =>
                    self.merge_type_extension(union_type, ext_path.as_path(), ext)?,

                Some(non_union_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_union_type.clone(),
                        extension_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ).into(),
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name.to_string(),
                        extension_type_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ).into(),
                    })
            }
        }
        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: <Self as TypeBuilder>::AstTypeDef,
    ) -> Result<()> {
        let file_position =
            loc::FilePosition::from_pos(file_path, def.position);

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        let mut member_type_refs = IndexMap::new();
        for member_type_name in &def.types {
            if member_type_refs.insert(
                member_type_name.to_string(),
                NamedGraphQLTypeRef::new(
                    member_type_name,
                    file_position.to_owned().into()
                ),
            ).is_some() {
                // TODO(!!): Duplicate member types!
            }
        }

        let member_type_refs =
            def.types
                .iter()
                .map(|type_name| (type_name.to_string(), NamedGraphQLTypeRef::new(
                    type_name,
                    file_position.clone().into(),
                )))
                .collect();

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Union(UnionType {
                def_location: file_position.into(),
                description: def.description.to_owned(),
                directives,
                name: def.name.to_string(),
                members: member_type_refs,
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        ext: <Self as TypeBuilder>::AstTypeExtension,
    ) -> Result<()> {
        let type_name = ext.name.as_str();
        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Union(union_type)) =>
                self.merge_type_extension(union_type, file_path, ext),

            Some(non_union_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_union_type.clone(),
                    extension_loc: loc::FilePosition::from_pos(
                        file_path,
                        ext.position,
                    ).into(),
                }),

            None => {
                self.extensions.push((file_path.to_path_buf(), ext));
                Ok(())
            },
        }
    }
}
