//! Centralized GraphQL specification (September 2025 edition)
//! URLs attached to build-level errors as
//! [`ErrorNote::spec`](crate::error_note::ErrorNote::spec) notes.
//!
//! Every error-construction site also carries an inline `//`
//! comment with the same URL directly above it (per project
//! convention). Rules that are enforced from multiple sites
//! reference these shared constants so the note values cannot
//! drift apart.

/// Grammar rule for enum values: a `Name`, but not `true`,
/// `false`, or `null`.
///
/// <https://spec.graphql.org/September2025/#sec-Enum-Value>
pub(crate) const ENUM_VALUE: &str =
    "https://spec.graphql.org/September2025/#sec-Enum-Value";

/// Enum type validation rules (e.g. "An Enum type must define
/// one or more unique enum values").
///
/// <https://spec.graphql.org/September2025/#sec-Enums.Type-Validation>
pub(crate) const ENUMS_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/#sec-Enums.Type-Validation";

/// Input object type validation rules (unique input field
/// names, one or more input fields, etc).
///
/// <https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation>
pub(crate) const INPUT_OBJECTS_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation";

/// Interface type validation rules (unique field names, one or
/// more fields, no self-implementation, etc).
///
/// <https://spec.graphql.org/September2025/#sec-Interfaces.Type-Validation>
pub(crate) const INTERFACES_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/#sec-Interfaces.Type-Validation";

/// Object type validation rules (unique field names, one or
/// more fields, unique argument names, unique `implements`
/// declarations, etc).
///
/// <https://spec.graphql.org/September2025/#sec-Objects.Type-Validation>
pub(crate) const OBJECTS_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/#sec-Objects.Type-Validation";

/// Reserved names rule: names must not begin with `__` (two
/// underscores) outside of the introspection system.
///
/// <https://spec.graphql.org/September2025/#sec-Names.Reserved-Names>
pub(crate) const RESERVED_NAMES: &str =
    "https://spec.graphql.org/September2025/#sec-Names.Reserved-Names";

/// Root operation type rules ("The query root operation type
/// must be provided and must be an Object type", etc).
///
/// <https://spec.graphql.org/September2025/#sec-Root-Operation-Types>
pub(crate) const ROOT_OPERATION_TYPES: &str =
    "https://spec.graphql.org/September2025/#sec-Root-Operation-Types";

/// Schema-level validity rules ("All types within a GraphQL
/// schema must have unique names", "All directives within a
/// GraphQL schema must have unique names", etc).
///
/// <https://spec.graphql.org/September2025/#sec-Schema>
pub(crate) const SCHEMA: &str =
    "https://spec.graphql.org/September2025/#sec-Schema";

/// Directive definition type validation rules (unique argument
/// names, no `__`-prefixed names, etc).
///
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives.Type-Validation>
pub(crate) const TYPE_SYSTEM_DIRECTIVES_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/\
    #sec-Type-System.Directives.Type-Validation";

/// Union type validation rules (one or more unique member
/// types, etc).
///
/// <https://spec.graphql.org/September2025/#sec-Unions.Type-Validation>
pub(crate) const UNIONS_TYPE_VALIDATION: &str =
    "https://spec.graphql.org/September2025/#sec-Unions.Type-Validation";
