use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use libgraphql_parser::ast;

/// Builder-stage enum value data before validation.
#[derive(Debug)]
#[allow(dead_code)]
pub struct EnumValueDefBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: EnumValueName,
    pub(crate) span: Span,
}

impl EnumValueDefBuilder {
    pub fn new(
        name: impl Into<EnumValueName>,
        span: Span,
    ) -> Self {
        Self {
            description: None,
            directives: vec![],
            name: name.into(),
            span,
        }
    }

    pub fn set_description(
        &mut self,
        desc: impl Into<String>,
    ) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }

    pub(crate) fn from_ast(
        ast_val: &ast::EnumValueDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        Self {
            description: ast_helpers::description_from_ast(
                &ast_val.description,
            ),
            directives: ast_val.directives.iter().map(|d| {
                ast_helpers::directive_annotation_from_ast(
                    d, source_map_id,
                )
            }).collect(),
            name: EnumValueName::new(ast_val.name.value.as_ref()),
            span: ast_helpers::span_from_ast(
                ast_val.span,
                source_map_id,
            ),
        }
    }
}
