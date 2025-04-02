use crate::ast;
use crate::loc;
use crate::Value;
use crate::types::NamedDirectiveRef;
use std::collections::BTreeMap;
use std::path::Path;

/// Represents a Directive annotation. Essentially a wrapper around
/// [NamedDirectiveRef], but includes an argument list.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation {
    pub(crate) args: BTreeMap<String, Value>,
    pub(crate) directive_ref: NamedDirectiveRef,
}
impl DirectiveAnnotation {
    pub fn from_ast<P: AsRef<Path>>(
        file_path: P,
        ast_annots: &[ast::operation::Directive],
    ) -> Vec<Self> {
        let file_path = file_path.as_ref();
        let mut annots = vec![];
        for ast_annot in ast_annots {
            let mut args = BTreeMap::new();
            for (arg_name, arg_val) in ast_annot.arguments.iter() {
                args.insert(arg_name.to_string(), Value::from_ast(
                    &arg_val,
                    loc::FilePosition::from_pos(
                        file_path,
                        ast_annot.position,
                    ),
                ));
            }

            annots.push(DirectiveAnnotation {
                args,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    loc::FilePosition::from_pos(
                        file_path,
                        ast_annot.position,
                    ),
                ),
            });
        }
        annots
    }
}

