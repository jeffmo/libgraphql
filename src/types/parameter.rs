use crate::ast;
use crate::loc;
use crate::types::TypeAnnotation;
use crate::Value;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) default_value: Option<Value>,
    pub(super) name: String,
    pub(super) type_ref: TypeAnnotation,
}
impl Parameter {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn default_value(&self) -> &Option<Value> {
        &self.default_value
    }

    pub(crate) fn from_ast(
        file_path: &Path,
        input_val: &ast::schema::InputValue,
    ) -> Self {
        let input_val_pos = loc::FilePosition::from_pos(
            file_path,
            input_val.position,
        );

        Parameter {
            def_location: loc::SchemaDefLocation::Schema(input_val_pos.clone()),
            default_value: input_val.default_value.as_ref().map(
                |val| Value::from_ast(val, input_val_pos.clone())
            ),
            name: input_val.name.to_owned(),
            type_ref: TypeAnnotation::from_ast_type(
                &input_val_pos.into(),
                &input_val.value_type,
            ),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_ref
    }
}
