use crate::ast;
use crate::file_reader;
use crate::loc;
use crate::schema::Schema;
use crate::schema_builder::EnumTypeBuilder;
use crate::schema_builder::InputObjectTypeBuilder;
use crate::schema_builder::InterfaceTypeBuilder;
use crate::schema_builder::ObjectTypeBuilder;
use crate::schema_builder::ScalarTypeBuilder;
use crate::schema_builder::TypeBuilder;
use crate::schema_builder::TypesMapBuilder;
use crate::schema_builder::UnionTypeBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

type Result<T> = std::result::Result<T, SchemaBuildError>;

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
pub enum GraphQLOperationType {
    Query,
    Mutation,
    Subscription,
}

/// Utility for building a [Schema].
#[derive(Debug)]
pub struct SchemaBuilder {
    directives: HashMap<String, Directive>,
    enum_builder: EnumTypeBuilder,
    inputobject_builder: InputObjectTypeBuilder,
    interface_builder: InterfaceTypeBuilder,
    query_type: Option<NamedTypeFilePosition>,
    mutation_type: Option<NamedTypeFilePosition>,
    object_builder: ObjectTypeBuilder,
    scalar_builder: ScalarTypeBuilder,
    str_load_counter: u16,
    subscription_type: Option<NamedTypeFilePosition>,
    types: HashMap<String, GraphQLType>,
    types_map_builder: TypesMapBuilder,
    union_builder: UnionTypeBuilder,
}
impl SchemaBuilder {
    pub fn build(mut self) -> Result<Schema> {
        self.inject_missing_builtin_directives();

        self.enum_builder.finalize(&mut self.types_map_builder)?;
        self.inputobject_builder.finalize(&mut self.types_map_builder)?;
        self.interface_builder.finalize(&mut self.types_map_builder)?;
        self.object_builder.finalize(&mut self.types_map_builder)?;
        self.scalar_builder.finalize(&mut self.types_map_builder)?;
        self.union_builder.finalize(&mut self.types_map_builder)?;
        // TODO(!!!): Implement the remaining TypeBuilders and finalize() them all here...

        // Fun side-quest: Check types eagerly while visiting them. When there's a possibility that
        // a type error could be resolved (or manifested) later, track a
        //self.check_types()?;
        let mut types = self.types_map_builder.into_types_map()?;
        types.extend(self.types);

        let query_typedefloc =
            if let Some(def) = self.query_type.take() {
                def
            } else {
                match types.get("Query") {
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
                match types.get("Mutation") {
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
                match types.get("Subscription") {
                    Some(GraphQLType::Object(obj_type)) => Some(NamedTypeFilePosition {
                        def_location: obj_type.def_location.clone(),
                        type_name: "Subscription".to_string(),
                    }),
                    _ => None,
                }
            };


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
            //types: self.types,
            types,
        })
    }

    pub fn new() -> Self {
        let types = HashMap::from([
            ("Boolean".to_string(), GraphQLType::Bool),
            ("Float".to_string(), GraphQLType::Float),
            ("ID".to_string(), GraphQLType::ID),
            ("Int".to_string(), GraphQLType::Int),
            ("String".to_string(), GraphQLType::String),
        ]);
        let types_map_builder = TypesMapBuilder::new();

        Self {
            directives: HashMap::new(),
            enum_builder: EnumTypeBuilder::new(),
            inputobject_builder: InputObjectTypeBuilder::new(),
            interface_builder: InterfaceTypeBuilder::new(),
            query_type: None,
            mutation_type: None,
            object_builder: ObjectTypeBuilder::new(),
            scalar_builder: ScalarTypeBuilder::new(),
            str_load_counter: 0,
            subscription_type: None,
            types,
            types_map_builder,
            union_builder: UnionTypeBuilder::new(),
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
                    operation: GraphQLOperationType::Query,
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
                    operation: GraphQLOperationType::Mutation,
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
                    operation: GraphQLOperationType::Subscription,
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
                self.enum_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    enum_def,
                ),

            ast::schema::TypeDefinition::InputObject(inputobj_def) =>
                self.inputobject_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    inputobj_def,
                ),

            ast::schema::TypeDefinition::Interface(iface_def) =>
                self.interface_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    iface_def,
                ),

            ast::schema::TypeDefinition::Scalar(scalar_def) =>
                self.scalar_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    scalar_def,
                ),

            ast::schema::TypeDefinition::Object(obj_def) =>
                self.object_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    obj_def,
                ),

            ast::schema::TypeDefinition::Union(union_def) =>
                self.union_builder.visit_type_def(
                    &mut self.types_map_builder,
                    file_path,
                    union_def,
                ),
        }
    }

    fn visit_type_extension(
        &mut self,
        file_path: &Path,
        ext: ast::schema::TypeExtension,
    ) -> Result<()> {
        use ast::schema::TypeExtension;
        match ext {
            TypeExtension::Enum(enum_ext) =>
                self.enum_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    enum_ext,
                ),

            TypeExtension::InputObject(inputobj_ext) =>
                self.inputobject_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    inputobj_ext,
                ),

            TypeExtension::Interface(iface_ext) =>
                self.interface_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    iface_ext,
                ),

            TypeExtension::Object(obj_ext) =>
                self.object_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    obj_ext,
                ),

            TypeExtension::Scalar(scalar_ext) =>
                self.scalar_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    scalar_ext,
                ),

            TypeExtension::Union(union_ext) =>
                self.union_builder.visit_type_extension(
                    &mut self.types_map_builder,
                    file_path,
                    union_ext,
                ),
        }
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
        operation: GraphQLOperationType,
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

    #[error("Enum types must define one or more unique variants")]
    EnumWithNoVariants {
        type_name: String,
        location: loc::FilePosition,
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
