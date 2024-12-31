use crate::ast;
use crate::file_reader;
use crate::loc;
use crate::schema::Schema;
use crate::types::Directive;
use crate::types::EnumVariant;
use crate::types::ObjectFieldDef;
use crate::types::InputFieldDef;
use crate::types::GraphQLEnumType;
use crate::types::GraphQLObjectType;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;
use crate::types::NamedDirectiveRef;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

type Result<T> = std::result::Result<T, SchemaBuildError>;
type TypecheckResult<T> = std::result::Result<T, SchemaTypecheckError>;

lazy_static::lazy_static! {
    static ref BUILTIN_DIRECTIVE_NAMES: HashSet<&'static str> = {
        HashSet::from([
            "skip",
            "include",
            "deprecated",
            "specifiedBy",
        ])
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphQLOperation {
    Query,
    Mutation,
    Subscription,
}

/// Utility for building a [Schema].
#[derive(Debug)]
pub struct SchemaBuilder {
    directives: HashMap<String, Directive>,
    query_type: Option<NamedTypeFilePosition>,
    mutation_type: Option<NamedTypeFilePosition>,
    str_load_counter: u16,
    subscription_type: Option<NamedTypeFilePosition>,
    types: HashMap<String, GraphQLType>,
    type_extensions: Vec<(PathBuf, ast::schema::TypeExtension)>,
}
impl SchemaBuilder {
    pub fn build(mut self) -> Result<Schema> {
        let query_typedefloc =
            if let Some(def) = self.query_type.take() {
                def
            } else {
                match self.types.get("Query") {
                    Some(GraphQLType::Object(obj_type)) => NamedTypeFilePosition {
                        def_location: obj_type.def_location.clone(),
                        type_name: "Query".to_string(),
                    },
                    _ => return Err(SchemaBuildError::NoQueryOperationTypeDefined),
                }
            };

        let mutation_type =
            if let Some(def) = self.mutation_type.take() {
                Some(def)
            } else {
                match self.types.get("Mutation") {
                    Some(GraphQLType::Object(obj_type)) => Some(NamedTypeFilePosition {
                        def_location: obj_type.def_location.clone(),
                        type_name: "Mutation".to_string(),
                    }),
                    _ => None,
                }
            };

        let subscription_type =
            if let Some(def) = self.subscription_type.take() {
                Some(def)
            } else {
                match self.types.get("Subscription") {
                    Some(GraphQLType::Object(obj_type)) => Some(NamedTypeFilePosition {
                        def_location: obj_type.def_location.clone(),
                        type_name: "Subscription".to_string(),
                    }),
                    _ => None,
                }
            };

        self.inject_missing_builtin_directives();
        self.merge_type_extensions()?;

        // Fun side-quest: Check types eagerly while visiting them. When there's a possibility that
        // a type error could be resolved (or manifested) later, track a
        self.check_types()?;

        Ok(Schema {
            directives: self.directives,
            query_type: NamedGraphQLTypeRef::new(
                query_typedefloc.type_name,
                query_typedefloc.def_location,
            ),
            mutation_type: mutation_type.map(|t| NamedGraphQLTypeRef::new(
                t.type_name,
                t.def_location,
            )),
            subscription_type: subscription_type.map(|t| NamedGraphQLTypeRef::new(
                t.type_name,
                t.def_location,
            )),
            types: self.types,
        })
    }

    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            query_type: None,
            mutation_type: None,
            str_load_counter: 0,
            subscription_type: None,
            type_extensions: vec![],
            types: HashMap::from([
                ("Boolean".to_string(), GraphQLType::Bool),
                ("Float".to_string(), GraphQLType::Float),
                ("ID".to_string(), GraphQLType::ID),
                ("Int".to_string(), GraphQLType::Int),
                ("String".to_string(), GraphQLType::String),
            ]),
        }
    }

    pub fn load_from_file(
        self,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        self.load_from_files(vec![file_path])
    }

    pub fn load_from_files(
        mut self,
        file_paths: Vec<impl AsRef<Path>>,
    ) -> Result<Self> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let file_content = file_reader::read_content(file_path)
                .map_err(|err| SchemaBuildError::SchemaFileReadError(
                    Box::new(err),
                ))?;
            self = self.load_from_str(
                Some(file_path.to_path_buf()),
                file_content.as_str(),
            )?;
        }
        Ok(self)
    }

    pub fn load_from_str(
        mut self,
        file_path: Option<PathBuf>,
        content: &str,
    ) -> Result<Self> {
        let file_path =
            if let Some(file_path) = file_path {
                file_path
            } else {
                let ctr = self.str_load_counter;
                self.str_load_counter += 1;
                PathBuf::from(format!("str://{}", ctr))
            };

        let doc =
            graphql_parser::schema::parse_schema::<String>(content)
                .map_err(|err| SchemaBuildError::SchemaParseError {
                    file: file_path.to_owned(),
                    err: err.to_string(),
                })?.into_static();

        for def in doc.definitions {
            self.visit_definition(file_path.as_path(), def)?;
        }

        Ok(self)
    }

    fn check_for_conflicting_type(
        &self,
        file_location: &loc::SchemaDefLocation,
        name: &str,
    ) -> Result<()> {
        if let Some(conflicting_type) = self.types.get(name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: name.to_string(),
                def1: conflicting_type.get_def_location().clone(),
                def2: file_location.clone(),
            })?;
        }
        Ok(())
    }

    fn check_types(&self) -> TypecheckResult<()> {
        // TODO: Typecheck the schema after we've fully parsed it.
        //
        //       Fun side-quest: Eagerly typecheck each definition as it is
        //       processed. If types are missing which are needed for
        //       typechecking, store a constraint in some kind of
        //       ProcessingContext struct to come back to later (either when the
        //       needed types get defined OR at the very end of processing).

        Ok(())
    }

   fn inject_missing_builtin_directives(&mut self) {
        if !self.directives.contains_key("skip") {
            self.directives.insert("skip".to_string(), Directive::Skip);
        }

        if !self.directives.contains_key("include") {
            self.directives.insert("include".to_string(), Directive::Include);
        }

        if !self.directives.contains_key("deprecated") {
            self.directives.insert("deprecated".to_string(), Directive::Deprecated);
        }

        if !self.directives.contains_key("specifiedBy") {
            self.directives.insert("specifiedBy".to_string(), Directive::SpecifiedBy);
        }
    }

    fn merge_enum_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::EnumTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Enum(GraphQLEnumType {
                def_location,
                directives,
                variants,
                ..
            })) => {
                directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));

                for ext_val in ext.values.iter() {
                    let ext_val_loc = loc::FilePosition::from_pos(
                        file_path,
                        ext_val.position,
                    );

                    // Error if this value is already defined.
                    if let Some(existing_value) = variants.get(ext_val.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                            enum_name: ext.name.to_string(),
                            enum_def_location: def_location.clone(),
                            value_def1: existing_value.def_location.clone(),
                            value_def2: ext_val_loc,
                        });
                    }
                    variants.insert(ext_val.name.to_string(), EnumVariant {
                        def_location: ext_val_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_inputobj_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::InputObjectTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::InputObject {
                directives: target_directives,
                fields: target_fields,
                ..
            }) => {
                target_directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_loc = loc::SchemaDefLocation::Schema(
                        loc::FilePosition::from_pos(
                            file_path,
                            ext_field.position,
                        )
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = target_fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            type_name: ext.name.to_string(),
                            field_name: ext_field.name.to_string(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        })?;
                    }
                    target_fields.insert(ext_field.name.to_string(), InputFieldDef {
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_interface_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::InterfaceTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Interface {
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_pos = loc::FilePosition::from_pos(
                        file_path,
                        ext_field.position,
                    );
                    let ext_field_loc = loc::SchemaDefLocation::Schema(
                        ext_field_pos.clone(),
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            type_name: ext.name.to_string(),
                            field_name: ext_field.name.to_string(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        })?;
                    }

                    fields.insert(ext_field.name.to_string(), ObjectFieldDef {
                        type_ref: GraphQLTypeRef::from_ast_type(
                            &ext_field_pos,
                            &ext_field.field_type,
                        ),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_object_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::ObjectTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Object(obj_type)) => {
                obj_type.directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_pos = loc::FilePosition::from_pos(
                        file_path,
                        ext_field.position,
                    );
                    let ext_field_loc = loc::SchemaDefLocation::Schema(
                        ext_field_pos.clone()
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = obj_type.fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            type_name: ext.name.to_string(),
                            field_name: ext_field.name.to_string(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        })?;
                    }
                    obj_type.fields.insert(ext_field.name.to_string(), ObjectFieldDef {
                        type_ref: GraphQLTypeRef::from_ast_type(
                            &ext_field_pos,
                            &ext_field.field_type,
                        ),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_scalar_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::ScalarTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Scalar {
                directives,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));
                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::TypeExtension,
    ) -> Result<()> {
        use ast::schema::TypeExtension;
        match ext {
            TypeExtension::Scalar(ext) =>
                self.merge_scalar_type_extension(file_path, ext),
            TypeExtension::Object(ext) =>
                self.merge_object_type_extension(file_path, ext),
            TypeExtension::Interface(ext) =>
                self.merge_interface_type_extension(file_path, ext),
            TypeExtension::Union(ext) =>
                self.merge_union_type_extension(file_path, ext),
            TypeExtension::Enum(ext) =>
                self.merge_enum_type_extension(file_path, ext),
            TypeExtension::InputObject(ext) =>
                self.merge_inputobj_type_extension(file_path, ext),
        }
    }

    fn merge_type_extensions(&mut self) -> Result<()> {
        while let Some((file_path, type_ext)) = self.type_extensions.pop() {
            self.merge_type_extension(file_path.as_path(), type_ext)?;
        }
        Ok(())
    }

    fn merge_union_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::UnionTypeExtension,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Union {
                directives,
                types: unioned_types,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    file_path,
                    &ext.directives,
                ));

                for ext_type in ext.types.iter() {
                    let ext_type_loc = loc::FilePosition::from_pos(
                        file_path,
                        ext.position,
                    );

                    // Error if this type is already specified as a member of this union.
                    if let Some(existing_value) = unioned_types.get(ext_type) {
                        return Err(SchemaBuildError::DuplicatedUnionMember {
                            type_name: ext.name.to_string(),
                            member1: existing_value.get_ref_location().clone(),
                            member2: ext_type_loc,
                        })?;
                    }
                    unioned_types.insert(ext_type.to_string(), GraphQLTypeRef::Named {
                        nullable: false,
                        type_ref: NamedGraphQLTypeRef::new(
                            ext_type,
                            ext_type_loc,
                        ),
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                schema_type: schema_type.clone(),
                extension_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn visit_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::Definition,
    ) -> Result<()> {
        use ast::schema::Definition;
        match def {
            Definition::SchemaDefinition(schema_def) =>
                self.visit_schemablock_definition(file_path, schema_def),
            Definition::TypeDefinition(type_def) =>
                self.visit_type_definition(file_path, type_def),
            Definition::TypeExtension(type_ext) =>
                self.visit_type_extension(file_path, type_ext),
            Definition::DirectiveDefinition(directive_def) =>
                self.visit_directive_definition(file_path, directive_def),
        }
    }

    fn visit_directive_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::DirectiveDefinition,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );

        if BUILTIN_DIRECTIVE_NAMES.contains(def.name.as_str()) {
            return Err(SchemaBuildError::RedefinitionOfBuiltinDirective {
                directive_name: def.name,
                location: file_position,
            })?;
        }

        if let Some(Directive::Custom {
            def_location,
            ..
        }) = self.directives.get(def.name.as_str()) {
            return Err(SchemaBuildError::DuplicateDirectiveDefinition {
                directive_name: def.name.clone(),
                location1: def_location.clone(),
                location2: file_position,
            })?;
        }

        self.directives.insert(def.name.to_string(), Directive::Custom {
            def_location: file_position,
            name: def.name.to_string(),
        });
        Ok(())
    }

    fn visit_enum_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::EnumType,
    ) -> Result<()> {
        let file_position =
            loc::FilePosition::from_pos(file_path, def.position);
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        let variants =
            def.values
                .iter()
                .map(|val| (val.name.to_string(), EnumVariant {
                    def_location: loc::FilePosition::from_pos(
                        file_path,
                        val.position,
                    ),
                }))
                .collect();

        self.types.insert(
            def.name.to_string(),
            GraphQLType::Enum(GraphQLEnumType {
                def_location: file_position,
                directives,
                name: def.name.to_string(),
                variants,
            }),
        );

        Ok(())
    }

    fn visit_inputobj_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::InputObjectType,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = inputobj_fields_from_ast(
            &schema_def_location,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::InputObject {
            def_location: file_position,
            directives,
            fields,
            name: def.name.to_string(),
        });

        Ok(())
    }

    fn visit_interface_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::InterfaceType,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = object_fields_from_ast(
            &file_position,
            &def.fields,
        );

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Interface {
            def_location: file_position,
            directives,
            fields,
            name: def.name.to_string(),
        });
        Ok(())
    }

    fn visit_object_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::ObjectType,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = object_fields_from_ast(
            &file_position,
            &def.fields,
        );

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Object(
            GraphQLObjectType {
                def_location: file_position,
                directives,
                fields,
                name: def.name.to_string(),
            }
        ));
        Ok(())
    }

    fn visit_scalar_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::ScalarType,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Scalar {
            def_location: schema_def_location,
            directives,
            name: def.name.to_string(),
        });
        Ok(())
    }

    fn visit_schemablock_definition(
        &mut self,
        file_path: &Path,
        schema_def: ast::schema::SchemaDefinition,
    ) -> Result<()> {
        if let Some(type_name) = &schema_def.query {
            let typedef_loc = NamedTypeFilePosition::from_pos(
                type_name.to_string(),
                file_path,
                schema_def.position,
            );
            if let Some(existing_typedef_loc) = &self.query_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Query,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                })?;
            }
            self.query_type = Some(typedef_loc);
        }

        if let Some(type_name) = &schema_def.mutation {
            let typedef_loc = NamedTypeFilePosition::from_pos(
                type_name.to_string(),
                file_path,
                schema_def.position,
            );
            if let Some(existing_typedef_loc) = &self.mutation_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Mutation,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                })?;
            }
            self.mutation_type = Some(typedef_loc);
        }

        if let Some(type_name) = &schema_def.subscription {
            let typedef_loc = NamedTypeFilePosition::from_pos(
                type_name.to_string(),
                file_path,
                schema_def.position,
            );
            if let Some(existing_typedef_loc) = &self.subscription_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Subscription,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                })?;
            }
            self.subscription_type = Some(typedef_loc);
        }

        Ok(())
    }

    fn visit_type_definition(
        &mut self,
        file_path: &Path,
        type_def: ast::schema::TypeDefinition,
    ) -> Result<()> {
        match type_def {
            ast::schema::TypeDefinition::Enum(enum_def) =>
                self.visit_enum_type_definition(file_path, enum_def),
            ast::schema::TypeDefinition::InputObject(inputobj_def) =>
                self.visit_inputobj_type_definition(file_path, inputobj_def),
            ast::schema::TypeDefinition::Interface(iface_def) =>
                self.visit_interface_type_definition(file_path, iface_def),
            ast::schema::TypeDefinition::Scalar(scalar_def) =>
                self.visit_scalar_type_definition(file_path, scalar_def),
            ast::schema::TypeDefinition::Object(obj_def) =>
                self.visit_object_type_definition(file_path, obj_def),
            ast::schema::TypeDefinition::Union(union_def) =>
                self.visit_union_type_definition(file_path, union_def),
        }
    }

    fn visit_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::TypeExtension,
    ) -> Result<()> {
        self.type_extensions.push((file_path.to_owned(), ext));
        Ok(())
    }

    fn visit_union_type_definition(
        &mut self,
        file_path: &Path,
        def: ast::schema::UnionType,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::Schema(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        let types = def.types.iter().map(|type_name| {
            (type_name.to_string(), GraphQLTypeRef::Named {
                nullable: false,
                type_ref: NamedGraphQLTypeRef::new(
                    type_name,
                    file_position.clone(),
                ),
            })
        }).collect();

        self.types.insert(def.name.to_string(), GraphQLType::Union {
            def_location: file_position,
            directives,
            name: def.name.to_string(),
            types,
        });

        Ok(())
    }
}
impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SchemaBuildError {
    #[error("Multiple directives were defined with the same name")]
    DuplicateDirectiveDefinition {
        directive_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },

    #[error("Multiple enum variants with the same name were defined on a single enum type")]
    DuplicateEnumValueDefinition {
        enum_name: String,
        enum_def_location: loc::FilePosition,
        value_def1: loc::FilePosition,
        value_def2: loc::FilePosition,
    },

    #[error("Multiple fields with the same name were defined on a single object type")]
    DuplicateFieldNameDefinition {
        type_name: String,
        field_name: String,
        field_def1: loc::SchemaDefLocation,
        field_def2: loc::SchemaDefLocation,
    },

    #[error("Multiple definitions of the same operation were defined")]
    DuplicateOperationDefinition {
        operation: GraphQLOperation,
        location1: NamedTypeFilePosition,
        location2: NamedTypeFilePosition,
    },

    #[error("Multiple GraphQL types with the same name were defined")]
    DuplicateTypeDefinition {
        type_name: String,
        def1: loc::SchemaDefLocation,
        def2: loc::SchemaDefLocation,
    },

    #[error("A union type specifies the same type as a member multiple times")]
    DuplicatedUnionMember {
        type_name: String,
        member1: loc::FilePosition,
        member2: loc::FilePosition,
    },

    #[error("Attempted to extend a type that is not defined elsewhere")]
    ExtensionOfUndefinedType {
        type_name: String,
        extension_type_loc: loc::FilePosition,
    },

    #[error("Attempted to extend a type using a name that corresponds to a different kind of type")]
    InvalidExtensionType {
        schema_type: GraphQLType,
        extension_loc: loc::FilePosition,
    },

    #[error("Attempted to build a schema that has no Query operation type defined")]
    NoQueryOperationTypeDefined,

    #[error("Attempted to redefine a builtin directive")]
    RedefinitionOfBuiltinDirective {
        directive_name: String,
        location: loc::FilePosition,
    },

    #[error("Failure while trying to read a schema file from disk")]
    SchemaFileReadError(Box<file_reader::ReadContentError>),

    #[error("Error parsing schema content")]
    SchemaParseError {
        file: PathBuf,
        err: String,
    },

    #[error("Error while checking the types of a loaded schema")]
    TypecheckError(Box<SchemaTypecheckError>),
}
impl std::convert::From<SchemaTypecheckError> for SchemaBuildError {
    fn from(err: SchemaTypecheckError) -> SchemaBuildError {
        SchemaBuildError::TypecheckError(Box::new(err))
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SchemaTypecheckError {
    // TODO
}

/// Represents the file location of a given type's definition in the schema.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedTypeFilePosition {
    pub def_location: loc::FilePosition,
    pub type_name: String,
}
impl NamedTypeFilePosition {
    pub(crate) fn from_pos(
        type_name: String,
        file: &Path,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            def_location: loc::FilePosition::from_pos(file, pos),
            type_name,
        }
    }
}

fn inputobj_fields_from_ast(
    schema_def_location: &loc::SchemaDefLocation,
    input_fields: &[ast::schema::InputValue],
) -> Result<HashMap<String, InputFieldDef>> {
    Ok(input_fields.iter().map(|input_field| {
        (input_field.name.to_string(), InputFieldDef {
            def_location: schema_def_location.clone(),
        })
    }).collect())
}

fn directive_refs_from_ast(
    file_path: &Path,
    directives: &[ast::operation::Directive],
) -> Vec<NamedDirectiveRef> {
    directives.iter().map(|d| NamedDirectiveRef::new(
        &d.name,
        loc::FilePosition::from_pos(
            file_path,
            d.position,
        ),
    )).collect()
}

fn object_fields_from_ast(
    ref_location: &loc::FilePosition,
    fields: &[ast::schema::Field],
) -> HashMap<String, ObjectFieldDef> {
    fields.iter().map(|field| {
        let field_def_position = loc::FilePosition::from_pos(
            ref_location.file.clone(),
            field.position,
        );
        (field.name.to_string(), ObjectFieldDef {
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
