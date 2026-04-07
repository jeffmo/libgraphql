use crate::error_note::ErrorNote;
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::schema_def::Schema;
use crate::schema::schema_build_error::SchemaBuildError;
use crate::schema::schema_build_error::SchemaBuildErrorKind;
use crate::schema::schema_errors::SchemaErrors;
use crate::schema_source_map::SchemaSourceMap;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::EnumValueDefBuilder;
use crate::type_builders::FieldDefBuilder;
use crate::type_builders::InputFieldDefBuilder;
use crate::type_builders::InputObjectTypeBuilder;
use crate::type_builders::InterfaceTypeBuilder;
use crate::type_builders::ObjectTypeBuilder;
use crate::type_builders::ParameterDefBuilder;
use crate::type_builders::ScalarTypeBuilder;
use crate::type_builders::UnionTypeBuilder;
use crate::types::DirectiveDefinition;
use crate::types::DirectiveDefinitionKind;
use crate::types::DirectiveLocationKind;
use crate::types::EnumType;
use crate::types::EnumValue;
use crate::types::FieldDefinition;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ParameterDefinition;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::value::Value;
use indexmap::IndexMap;
use libgraphql_parser::ast;

/// Accumulates GraphQL type definitions, directive definitions,
/// and schema metadata, then validates and produces an immutable
/// [`Schema`].
///
/// Supports both programmatic construction via
/// [`absorb_type()`](Self::absorb_type) /
/// [`absorb_directive()`](Self::absorb_directive) and loading
/// from parsed schema strings via
/// [`load_str()`](Self::load_str).
///
/// See [Schema](https://spec.graphql.org/September2025/#sec-Schema).
pub struct SchemaBuilder {
    directive_defs: IndexMap<DirectiveName, DirectiveDefinition>,
    errors: Vec<SchemaBuildError>,
    mutation_type_name: Option<(TypeName, Span)>,
    query_type_name: Option<(TypeName, Span)>,
    source_maps: Vec<SchemaSourceMap>,
    subscription_type_name: Option<(TypeName, Span)>,
    types: IndexMap<TypeName, GraphQLType>,
}

/// Wraps the six type-builder kinds so that
/// [`SchemaBuilder::absorb_type()`] can accept any of them
/// via `impl Into<TypeBuilderKind>`.
pub(crate) enum TypeBuilderKind {
    Enum(EnumTypeBuilder),
    InputObject(InputObjectTypeBuilder),
    Interface(InterfaceTypeBuilder),
    Object(ObjectTypeBuilder),
    Scalar(ScalarTypeBuilder),
    Union(UnionTypeBuilder),
}

