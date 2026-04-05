pub(crate) mod graphql_name;

mod directive_name;
mod enum_value_name;
mod field_name;
mod fragment_name;
mod type_name;
mod variable_name;

pub use crate::names::directive_name::DirectiveName;
pub use crate::names::enum_value_name::EnumValueName;
pub use crate::names::field_name::FieldName;
pub use crate::names::fragment_name::FragmentName;
pub use crate::names::type_name::TypeName;
pub use crate::names::variable_name::VariableName;

#[cfg(test)]
mod tests;
