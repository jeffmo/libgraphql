use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use libgraphql_parser::ast;

/// Builder for constructing a
/// [`ScalarType`](crate::types::ScalarType).
///
/// See [Scalars](https://spec.graphql.org/September2025/#sec-Scalars).
#[derive(Debug)]
#[allow(dead_code)]
pub struct ScalarTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

#[allow(clippy::result_large_err)]
impl ScalarTypeBuilder {
    pub fn new(
        name: impl Into<TypeName>,
        span: Span,
    ) -> Result<Self, SchemaBuildError> {
        let name = name.into();
        if name.as_str().starts_with("__") {
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
        })
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
        ast_scalar: &ast::ScalarTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_scalar.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_scalar.description,
            ),
            directives: vec![],
            errors: vec![],
            name: TypeName::new(ast_scalar.name.value.as_ref()),
            span,
        };
        if builder.name.as_str().starts_with("__") {
            builder.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: builder.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        for dir in &ast_scalar.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        builder
    }
}
