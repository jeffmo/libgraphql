use crate::names::DirectiveName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::parameter_def_builder::ParameterDefBuilder;
use crate::types::DirectiveLocationKind;
use libgraphql_parser::ast;

/// Builder for constructing a
/// [`DirectiveDefinition`](crate::types::DirectiveDefinition).
///
/// See [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
#[derive(Debug)]
#[allow(dead_code)]
pub struct DirectiveBuilder {
    pub(crate) description: Option<String>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) is_repeatable: bool,
    pub(crate) locations: Vec<DirectiveLocationKind>,
    pub(crate) name: DirectiveName,
    pub(crate) parameters: Vec<ParameterDefBuilder>,
    pub(crate) span: Span,
}

#[allow(clippy::result_large_err)]
impl DirectiveBuilder {
    /// Creates a new builder. Returns `Err` if `name` starts with
    /// `__` (reserved prefix per the GraphQL spec).
    pub fn new(
        name: impl Into<DirectiveName>,
        span: Span,
    ) -> Result<Self, SchemaBuildError> {
        let name = name.into();
        if name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedDirectiveName {
                    name: name.to_string(),
                },
                span,
                vec![],
            ));
        }
        Ok(Self {
            description: None,
            errors: vec![],
            is_repeatable: false,
            locations: vec![],
            name,
            parameters: vec![],
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

    /// Sets whether this directive is repeatable.
    pub fn set_repeatable(
        &mut self,
        repeatable: bool,
    ) -> &mut Self {
        self.is_repeatable = repeatable;
        self
    }

    /// Appends a valid directive location.
    pub fn add_location(
        &mut self,
        location: DirectiveLocationKind,
    ) -> &mut Self {
        self.locations.push(location);
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
                    field_name: format!("@{}", self.name),
                    param_name: param.name.to_string(),
                    type_name: None,
                },
                param.span,
                vec![],
            ));
        }
        if self.parameters.iter().any(|p| p.name == param.name) {
            // https://spec.graphql.org/September2025/#sec-Type-System.Directives.Type-Validation
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateParameterDefinition {
                    field_name: format!("@{}", self.name),
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
        ast_dir: &ast::DirectiveDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_dir.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_dir.description,
            ),
            errors: vec![],
            is_repeatable: ast_dir.repeatable,
            locations: ast_dir.locations.iter()
                .map(|loc| loc.kind)
                .collect(),
            name: DirectiveName::new(ast_dir.name.value.as_ref()),
            parameters: vec![],
            span,
        };
        if builder.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            builder.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedDirectiveName {
                    name: builder.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        for param in &ast_dir.arguments {
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
