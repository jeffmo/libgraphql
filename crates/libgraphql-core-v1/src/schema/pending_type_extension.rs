use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::TypeName;
use crate::schema::schema_build_error::SchemaBuildError;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::EnumValueDefBuilder;
use crate::type_builders::FieldDefBuilder;
use crate::type_builders::InputFieldDefBuilder;
use crate::types::GraphQLTypeKind;
use libgraphql_parser::ast;

/// An owned, deferred representation of a parsed type extension.
///
/// [`SchemaBuilder`](crate::schema::SchemaBuilder) converts each
/// `extend <kind> Name ...` AST node into this owned form
/// immediately (the parser AST borrows source text and cannot
/// outlive `load_str()`). If the extension's target type is
/// already registered, the extension is applied right away;
/// otherwise it is held pending until the target's definition
/// arrives -- extensions may legally precede the definition in
/// load order. Extensions still pending when
/// [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build)
/// runs produce `ExtensionOfUndefinedType` errors.
///
/// See [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions).
#[derive(Debug)]
pub(crate) enum PendingTypeExtension {
    Enum(PendingEnumTypeExtension),
    InputObject(PendingInputObjectTypeExtension),
    Interface(PendingFieldedTypeExtension),
    Object(PendingFieldedTypeExtension),
    Scalar(PendingScalarTypeExtension),
    Union(PendingUnionTypeExtension),
}

/// Owned contributions of an `extend enum Name ...` extension.
///
/// See [Enum Extensions](https://spec.graphql.org/September2025/#sec-Enum-Extensions).
#[derive(Debug)]
pub(crate) struct PendingEnumTypeExtension {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
    pub(crate) values: Vec<EnumValueDefBuilder>,
}

/// Owned contributions of an `extend type Name ...` or
/// `extend interface Name ...` extension (object and interface
/// extensions share the same shape: fields, `implements`
/// declarations, and directives).
///
/// See
/// [Object Extensions](https://spec.graphql.org/September2025/#sec-Object-Extensions)
/// and
/// [Interface Extensions](https://spec.graphql.org/September2025/#sec-Interface-Extensions).
#[derive(Debug)]
pub(crate) struct PendingFieldedTypeExtension {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: Vec<FieldDefBuilder>,
    pub(crate) implements: Vec<Located<TypeName>>,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

/// Owned contributions of an `extend input Name ...` extension.
///
/// See [Input Object Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions).
#[derive(Debug)]
pub(crate) struct PendingInputObjectTypeExtension {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: Vec<InputFieldDefBuilder>,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

/// Owned contributions of an `extend scalar Name ...` extension
/// (scalar extensions may only contribute directives).
///
/// See [Scalar Extensions](https://spec.graphql.org/September2025/#sec-Scalar-Extensions).
#[derive(Debug)]
pub(crate) struct PendingScalarTypeExtension {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

/// Owned contributions of an `extend union Name ...` extension.
///
/// See [Union Extensions](https://spec.graphql.org/September2025/#sec-Union-Extensions).
#[derive(Debug)]
pub(crate) struct PendingUnionTypeExtension {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) members: Vec<Located<TypeName>>,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

impl PendingTypeExtension {
    /// Converts a parsed type extension AST node into an owned
    /// `PendingTypeExtension`. Returns `Err` with all collected
    /// conversion errors if any are found (matching the eager
    /// `from_ast()` convention used by the type builders).
    pub(crate) fn from_ast(
        ext: &ast::TypeExtension<'_>,
        source_map_id: SourceMapId,
    ) -> Result<Self, Vec<SchemaBuildError>> {
        match ext {
            ast::TypeExtension::Enum(e) => {
                Ok(Self::Enum(PendingEnumTypeExtension {
                    directives: directives_from_ast(
                        &e.directives, source_map_id,
                    ),
                    span: ast_helpers::span_from_ast(
                        e.span, source_map_id,
                    ),
                    type_name: TypeName::new(e.name.value.as_ref()),
                    values: e.values.iter().map(|v| {
                        EnumValueDefBuilder::from_ast(v, source_map_id)
                    }).collect(),
                }))
            },
            ast::TypeExtension::InputObject(io) => {
                Ok(Self::InputObject(PendingInputObjectTypeExtension {
                    directives: directives_from_ast(
                        &io.directives, source_map_id,
                    ),
                    fields: io.fields.iter().map(|f| {
                        InputFieldDefBuilder::from_ast(f, source_map_id)
                    }).collect(),
                    span: ast_helpers::span_from_ast(
                        io.span, source_map_id,
                    ),
                    type_name: TypeName::new(io.name.value.as_ref()),
                }))
            },
            ast::TypeExtension::Interface(i) => {
                Ok(Self::Interface(fielded_from_ast(
                    &i.directives,
                    &i.fields,
                    &i.implements,
                    &i.name,
                    i.span,
                    source_map_id,
                )?))
            },
            ast::TypeExtension::Object(o) => {
                Ok(Self::Object(fielded_from_ast(
                    &o.directives,
                    &o.fields,
                    &o.implements,
                    &o.name,
                    o.span,
                    source_map_id,
                )?))
            },
            ast::TypeExtension::Scalar(s) => {
                Ok(Self::Scalar(PendingScalarTypeExtension {
                    directives: directives_from_ast(
                        &s.directives, source_map_id,
                    ),
                    span: ast_helpers::span_from_ast(
                        s.span, source_map_id,
                    ),
                    type_name: TypeName::new(s.name.value.as_ref()),
                }))
            },
            ast::TypeExtension::Union(u) => {
                Ok(Self::Union(PendingUnionTypeExtension {
                    directives: directives_from_ast(
                        &u.directives, source_map_id,
                    ),
                    members: u.members.iter().map(|m| Located {
                        value: TypeName::new(m.value.as_ref()),
                        span: ast_helpers::span_from_ast(
                            m.span, source_map_id,
                        ),
                    }).collect(),
                    span: ast_helpers::span_from_ast(
                        u.span, source_map_id,
                    ),
                    type_name: TypeName::new(u.name.value.as_ref()),
                }))
            },
        }
    }

