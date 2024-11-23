use crate::ast;
use crate::directives::Directive;
use crate::directives::DirectiveReference;
use crate::types::EnumValue;
use crate::types::FieldType;
use crate::types::InputFieldType;
use crate::types::SchemaType;
use crate::types::SchemaTypeReference;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct SchemaBuilder {
    directives: HashMap<String, Directive>,
    query_type: Option<TypeDefFileLocation>,
    mutation_type: Option<TypeDefFileLocation>,
    subscription_type: Option<TypeDefFileLocation>,
    types: HashMap<String, SchemaType>,
    type_extensions: Vec<(PathBuf, ast::schema::TypeExtension)>,
}
impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            type_extensions: vec![],
            types: HashMap::new(),
        }
    }

    pub fn load_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> Result<(), SchemaBuildError> {
        self.load_files(vec![file_path])
    }

    pub fn load_files<P: AsRef<Path>>(
        &mut self,
        file_paths: Vec<P>,
    ) -> Result<(), SchemaBuildError> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();

            if !file_path.is_file() {
                return Err(SchemaBuildError::PathIsNotAFile(
                    file_path.to_path_buf()
                ));
            }

            let bytes = std::fs::read(file_path)
                .map_err(|err| SchemaBuildError::SchemaFileReadError {
                    path: file_path.to_path_buf(),
                    err,
                })?;

            let content = String::from_utf8(bytes).map_err(
                |err| SchemaBuildError::SchemaFileDecodeError {
                    path: file_path.to_path_buf(),
                    err,
                }
            )?;

            let doc =
                graphql_parser::schema::parse_schema::<String>(content.as_str())
                    .map_err(|err| SchemaBuildError::SchemaParseError {
                        file: file_path.to_path_buf(),
                        err,
                    })?.into_static();

            for def in doc.definitions {
                self.visit_definition(file_path.to_path_buf(), def)?;
            }
        }

        // TODO: Typecheck the schema after we've fully parsed it.
        //
        //       Fun approach: Eagerly typecheck each definition as it is
        //       processed IF all of the types needed are already defined. If
        //       types are missing which are needed for typechecking, store a
        //       constraint in some kind of ProcessingContext struct to come
        //       back to later (either when the needed types get defined OR at
        //       the very end of processing).

        Ok(())
    }

    fn check_for_conflicting_type(
        &self,
        file_location: &ast::FileLocation,
        name: &str,
    ) -> Result<(), SchemaBuildError> {
        if let Some(conflicting_type) = self.types.get(name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: name.to_string(),
                location1: conflicting_type.get_def_location().clone(),
                location2: file_location.clone(),
            });
        }
        Ok(())
    }

    fn merge_enum_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::EnumTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::Enum {
                def_location,
                directives,
                values,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_val in ext.values.iter() {
                    let ext_val_loc = ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        ext_val.position,
                    );

                    // Error if this value is already defined.
                    if let Some(existing_value) = values.get(ext_val.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                            enum_name: ext.name.to_string(),
                            enum_def_location: def_location.clone(),
                            value_def1: existing_value.def_location.clone(),
                            value_def2: ext_val_loc,
                        });
                    }
                    values.insert(ext_val.name.to_string(), EnumValue {
                        def_ast: ext_val.clone(),
                        def_location: ext_val_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn merge_inputobj_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::InputObjectTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::InputObject {
                def_location,
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_loc = ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        ext_field.position,
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            field_name: ext.name.to_string(),
                            field_def_location: def_location.clone(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        });
                    }
                    fields.insert(ext_field.name.to_string(), InputFieldType {
                        def_ast: ext_field.clone(),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn merge_interface_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::InterfaceTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::Interface {
                def_location,
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_loc = ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        ext_field.position,
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            field_name: ext.name.to_string(),
                            field_def_location: def_location.clone(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        });
                    }
                    fields.insert(ext_field.name.to_string(), FieldType {
                        def_ast: ext_field.clone(),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn merge_object_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::ObjectTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::Object {
                def_location,
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_loc = ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        ext_field.position,
                    );

                    // Error if this field is already defined.
                    if let Some(existing_field) = fields.get(ext_field.name.as_str()) {
                        return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                            field_name: ext.name.to_string(),
                            field_def_location: def_location.clone(),
                            field_def1: existing_field.def_location.clone(),
                            field_def2: ext_field_loc,
                        });
                    }
                    fields.insert(ext_field.name.to_string(), FieldType {
                        def_ast: ext_field.clone(),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn merge_scalar_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::ScalarTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::Scalar {
                directives,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));
                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn merge_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::TypeExtension,
    ) -> Result<(), SchemaBuildError> {
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

    fn merge_union_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::UnionTypeExtension,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(SchemaType::Union {
                def_location,
                directives,
                types,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_type in ext.types.iter() {
                    let ext_type_loc = ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        ext.position,
                    );

                    // Error if this type is already specified as a member of this union.
                    if let Some(existing_value) = types.get(ext_type) {
                        return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                            enum_name: ext.name.to_string(),
                            enum_def_location: def_location.clone(),
                            value_def1: existing_value.location.clone(),
                            value_def2: ext_type_loc,
                        });
                    }
                    types.insert(ext_type.to_string(), SchemaTypeReference {
                        type_name: ext_type.to_string(),
                        location: ext_type_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => return Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_location,
            }),

            None => return Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_location,
            }),
        }
    }

    fn visit_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::Definition,
    ) -> Result<(), SchemaBuildError> {
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
        file_path: PathBuf,
        def: ast::schema::DirectiveDefinition,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(file_path, def.position);
        let builtin_names = &crate::directives::BUILTIN_DIRECTIVE_NAMES;
        if builtin_names.contains(def.name.as_str()) {
            return Err(SchemaBuildError::RedefinitionOfBuiltinDirective {
                directive_name: def.name,
                location: file_location,
            });
        }

        if let Some(Directive::Custom {
            def_location,
            ..
        }) = self.directives.get(def.name.as_str()) {
            return Err(SchemaBuildError::DuplicateDirectiveDefinition {
                directive_name: def.name.clone(),
                location1: def_location.clone(),
                location2: file_location,
            });
        }

        self.directives.insert(def.name.to_string(), Directive::Custom {
            def_ast: def,
            def_location: file_location,
        });
        Ok(())
    }

    fn visit_enum_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::EnumType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(file_path.clone(), def.position);
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        let values =
            def.values
                .iter()
                .map(|val| (val.name.to_string(), EnumValue {
                    def_location: ast::FileLocation::from_pos(
                        file_path.to_path_buf(),
                        val.position,
                    ),
                    def_ast: val.clone(),
                }))
                .collect();

        self.types.insert(def.name.to_string(), SchemaType::Enum {
            def_ast: def,
            def_location: file_location,
            directives,
            values,
        });

        Ok(())
    }

    fn visit_inputobj_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::InputObjectType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let fields = build_inputfields_map(
            &file_location,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), SchemaType::InputObject {
            def_ast: def,
            def_location: file_location.clone(),
            directives,
            fields,
        });

        Ok(())
    }

    fn visit_interface_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::InterfaceType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let fields = build_fields_map(
            &file_location,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), SchemaType::Interface {
            def_ast: def,
            def_location: file_location.clone(),
            directives,
            fields,
        });
        Ok(())
    }

    fn visit_object_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::ObjectType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let fields = build_fields_map(
            &file_location,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), SchemaType::Object {
            def_ast: def,
            def_location: file_location.clone(),
            directives,
            fields,
        });
        Ok(())
    }

    fn visit_scalar_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::ScalarType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.clone(),
            def.position,
        );
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), SchemaType::Scalar {
            def_ast: def,
            def_location: file_location,
            directives,
        });
        Ok(())
    }

    fn visit_schemablock_definition(
        &mut self,
        file_path: PathBuf,
        schema_def: ast::schema::SchemaDefinition,
    ) -> Result<(), SchemaBuildError> {
        if let Some(type_name) = &schema_def.query {
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
                schema_def.position.clone(),
            );
            if let Some(existing_typedef_loc) = &self.query_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Query,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                });
            }
            self.query_type = Some(typedef_loc);
        }

        if let Some(type_name) = &schema_def.mutation {
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
                schema_def.position.clone(),
            );
            if let Some(existing_typedef_loc) = &self.mutation_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Mutation,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                });
            }
            self.mutation_type = Some(typedef_loc);
        }

        if let Some(type_name) = &schema_def.subscription {
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
                schema_def.position.clone(),
            );
            if let Some(existing_typedef_loc) = &self.subscription_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Subscription,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                });
            }
            self.mutation_type = Some(typedef_loc);
        }

        Ok(())
    }

    fn visit_type_definition(
        &mut self,
        file_path: PathBuf,
        type_def: ast::schema::TypeDefinition,
    ) -> Result<(), SchemaBuildError> {
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
        file_path: PathBuf,
        ext: ast::schema::TypeExtension,
    ) -> Result<(), SchemaBuildError> {
        Ok(self.type_extensions.push((file_path, ext)))
    }

    fn visit_union_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::UnionType,
    ) -> Result<(), SchemaBuildError> {
        let file_location = ast::FileLocation::from_pos(
            file_path.clone(),
            def.position,
        );
        self.check_for_conflicting_type(&file_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        let types = schematype_refs_from_ast(
            &ast::FileLocation::from_pos(
                file_path.to_path_buf(),
                def.position,
            ),
            &def.types,
        );

        self.types.insert(def.name.to_string(), SchemaType::Union {
            def_ast: def,
            def_location: file_location,
            directives,
            types,
        });

        Ok(())
    }
}
impl std::convert::TryFrom<SchemaBuilder> for Schema {
    type Error = SchemaBuildError;

