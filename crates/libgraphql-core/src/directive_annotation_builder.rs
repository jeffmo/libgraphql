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
        directives: &[ast::operation::Directive],
    ) -> Vec<DirectiveAnnotation> {
        directives.iter().map(|ast_annot| {
            let annot_srcloc =
                annotated_item_srcloc.with_ast_position(&ast_annot.position);
            let mut arguments = IndexMap::new();
            for (arg_name, ast_arg) in ast_annot.arguments.iter() {
                arguments.insert(
                    arg_name.to_string(),
                    Value::from_ast(ast_arg, &annot_srcloc),
                );
            }
            DirectiveAnnotation {
                arguments,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    annot_srcloc,
                ),
            }
        }).collect()
    }
}
