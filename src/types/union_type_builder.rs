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
    extensions: Vec<(Option<PathBuf>, ast::schema::UnionTypeExtension)>,
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
        ext_file_path: Option<&Path>,
        ext: &ast::schema::UnionTypeExtension,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_ast_position(
            ext_file_path,
            &ext.position,
        );
        type_.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            &ext_srcloc,
            &ext.directives,
        ));

        for ext_member_name in ext.types.iter() {
            let ext_member_srcloc = ext_srcloc.with_ast_position(&ext.position);

            // Error if this type is already present in this union defined.
            if let Some(existing_type) = type_.members.get(ext_member_name.as_str()) {
                return Err(SchemaBuildError::DuplicatedUnionMember {
                    type_name: ext_member_name.to_string(),
                    member1: existing_type.ref_location().to_owned(),
                    member2: ext_member_srcloc.to_owned(),
                });
            }
            type_.members.insert(ext_member_name.to_string(), NamedGraphQLTypeRef::new(
                ext_member_name,
                ext_member_srcloc,
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
                    self.merge_type_extension(union_type, ext_path.as_deref(), &ext)?,

                Some(non_union_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_union_type.clone(),
                        extension_location: loc::SourceLocation::from_schema_ast_position(
                            ext_path.as_deref(),
                            &ext.position,
                        ),
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name.to_string(),
                        extension_location: loc::SourceLocation::from_schema_ast_position(
                            ext_path.as_deref(),
                            &ext.position,
                        ),
                    })
            };
        }
        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &<Self as TypeBuilder>::AstTypeDef,
    ) -> Result<()> {
        let uniondef_srcloc = loc::SourceLocation::from_schema_ast_position(
            file_path,
            &def.position,
        );

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            &uniondef_srcloc,
            &def.directives,
        );

        let mut member_type_refs =
            IndexMap::<String, NamedGraphQLTypeRef>::new();
        for member_type_name in &def.types {
            let member_type_name = member_type_name.to_string();
            if let Some(existing_value) = member_type_refs.get(member_type_name.as_str()) {
                return Err(SchemaBuildError::DuplicatedUnionMember {
                    type_name: member_type_name,
                    member1: existing_value.ref_location().to_owned(),
                    member2: uniondef_srcloc,
                });
            } else {
                member_type_refs.insert(
                    member_type_name.to_string(),
                    NamedGraphQLTypeRef::new(
                        member_type_name,
                        uniondef_srcloc.to_owned(),
                    ),
                );
            }
        }

        let member_type_refs =
            def.types
                .iter()
                .map(|type_name| (type_name.to_string(), NamedGraphQLTypeRef::new(
                    type_name,
                    uniondef_srcloc.to_owned(),
                )))
                .collect();

        types_builder.add_new_type(
            def.name.as_str(),
            &uniondef_srcloc.to_owned(),
            GraphQLType::Union(UnionType {
                def_location: uniondef_srcloc,
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
        file_path: Option<&Path>,
        ext: <Self as TypeBuilder>::AstTypeExtension,
    ) -> Result<()> {
        let type_name = ext.name.as_str();
        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Union(union_type)) =>
                self.merge_type_extension(union_type, file_path, &ext),

            Some(non_union_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_union_type.clone(),
                    extension_location: loc::SourceLocation::from_schema_ast_position(
                        file_path,
                        &ext.position,
                    ),
                }),

            None => {
                self.extensions.push(
                    (file_path.map(|p| p.to_path_buf()), ext)
                );
                Ok(())
            },
        }
    }
}