    fn try_from(builder: SchemaBuilder) -> Result<Schema, Self::Error> {
        let query_type =
            if let Some(def) = builder.query_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Query") {
                    Some(SchemaType::Object { .. }) => "Query".to_string(),
                    _ => return Err(SchemaBuildError::NoQueryTypeDefined),
                }
            };

        let mutation_type =
            if let Some(def) = builder.mutation_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Mutation") {
                    Some(SchemaType::Object { .. }) => "Mutation".to_string(),
                    _ => return Err(SchemaBuildError::NoMutationTypeDefined),
                }
            };

        let subscription_type =
            if let Some(def) = builder.subscription_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Subscription") {
                    Some(SchemaType::Object { .. }) => "Subscription".to_string(),
                    _ => return Err(SchemaBuildError::NoSubscriptionTypeDefined),
                }
            };

        let mut directives: HashMap<String, Directive> = builder.directives.into_iter().map(
            |(dir_name, dir)| (dir_name, dir.into())
        ).collect();

        if !directives.contains_key("skip") {
            directives.insert("skip".to_string(), Directive::Skip);
        }

        Ok(Schema {
            directives,
            query_type,
            mutation_type,
            subscription_type,
            types: builder.types,
        })
    }
}

#[derive(Debug)]
pub struct Schema {
    directives: HashMap<String, Directive>,
    query_type: String,
    mutation_type: String,
    subscription_type: String,
    types: HashMap<String, SchemaType>,
}
impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }
}