impl TypeBuilderKind {
    pub(crate) fn name(&self) -> &TypeName {
        match self {
            Self::Enum(b) => &b.name,
            Self::InputObject(b) => &b.name,
            Self::Interface(b) => &b.name,
            Self::Object(b) => &b.name,
            Self::Scalar(b) => &b.name,
            Self::Union(b) => &b.name,
        }
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Enum(b) => b.span,
            Self::InputObject(b) => b.span,
            Self::Interface(b) => b.span,
            Self::Object(b) => b.span,
            Self::Scalar(b) => b.span,
            Self::Union(b) => b.span,
        }
    }

    /// Converts the builder into a finalized [`GraphQLType`],
    /// draining any builder-accumulated errors into `errors`.
    pub(crate) fn into_graphql_type(
        self,
        errors: &mut Vec<SchemaBuildError>,
    ) -> GraphQLType {
        match self {
            Self::Enum(b) => {
                errors.extend(b.errors);
                let type_name = b.name.clone();
                GraphQLType::Enum(Box::new(EnumType {
                    description: b.description,
                    directives: b.directives,
                    name: b.name,
                    span: b.span,
                    values: b.values.into_iter().map(|v| {
                        let ev = enum_value_from_builder(
                            v, &type_name,
                        );
                        (ev.name.clone(), ev)
                    }).collect(),
                }))
            },
            Self::InputObject(b) => {
                errors.extend(b.errors);
                let type_name = b.name.clone();
                GraphQLType::InputObject(Box::new(InputObjectType {
                    description: b.description,
                    directives: b.directives,
                    fields: b.fields.into_iter().map(|f| {
                        let field = input_field_from_builder(
                            f, &type_name,
                        );
                        (field.name.clone(), field)
                    }).collect(),
                    name: b.name,
                    span: b.span,
                }))
            },
            Self::Interface(b) => {
                errors.extend(b.errors);
                let type_name = b.name.clone();
                GraphQLType::Interface(Box::new(InterfaceType(
                    FieldedTypeData {
                        description: b.description,
                        directives: b.directives,
                        fields: b.fields.into_iter().map(|f| {
                            let field = field_def_from_builder(
                                f, &type_name, errors,
                            );
                            (field.name.clone(), field)
                        }).collect(),
                        interfaces: b.implements,
                        name: b.name,
                        span: b.span,
                    },
                )))
            },
            Self::Object(b) => {
                errors.extend(b.errors);
                let type_name = b.name.clone();
                GraphQLType::Object(Box::new(ObjectType(
                    FieldedTypeData {
                        description: b.description,
                        directives: b.directives,
                        fields: b.fields.into_iter().map(|f| {
                            let field = field_def_from_builder(
                                f, &type_name, errors,
                            );
                            (field.name.clone(), field)
                        }).collect(),
                        interfaces: b.implements,
                        name: b.name,
                        span: b.span,
                    },
                )))
            },
            Self::Scalar(b) => {
                errors.extend(b.errors);
                GraphQLType::Scalar(Box::new(ScalarType {
                    description: b.description,
                    directives: b.directives,
                    kind: ScalarKind::Custom,
                    name: b.name,
                    span: b.span,
                }))
            },
            Self::Union(b) => {
                errors.extend(b.errors);
                GraphQLType::Union(Box::new(UnionType {
                    description: b.description,
                    directives: b.directives,
                    members: b.members,
                    name: b.name,
                    span: b.span,
                }))
            },
        }
    }
}

impl From<EnumTypeBuilder> for TypeBuilderKind {
    fn from(b: EnumTypeBuilder) -> Self { Self::Enum(b) }
}
impl From<InputObjectTypeBuilder> for TypeBuilderKind {
    fn from(b: InputObjectTypeBuilder) -> Self { Self::InputObject(b) }
}
impl From<InterfaceTypeBuilder> for TypeBuilderKind {
    fn from(b: InterfaceTypeBuilder) -> Self { Self::Interface(b) }
}
impl From<ObjectTypeBuilder> for TypeBuilderKind {
    fn from(b: ObjectTypeBuilder) -> Self { Self::Object(b) }
}
impl From<ScalarTypeBuilder> for TypeBuilderKind {
    fn from(b: ScalarTypeBuilder) -> Self { Self::Scalar(b) }
}
impl From<UnionTypeBuilder> for TypeBuilderKind {
    fn from(b: UnionTypeBuilder) -> Self { Self::Union(b) }
}

// ---------------------------------------------------------
// Builder-to-finalized-type conversion helpers
// ---------------------------------------------------------

