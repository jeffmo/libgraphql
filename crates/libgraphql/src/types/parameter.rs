use crate::ast;
use crate::loc;
use crate::types::TypeAnnotation;
use crate::Value;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub(super) def_location: loc::SourceLocation,
    pub(super) default_value: Option<Value>,
    pub(super) name: String,
    pub(super) type_annotation: TypeAnnotation,
}
impl Parameter {
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    pub fn default_value(&self) -> &Option<Value> {
        &self.default_value
    }

    pub(crate) fn from_ast(
        file_path: Option<&Path>,
        param: &ast::schema::InputValue,
    ) -> Self {
        let paramdef_srcloc = loc::SourceLocation::from_schema_ast_position(
            file_path,
            &param.position,
        );

        Parameter {
            default_value: param.default_value.as_ref().map(
                |val| Value::from_ast(val, &paramdef_srcloc)
            ),
            name: param.name.to_owned(),
            type_annotation: TypeAnnotation::from_ast_type(
                &paramdef_srcloc,
                &param.value_type,
            ),
            def_location: paramdef_srcloc,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
