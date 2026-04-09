use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::TypeName;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::type_builders::ast_helpers;
use crate::type_builders::conversion_helpers::field_def_from_builder;
use crate::type_builders::field_def_builder::FieldDefBuilder;
use crate::type_builders::into_graphql_type::IntoGraphQLType;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::ObjectType;
use libgraphql_parser::ast;

/// Builder for constructing an
/// [`ObjectType`](crate::types::ObjectType).
///
/// See [Objects](https://spec.graphql.org/September2025/#sec-Objects).
#[derive(Debug)]
pub struct ObjectTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: Vec<FieldDefBuilder>,
    pub(crate) implements: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

// TODO: SchemaBuildError is large due to SchemaBuildErrorKind
// variants + Vec<ErrorNote>. Consider boxing the error or
// using an error index to reduce Result size.
#[allow(clippy::result_large_err)]
impl ObjectTypeBuilder {
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
            fields: vec![],
            implements: vec![],
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

    /// Appends a field. Returns `Err` on duplicate name or `__`
    /// prefix.
    pub fn add_field(
        &mut self,
        field: FieldDefBuilder,
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
            // https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
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

    /// Declares that this type implements an interface. Returns
    /// `Err` on duplicate interface.
    pub fn add_implements(
        &mut self,
        iface: impl Into<TypeName>,
        span: Span,
    ) -> Result<&mut Self, SchemaBuildError> {
        let iface = iface.into();
        if self.implements.iter().any(|l| l.value == iface) {
            // https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
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

    /// Appends an applied directive annotation.
    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }

    /// Constructs a builder from a parsed AST node. Returns
    /// `Err` with all collected validation errors if any are
    /// found during construction.
    pub(crate) fn from_ast(
        ast_obj: &ast::ObjectTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Result<Self, Vec<SchemaBuildError>> {
        let mut errors = vec![];
        let span = ast_helpers::span_from_ast(
            ast_obj.span, source_map_id,
        );
        let mut builder = Self {
            description: ast_helpers::description_from_ast(
                &ast_obj.description,
            ),
            directives: vec![],
            fields: vec![],
            implements: vec![],
            name: TypeName::new(ast_obj.name.value.as_ref()),
            span,
        };
        if builder.name.as_str().starts_with("__") {
            // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
            errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: builder.name.to_string(),
                },
                ast_helpers::span_from_ast(
                    ast_obj.name.span, source_map_id,
                ),
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
                errors.push(e);
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
            match FieldDefBuilder::from_ast(field, source_map_id) {
                Ok(field_builder) => {
                    if let Err(e) = builder.add_field(field_builder) {
                        errors.push(e);
                    }
                },
                Err(field_errors) => {
                    errors.extend(field_errors);
                },
            }
        }
        if errors.is_empty() {
            Ok(builder)
        } else {
            Err(errors)
        }
    }
}

impl IntoGraphQLType for ObjectTypeBuilder {
    fn type_name(&self) -> &TypeName { &self.name }
    fn type_span(&self) -> Span { self.span }

    fn into_graphql_type(self) -> GraphQLType {
        let type_name = self.name.clone();
        GraphQLType::Object(Box::new(ObjectType(
            FieldedTypeData {
                description: self.description,
                directives: self.directives,
                fields: self.fields.into_iter().map(|f| {
                    let field = field_def_from_builder(
                        f, &type_name,
                    );
                    (field.name.clone(), field)
                }).collect(),
                interfaces: self.implements,
                name: self.name,
                span: self.span,
            },
        )))
    }
}