fn read_schema_file(path: &Path) -> Result<String, SchemaBuildError> {
    let bytes = std::fs::read(path)
        .map_err(|err| SchemaBuildError::SchemaFileReadError {
            path: path.to_path_buf(),
            err,
        })?;

    Ok(String::from_utf8(bytes)
        .map_err(|err| SchemaBuildError::SchemaFileDecodeError {
            path: path.to_path_buf(),
            err,
        })?)
}

#[derive(Debug)]
pub enum SchemaBuildError {
    DuplicateDirectiveDefinition {
        directive_name: String,
        location1: ast::FileLocation,
        location2: ast::FileLocation,
    },
    DuplicateEnumValueDefinition {
        enum_name: String,
        enum_def_location: ast::FileLocation,
        value_def1: ast::FileLocation,
        value_def2: ast::FileLocation,
    },
    DuplicateFieldNameDefinition {
        field_name: String,
        field_def_location: ast::FileLocation,
        field_def1: ast::FileLocation,
        field_def2: ast::FileLocation,
    },
    DuplicateOperationDefinition {
        operation: GraphQLOperation,
        location1: TypeDefFileLocation,
        location2: TypeDefFileLocation,
    },
    DuplicateTypeDefinition {
        type_name: String,
        location1: ast::FileLocation,
        location2: ast::FileLocation,
    },
    ExtensionOfUndefinedType {
        type_name: String,
        extension_type_loc: ast::FileLocation,
    },
    InvalidExtensionType {
        type_name: String,
        schema_type: SchemaType,
        extension_type_loc: ast::FileLocation,
    },
    NoQueryTypeDefined,
    NoMutationTypeDefined,
    NoSubscriptionTypeDefined,
    PathIsNotAFile(PathBuf),
    RedefinitionOfBuiltinDirective {
        directive_name: String,
        location: ast::FileLocation,
    },
    SchemaFileDecodeError {
        path: PathBuf,
        err: std::string::FromUtf8Error,
    },
    SchemaFileReadError {
        path: PathBuf,
        err: std::io::Error,
    },
    SchemaParseError {
        file: PathBuf,
        err: ast::schema::ParseError,
    },
}

