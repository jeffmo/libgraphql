use crate::ast;
use std::collections::HashSet;
use std::path::PathBuf;

lazy_static::lazy_static! {
    pub static ref BUILTIN_DIRECTIVE_NAMES: HashSet<&'static str> = {
        HashSet::from([
            "skip",
            "include",
            "deprecated",
            "specifiedBy",
        ])
    };
}

#[derive(Debug)]
pub enum Directive {
    Custom {
        def_ast: ast::schema::DirectiveDefinition,
        def_location: ast::FileLocation,
        // TODO: parameters
    },
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}
impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Custom { def_ast, .. } => def_ast.name.as_str(),
            Directive::Deprecated => "deprecated",
            Directive::Include => "include",
            Directive::Skip => "skip",
            Directive::SpecifiedBy => "specifiedBy",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirectiveReference {
    // TODO: arguments
    pub directive_name: String,
    pub location: ast::FileLocation,
}
impl DirectiveReference {
    pub fn from_ast(
        file_path: &PathBuf,
        ast: &ast::query::Directive,
    ) -> Self {
        DirectiveReference {
            directive_name: ast.name.to_string(),
            location: ast::FileLocation::from_pos(
                file_path.to_path_buf(),
                ast.position,
            ),
        }
    }
}
