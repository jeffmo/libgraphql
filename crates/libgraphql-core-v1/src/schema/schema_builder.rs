use crate::error_note::ErrorNote;
use crate::error_note::ErrorNoteKind;
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::operation_kind::OperationKind;
use crate::schema::pending_type_extension::PendingEnumTypeExtension;
use crate::schema::pending_type_extension::PendingFieldedTypeExtension;
use crate::schema::pending_type_extension::PendingInputObjectTypeExtension;
use crate::schema::pending_type_extension::PendingTypeExtension;
use crate::schema::pending_type_extension::PendingUnionTypeExtension;
use crate::schema::schema_build_error::SchemaBuildError;
use crate::schema::schema_build_error::SchemaBuildErrorKind;
use crate::schema::schema_def::Schema;
use crate::schema::schema_errors::SchemaErrors;
use crate::schema_source_map::SchemaSourceMap;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::spec_urls;
use crate::type_builders::ast_helpers;
use crate::type_builders::conversion_helpers::enum_value_from_builder;
use crate::type_builders::conversion_helpers::field_def_from_builder;
use crate::type_builders::conversion_helpers::input_field_from_builder;
use crate::type_builders::conversion_helpers::param_def_from_builder;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::InputObjectTypeBuilder;
use crate::type_builders::InterfaceTypeBuilder;
use crate::type_builders::IntoGraphQLType;
use crate::type_builders::ObjectTypeBuilder;
use crate::type_builders::ScalarTypeBuilder;
use crate::type_builders::UnionTypeBuilder;
use crate::types::DeprecationState;
use crate::types::DirectiveDefinition;
use crate::types::DirectiveDefinitionKind;
use crate::types::DirectiveLocationKind;
use crate::types::EnumType;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InputObjectType;
use crate::types::ParameterDefinition;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::validators::InputObjectTypeValidator;
use crate::validators::ObjectOrInterfaceTypeValidator;
use crate::validators::UnionTypeValidator;
use crate::validators::validate_directive_definitions;
use crate::value::Value;
use indexmap::IndexMap;
use libgraphql_parser::ast;
use libgraphql_parser::ByteSpan;
use libgraphql_parser::GraphQLErrorNoteKind;
use std::path::Path;
use std::path::PathBuf;

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
    /// Type extensions whose target type has not been defined
    /// yet. Keyed by target type name; each entry holds the
    /// extensions in arrival order. Applied when the target's
    /// definition is absorbed; any still pending at `build()`
    /// produce `ExtensionOfUndefinedType` errors.
    pending_extensions: IndexMap<TypeName, Vec<PendingTypeExtension>>,
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
            pending_extensions: IndexMap::new(),
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
    /// See [Built-in
    /// Scalars](https://spec.graphql.org/September2025/#sec-Scalars.Built-in-Scalars).
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
    /// See [Built-in Directives][built-in-directives].
    ///
    /// [built-in-directives]:
    ///   https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives
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
                            DeprecationState::DEFAULT_REASON.to_string(),
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
            // https://spec.graphql.org/September2025/#sec-Schema
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
                    ErrorNote::spec(spec_urls::SCHEMA),
                ],
            ));
        }

        // Convert builder to GraphQLType and insert
        let graphql_type = builder.into_graphql_type();
        self.types.insert(name.clone(), graphql_type);

        // Apply any extensions that arrived before this type's
        // definition (extensions may legally precede the
        // definition in load order), in arrival order. Merge
        // errors are deferred to `build()`.
        //
        // https://spec.graphql.org/September2025/#sec-Type-Extensions
        if let Some(pending) = self.pending_extensions.shift_remove(&name) {
            for ext in pending {
                self.apply_type_extension(ext);
            }
        }
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
                // "All directives within a GraphQL schema must
                // have unique names" -- built-in directives are
                // provided by the implementation, so redefining
                // one always collides.
                //
                // https://spec.graphql.org/September2025/#sec-Schema
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
                        ErrorNote::spec(spec_urls::SCHEMA),
                    ],
                ));
            }
            // https://spec.graphql.org/September2025/#sec-Schema
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
                    ErrorNote::spec(spec_urls::SCHEMA),
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
    ///
    /// The registered source map carries no source label; when
    /// loading multiple sources, prefer
    /// [`load_str_with_label()`](Self::load_str_with_label) so
    /// diagnostics can identify which source a location refers
    /// to.
    pub fn load_str(
        &mut self,
        source: &str,
    ) -> Result<&mut Self, Vec<SchemaBuildError>> {
        self.load_str_impl(source, /* label = */ None)
    }

    /// Like [`load_str()`](Self::load_str), but labels the
    /// registered [`SchemaSourceMap`] with `label` (typically
    /// the path the source text was read from).
    ///
    /// The label is stored as the source map's
    /// [`file_path()`](SchemaSourceMap::file_path) and is
    /// surfaced by diagnostics that resolve spans from this
    /// source, which is especially useful when a schema is
    /// assembled from multiple source strings.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libgraphql_core_v1 as libgraphql_core;
    /// use libgraphql_core::schema::SchemaBuilder;
    /// use std::path::Path;
    ///
    /// let mut builder = SchemaBuilder::new();
    /// builder.load_str_with_label(
    ///     "type Query { hello: String }",
    ///     "schemas/main.graphql",
    /// ).unwrap();
    /// let schema = builder.build().unwrap();
    ///
    /// assert_eq!(
    ///     schema.source_maps()[1].file_path(),
    ///     Some(Path::new("schemas/main.graphql")),
    /// );
    /// ```
    pub fn load_str_with_label(
        &mut self,
        source: &str,
        label: impl AsRef<Path>,
    ) -> Result<&mut Self, Vec<SchemaBuildError>> {
        self.load_str_impl(source, Some(label.as_ref().to_path_buf()))
    }

    /// Convenience: creates a new `SchemaBuilder` and loads
    /// `source` into it in one step.
    ///
    /// Equivalent to [`SchemaBuilder::new()`](Self::new)
    /// followed by [`load_str()`](Self::load_str). Additional
    /// sources can still be loaded into the returned builder
    /// before calling [`build()`](Self::build).
    ///
    /// The error type mirrors `load_str`'s
    /// (`Vec<SchemaBuildError>`, the load-phase shape) rather than
    /// [`build()`](Self::build)'s `SchemaErrors` — `from_str` is a
    /// load-phase API; only `build*` methods return `SchemaErrors`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libgraphql_core_v1 as libgraphql_core;
    /// use libgraphql_core::schema::SchemaBuilder;
    ///
    /// let builder = SchemaBuilder::from_str(
    ///     "type Query { hello: String }",
    /// ).unwrap();
    /// let schema = builder.build().unwrap();
    ///
    /// assert!(schema.object_type("Query").is_some());
    /// ```
    // An inherent `from_str` is intentional (mirroring the other
    // string entry points and the v0 API) rather than a
    // `std::str::FromStr` impl, which would force callers to
    // import the trait for a builder-construction convenience.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(source: &str) -> Result<Self, Vec<SchemaBuildError>> {
        let mut builder = Self::new();
        builder.load_str(source)?;
        Ok(builder)
    }

    /// Like [`from_str()`](Self::from_str), but labels the
    /// registered [`SchemaSourceMap`] with `label` (see
    /// [`load_str_with_label()`](Self::load_str_with_label)).
    pub fn from_str_with_label(
        source: &str,
        label: impl AsRef<Path>,
    ) -> Result<Self, Vec<SchemaBuildError>> {
        let mut builder = Self::new();
        builder.load_str_with_label(source, label)?;
        Ok(builder)
    }

    /// Shared implementation for the `load_str` family:
    /// registers a [`SchemaSourceMap`] (labeled with `label`,
    /// if provided) for `source`, parses it, and loads all
    /// definitions into this builder.
    fn load_str_impl(
        &mut self,
        source: &str,
        label: Option<PathBuf>,
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
                // No spec note: this is an implementation limit
                // of this crate (u16 source-map IDs), not a rule
                // from the GraphQL specification.
                return Err(vec![SchemaBuildError::new(
                    SchemaBuildErrorKind::SourceMapLimitExceeded,
                    Span::builtin(),
                    vec![],
                )]);
            },
        };
        self.source_maps.push(
            SchemaSourceMap::from_source(source, label),
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
                    // No schema-level spec note: parse errors
                    // are grammar-level, and any notes (spec
                    // ones included) are supplied by the parser
                    // and translated verbatim above.
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
    /// absorbs type definitions, directive definitions,
    /// `schema { ... }` definitions, type extensions, and
    /// schema extensions. Skips operation definitions and
    /// fragment definitions (which are not schema-level
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
                ast::Definition::SchemaExtension(se) => {
                    self.load_schema_extension(se, source_map_id);
                },
                ast::Definition::TypeExtension(te) => {
                    self.load_type_extension(te, source_map_id);
                },
                // Skip operations and fragments
                ast::Definition::OperationDefinition(_)
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
        self.load_root_operations(&sd.root_operations, source_map_id);
    }

    /// Processes an `extend schema { ... }` extension, merging
    /// its root operation type bindings with the same duplicate
    /// handling as a `schema { ... }` definition.
    ///
    /// Schema-level directive annotations are not yet stored on
    /// [`Schema`] (they are dropped for `schema { ... }`
    /// definitions too), so extension directives are likewise
    /// not retained here.
    ///
    /// See [Schema Extension](https://spec.graphql.org/September2025/#sec-Schema-Extension).
    fn load_schema_extension(
        &mut self,
        se: &ast::SchemaExtension<'_>,
        source_map_id: SourceMapId,
    ) {
        self.load_root_operations(&se.root_operations, source_map_id);
    }

    /// Binds root operation types (query, mutation,
    /// subscription) from a `schema { ... }` definition or an
    /// `extend schema { ... }` extension. Rebinding an
    /// already-bound root operation kind is an error.
    ///
    /// See [Root Operation
    /// Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    fn load_root_operations(
        &mut self,
        root_operations: &[ast::RootOperationTypeDefinition<'_>],
        source_map_id: SourceMapId,
    ) {
        for root_op in root_operations {
            let type_name = TypeName::new(
                root_op.named_type.value.as_ref(),
            );
            let span = ast_helpers::span_from_ast(
                root_op.span, source_map_id,
            );
            let operation: OperationKind =
                root_op.operation_kind.into();
            let slot = match operation {
                OperationKind::Query => {
                    &mut self.query_type_name
                },
                OperationKind::Mutation => {
                    &mut self.mutation_type_name
                },
                OperationKind::Subscription => {
                    &mut self.subscription_type_name
                },
            };
            if let Some((existing_name, existing_span)) = slot {
                // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
                self.errors.push(SchemaBuildError::new(
                    SchemaBuildErrorKind::DuplicateOperationDefinition {
                        operation,
                        type_name: existing_name.to_string(),
                    },
                    span,
                    vec![
                        ErrorNote::general_with_span(
                            "first defined here",
                            *existing_span,
                        ),
                        ErrorNote::spec(
                            "https://spec.graphql.org/September2025/\
                            #sec-Root-Operation-Types",
                        ),
                    ],
                ));
            } else {
                *slot = Some((type_name, span));
            }
        }
    }

    /// Processes an `extend <kind> Name ...` type extension.
    ///
    /// If the target type is already registered, the extension
    /// is merged into it immediately. Otherwise the extension
    /// is stored pending and applied when (if) the target's
    /// definition is absorbed -- extensions may legally precede
    /// the definition in load order. Extensions still pending
    /// at [`build()`](Self::build) time produce
    /// `ExtensionOfUndefinedType` errors.
    ///
    /// See [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions).
    fn load_type_extension(
        &mut self,
        te: &ast::TypeExtension<'_>,
        source_map_id: SourceMapId,
    ) {
        let ext = match PendingTypeExtension::from_ast(te, source_map_id) {
            Ok(ext) => ext,
            Err(errs) => {
                self.errors.extend(errs);
                return;
            },
        };
        if self.types.contains_key(ext.type_name()) {
            self.apply_type_extension(ext);
        } else {
            self.pending_extensions
                .entry(ext.type_name().clone())
                .or_default()
                .push(ext);
        }
    }

    /// Merges a type extension into its (already-registered)
    /// target type in place. If the extension's kind does not
    /// match the target type's kind, an
    /// `InvalidExtensionTypeKind` error is recorded and the
    /// extension is not applied.
    ///
    /// See [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions).
    fn apply_type_extension(&mut self, ext: PendingTypeExtension) {
        let ext_kind = ext.extension_kind();
        let ext_span = ext.span();
        let spec_url = ext.spec_url();
        let Some(target) = self.types.get_mut(ext.type_name()) else {
            // Callers only invoke this for registered targets.
            return;
        };
        match (target, ext) {
            (GraphQLType::Enum(t), PendingTypeExtension::Enum(pe)) => {
                merge_enum_type_extension(
                    t, pe, spec_url, &mut self.errors,
                );
            },
            (
                GraphQLType::InputObject(t),
                PendingTypeExtension::InputObject(pe),
            ) => {
                merge_input_object_type_extension(
                    t, pe, spec_url, &mut self.errors,
                );
            },
            (
                GraphQLType::Interface(t),
                PendingTypeExtension::Interface(pe),
            ) => {
                merge_fielded_type_extension(
                    &mut t.0, pe, spec_url, &mut self.errors,
                );
            },
            (GraphQLType::Object(t), PendingTypeExtension::Object(pe)) => {
                merge_fielded_type_extension(
                    &mut t.0, pe, spec_url, &mut self.errors,
                );
            },
            (GraphQLType::Scalar(t), PendingTypeExtension::Scalar(pe)) => {
                // Scalar extensions may only contribute
                // directives.
                //
                // https://spec.graphql.org/September2025/#sec-Scalar-Extensions
                t.directives.extend(pe.directives);
            },
            (GraphQLType::Union(t), PendingTypeExtension::Union(pe)) => {
                merge_union_type_extension(
                    t, pe, spec_url, &mut self.errors,
                );
            },
            (target, ext) => {
                // https://spec.graphql.org/September2025/#sec-Type-Extensions
                self.errors.push(SchemaBuildError::new(
                    SchemaBuildErrorKind::InvalidExtensionTypeKind {
                        actual_kind: target.type_kind(),
                        extension_kind: ext_kind,
                        type_name: ext.type_name().to_string(),
                    },
                    ext_span,
                    vec![
                        ErrorNote::general_with_span(
                            "target type defined here",
                            target.span(),
                        ),
                        ErrorNote::spec(spec_url),
                    ],
                ));
            },
        }
    }

    /// Validates and finalizes the schema.
    ///
    /// Resolves root operation types, validates all type and
    /// directive definitions, and returns an immutable [`Schema`]
    /// on success. On failure, returns a [`SchemaErrors`]
    /// containing all accumulated errors.
    ///
    /// # Validation phases
    ///
    /// 1. **Root query type resolution** -- uses the explicit
    ///    `schema { query: ... }` binding if present, otherwise
    ///    defaults to `"Query"` per the
    ///    [spec](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    /// 2. **Root type validation** -- ensures query, mutation, and
    ///    subscription root types exist and are object types.
    /// 3. **Empty type checks** -- rejects object/interface types
    ///    with no fields, unions with no members, and enums with
    ///    no values.
    /// 4. **Type-system validators** -- runs the (internal)
    ///    `ObjectOrInterfaceTypeValidator`,
    ///    `UnionTypeValidator`,
    ///    `InputObjectTypeValidator`, and
    ///    `validate_directive_definitions` passes to enforce
    ///    cross-type reference rules.
    ///
    /// See [Schema](https://spec.graphql.org/September2025/#sec-Schema).
    // TODO: SchemaErrors wraps Vec<SchemaBuildError> which is
    // large. Consider boxing once error strategy is finalized.
    #[allow(clippy::result_large_err)]
    pub fn build(mut self) -> Result<Schema, SchemaErrors> {
        // Any extension still pending at build time targets a
        // type that was never defined.
        let pending_extensions =
            std::mem::take(&mut self.pending_extensions);
        for (_, extensions) in pending_extensions {
            for ext in extensions {
                // https://spec.graphql.org/September2025/#sec-Type-Extensions
                self.errors.push(SchemaBuildError::new(
                    SchemaBuildErrorKind::ExtensionOfUndefinedType {
                        type_name: ext.type_name().to_string(),
                    },
                    ext.span(),
                    vec![ErrorNote::spec(ext.spec_url())],
                ));
            }
        }

        // Step 1: Resolve root query type name.
        //
        // If an explicit `schema { query: ... }` was provided, use
        // that binding. Otherwise, default to "Query" per the spec:
        // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
        let query_type_name = match &self.query_type_name {
            Some((name, _)) => name.clone(),
            None => TypeName::new("Query"),
        };

        if !self.types.contains_key(&query_type_name) {
            // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
            self.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::NoQueryOperationTypeDefined,
                self.query_type_name
                    .as_ref()
                    .map(|(_, span)| *span)
                    .unwrap_or(Span::builtin()),
                vec![ErrorNote::spec(spec_urls::ROOT_OPERATION_TYPES)],
            ));
        }

        // Step 2: Validate root types are object types.
        //
        // Clone names/spans up front to avoid borrowing `self`
        // immutably while calling `validate_root_type` mutably.
        let query_span = self.query_type_name
            .as_ref()
            .map(|(_, span)| *span)
            .unwrap_or(Span::builtin());
        let mutation_binding = self.mutation_type_name
            .as_ref()
            .map(|(name, span)| (name.clone(), *span));
        let subscription_binding = self.subscription_type_name
            .as_ref()
            .map(|(name, span)| (name.clone(), *span));

        self.validate_root_type(
            OperationKind::Query, Some(&query_type_name), query_span,
        );
        if let Some((ref name, span)) = mutation_binding {
            self.validate_root_type(
                OperationKind::Mutation, Some(name), span,
            );
        }
        if let Some((ref name, span)) = subscription_binding {
            self.validate_root_type(
                OperationKind::Subscription, Some(name), span,
            );
        }

        // Step 3: Check for empty types (build-level checks).
        for graphql_type in self.types.values() {
            match graphql_type {
                GraphQLType::Object(obj) => {
                    if obj.fields().is_empty() {
                        // https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
                        self.errors.push(SchemaBuildError::new(
                            SchemaBuildErrorKind::EmptyObjectOrInterfaceType {
                                type_kind: graphql_type.type_kind(),
                                type_name: obj.name().to_string(),
                            },
                            obj.span(),
                            vec![
                                ErrorNote::spec(
                                    spec_urls::OBJECTS_TYPE_VALIDATION,
                                ),
                            ],
                        ));
                    }
                },
                GraphQLType::Interface(iface) => {
                    if iface.fields().is_empty() {
                        // https://spec.graphql.org/September2025/#sec-Interfaces.Type-Validation
                        self.errors.push(SchemaBuildError::new(
                            SchemaBuildErrorKind::EmptyObjectOrInterfaceType {
                                type_kind: graphql_type.type_kind(),
                                type_name: iface.name().to_string(),
                            },
                            iface.span(),
                            vec![
                                ErrorNote::spec(
                                    spec_urls::INTERFACES_TYPE_VALIDATION,
                                ),
                            ],
                        ));
                    }
                },
                GraphQLType::Union(union_t) => {
                    if union_t.members().is_empty() {
                        // https://spec.graphql.org/September2025/#sec-Unions.Type-Validation
                        self.errors.push(SchemaBuildError::new(
                            SchemaBuildErrorKind::EmptyUnionType {
                                type_name: union_t.name().to_string(),
                            },
                            union_t.span(),
                            vec![
                                ErrorNote::spec(
                                    spec_urls::UNIONS_TYPE_VALIDATION,
                                ),
                            ],
                        ));
                    }
                },
                GraphQLType::Enum(enum_t) if enum_t.values().is_empty() => {
                    // https://spec.graphql.org/September2025/#sec-Enums.Type-Validation
                    self.errors.push(SchemaBuildError::new(
                        SchemaBuildErrorKind::EnumWithNoValues {
                            type_name: enum_t.name().to_string(),
                        },
                        enum_t.span(),
                        vec![
                            ErrorNote::spec(
                                spec_urls::ENUMS_TYPE_VALIDATION,
                            ),
                        ],
                    ));
                },
                _ => {},
            }
        }

        // Step 4: Run validators.
        let mut validation_errors = Vec::new();

        for graphql_type in self.types.values() {
            match graphql_type {
                GraphQLType::Object(obj) => {
                    let errs = ObjectOrInterfaceTypeValidator::new(
                        obj.as_ref(),
                        &self.types,
                    ).validate();
                    validation_errors.extend(errs);
                },
                GraphQLType::Interface(iface) => {
                    let errs = ObjectOrInterfaceTypeValidator::new(
                        iface.as_ref(),
                        &self.types,
                    ).validate();
                    validation_errors.extend(errs);
                },
                GraphQLType::Union(union_t) => {
                    let errs = UnionTypeValidator::new(
                        union_t.as_ref(),
                        &self.types,
                    ).validate();
                    validation_errors.extend(errs);
                },
                GraphQLType::InputObject(input_obj) => {
                    let errs = InputObjectTypeValidator::new(
                        input_obj.as_ref(),
                        &self.types,
                    ).validate();
                    validation_errors.extend(errs);
                },
                _ => {},
            }
        }

        // Validate directive definitions.
        let directive_errs = validate_directive_definitions(
            &self.directive_defs,
            &self.types,
        );
        validation_errors.extend(directive_errs);

        // Wrap TypeValidationErrors into SchemaBuildErrors,
        // propagating the inner notes (every TypeValidationError
        // carries its own spec-reference note) so that
        // SchemaBuildError::notes() surfaces them uniformly.
        for tve in validation_errors {
            let span = tve.span();
            let notes = tve.notes().to_vec();
            self.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::TypeValidation(tve),
                span,
                notes,
            ));
        }

        // Step 5: Return result.
        if !self.errors.is_empty() {
            return Err(SchemaErrors::new(self.errors));
        }

        Ok(Schema {
            directive_defs: self.directive_defs,
            mutation_type_name: self.mutation_type_name
                .map(|(name, _)| name),
            query_type_name,
            source_maps: self.source_maps,
            subscription_type_name: self.subscription_type_name
                .map(|(name, _)| name),
            types: self.types,
        })
    }

    /// Convenience: parse a schema string and build in one step.
    // TODO: SchemaErrors wraps Vec<SchemaBuildError> which is
    // large. Consider boxing once error strategy is finalized.
    #[allow(clippy::result_large_err)]
    pub fn build_from_str(
        source: &str,
    ) -> Result<Schema, SchemaErrors> {
        Self::from_str(source)
            .map_err(SchemaErrors::new)
            .and_then(Self::build)
    }

    /// Like [`build_from_str()`](Self::build_from_str), but
    /// labels the registered [`SchemaSourceMap`] with `label`
    /// (see [`load_str_with_label()`](Self::load_str_with_label)).
    // TODO: SchemaErrors wraps Vec<SchemaBuildError> which is
    // large. Consider boxing once error strategy is finalized.
    #[allow(clippy::result_large_err)]
    pub fn build_from_str_with_label(
        source: &str,
        label: impl AsRef<Path>,
    ) -> Result<Schema, SchemaErrors> {
        Self::from_str_with_label(source, label)
            .map_err(SchemaErrors::new)
            .and_then(Self::build)
    }

    // ---------------------------------------------------------
    // Root type validation helper
    // ---------------------------------------------------------

    /// Validates that a root operation type (if it exists in the
    /// type map) is an object type. Emits
    /// `RootOperationTypeNotDefined` (for mutation/subscription
    /// only -- query uses `NoQueryOperationTypeDefined` instead)
    /// or `RootOperationTypeNotObjectType`.
    fn validate_root_type(
        &mut self,
        operation: OperationKind,
        type_name: Option<&TypeName>,
        span: Span,
    ) {
        let Some(name) = type_name else { return };
        let Some(graphql_type) = self.types.get(name) else {
            // Only emit RootOperationTypeNotDefined for
            // mutation/subscription. Query missing is handled
            // separately via NoQueryOperationTypeDefined.
            if operation != OperationKind::Query {
                // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
                self.errors.push(SchemaBuildError::new(
                    SchemaBuildErrorKind::RootOperationTypeNotDefined {
                        operation,
                        type_name: name.to_string(),
                    },
                    span,
                    vec![
                        ErrorNote::spec(
                            spec_urls::ROOT_OPERATION_TYPES,
                        ),
                    ],
                ));
            }
            return;
        };
        if !matches!(graphql_type, GraphQLType::Object(_)) {
            // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
            self.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::RootOperationTypeNotObjectType {
                    actual_kind: graphql_type.type_kind(),
                    operation,
                    type_name: name.to_string(),
                },
                span,
                vec![ErrorNote::spec(spec_urls::ROOT_OPERATION_TYPES)],
            ));
        }
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
}