#[derive(Debug)]
pub enum GraphQLOperation {
    Query,
    Mutation,
    Subscription,
}

/// Represents the location of a given type's definition in the schema.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TypeDefFileLocation {
    pub location: ast::FileLocation,
    pub type_name: String,
}
impl TypeDefFileLocation {
    pub(crate) fn from_pos(
        type_name: String,
        file: PathBuf,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            location: ast::FileLocation::from_pos(file, pos),
            type_name,
        }
    }
}

fn build_fields_map(
    file_location: &ast::FileLocation,
    fields: &Vec<ast::schema::Field>,
) -> Result<HashMap<String, FieldType>, SchemaBuildError> {
    Ok(fields.into_iter().map(|field| {
        (field.name.to_string(), FieldType {
            def_location: ast::FileLocation::from_pos(
                file_location.file.to_path_buf(),
                field.position,
            ),
            def_ast: field.clone(),
        })
    }).collect())
}

fn build_inputfields_map(
    file_location: &ast::FileLocation,
    input_fields: &Vec<ast::schema::InputValue>,
) -> Result<HashMap<String, InputFieldType>, SchemaBuildError> {
    Ok(input_fields.into_iter().map(|input_field| {
        (input_field.name.to_string(), InputFieldType {
            def_location: ast::FileLocation::from_pos(
                file_location.file.to_path_buf(),
                input_field.position,
            ),
            def_ast: input_field.clone(),
        })
    }).collect())
}

fn directive_refs_from_ast(
    file_path: &PathBuf,
    directives: &Vec<ast::query::Directive>,
) -> Vec<DirectiveReference> {
    directives.iter().map(|d| DirectiveReference {
        directive_name: d.name.to_string(),
        location: ast::FileLocation::from_pos(
            file_path.to_path_buf(),
            d.position,
        ),
    }).collect()
}

fn schematype_refs_from_ast(
    location: &ast::FileLocation,
    types: &Vec<String>,
) -> HashMap<String, SchemaTypeReference> {
    types.iter().map(|t| (t.to_string(), SchemaTypeReference {
        type_name: t.to_string(),
        location: location.clone(),
    })).collect()
}
