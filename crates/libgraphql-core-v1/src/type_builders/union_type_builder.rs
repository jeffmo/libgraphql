use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use libgraphql_parser::ast;

/// Builder for constructing a
/// [`UnionType`](crate::types::UnionType).
///
/// See [Unions](https://spec.graphql.org/September2025/#sec-Unions).
#[derive(Debug)]
#[allow(dead_code)]
pub struct UnionTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) members: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

#[allow(clippy::result_large_err)]
impl UnionTypeBuilder {
    /// Creates a new builder. Returns `Err` if `name` starts with
    /// `__` (reserved prefix per the GraphQL spec).
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
            members: vec![],
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

    /// Appends a union member type. Returns `Err` on duplicate.
    pub fn add_member(
        &mut self,
        member: impl Into<TypeName>,
        span: Span,
    ) -> Result<&mut Self, SchemaBuildError> {
        let member = member.into();
        if self.members.iter().any(|m| m.value == member) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateUnionMember {
                    member_name: member.to_string(),
                    type_name: self.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        self.members.push(Located { value: member, span });
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
        ast_union: &ast::UnionTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_union.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_union.description,
            ),
            directives: vec![],
            errors: vec![],
            members: vec![],
            name: TypeName::new(ast_union.name.value.as_ref()),
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
        for member in &ast_union.members {
            let member_span = ast_helpers::span_from_ast(
                member.span, source_map_id,
            );
            if let Err(e) = builder.add_member(
                member.value.as_ref(), member_span,
            ) {
                builder.errors.push(e);
            }
        }
        for dir in &ast_union.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        builder
    }
}