// ---------------------------------------------------------
// Type extension merge helpers
// ---------------------------------------------------------

/// Merges an enum type extension into the stored [`EnumType`]
/// in place: extension values are appended (duplicates
/// rejected) and extension directives are appended.
///
/// See [Enum Extensions](https://spec.graphql.org/September2025/#sec-Enum-Extensions).
fn merge_enum_type_extension(
    enum_type: &mut EnumType,
    ext: PendingEnumTypeExtension,
    spec_url: &'static str,
    errors: &mut Vec<SchemaBuildError>,
) {
    for value in ext.values {
        if let Some(existing) = enum_type.values.get(&value.name) {
            // https://spec.graphql.org/September2025/#sec-Enum-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateEnumValueDefinition {
                    type_name: enum_type.name.to_string(),
                    value_name: value.name.to_string(),
                },
                value.span,
                vec![
                    ErrorNote::general_with_span(
                        "first defined here",
                        existing.span(),
                    ),
                    ErrorNote::spec(spec_url),
                ],
            ));
            continue;
        }
        let enum_value = enum_value_from_builder(value, &enum_type.name);
        enum_type.values.insert(enum_value.name.clone(), enum_value);
    }
    enum_type.directives.extend(ext.directives);
}

/// Merges an object or interface type extension into the stored
/// type's [`FieldedTypeData`] in place: extension fields and
/// `implements` declarations are appended (duplicates rejected,
/// `__`-prefixed field names rejected) and extension directives
/// are appended.
///
/// See
/// [Object Extensions](https://spec.graphql.org/September2025/#sec-Object-Extensions)
/// and
/// [Interface Extensions](https://spec.graphql.org/September2025/#sec-Interface-Extensions).
fn merge_fielded_type_extension(
    data: &mut FieldedTypeData,
    ext: PendingFieldedTypeExtension,
    spec_url: &'static str,
    errors: &mut Vec<SchemaBuildError>,
) {
    for iface in ext.implements {
        let existing = data.interfaces
            .iter()
            .find(|l| l.value == iface.value);
        if let Some(existing) = existing {
            // https://spec.graphql.org/September2025/#sec-Object-Extensions
            // https://spec.graphql.org/September2025/#sec-Interface-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateInterfaceImplementsDeclaration {
                    interface_name: iface.value.to_string(),
                    type_name: data.name.to_string(),
                },
                iface.span,
                vec![
                    ErrorNote::general_with_span(
                        "first declared here",
                        existing.span,
                    ),
                    ErrorNote::spec(spec_url),
                ],
            ));
            continue;
        }
        data.interfaces.push(iface);
    }
    for field in ext.fields {
        if field.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                    field_name: field.name.to_string(),
                    type_name: data.name.to_string(),
                },
                field.span,
                vec![ErrorNote::spec(spec_urls::RESERVED_NAMES)],
            ));
            continue;
        }
        if let Some(existing) = data.fields.get(&field.name) {
            // https://spec.graphql.org/September2025/#sec-Object-Extensions
            // https://spec.graphql.org/September2025/#sec-Interface-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                    field_name: field.name.to_string(),
                    type_name: data.name.to_string(),
                },
                field.span,
                vec![
                    ErrorNote::general_with_span(
                        "first defined here",
                        existing.span(),
                    ),
                    ErrorNote::spec(spec_url),
                ],
            ));
            continue;
        }
        let field_def = field_def_from_builder(field, &data.name);
        data.fields.insert(field_def.name.clone(), field_def);
    }
    data.directives.extend(ext.directives);
}

