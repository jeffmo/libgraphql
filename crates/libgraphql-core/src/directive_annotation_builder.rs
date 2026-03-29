use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::NamedDirectiveRef;
use crate::Value;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct DirectiveAnnotationBuilder;
impl DirectiveAnnotationBuilder {
    pub fn from_ast(
        annotated_item_srcloc: &loc::SourceLocation,
        source_map: &ast::SourceMap<'_>,
        directives: &[ast::DirectiveAnnotation<'_>],
    ) -> Vec<DirectiveAnnotation> {
        directives.iter().map(|ast_annot| {
            let annot_srcloc =
                annotated_item_srcloc.with_span(ast_annot.span, source_map);
            let mut arguments = IndexMap::new();
            for arg in ast_annot.arguments.iter() {
                arguments.insert(
                    arg.name.value.to_string(),
                    Value::from_ast(&arg.value, &annot_srcloc),
                );
            }
            DirectiveAnnotation {
                arguments,
                directive_ref: NamedDirectiveRef::new(
                    ast_annot.name.value.as_ref(),
                    annot_srcloc,
                ),
            }
        }).collect()
    }
}
