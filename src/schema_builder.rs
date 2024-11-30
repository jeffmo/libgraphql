use crate::ast;
use crate::loc;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use crate::schema::TypeDefFileLocation;
use crate::types::Directive;
use crate::types::EnumValue;
use crate::types::ObjectFieldDef;
use crate::types::InputFieldDef;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

type BuildResult<T> = std::result::Result<T, Box<SchemaBuildError>>;
type TypecheckResult<T> = std::result::Result<T, Box<SchemaTypecheckError>>;

lazy_static::lazy_static! {
    pub static ref BUILTIN_DIRECTIVE_NAMES: HashSet<&'static str> = {
        HashSet::from([
            "skip",
            "include",
            "deprecated",
            "specifiedBy",
        ])
    };
}

#[derive(Debug)]
pub enum GraphQLOperation {
    Query,
    Mutation,
    Subscription,
}

/// Utility for building a [Schema].
#[derive(Debug)]
pub struct SchemaBuilder {
    directives: HashMap<String, Directive>,
    query_type: Option<TypeDefFileLocation>,
    mutation_type: Option<TypeDefFileLocation>,
    subscription_type: Option<TypeDefFileLocation>,
    types: HashMap<String, GraphQLType>,
    type_extensions: Vec<(PathBuf, ast::schema::TypeExtension)>,
}
impl SchemaBuilder {
    pub fn check_types(&self) -> TypecheckResult<()> {
        // TODO: Typecheck the schema after we've fully parsed it.
        //
        //       Fun side-quest: Eagerly typecheck each definition as it is
        //       processed. If types are missing which are needed for
        //       typechecking, store a constraint in some kind of
        //       ProcessingContext struct to come back to later (either when the
        //       needed types get defined OR at the very end of processing).

        Ok(())
    }

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
    ) -> BuildResult<()> {
        self.load_files(vec![file_path])
    }

    pub fn load_files<P: AsRef<Path>>(
        &mut self,
        file_paths: Vec<P>,
    ) -> BuildResult<()> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();

            if !file_path.is_file() {
                return Err(SchemaBuildError::PathIsNotAFile(
                    file_path.to_path_buf()
                ))?;
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
        Ok(())
    }

    fn check_for_conflicting_type(
        &self,
        file_location: &loc::SchemaDefLocation,
        name: &str,
    ) -> BuildResult<()> {
        if let Some(conflicting_type) = self.types.get(name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: name.to_string(),
                def1: conflicting_type.get_def_location().clone(),
                def2: file_location.clone(),
            })?;
        }
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
        file_path: PathBuf,
        ext: ast::schema::EnumTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Enum {
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
                    let ext_val_loc = loc::FilePosition::from_pos(
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
                        }.into());
                    }
                    values.insert(ext_val.name.to_string(), EnumValue {
                        def_location: ext_val_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_inputobj_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::InputObjectTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::InputObject {
                directives: target_directives,
                fields: target_fields,
                ..
            }) => {
                target_directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_loc = loc::SchemaDefLocation::SchemaFile(
                        loc::FilePosition::from_pos(
                            file_path.to_path_buf(),
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
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_interface_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::InterfaceTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Interface {
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_pos = loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        ext_field.position,
                    );
                    let ext_field_loc = loc::SchemaDefLocation::SchemaFile(
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
                        type_ref: schematyperef_from_ast(
                            &ext_field_pos,
                            &ext_field.field_type,
                        ),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_object_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::ObjectTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Object {
                directives,
                fields,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_field in ext.fields.iter() {
                    let ext_field_pos =loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        ext_field.position,
                    );
                    let ext_field_loc = loc::SchemaDefLocation::SchemaFile(
                        ext_field_pos.clone()
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
                        type_ref: schematyperef_from_ast(
                            &ext_field_pos,
                            &ext_field.field_type,
                        ),
                        def_location: ext_field_loc,
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_scalar_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::ScalarTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Scalar {
                directives,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));
                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn merge_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::TypeExtension,
    ) -> BuildResult<()> {
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

    fn merge_type_extensions(&mut self) -> BuildResult<()> {
        while let Some((file_path, type_ext)) = self.type_extensions.pop() {
            self.merge_type_extension(file_path, type_ext)?;
        }
        Ok(())
    }

    fn merge_union_type_extension(
        &mut self,
        file_path: PathBuf,
        ext: ast::schema::UnionTypeExtension,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            ext.position,
        );

        match self.types.get_mut(ext.name.as_str()) {
            Some(GraphQLType::Union {
                directives,
                types: unioned_types,
                ..
            }) => {
                directives.append(&mut directive_refs_from_ast(
                    &file_path,
                    &ext.directives,
                ));

                for ext_type in ext.types.iter() {
                    let ext_type_loc = loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
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
                        type_ref: NamedRef::<GraphQLType>::new(
                            ext_type.to_string(),
                            ext_type_loc,
                        ),
                    });
                }

                Ok(())
            },

            Some(schema_type) => Err(SchemaBuildError::InvalidExtensionType {
                type_name: ext.name.to_string(),
                schema_type: schema_type.clone(),
                extension_type_loc: file_position,
            })?,

            None => Err(SchemaBuildError::ExtensionOfUndefinedType {
                type_name: ext.name.to_string(),
                extension_type_loc: file_position,
            })?,
        }
    }

    fn visit_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::Definition,
    ) -> BuildResult<()> {
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
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(file_path, def.position);

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
        file_path: PathBuf,
        def: ast::schema::EnumType,
    ) -> BuildResult<()> {
        let file_position =
            loc::FilePosition::from_pos(file_path.clone(), def.position);
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        let values =
            def.values
                .iter()
                .map(|val| (val.name.to_string(), EnumValue {
                    def_location: loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        val.position,
                    ),
                }))
                .collect();

        self.types.insert(def.name.to_string(), GraphQLType::Enum {
            def_location: file_position,
            directives,
            values,
        });

        Ok(())
    }

    fn visit_inputobj_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::InputObjectType,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = inputobj_fields_from_ast(
            &schema_def_location,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::InputObject {
            def_location: file_position,
            directives,
            fields,
        });

        Ok(())
    }

    fn visit_interface_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::InterfaceType,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = object_fields_from_ast(
            &file_position,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Interface {
            def_location: file_position,
            directives,
            fields,
        });
        Ok(())
    }

    fn visit_object_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::ObjectType,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let fields = object_fields_from_ast(
            &file_position,
            &def.fields,
        )?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Object {
            def_location: file_position,
            directives,
            fields,
        });
        Ok(())
    }

    fn visit_scalar_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::ScalarType,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.clone(),
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        self.types.insert(def.name.to_string(), GraphQLType::Scalar {
            def_location: schema_def_location,
            directives,
        });
        Ok(())
    }

    fn visit_schemablock_definition(
        &mut self,
        file_path: PathBuf,
        schema_def: ast::schema::SchemaDefinition,
    ) -> BuildResult<()> {
        if let Some(type_name) = &schema_def.query {
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
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
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
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
            let typedef_loc = TypeDefFileLocation::from_pos(
                type_name.to_string(),
                file_path.to_path_buf(),
                schema_def.position,
            );
            if let Some(existing_typedef_loc) = &self.subscription_type {
                return Err(SchemaBuildError::DuplicateOperationDefinition {
                    operation: GraphQLOperation::Subscription,
                    location1: existing_typedef_loc.clone(),
                    location2: typedef_loc,
                })?;
            }
            self.mutation_type = Some(typedef_loc);
        }

        Ok(())
    }

    fn visit_type_definition(
        &mut self,
        file_path: PathBuf,
        type_def: ast::schema::TypeDefinition,
    ) -> BuildResult<()> {
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
    ) -> BuildResult<()> {
        self.type_extensions.push((file_path, ext));
        Ok(())
    }

    fn visit_union_type_definition(
        &mut self,
        file_path: PathBuf,
        def: ast::schema::UnionType,
    ) -> BuildResult<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path.clone(),
            def.position,
        );
        let schema_def_location = loc::SchemaDefLocation::SchemaFile(
            file_position.clone(),
        );
        self.check_for_conflicting_type(&schema_def_location, def.name.as_str())?;

        let directives = directive_refs_from_ast(
            &file_path,
            &def.directives,
        );

        let types = def.types.iter().map(|type_name| {
            (type_name.to_string(), GraphQLTypeRef::Named {
                nullable: false,
                type_ref: NamedRef::<GraphQLType>::new(
                    type_name.to_string(),
                    file_position.clone(),
                ),
            })
        }).collect();

        self.types.insert(def.name.to_string(), GraphQLType::Union {
            def_location: file_position,
            directives,
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
impl std::convert::TryFrom<SchemaBuilder> for Schema {
    type Error = Box<SchemaBuildError>;

    fn try_from(mut builder: SchemaBuilder) -> BuildResult<Schema> {
        let query_type =
            if let Some(ref def) = builder.query_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Query") {
                    Some(GraphQLType::Object { .. }) => "Query".to_string(),
                    _ => return Err(SchemaBuildError::NoQueryTypeDefined)?,
                }
            };

        let mutation_type =
            if let Some(ref def) = builder.mutation_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Mutation") {
                    Some(GraphQLType::Object { .. }) => "Mutation".to_string(),
                    _ => return Err(SchemaBuildError::NoMutationTypeDefined)?,
                }
            };

        let subscription_type =
            if let Some(ref def) = builder.subscription_type {
                def.type_name.to_string()
            } else {
                match builder.types.get("Subscription") {
                    Some(GraphQLType::Object { .. }) => "Subscription".to_string(),
                    _ => return Err(SchemaBuildError::NoSubscriptionTypeDefined)?,
                }
            };

        builder.inject_missing_builtin_directives();
        builder.merge_type_extensions()?;

        // Fun side-quest: Check types eagerly while visiting them. When there's a possibility that
        // a type error could be resolved (or manifested) later, track a
        builder.check_types()?;

        Ok(Schema {
            directives: builder.directives,
            query_type,
            mutation_type,
            subscription_type,
            types: builder.types,
        })
    }
}

