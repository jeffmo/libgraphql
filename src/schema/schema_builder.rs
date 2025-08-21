use crate::ast;
use crate::file_reader;
use crate::loc;
use crate::operation::OperationKind;
use crate::schema::Schema;
use crate::schema::TypeValidationError;
use crate::types::Directive;
use crate::types::EnumTypeBuilder;
use crate::types::GraphQLType;
use crate::types::InterfaceTypeBuilder;
use crate::types::InputObjectTypeBuilder;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectTypeBuilder;
use crate::types::Parameter;
use crate::types::ScalarTypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::UnionTypeBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use thiserror::Error;

type Result<T> = std::result::Result<T, SchemaBuildError>;

fn builtin_directive_names() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        HashSet::from([
            "skip",
            "include",
            "deprecated",
            "specifiedBy",
        ])
    })
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
    directive_defs: HashMap<String, Directive>,
    enum_builder: EnumTypeBuilder,
    inputobject_builder: InputObjectTypeBuilder,
    interface_builder: InterfaceTypeBuilder,
    query_type: Option<NamedTypeDefLocation>,
    mutation_type: Option<NamedTypeDefLocation>,
    object_builder: ObjectTypeBuilder,
    scalar_builder: ScalarTypeBuilder,
    str_load_counter: u16,
    subscription_type: Option<NamedTypeDefLocation>,
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

        // Fun side-quest: Check types eagerly while visiting them. When there's a possibility that
        // a type error could be resolved (or manifested) later, track a
        //self.check_types()?;
        let types = self.types_map_builder.into_types_map()?;

        let query_typedefloc =
            if let Some(def) = self.query_type.take() {
                def
            } else {
                match types.get("Query") {
                    Some(GraphQLType::Object(obj_type)) => NamedTypeDefLocation {
                        def_location: obj_type.def_location().clone(),
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
                    Some(GraphQLType::Object(obj_type)) => Some(NamedTypeDefLocation {
                        def_location: obj_type.def_location().clone(),
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
                    Some(GraphQLType::Object(obj_type)) => Some(NamedTypeDefLocation {
                        def_location: obj_type.def_location().clone(),
                        type_name: "Subscription".to_string(),
                    }),
                    _ => None,
                }
            };

        Ok(Schema {
            directive_defs: self.directive_defs,
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
            types,
        })
    }

    pub fn new() -> Self {
        let types_map_builder = TypesMapBuilder::new();

        Self {
            directive_defs: HashMap::new(),
            enum_builder: EnumTypeBuilder::new(),
            inputobject_builder: InputObjectTypeBuilder::new(),
            interface_builder: InterfaceTypeBuilder::new(),
            query_type: None,
            mutation_type: None,
            object_builder: ObjectTypeBuilder::new(),
            scalar_builder: ScalarTypeBuilder::new(),
            str_load_counter: 0,
            subscription_type: None,
            types_map_builder,
            union_builder: UnionTypeBuilder::new(),
        }
    }

    pub fn load_file(
        self,
        file_path: impl AsRef<Path>,
    ) -> Result<Self> {
        self.load_files(vec![file_path])
    }

    pub fn load_files(
        mut self,
        file_paths: Vec<impl AsRef<Path>>,
    ) -> Result<Self> {
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let file_content = file_reader::read_content(file_path)
                .map_err(|err| SchemaBuildError::SchemaFileReadError(
                    Box::new(err),
                ))?;
            self = self.load_str(
                Some(file_path.to_path_buf()),
                file_content.as_str(),
            )?;
        }
        Ok(self)
    }

    pub fn load_str(
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
                PathBuf::from(format!("str://{ctr}"))
            };

        let ast_doc =
            graphql_parser::schema::parse_schema::<String>(content)
                .map_err(|err| SchemaBuildError::ParseError {
                    file: file_path.to_owned(),
                    err: err.to_string(),
                })?.into_static();

        for def in ast_doc.definitions {
            self.visit_ast_def(file_path.as_path(), def)?;
        }

        Ok(self)
    }

    fn inject_missing_builtin_directives(&mut self) {
        if !self.directive_defs.contains_key("skip") {
            self.directive_defs.insert("skip".to_string(), Directive::Skip);
        }

        if !self.directive_defs.contains_key("include") {
            self.directive_defs.insert("include".to_string(), Directive::Include);
        }

        if !self.directive_defs.contains_key("deprecated") {
            self.directive_defs.insert("deprecated".to_string(), Directive::Deprecated);
        }

        if !self.directive_defs.contains_key("specifiedBy") {
            self.directive_defs.insert("specifiedBy".to_string(), Directive::SpecifiedBy);
        }
    }

    fn visit_ast_def(
        &mut self,
        file_path: &Path,
        def: ast::schema::Definition,
    ) -> Result<()> {
        use ast::schema::Definition;
        match def {
            Definition::SchemaDefinition(schema_def) =>
                self.visit_ast_schemablock_def(file_path, schema_def),
            Definition::TypeDefinition(type_def) =>
                self.visit_ast_type_def(file_path, type_def),
            Definition::TypeExtension(type_ext) =>
                self.visit_ast_type_extension(file_path, type_ext),
            Definition::DirectiveDefinition(directive_def) =>
                self.visit_ast_directive_def(file_path, directive_def),
        }
    }

    fn visit_ast_directive_def(
        &mut self,
        file_path: &Path,
        def: ast::schema::DirectiveDefinition,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );

        if builtin_directive_names().contains(def.name.as_str()) {
            return Err(SchemaBuildError::RedefinitionOfBuiltinDirective {
                directive_name: def.name,
                location: file_position.into(),
            })?;
        }

        if def.name.starts_with("__") {
            return Err(SchemaBuildError::InvalidDunderPrefixedDirectiveName {
                def_location: file_position.into(),
                directive_name: def.name.to_string(),
            });
        }

        if let Some(Directive::Custom {
            def_location,
            ..
        }) = self.directive_defs.get(def.name.as_str()) {
            return Err(SchemaBuildError::DuplicateDirectiveDefinition {
                directive_name: def.name.clone(),
                location1: def_location.clone().into(),
                location2: file_position.into(),
            })?;
        }

        self.directive_defs.insert(def.name.to_string(), Directive::Custom {
            def_location: file_position,
            description: def.description.to_owned(),
            name: def.name.to_string(),
            params: def.arguments.iter().map(|input_val| (
                input_val.name.to_string(),
                Parameter::from_ast(
                    file_path,
                    input_val,
                ),
            )).collect()
        });

        Ok(())
    }

    fn visit_ast_schemablock_def(
        &mut self,
        file_path: &Path,
        schema_def: ast::schema::SchemaDefinition,
    ) -> Result<()> {
        if let Some(type_name) = &schema_def.query {
            let typedef_loc = NamedTypeDefLocation::from_pos(
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
            let typedef_loc = NamedTypeDefLocation::from_pos(
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
            let typedef_loc = NamedTypeDefLocation::from_pos(
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

        // As per spec:
        //
        // > The query, mutation, and subscription root types must all be
        // > different types if provided.
        //
        // https://spec.graphql.org/October2021/#sel-FAHTRLCAACG0B57a
        if let (Some(query_type), Some(mut_type)) = (&self.query_type, &self.mutation_type) {
            if query_type.type_name == mut_type.type_name {
                // Query and Mutation operations use the same type
                return Err(SchemaBuildError::NonUniqueOperationTypes {
                    reused_type_name: query_type.type_name.to_owned(),
                    operation1: OperationKind::Query,
                    operation1_loc: query_type.def_location.to_owned(),
                    operation2: OperationKind::Mutation,
                    operation2_loc: mut_type.def_location.to_owned(),
                });
            }
        }

        if let (Some(query_type), Some(sub_type)) = (&self.query_type, &self.subscription_type) {
            if query_type.type_name == sub_type.type_name {
                // Query and Subscription operations use the same type
                return Err(SchemaBuildError::NonUniqueOperationTypes {
                    reused_type_name: query_type.type_name.to_owned(),
                    operation1: OperationKind::Query,
                    operation1_loc: query_type.def_location.to_owned(),
                    operation2: OperationKind::Subscription,
                    operation2_loc: sub_type.def_location.to_owned(),
                });
            }
        }

        if let (Some(mut_type), Some(sub_type)) = (&self.mutation_type, &self.subscription_type) {
            if mut_type.type_name == sub_type.type_name {
                // Subscription and Mutation operations use the same type
                return Err(SchemaBuildError::NonUniqueOperationTypes {
                    reused_type_name: mut_type.type_name.to_owned(),
                    operation1: OperationKind::Mutation,
                    operation1_loc: mut_type.def_location.to_owned(),
                    operation2: OperationKind::Subscription,
                    operation2_loc: sub_type.def_location.to_owned(),
                });
            }
        }

        Ok(())
    }

    fn visit_ast_type_def(
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

    fn visit_ast_type_extension(
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
        location1: loc::SchemaDefLocation,
        location2: loc::SchemaDefLocation,
    },

    #[error("Multiple enum variants with the same name were defined on a single enum type")]
    DuplicateEnumValueDefinition {
        enum_name: String,
        enum_def_location: loc::SchemaDefLocation,
        value_def1: loc::SchemaDefLocation,
        value_def2: loc::SchemaDefLocation,
    },

    #[error("Multiple fields with the same name were defined on a single object type")]
    DuplicateFieldNameDefinition {
        type_name: String,
        field_name: String,
        field_def1: loc::SchemaDefLocation,
        field_def2: loc::SchemaDefLocation,
    },

    #[error(
        "The `{type_name}` type declares that it implements the \
        `{duplicated_interface_name}` interface more than once"
    )]
    DuplicateInterfaceImplementsDeclaration {
        def_location: loc::SchemaDefLocation,
        duplicated_interface_name: String,
        type_name: String,
    },

    #[error("Multiple definitions of the same operation were defined")]
    DuplicateOperationDefinition {
        operation: GraphQLOperationType,
        location1: NamedTypeDefLocation,
        location2: NamedTypeDefLocation,
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
        member1: loc::SchemaDefLocation,
        member2: loc::SchemaDefLocation,
    },

    #[error("Enum types must define one or more unique variants")]
    EnumWithNoVariants {
        type_name: String,
        location: loc::SchemaDefLocation,
    },

    #[error("Attempted to extend a type that is not defined elsewhere")]
    ExtensionOfUndefinedType {
        type_name: String,
        extension_type_loc: loc::SchemaDefLocation,
    },

    #[error("Attempted to extend a type using a name that corresponds to a different kind of type")]
    InvalidExtensionType {
        schema_type: GraphQLType,
        extension_loc: loc::SchemaDefLocation,
    },

    #[error("Custom directive names must not start with `__`")]
    InvalidDunderPrefixedDirectiveName {
        def_location: loc::SchemaDefLocation,
        directive_name: String,
    },

    #[error("Field names must not start with `__`")]
    InvalidDunderPrefixedFieldName {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        type_name: String,
    },

    #[error("Parameter names must not start with `__`")]
    InvalidDunderPrefixedParamName {
        def_location: loc::SchemaDefLocation,
        field_name: String,
        param_name: String,
        type_name: String,
    },

    #[error("Type names must not start with `__`")]
    InvalidDunderPrefixedTypeName {
        def_location: loc::SchemaDefLocation,
        type_name: String,
    },

    #[error(
        "Interface types may not declare that they implement themselves: The \
        `{interface_name}` interface does just that"
    )]
    InvalidSelfImplementingInterface {
        def_location: loc::SchemaDefLocation,
        interface_name: String,
    },

    #[error("Attempted to build a schema that has no Query operation type defined")]
    NoQueryOperationTypeDefined,

    #[error(
        "The {operation1:?} and {operation2:?} root operation are defined with \
        the same GraphQL type, but this is not allowed in GraphQL. All root \
        operations must be defined with different types."
    )]
    NonUniqueOperationTypes {
        reused_type_name: String,
        operation1: OperationKind,
        operation1_loc: loc::SchemaDefLocation,
        operation2: OperationKind,
        operation2_loc: loc::SchemaDefLocation
    },

    #[error("Error parsing schema string")]
    ParseError {
        file: PathBuf,
        err: String,
    },

    #[error("Attempted to redefine a builtin directive")]
    RedefinitionOfBuiltinDirective {
        directive_name: String,
        location: loc::SchemaDefLocation,
    },

    #[error("Failure while trying to read a schema file from disk")]
    SchemaFileReadError(Box<file_reader::ReadContentError>),

    #[error(
        "Encountered the following type-validation errors while building the \
        schema:\n\n{}",
        errors.iter()
            .map(|s| format!("  * {s}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )]
    TypeValidationErrors {
        errors: Vec<TypeValidationError>,
    },
}

/// Represents the file location of a given type's definition in the schema.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedTypeDefLocation {
    pub def_location: loc::SchemaDefLocation,
    pub type_name: String,
}
impl NamedTypeDefLocation {
    pub(crate) fn from_pos(
        type_name: String,
        file: &Path,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            def_location: loc::FilePosition::from_pos(file, pos).into(),
            type_name,
        }
    }
}