fn field_def_from_builder(
    b: FieldDefBuilder,
    parent_type_name: &TypeName,
    errors: &mut Vec<SchemaBuildError>,
) -> FieldDefinition {
    errors.extend(b.errors);
    FieldDefinition {
        description: b.description,
        directives: b.directives,
        name: b.name,
        parameters: b.parameters.into_iter().map(|p| {
            let param = param_def_from_builder(p);
            (param.name.clone(), param)
        }).collect(),
        parent_type_name: parent_type_name.clone(),
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

fn param_def_from_builder(b: ParameterDefBuilder) -> ParameterDefinition {
    ParameterDefinition {
        default_value: b.default_value,
        description: b.description,
        directives: b.directives,
        name: b.name,
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

fn input_field_from_builder(
    b: InputFieldDefBuilder,
    parent_type_name: &TypeName,
) -> InputField {
    InputField {
        default_value: b.default_value,
        description: b.description,
        directives: b.directives,
        name: b.name,
        parent_type_name: parent_type_name.clone(),
        span: b.span,
        type_annotation: b.type_annotation,
    }
}

fn enum_value_from_builder(
    b: EnumValueDefBuilder,
    parent_type_name: &TypeName,
) -> EnumValue {
    EnumValue {
        description: b.description,
        directives: b.directives,
        name: b.name,
        parent_type_name: parent_type_name.clone(),
        span: b.span,
    }
}

// ---------------------------------------------------------
// SchemaBuilder implementation
// ---------------------------------------------------------

impl Default for SchemaBuilder {
    fn default() -> Self { Self::new() }
}

// TODO: SchemaBuildError is large due to SchemaBuildErrorKind
// variants + Vec<ErrorNote>. Consider boxing the error or
// using an error index to reduce Result size.
#[allow(clippy::result_large_err)]
impl SchemaBuilder {
    /// Creates a new `SchemaBuilder` pre-seeded with the five
    /// built-in scalar types and five built-in directives.
    pub fn new() -> Self {
        let mut builder = Self {
            directive_defs: IndexMap::new(),
            errors: vec![],
            mutation_type_name: None,
            query_type_name: None,
            source_maps: vec![SchemaSourceMap::builtin()],
            subscription_type_name: None,
            types: IndexMap::new(),
        };
        builder.seed_builtin_scalars();
        builder.seed_builtin_directives();
        builder
    }

    /// Seeds the five built-in scalar types: `Boolean`, `Float`,
    /// `ID`, `Int`, `String`.
    ///
    /// See [Built-in Scalars](https://spec.graphql.org/September2025/#sec-Scalars.Built-in-Scalars).
    fn seed_builtin_scalars(&mut self) {
        for (kind, name) in [
            (ScalarKind::Boolean, "Boolean"),
            (ScalarKind::Float, "Float"),
            (ScalarKind::ID, "ID"),
            (ScalarKind::Int, "Int"),
            (ScalarKind::String, "String"),
        ] {
            self.types.insert(
                TypeName::new(name),
                GraphQLType::Scalar(Box::new(ScalarType {
                    description: None,
                    directives: vec![],
                    kind,
                    name: TypeName::new(name),
                    span: Span::builtin(),
                })),
            );
        }
    }

    /// Seeds the five built-in directives: `@skip`, `@include`,
    /// `@deprecated`, `@specifiedBy`, `@oneOf`.
    ///
    /// See [Built-in Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives).
    fn seed_builtin_directives(&mut self) {
        // @skip(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
        self.directive_defs.insert(
            DirectiveName::new("skip"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::Skip,
                locations: vec![
                    DirectiveLocationKind::Field,
                    DirectiveLocationKind::FragmentSpread,
                    DirectiveLocationKind::InlineFragment,
                ],
                name: DirectiveName::new("skip"),
                parameters: IndexMap::from_iter([(
                    FieldName::new("if"),
                    ParameterDefinition {
                        default_value: None,
                        description: None,
                        directives: vec![],
                        name: FieldName::new("if"),
                        span: Span::builtin(),
                        type_annotation: TypeAnnotation::named(
                            "Boolean",
                            /* nullable = */ false,
                        ),
                    },
                )]),
                span: Span::builtin(),
            },
        );

        // @include(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
        self.directive_defs.insert(
            DirectiveName::new("include"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::Include,
                locations: vec![
                    DirectiveLocationKind::Field,
                    DirectiveLocationKind::FragmentSpread,
                    DirectiveLocationKind::InlineFragment,
                ],
                name: DirectiveName::new("include"),
                parameters: IndexMap::from_iter([(
                    FieldName::new("if"),
                    ParameterDefinition {
                        default_value: None,
                        description: None,
                        directives: vec![],
                        name: FieldName::new("if"),
                        span: Span::builtin(),
                        type_annotation: TypeAnnotation::named(
                            "Boolean",
                            /* nullable = */ false,
                        ),
                    },
                )]),
                span: Span::builtin(),
            },
        );

        // @deprecated(reason: String = "No longer supported")
        // on FIELD_DEFINITION | ARGUMENT_DEFINITION |
        //    INPUT_FIELD_DEFINITION | ENUM_VALUE
        self.directive_defs.insert(
            DirectiveName::new("deprecated"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::Deprecated,
                locations: vec![
                    DirectiveLocationKind::ArgumentDefinition,
                    DirectiveLocationKind::EnumValue,
                    DirectiveLocationKind::FieldDefinition,
                    DirectiveLocationKind::InputFieldDefinition,
                ],
                name: DirectiveName::new("deprecated"),
                parameters: IndexMap::from_iter([(
                    FieldName::new("reason"),
                    ParameterDefinition {
                        default_value: Some(Value::String(
                            "No longer supported".to_string(),
                        )),
                        description: None,
                        directives: vec![],
                        name: FieldName::new("reason"),
                        span: Span::builtin(),
                        type_annotation: TypeAnnotation::named(
                            "String",
                            /* nullable = */ true,
                        ),
                    },
                )]),
                span: Span::builtin(),
            },
        );

        // @specifiedBy(url: String!) on SCALAR
        self.directive_defs.insert(
            DirectiveName::new("specifiedBy"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::SpecifiedBy,
                locations: vec![DirectiveLocationKind::Scalar],
                name: DirectiveName::new("specifiedBy"),
                parameters: IndexMap::from_iter([(
                    FieldName::new("url"),
                    ParameterDefinition {
                        default_value: None,
                        description: None,
                        directives: vec![],
                        name: FieldName::new("url"),
                        span: Span::builtin(),
                        type_annotation: TypeAnnotation::named(
                            "String",
                            /* nullable = */ false,
                        ),
                    },
                )]),
                span: Span::builtin(),
            },
        );

        // @oneOf on INPUT_OBJECT
        self.directive_defs.insert(
            DirectiveName::new("oneOf"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::OneOf,
                locations: vec![DirectiveLocationKind::InputObject],
                name: DirectiveName::new("oneOf"),
                parameters: IndexMap::new(),
                span: Span::builtin(),
            },
        );
    }

    /// Registers a type builder with the schema. Converts the
    /// builder to a finalized [`GraphQLType`], checks for
    /// duplicate type names, and inserts.
    ///
    /// Builder-accumulated errors are drained into the
    /// schema builder's error list.
    //
    // TypeBuilderKind is pub(crate) intentionally: callers pass
    // concrete builder types (e.g. ObjectTypeBuilder) which impl
    // Into<TypeBuilderKind>. The enum itself is an internal
    // dispatch mechanism, not a public API surface.
    #[allow(private_bounds)]
    pub fn absorb_type(
        &mut self,
        builder: impl Into<TypeBuilderKind>,
    ) -> Result<&mut Self, SchemaBuildError> {
        let kind: TypeBuilderKind = builder.into();
        let name = kind.name().clone();
        let span = kind.span();

        // Check duplicate
        if let Some(existing) = self.types.get(&name) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateTypeDefinition {
                    first_defined_span: existing.span(),
                    type_name: name.to_string(),
                },
                span,
                vec![
                    ErrorNote::general_with_span(
                        "first defined here",
                        existing.span(),
                    ),
                ],
            ));
        }

        // Convert builder to GraphQLType and insert
        let graphql_type = kind.into_graphql_type(
            &mut self.errors,
        );
        self.types.insert(name, graphql_type);
        Ok(self)
    }

    /// Registers a directive builder with the schema. Checks
    /// for duplicate directive names and rejects redefinition
    /// of built-in directives.
    pub fn absorb_directive(
        &mut self,
        builder: DirectiveBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        let name = builder.name.clone();
        let span = builder.span;

        // Reject redefinition of built-in directives
        if let Some(existing) = self.directive_defs.get(&name) {
            if existing.is_builtin() {
                // https://spec.graphql.org/September2025/#sec-Type-System.Directives
                return Err(SchemaBuildError::new(
                    SchemaBuildErrorKind::RedefinitionOfBuiltinDirective {
                        name: name.to_string(),
                    },
                    span,
                    vec![],
                ));
            }
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateDirectiveDefinition {
                    name: name.to_string(),
                },
                span,
                vec![],
            ));
        }

        self.errors.extend(builder.errors);
        let def = DirectiveDefinition {
            description: builder.description,
            is_repeatable: builder.is_repeatable,
            kind: DirectiveDefinitionKind::Custom,
            locations: builder.locations,
            name: builder.name,
            parameters: builder.parameters.into_iter().map(|p| {
                let param = param_def_from_builder(p);
                (param.name.clone(), param)
            }).collect(),
            span: builder.span,
        };
        self.directive_defs.insert(name, def);
        Ok(self)
    }

    /// Parses `source` as a GraphQL schema document and loads
    /// all definitions into this builder.
    ///
    /// Returns `&mut Self` for method chaining. Parse errors and
    /// validation errors are collected into the returned `Err`
    /// variant.
    pub fn load_str(
        &mut self,
        source: &str,
    ) -> Result<&mut Self, Vec<SchemaBuildError>> {
        let parse_result =
            libgraphql_parser::parse_schema(source);

        // Report parse-level errors
        if parse_result.has_errors() {
            let parse_errors: Vec<SchemaBuildError> =
                parse_result.errors().iter().map(|e| {
                    SchemaBuildError::new(
                        SchemaBuildErrorKind::ParseError {
                            message: e.to_string(),
                        },
                        Span::builtin(),
                        vec![],
                    )
                }).collect();
            return Err(parse_errors);
        }

        // Register source map
        let source_text = parse_result
            .source_map()
            .source()
            .unwrap_or("");
        let source_map_id = SourceMapId(
            self.source_maps.len() as u16,
        );
        self.source_maps.push(
            SchemaSourceMap::from_source(source_text, None),
        );

        let doc = parse_result.ast();
        self.load_document(doc, source_map_id);
        Ok(self)
    }

    /// Iterates over all definitions in a parsed document and
    /// absorbs type definitions, directive definitions, and
    /// schema definitions. Skips extensions, operations, and
    /// fragments (which are not schema-level definitions).
    fn load_document(
        &mut self,
        doc: &ast::Document<'_>,
        source_map_id: SourceMapId,
    ) {
        for def in &doc.definitions {
            match def {
                ast::Definition::TypeDefinition(td) => {
                    self.load_type_definition(td, source_map_id);
                },
                ast::Definition::DirectiveDefinition(dd) => {
                    let builder = DirectiveBuilder::from_ast(
                        dd, source_map_id,
                    );
                    if let Err(e) = self.absorb_directive(builder) {
                        self.errors.push(e);
                    }
                },
                ast::Definition::SchemaDefinition(sd) => {
                    self.load_schema_definition(sd, source_map_id);
                },
                // Skip extensions, operations, fragments
                ast::Definition::SchemaExtension(_)
                | ast::Definition::TypeExtension(_)
                | ast::Definition::OperationDefinition(_)
                | ast::Definition::FragmentDefinition(_) => {},
            }
        }
    }

    /// Dispatches a parsed type definition to the appropriate
    /// builder's `from_ast()` and absorbs the result.
    fn load_type_definition(
        &mut self,
        td: &ast::TypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) {
        let result = match td {
            ast::TypeDefinition::Enum(e) => {
                self.absorb_type(
                    EnumTypeBuilder::from_ast(e, source_map_id),
                )
            },
            ast::TypeDefinition::InputObject(io) => {
                self.absorb_type(
                    InputObjectTypeBuilder::from_ast(
                        io, source_map_id,
                    ),
                )
            },
            ast::TypeDefinition::Interface(i) => {
                self.absorb_type(
                    InterfaceTypeBuilder::from_ast(
                        i, source_map_id,
                    ),
                )
            },
            ast::TypeDefinition::Object(o) => {
                self.absorb_type(
                    ObjectTypeBuilder::from_ast(o, source_map_id),
                )
            },
            ast::TypeDefinition::Scalar(s) => {
                self.absorb_type(
                    ScalarTypeBuilder::from_ast(s, source_map_id),
                )
            },
            ast::TypeDefinition::Union(u) => {
                self.absorb_type(
                    UnionTypeBuilder::from_ast(u, source_map_id),
                )
            },
        };
        if let Err(e) = result {
            self.errors.push(e);
        }
    }

    /// Processes a `schema { ... }` definition, extracting root
    /// operation type bindings (query, mutation, subscription).
    fn load_schema_definition(
        &mut self,
        sd: &ast::SchemaDefinition<'_>,
        source_map_id: SourceMapId,
    ) {
        for root_op in &sd.root_operations {
            let type_name = TypeName::new(
                root_op.named_type.value.as_ref(),
            );
            let span = ast_helpers::span_from_ast(
                root_op.span, source_map_id,
            );
            let slot = match root_op.operation_kind {
                ast::OperationKind::Query => {
                    &mut self.query_type_name
                },
                ast::OperationKind::Mutation => {
                    &mut self.mutation_type_name
                },
                ast::OperationKind::Subscription => {
                    &mut self.subscription_type_name
                },
            };
            let op_str = match root_op.operation_kind {
                ast::OperationKind::Query => "query",
                ast::OperationKind::Mutation => "mutation",
                ast::OperationKind::Subscription => "subscription",
            };
            if let Some((existing_name, existing_span)) = slot {
                // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
                self.errors.push(SchemaBuildError::new(
                    SchemaBuildErrorKind::DuplicateOperationDefinition {
                        operation: op_str.to_string(),
                        type_name: existing_name.to_string(),
                    },
                    span,
                    vec![
                        ErrorNote::general_with_span(
                            "first defined here",
                            *existing_span,
                        ),
                    ],
                ));
            } else {
                *slot = Some((type_name, span));
            }
        }
    }

    /// Validates and finalizes the schema. Placeholder for
    /// Task 16.
    // TODO: SchemaErrors wraps Vec<SchemaBuildError> which is
    // large. Consider boxing once error strategy is finalized.
    #[allow(clippy::result_large_err)]
    pub fn build(self) -> Result<Schema, SchemaErrors> {
        todo!()
    }

    /// Convenience: parse a schema string and build in one step.
    // TODO: SchemaErrors wraps Vec<SchemaBuildError> which is
    // large. Consider boxing once error strategy is finalized.
    #[allow(clippy::result_large_err)]
    pub fn build_from_str(
        source: &str,
    ) -> Result<Schema, SchemaErrors> {
        let mut sb = Self::new();
        sb.load_str(source).map_err(SchemaErrors::new)?;
        sb.build()
    }

    // ---------------------------------------------------------
    // Test accessors
    // ---------------------------------------------------------

    /// Returns the registered types (for test inspection).
    pub(crate) fn types(&self) -> &IndexMap<TypeName, GraphQLType> {
        &self.types
    }

    /// Returns the registered directive definitions (for test
    /// inspection).
    pub(crate) fn directive_defs(
        &self,
    ) -> &IndexMap<DirectiveName, DirectiveDefinition> {
        &self.directive_defs
    }

    /// Returns the query root type name binding (for test
    /// inspection).
    pub(crate) fn query_type_name(&self) -> Option<&(TypeName, Span)> {
        self.query_type_name.as_ref()
    }

    /// Returns the mutation root type name binding (for test
    /// inspection).
    // TODO: Remove #[allow(dead_code)] once mutation root type
    // tests are added or build() consumes this field.
    #[allow(dead_code)]
    pub(crate) fn mutation_type_name(&self) -> Option<&(TypeName, Span)> {
        self.mutation_type_name.as_ref()
    }
}