/// Merges an input object type extension into the stored
/// [`InputObjectType`] in place: extension input fields are
/// appended (duplicates rejected, `__`-prefixed field names
/// rejected) and extension directives are appended.
///
/// See [Input Object
/// Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions).
fn merge_input_object_type_extension(
    input_object_type: &mut InputObjectType,
    ext: PendingInputObjectTypeExtension,
    spec_url: &'static str,
    errors: &mut Vec<SchemaBuildError>,
) {
    for field in ext.fields {
        if field.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                    field_name: field.name.to_string(),
                    type_name: input_object_type.name.to_string(),
                },
                field.span,
                vec![ErrorNote::spec(spec_urls::RESERVED_NAMES)],
            ));
            continue;
        }
        if let Some(existing) = input_object_type.fields.get(&field.name) {
            // https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                    field_name: field.name.to_string(),
                    type_name: input_object_type.name.to_string(),
                },
                field.span,
                vec![
                    ErrorNote::general_with_span(
                        "first defined here",
                        existing.span(),
                    ),
                    ErrorNote::spec(spec_url),
                ],
            ));
            continue;
        }
        let input_field = input_field_from_builder(
            field, &input_object_type.name,
        );
        input_object_type.fields.insert(
            input_field.name.clone(), input_field,
        );
    }
    for directive in ext.directives {
        if directive.name().as_str() == "oneOf" {
            // Input Object Extensions rule 5: "The `@oneOf`
            // directive must not be provided by an Input Object
            // type extension."
            // https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::OneOfDirectiveProvidedByInputObjectExtension {
                    type_name: input_object_type.name.to_string(),
                },
                directive.span(),
                vec![ErrorNote::spec(spec_url)],
            ));
            continue;
        }
        input_object_type.directives.push(directive);
    }
}

/// Merges a union type extension into the stored [`UnionType`]
/// in place: extension members are appended (duplicates
/// rejected) and extension directives are appended.
///
/// See [Union Extensions](https://spec.graphql.org/September2025/#sec-Union-Extensions).
fn merge_union_type_extension(
    union_type: &mut UnionType,
    ext: PendingUnionTypeExtension,
    spec_url: &'static str,
    errors: &mut Vec<SchemaBuildError>,
) {
    for member in ext.members {
        let existing = union_type.members
            .iter()
            .find(|m| m.value == member.value);
        if let Some(existing) = existing {
            // https://spec.graphql.org/September2025/#sec-Union-Extensions
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateUnionMember {
                    member_name: member.value.to_string(),
                    type_name: union_type.name.to_string(),
                },
                member.span,
                vec![
                    ErrorNote::general_with_span(
                        "first defined here",
                        existing.span,
                    ),
                    ErrorNote::spec(spec_url),
                ],
            ));
            continue;
        }
        union_type.members.push(member);
    }
    union_type.directives.extend(ext.directives);
}

// ---------------------------------------------------------
// AST conversion helpers
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
