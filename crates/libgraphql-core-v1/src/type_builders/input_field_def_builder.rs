use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::types::TypeAnnotation;
use crate::value::Value;
use libgraphql_parser::ast;

/// Builder-stage input field data before validation.
#[derive(Debug)]
#[allow(dead_code)]
pub struct InputFieldDefBuilder {
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl InputFieldDefBuilder {
    /// Creates a new input field definition builder.
    pub fn new(
        name: impl Into<FieldName>,
        type_annotation: TypeAnnotation,
        span: Span,
    ) -> Self {
        Self {
            default_value: None,
            description: None,
            directives: vec![],
            name: name.into(),
            span,
            type_annotation,
        }
    }

    /// Sets the default value for this input field.
    pub fn set_default_value(
        &mut self,
        value: Value,
    ) -> &mut Self {
        self.default_value = Some(value);
        self
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

    /// Constructs a builder from a parsed AST node.
    pub(crate) fn from_ast(
        ast_field: &ast::InputValueDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        Self {
            default_value: ast_field.default_value
                .as_ref()
                .map(ast_helpers::value_from_ast),
            description: ast_helpers::description_from_ast(
                &ast_field.description,
            ),
            directives: ast_field.directives.iter().map(|d| {
                ast_helpers::directive_annotation_from_ast(
                    d, source_map_id,
                )
            }).collect(),
            name: FieldName::new(ast_field.name.value.as_ref()),
            span: ast_helpers::span_from_ast(
                ast_field.span,
                source_map_id,
            ),
            type_annotation: ast_helpers::type_annotation_from_ast(
                &ast_field.value_type,
                source_map_id,
            ),
        }
    }
}