#[derive(Debug)]
pub enum SchemaBuildError {
    DuplicateDirectiveDefinition {
        directive_name: String,
        location1: loc::FilePosition,
        location2: loc::FilePosition,
    },
    DuplicateEnumValueDefinition {
        enum_name: String,
        enum_def_location: loc::FilePosition,
        value_def1: loc::FilePosition,
        value_def2: loc::FilePosition,
    },
    DuplicateFieldNameDefinition {
        type_name: String,
        field_name: String,
        field_def1: loc::SchemaDefLocation,
        field_def2: loc::SchemaDefLocation,
    },
    DuplicateOperationDefinition {
        operation: GraphQLOperation,
        location1: TypeDefFileLocation,
        location2: TypeDefFileLocation,
    },
    DuplicateTypeDefinition {
        type_name: String,
        def1: loc::SchemaDefLocation,
        def2: loc::SchemaDefLocation,
    },
    DuplicatedUnionMember {
        type_name: String,
        member1: loc::FilePosition,
        member2: loc::FilePosition,
    },
    ExtensionOfUndefinedType {
        type_name: String,
        extension_type_loc: loc::FilePosition,
    },
    InvalidExtensionType {
        type_name: String,
        schema_type: GraphQLType,
        extension_type_loc: loc::FilePosition,
    },
    NoQueryTypeDefined,
    NoMutationTypeDefined,
    NoSubscriptionTypeDefined,
    PathIsNotAFile(PathBuf),
    RedefinitionOfBuiltinDirective {
        directive_name: String,
        location: loc::FilePosition,
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
    TypecheckError(Box<SchemaTypecheckError>),
}
impl std::convert::From<Box<SchemaTypecheckError>> for Box<SchemaBuildError> {
    fn from(err: Box<SchemaTypecheckError>) -> Box<SchemaBuildError> {
        Box::new(SchemaBuildError::TypecheckError(err))
    }
}

#[derive(Debug)]
pub enum SchemaTypecheckError {
    // TODO
}

fn inputobj_fields_from_ast(
    schema_def_location: &loc::SchemaDefLocation,
    input_fields: &[ast::schema::InputValue],
) -> BuildResult<HashMap<String, InputFieldDef>> {
    Ok(input_fields.iter().map(|input_field| {
        (input_field.name.to_string(), InputFieldDef {
            def_location: schema_def_location.clone(),
        })
    }).collect())
}

fn directive_refs_from_ast(
    file_path: &Path,
    directives: &[ast::query::Directive],
) -> Vec<NamedRef<Directive>> {
    directives.iter().map(|d| NamedRef::<Directive>::new(
        d.name.to_string(),
        loc::FilePosition::from_pos(
            file_path.to_path_buf(),
            d.position,
        ),
    )).collect()
}

fn object_fields_from_ast(
   ref_location: &loc::FilePosition,
    fields: &[ast::schema::Field],
) -> BuildResult<HashMap<String, ObjectFieldDef>> {
    Ok(fields.iter().map(|field| (field.name.to_string(), ObjectFieldDef {
        def_location: loc::SchemaDefLocation::SchemaFile(ref_location.clone()),
        type_ref: schematyperef_from_ast(
            ref_location,
            &field.field_type,
        ),
    })).collect())
}

fn schematyperef_from_ast(
    ref_location: &loc::FilePosition,
    ast: &ast::query::Type,
) -> GraphQLTypeRef {
    schematyperef_from_ast_impl(ref_location, ast, true)
}

fn schematyperef_from_ast_impl(
    ref_location: &loc::FilePosition,
    ast: &ast::query::Type,
    nullable: bool,
) -> GraphQLTypeRef {
    match ast {
        ast::query::Type::ListType(inner) =>
            GraphQLTypeRef::List {
                inner_type_ref: Box::new(
                    schematyperef_from_ast_impl(ref_location, inner, true)
                ),
                nullable,
                ref_location: ref_location.clone(),
            },

        ast::query::Type::NamedType(name) =>
            GraphQLTypeRef::Named {
                nullable,
                type_ref: NamedRef::<GraphQLType>::new(
                    name.to_string(),
                    ref_location.clone(),
                ),
            },

        ast::query::Type::NonNullType(inner) =>
            schematyperef_from_ast_impl(ref_location, inner, false),
    }
}
