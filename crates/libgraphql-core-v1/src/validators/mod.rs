/// Type-system validators for cross-type validation.
///
/// These validators are designed to be called from
/// `SchemaBuilder::build()` (Task 16) to enforce the GraphQL
/// specification's type-system rules. They are not yet wired
/// into the build pipeline — `build()` is currently `todo!()`.
mod directive_definition_validator;
mod edit_distance;
mod input_object_type_validator;
mod object_or_interface_type_validator;
mod union_type_validator;

pub(crate) use crate::validators::directive_definition_validator::validate_directive_definitions;
pub(crate) use crate::validators::input_object_type_validator::InputObjectTypeValidator;
pub(crate) use crate::validators::object_or_interface_type_validator::ObjectOrInterfaceTypeValidator;
pub(crate) use crate::validators::union_type_validator::UnionTypeValidator;

#[cfg(test)]
mod tests;
