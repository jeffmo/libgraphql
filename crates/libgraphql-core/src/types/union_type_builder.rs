use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::UnionType;
use indexmap::IndexMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Owned representation of a deferred union type extension.
#[derive(Debug)]
struct DeferredUnionExtension {
    directives: Vec<DirectiveAnnotation>,
    members: IndexMap<String, NamedGraphQLTypeRef>,
    name: String,
    srcloc: loc::SourceLocation,
}

#[derive(Debug)]
pub(crate) struct UnionTypeBuilder {
    extensions: Vec<DeferredUnionExtension>,
}

impl UnionTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_extension(
        &mut self,
        type_: &mut UnionType,
        ext: &mut DeferredUnionExtension,
    ) -> Result<()> {
        type_.directives.append(&mut ext.directives);

        for (member_name, member_ref) in ext.members.drain(..) {
            // Error if this type is already present in this union.
            if let Some(existing_type) = type_.members.get(member_name.as_str()) {
                return Err(SchemaBuildError::DuplicatedUnionMember {
                    type_name: member_name,
                    member1: existing_type.ref_location().to_owned(),
                    member2: member_ref.ref_location().to_owned(),
                });
            }
            type_.members.insert(member_name, member_ref);
        }

        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &ast::UnionTypeDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let uniondef_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            def.span,
            source_map,
        );

        let directives = DirectiveAnnotationBuilder::from_ast(
            &uniondef_srcloc,
            source_map,
            &def.directives,
        );

        let mut member_type_refs =
            IndexMap::<String, NamedGraphQLTypeRef>::new();
        for member_name in &def.members {
            let member_type_name = member_name.value.as_ref().to_string();
            if let Some(existing_value) =
                member_type_refs.get(member_type_name.as_str())
            {
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

        types_builder.add_new_type(
            def.name.value.as_ref(),
            &uniondef_srcloc.to_owned(),
            GraphQLType::Union(UnionType {
                def_location: uniondef_srcloc,
                description: def.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives,
                name: def.name.value.as_ref().to_string(),
                members: member_type_refs,
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: &ast::UnionTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            ext.span,
            source_map,
        );
        let type_name = ext.name.value.as_ref();

        let ext_directives = DirectiveAnnotationBuilder::from_ast(
            &ext_srcloc,
            source_map,
            &ext.directives,
        );

        let mut ext_members = IndexMap::new();
        for member_name in ext.members.iter() {
            let name_str = member_name.value.as_ref().to_string();
            ext_members.insert(
                name_str.clone(),
                NamedGraphQLTypeRef::new(
                    name_str,
                    ext_srcloc.to_owned(),
                ),
            );
        }

        let mut deferred = DeferredUnionExtension {
            directives: ext_directives,
            members: ext_members,
            name: type_name.to_string(),
            srcloc: ext_srcloc.clone(),
        };

        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Union(union_type)) =>
                self.merge_extension(union_type, &mut deferred),

            Some(non_union_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_union_type.clone(),
                    extension_location: ext_srcloc,
                }),

            None => {
                self.extensions.push(deferred);
                Ok(())
            },
        }
    }
}

impl TypeBuilder for UnionTypeBuilder {
    fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some(mut ext) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Union(union_type)) =>
                    self.merge_extension(union_type, &mut ext)?,

                Some(non_union_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_union_type.clone(),
                        extension_location: ext.srcloc,
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name,
                        extension_location: ext.srcloc,
                    }),
            };
        }
        Ok(())
    }
}