    /// The [`GraphQLTypeKind`] this extension's `extend <kind>`
    /// keyword corresponds to (used for kind-mismatch
    /// diagnostics).
    pub(crate) fn extension_kind(&self) -> GraphQLTypeKind {
        match self {
            Self::Enum(_) => GraphQLTypeKind::Enum,
            Self::InputObject(_) => GraphQLTypeKind::InputObject,
            Self::Interface(_) => GraphQLTypeKind::Interface,
            Self::Object(_) => GraphQLTypeKind::Object,
            Self::Scalar(_) => GraphQLTypeKind::Scalar,
            Self::Union(_) => GraphQLTypeKind::Union,
        }
    }

    /// The source span of the whole `extend ...` production.
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Enum(ext) => ext.span,
            Self::InputObject(ext) => ext.span,
            Self::Interface(ext) => ext.span,
            Self::Object(ext) => ext.span,
            Self::Scalar(ext) => ext.span,
            Self::Union(ext) => ext.span,
        }
    }

    /// The URL of the spec section defining this extension
    /// kind's validation rules.
    pub(crate) fn spec_url(&self) -> &'static str {
        match self {
            Self::Enum(_) => {
                "https://spec.graphql.org/September2025/#sec-Enum-Extensions"
            },
            Self::InputObject(_) => {
                "https://spec.graphql.org/September2025/#sec-Input-Object-Extensions"
            },
            Self::Interface(_) => {
                "https://spec.graphql.org/September2025/#sec-Interface-Extensions"
            },
            Self::Object(_) => {
                "https://spec.graphql.org/September2025/#sec-Object-Extensions"
            },
            Self::Scalar(_) => {
                "https://spec.graphql.org/September2025/#sec-Scalar-Extensions"
            },
            Self::Union(_) => {
                "https://spec.graphql.org/September2025/#sec-Union-Extensions"
            },
        }
    }

    /// The name of the type this extension targets.
    pub(crate) fn type_name(&self) -> &TypeName {
        match self {
            Self::Enum(ext) => &ext.type_name,
            Self::InputObject(ext) => &ext.type_name,
            Self::Interface(ext) => &ext.type_name,
            Self::Object(ext) => &ext.type_name,
            Self::Scalar(ext) => &ext.type_name,
            Self::Union(ext) => &ext.type_name,
        }
    }
}

/// Converts a slice of AST directive annotations into owned
/// [`DirectiveAnnotation`]s.
fn directives_from_ast(
    ast_directives: &[ast::DirectiveAnnotation<'_>],
    source_map_id: SourceMapId,
) -> Vec<DirectiveAnnotation> {
    ast_directives.iter().map(|d| {
        ast_helpers::directive_annotation_from_ast(d, source_map_id)
    }).collect()
}

/// Shared conversion for object and interface type extensions
/// (identical AST shapes).
fn fielded_from_ast(
    ast_directives: &[ast::DirectiveAnnotation<'_>],
    ast_fields: &[ast::FieldDefinition<'_>],
    ast_implements: &[ast::Name<'_>],
    ast_name: &ast::Name<'_>,
    ast_span: libgraphql_parser::ByteSpan,
    source_map_id: SourceMapId,
) -> Result<PendingFieldedTypeExtension, Vec<SchemaBuildError>> {
    let mut errors = vec![];
    let mut fields = vec![];
    for field in ast_fields {
        match FieldDefBuilder::from_ast(field, source_map_id) {
            Ok(field_builder) => fields.push(field_builder),
            Err(field_errors) => errors.extend(field_errors),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(PendingFieldedTypeExtension {
        directives: directives_from_ast(ast_directives, source_map_id),
        fields,
        implements: ast_implements.iter().map(|iface| Located {
            value: TypeName::new(iface.value.as_ref()),
            span: ast_helpers::span_from_ast(iface.span, source_map_id),
        }).collect(),
        span: ast_helpers::span_from_ast(ast_span, source_map_id),
        type_name: TypeName::new(ast_name.value.as_ref()),
    })
}
