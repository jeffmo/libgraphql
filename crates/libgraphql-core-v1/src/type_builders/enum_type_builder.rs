use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::enum_value_def_builder::EnumValueDefBuilder;
use libgraphql_parser::ast;

/// Builder for constructing an
/// [`EnumType`](crate::types::EnumType).
///
/// See [Enums](https://spec.graphql.org/September2025/#sec-Enums).
#[derive(Debug)]
#[allow(dead_code)]
pub struct EnumTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
    pub(crate) values: Vec<EnumValueDefBuilder>,
}

#[allow(clippy::result_large_err)]
impl EnumTypeBuilder {
    /// Creates a new builder. Returns `Err` if `name` starts with
    /// `__` (reserved prefix per the GraphQL spec).
    pub fn new(
        name: impl Into<TypeName>,
        span: Span,
    ) -> Result<Self, SchemaBuildError> {
        let name = name.into();
        if name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: name.to_string(),
                },
                span,
                vec![],
            ));
        }
        Ok(Self {
            description: None,
            directives: vec![],
            errors: vec![],
            name,
            span,
            values: vec![],
        })
    }

    /// Sets the optional description string.
    pub fn set_description(
        &mut self,
        desc: impl Into<String>,
    ) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    /// Appends an enum value. Returns `Err` on duplicate or
    /// reserved name (`true`, `false`, `null`).
    pub fn add_value(
        &mut self,
        value: EnumValueDefBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        let name_str = value.name.as_str();
        if name_str == "true"
            || name_str == "false"
            || name_str == "null"
        {
            // https://spec.graphql.org/September2025/#sec-Enums.Type-Validation
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidEnumValueName {
                    type_name: self.name.to_string(),
                    value_name: value.name.to_string(),
                },
                value.span,
                vec![],
            ));
        }
        if self.values.iter().any(|v| v.name == value.name) {
            // https://spec.graphql.org/September2025/#sec-Enums.Type-Validation
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateEnumValueDefinition {
                    type_name: self.name.to_string(),
                    value_name: value.name.to_string(),
                },
                value.span,
                vec![],
            ));
        }
        self.values.push(value);
        Ok(self)
    }

    /// Appends an applied directive annotation.
    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }

    /// Constructs a builder from a parsed AST node, collecting
    /// validation errors internally instead of propagating them.
    pub(crate) fn from_ast(
        ast_enum: &ast::EnumTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_enum.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_enum.description,
            ),
            directives: vec![],
            errors: vec![],
            name: TypeName::new(ast_enum.name.value.as_ref()),
            span,
            values: vec![],
        };
        if builder.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            builder.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: builder.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        for dir in &ast_enum.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        for val in &ast_enum.values {
            let val_builder = EnumValueDefBuilder::from_ast(
                val, source_map_id,
            );
            if let Err(e) = builder.add_value(val_builder) {
                builder.errors.push(e);
            }
        }
        builder
    }
}
