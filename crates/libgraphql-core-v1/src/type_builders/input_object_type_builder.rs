use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::input_field_def_builder::InputFieldDefBuilder;
use libgraphql_parser::ast;

/// Builder for constructing an
/// [`InputObjectType`](crate::types::InputObjectType).
///
/// See [Input Objects](https://spec.graphql.org/September2025/#sec-Input-Objects).
#[derive(Debug)]
#[allow(dead_code)]
pub struct InputObjectTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) fields: Vec<InputFieldDefBuilder>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

#[allow(clippy::result_large_err)]
impl InputObjectTypeBuilder {
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
            fields: vec![],
            name,
            span,
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

    /// Appends an input field. Returns `Err` on duplicate name or
    /// `__` prefix.
    pub fn add_field(
        &mut self,
        field: InputFieldDefBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        if field.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                    field_name: field.name.to_string(),
                    type_name: self.name.to_string(),
                },
                field.span,
                vec![],
            ));
        }
        if self.fields.iter().any(|f| f.name == field.name) {
            // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                    field_name: field.name.to_string(),
                    type_name: self.name.to_string(),
                },
                field.span,
                vec![],
            ));
        }
        self.fields.push(field);
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
        ast_input: &ast::InputObjectTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_input.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_input.description,
            ),
            directives: vec![],
            errors: vec![],
            fields: vec![],
            name: TypeName::new(ast_input.name.value.as_ref()),
            span,
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
        for dir in &ast_input.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        for field in &ast_input.fields {
            let field_builder = InputFieldDefBuilder::from_ast(
                field, source_map_id,
            );
            if let Err(e) = builder.add_field(field_builder) {
                builder.errors.push(e);
            }
        }
        builder
    }
}
