use crate::error_note::ErrorNote;
use crate::error_note::ErrorNoteKind;
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
use crate::type_builders::conversion_helpers::param_def_from_builder;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::InputObjectTypeBuilder;
use crate::type_builders::InterfaceTypeBuilder;
use crate::type_builders::IntoGraphQLType;
use crate::type_builders::ObjectTypeBuilder;
use crate::type_builders::ScalarTypeBuilder;
use crate::type_builders::UnionTypeBuilder;
use crate::types::DirectiveDefinition;
use crate::types::DirectiveDefinitionKind;
use crate::types::DirectiveLocationKind;
use crate::types::GraphQLType;
use crate::types::ParameterDefinition;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::value::Value;
use indexmap::IndexMap;
use libgraphql_parser::ast;
use libgraphql_parser::ByteSpan;
use libgraphql_parser::GraphQLErrorNoteKind;

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

        // @deprecated(reason: String! = "No longer supported")
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
                            /* nullable = */ false,
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

    /// Registers a type builder with the schema. Accepts any
    /// `impl` [`IntoGraphQLType`] (all six type builders
    /// implement this trait). Converts the builder to a
    /// finalized [`GraphQLType`], checks for duplicate type
    /// names, and inserts.
    pub fn absorb_type(
        &mut self,
        builder: impl IntoGraphQLType,
    ) -> Result<&mut Self, SchemaBuildError> {
        let name = builder.type_name().clone();
        let span = builder.type_span();

        // Check duplicate
        if let Some(existing) = self.types.get(&name) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateTypeDefinition {
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
        let graphql_type = builder.into_graphql_type();
        self.types.insert(name, graphql_type);
        Ok(self)
    }

    /// Registers a directive builder with the schema.
    ///
    /// Rejects redefinition of the five built-in directives
    /// (`@skip`, `@include`, `@deprecated`, `@specifiedBy`,
    /// `@oneOf`) and duplicate custom directive names.
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
                    vec![
                        ErrorNote::general_with_span(
                            "first defined here",
                            existing.span(),
                        ),
                    ],
                ));
            }
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateDirectiveDefinition {
                    name: name.to_string(),
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
    /// Registers a [`SchemaSourceMap`] for the source text so
    /// that spans within it can be resolved to line/column
    /// later. Returns `&mut Self` for method chaining. Parse
    /// errors are collected into the returned `Err` variant
    /// with their original parser spans translated to our
    /// [`Span`] type.
    pub fn load_str(
        &mut self,
        source: &str,
    ) -> Result<&mut Self, Vec<SchemaBuildError>> {
        let parse_result =
            libgraphql_parser::parse_schema(source);

        // Register source map BEFORE checking parse errors
        // so we have a source_map_id for span translation.
        let source_map_id = match u16::try_from(
            self.source_maps.len(),
        ) {
            Ok(id) => SourceMapId(id),
            Err(_) => {
                return Err(vec![SchemaBuildError::new(
                    SchemaBuildErrorKind::SourceMapLimitExceeded,
                    Span::builtin(),
                    vec![],
                )]);
            },
        };
        self.source_maps.push(
            SchemaSourceMap::from_source(source, None),
        );

        // Report parse-level errors with proper spans
        if parse_result.has_errors() {
            let parse_errors: Vec<SchemaBuildError> =
                parse_result.errors().iter().map(|e| {
                    let error_span =
                        translate_parser_span(
                            e.source_span(), source_map_id,
                        );
                    let notes = e.notes().iter().map(|n| {
                        let note_span =
                            n.span.as_ref().map(|s| {
                                translate_parser_span(
                                    s, source_map_id,
                                )
                            });
                        let kind = match n.kind {
                            GraphQLErrorNoteKind::General => {
                                ErrorNoteKind::General
                            },
                            GraphQLErrorNoteKind::Help => {
                                ErrorNoteKind::Help
                            },
                            GraphQLErrorNoteKind::Spec => {
                                ErrorNoteKind::Spec
                            },
                        };
                        ErrorNote {
                            kind,
                            message: n.message.clone(),
                            span: note_span,
                        }
                    }).collect();
                    SchemaBuildError::new(
                        SchemaBuildErrorKind::ParseError {
                            message: e.message().to_string(),
                        },
                        error_span,
                        notes,
                    )
                }).collect();
            return Err(parse_errors);
        }

        let doc = parse_result.ast();
        self.load_document(doc, source_map_id);
        Ok(self)
    }

    /// Iterates over all definitions in a parsed document and
    /// absorbs type definitions, directive definitions, and
    /// `schema { ... }` definitions. Skips schema extensions,
    /// type extensions, operation definitions, and fragment
    /// definitions (which are not first-pass schema-level
    /// definitions).
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
                    match DirectiveBuilder::from_ast(
                        dd, source_map_id,
                    ) {
                        Ok(builder) => {
                            if let Err(e) =
                                self.absorb_directive(builder)
                            {
                                self.errors.push(e);
                            }
                        },
                        Err(errs) => {
                            self.errors.extend(errs);
                        },
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
        macro_rules! absorb_from_ast {
            ($builder:ident, $ast_node:expr) => {
                match $builder::from_ast($ast_node, source_map_id) {
                    Ok(builder) => {
                        if let Err(e) = self.absorb_type(builder) {
                            self.errors.push(e);
                        }
                    },
                    Err(errs) => {
                        self.errors.extend(errs);
                    },
                }
            };
        }
        match td {
            ast::TypeDefinition::Enum(e) => {
                absorb_from_ast!(EnumTypeBuilder, e);
            },
            ast::TypeDefinition::InputObject(io) => {
                absorb_from_ast!(InputObjectTypeBuilder, io);
            },
            ast::TypeDefinition::Interface(i) => {
                absorb_from_ast!(InterfaceTypeBuilder, i);
            },
            ast::TypeDefinition::Object(o) => {
                absorb_from_ast!(ObjectTypeBuilder, o);
            },
            ast::TypeDefinition::Scalar(s) => {
                absorb_from_ast!(ScalarTypeBuilder, s);
            },
            ast::TypeDefinition::Union(u) => {
                absorb_from_ast!(UnionTypeBuilder, u);
            },
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

    /// Returns accumulated errors (for test inspection).
    pub(crate) fn errors(&self) -> &[SchemaBuildError] {
        &self.errors
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

// ---------------------------------------------------------
// Parser span translation helper
// ---------------------------------------------------------

/// Translates a parser [`SourceSpan`](libgraphql_parser::SourceSpan)
/// into our [`Span`] type by extracting byte offsets and
/// attaching the given `source_map_id`.
fn translate_parser_span(
    source_span: &libgraphql_parser::SourceSpan,
    source_map_id: SourceMapId,
) -> Span {
    let start = source_span
        .start_inclusive
        .byte_offset() as u32;
    let end = source_span
        .end_exclusive
        .byte_offset() as u32;
    Span::new(ByteSpan::new(start, end), source_map_id)
}
