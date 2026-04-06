use crate::directive_annotation::DirectiveAnnotation;
use crate::names::DirectiveName;
use crate::names::EnumValueName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::names::VariableName;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::types::ListTypeAnnotation;
use crate::types::NamedTypeAnnotation;
use crate::types::TypeAnnotation;
use crate::value::Value;
use libgraphql_parser::ByteSpan;
use libgraphql_parser::ast;

pub(crate) fn span_from_ast(
    byte_span: ByteSpan,
    source_map_id: SourceMapId,
) -> Span {
    Span::new(byte_span, source_map_id)
}

pub(crate) fn type_annotation_from_ast(
    ast_annot: &ast::TypeAnnotation<'_>,
    source_map_id: SourceMapId,
) -> TypeAnnotation {
    match ast_annot {
        ast::TypeAnnotation::Named(named) => {
            TypeAnnotation::Named(NamedTypeAnnotation {
                nullable: named.nullable(),
                span: span_from_ast(named.span, source_map_id),
                type_name: TypeName::new(named.name.value.as_ref()),
            })
        },
        ast::TypeAnnotation::List(list) => {
            TypeAnnotation::List(ListTypeAnnotation {
                inner: Box::new(type_annotation_from_ast(
                    &list.element_type,
                    source_map_id,
                )),
                nullable: list.nullable(),
                span: span_from_ast(list.span, source_map_id),
            })
        },
    }
}

pub(crate) fn value_from_ast(ast_val: &ast::Value<'_>) -> Value {
    match ast_val {
        ast::Value::Boolean(v) => Value::Boolean(v.value),
        ast::Value::Enum(v) => {
            Value::Enum(EnumValueName::new(v.value.as_ref()))
        },
        ast::Value::Float(v) => Value::Float(v.value),
        ast::Value::Int(v) => Value::Int(v.value),
        ast::Value::List(v) => {
            Value::List(v.values.iter().map(value_from_ast).collect())
        },
        ast::Value::Null(_) => Value::Null,
        ast::Value::Object(v) => Value::Object(
            v.fields.iter().map(|f| {
                (
                    FieldName::new(f.name.value.as_ref()),
                    value_from_ast(&f.value),
                )
            }).collect(),
        ),
        ast::Value::String(v) => {
            Value::String(v.value.to_string())
        },
        ast::Value::Variable(v) => {
            Value::VarRef(VariableName::new(v.name.value.as_ref()))
        },
    }
}

pub(crate) fn directive_annotation_from_ast(
    ast_dir: &ast::DirectiveAnnotation<'_>,
    source_map_id: SourceMapId,
) -> DirectiveAnnotation {
    DirectiveAnnotation {
        arguments: ast_dir.arguments.iter().map(|arg| {
            (
                FieldName::new(arg.name.value.as_ref()),
                value_from_ast(&arg.value),
            )
        }).collect(),
        name: DirectiveName::new(ast_dir.name.value.as_ref()),
        span: span_from_ast(ast_dir.span, source_map_id),
    }
}

pub(crate) fn description_from_ast(
    desc: &Option<ast::StringValue<'_>>,
) -> Option<String> {
    desc.as_ref().map(|d| d.value.to_string())
}
