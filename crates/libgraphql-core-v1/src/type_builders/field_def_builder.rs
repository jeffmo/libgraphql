use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::parameter_def_builder::ParameterDefBuilder;
use crate::types::TypeAnnotation;
use libgraphql_parser::ast;

/// Builder-stage field definition data before validation.
#[derive(Debug)]
pub struct FieldDefBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) name: FieldName,
    pub(crate) parameters: Vec<ParameterDefBuilder>,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

// TODO: SchemaBuildError is large due to SchemaBuildErrorKind
// variants + Vec<ErrorNote>. Consider boxing the error or
// using an error index to reduce Result size.
#[allow(clippy::result_large_err)]
impl FieldDefBuilder {
    /// Creates a new field definition builder.
    pub fn new(
        name: impl Into<FieldName>,
        type_annotation: TypeAnnotation,
        span: Span,
    ) -> Self {
        Self {
            description: None,
            directives: vec![],
            errors: vec![],
            name: name.into(),
            parameters: vec![],
            span,
            type_annotation,
        }
    }

    /// Sets the optional description string.
    pub fn set_description(
        &mut self,
        desc: impl Into<String>,
    ) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    /// Appends an applied directive annotation.
    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }

    /// Appends a parameter. Returns `Err` on duplicate name or
    /// `__` prefix.
    pub fn add_parameter(
        &mut self,
        param: ParameterDefBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        if param.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedParamName {
                    field_name: self.name.to_string(),
                    param_name: param.name.to_string(),
                    type_name: None,
                },
                param.span,
                vec![],
            ));
        }
        if self.parameters.iter().any(|p| p.name == param.name) {
            // https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateParameterDefinition {
                    field_name: self.name.to_string(),
                    param_name: param.name.to_string(),
                    type_name: None,
                },
                param.span,
                vec![],
            ));
        }
        self.parameters.push(param);
        Ok(self)
    }

    /// Constructs a builder from a parsed AST node, collecting
    /// validation errors internally instead of propagating them.
    pub(crate) fn from_ast(
        ast_field: &ast::FieldDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_field.span,
            source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_field.description,
            ),
            directives: ast_field.directives.iter().map(|d| {
                ast_helpers::directive_annotation_from_ast(
                    d, source_map_id,
                )
            }).collect(),
            errors: vec![],
            name: FieldName::new(ast_field.name.value.as_ref()),
            parameters: vec![],
            span,
            type_annotation: ast_helpers::type_annotation_from_ast(
                &ast_field.field_type,
                source_map_id,
            ),
        };
        for param in &ast_field.parameters {
            let param_builder = ParameterDefBuilder::from_ast(
                param, source_map_id,
            );
            if let Err(e) = builder.add_parameter(param_builder) {
                builder.errors.push(e);
            }
        }
        builder
    }
}
