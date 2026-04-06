use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::field_def_builder::FieldDefBuilder;
use libgraphql_parser::ast;

/// Builder for constructing an
/// [`ObjectType`](crate::types::ObjectType).
///
/// See [Objects](https://spec.graphql.org/September2025/#sec-Objects).
#[derive(Debug)]
#[allow(dead_code)]
pub struct ObjectTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) fields: Vec<FieldDefBuilder>,
    pub(crate) implements: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

#[allow(clippy::result_large_err)]
impl ObjectTypeBuilder {
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
            fields: vec![],
            implements: vec![],
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

    pub fn add_field(
        &mut self,
        field: FieldDefBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        if field.name.as_str().starts_with("__") {
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

    pub fn add_implements(
        &mut self,
        iface: impl Into<TypeName>,
        span: Span,
    ) -> Result<&mut Self, SchemaBuildError> {
        let iface = iface.into();
        if self.implements.iter().any(|l| l.value == iface) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateInterfaceImplementsDeclaration {
                    interface_name: iface.to_string(),
                    type_name: self.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        self.implements.push(Located { value: iface, span });
        Ok(self)
    }

    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }

    pub(crate) fn from_ast(
        ast_obj: &ast::ObjectTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_obj.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_obj.description,
            ),
            directives: vec![],
            errors: vec![],
            fields: vec![],
            implements: vec![],
            name: TypeName::new(ast_obj.name.value.as_ref()),
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
        for iface in &ast_obj.implements {
            let iface_span = ast_helpers::span_from_ast(
                iface.span, source_map_id,
            );
            if let Err(e) = builder.add_implements(
                iface.value.as_ref(), iface_span,
            ) {
                builder.errors.push(e);
            }
        }
        for dir in &ast_obj.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        for field in &ast_obj.fields {
            let field_builder = FieldDefBuilder::from_ast(
                field, source_map_id,
            );
            if let Err(e) = builder.add_field(field_builder) {
                builder.errors.push(e);
            }
        }
        builder
    }
}
