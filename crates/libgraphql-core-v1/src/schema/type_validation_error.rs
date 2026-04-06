use crate::error_note::ErrorNote;
use crate::span::Span;

/// A type-system validation error discovered during
/// [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build).
///
/// These errors represent violations of the
/// [Type System](https://spec.graphql.org/September2025/#sec-Type-System)
/// rules — interface contract mismatches, invalid type
/// references, circular input object chains, etc.
///
/// Wrapped by
/// [`SchemaBuildErrorKind::TypeValidation`](crate::schema::SchemaBuildErrorKind::TypeValidation)
/// when surfaced through
/// [`SchemaBuildError`](crate::schema::SchemaBuildError).
#[derive(Clone, Debug, PartialEq)]
pub struct TypeValidationError {
    kind: TypeValidationErrorKind,
    notes: Vec<ErrorNote>,
    span: Span,
}

impl TypeValidationError {
    pub(crate) fn new(
        kind: TypeValidationErrorKind,
        span: Span,
        notes: Vec<ErrorNote>,
    ) -> Self {
        Self { kind, notes, span }
    }

    pub fn kind(&self) -> &TypeValidationErrorKind { &self.kind }
    pub fn notes(&self) -> &[ErrorNote] { &self.notes }
    pub fn span(&self) -> Span { self.span }
}

impl std::fmt::Display for TypeValidationError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for TypeValidationError {}

/// Categorized validation error kind for programmatic matching.
///
/// `#[non_exhaustive]` — new variants may be added in minor
/// releases. Always include a wildcard arm in `match`
/// expressions.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum TypeValidationErrorKind {
    #[error(
        "circular non-nullable input field chain: {}",
        circular_field_path.join(" -> "),
    )]
    CircularInputFieldChain {
        circular_field_path: Vec<String>,
    },

    #[error(
        "`{type_name}` declares it implements \
        `{non_interface_type_name}`, but \
        `{non_interface_type_name}` is not an interface type"
    )]
    ImplementsNonInterfaceType {
        non_interface_type_name: String,
        type_name: String,
    },

    #[error(
        "`{type_name}` declares it implements \
        `{undefined_interface_name}`, but \
        `{undefined_interface_name}` is not defined"
    )]
    ImplementsUndefinedInterface {
        type_name: String,
        undefined_interface_name: String,
    },

    #[error(
        "input field `{parent_type_name}.{field_name}` has \
        type `{invalid_type_name}` which is not an input type"
    )]
    InvalidInputFieldWithOutputType {
        field_name: String,
        invalid_type_name: String,
        parent_type_name: String,
    },

    #[error(
        "`{type_name}.{field_name}` defines parameter \
        `{parameter_name}` with type `{actual_type}`, but \
        `{interface_name}.{field_name}` defines it as \
        `{expected_type}`"
    )]
    InvalidInterfaceSpecifiedFieldParameterType {
        actual_type: String,
        expected_type: String,
        field_name: String,
        interface_name: String,
        parameter_name: String,
        type_name: String,
    },

    #[error(
        "`{type_name}.{field_name}` has return type \
        `{actual_type}` which is not a valid subtype of \
        `{interface_name}.{field_name}` return type \
        `{expected_type}`"
    )]
    InvalidInterfaceSpecifiedFieldType {
        actual_type: String,
        expected_type: String,
        field_name: String,
        interface_name: String,
        type_name: String,
    },

    #[error(
        "output field `{parent_type_name}.{field_name}` has \
        type `{input_type_name}` which is not an output type"
    )]
    InvalidOutputFieldWithInputType {
        field_name: String,
        input_type_name: String,
        parent_type_name: String,
    },

    #[error(
        "parameter `{type_name}.{field_name}({parameter_name})` \
        has type `{invalid_type_name}` which is not an input type"
    )]
    InvalidParameterWithOutputOnlyType {
        field_name: String,
        invalid_type_name: String,
        parameter_name: String,
        type_name: String,
    },

    #[error(
        "additional parameter `{parameter_name}` on \
        `{type_name}.{field_name}` (not in \
        `{interface_name}.{field_name}`) must not be required"
    )]
    InvalidRequiredAdditionalParameterOnInterfaceSpecifiedField {
        field_name: String,
        interface_name: String,
        parameter_name: String,
        type_name: String,
    },

    #[error(
        "union member `{member_name}` on `{union_type_name}` \
        is not an object type"
    )]
    InvalidUnionMemberTypeKind {
        member_name: String,
        union_type_name: String,
    },

    #[error(
        "`{type_name}` implements `{interface_name}` but does \
        not define field `{field_name}`"
    )]
    MissingInterfaceSpecifiedField {
        field_name: String,
        interface_name: String,
        type_name: String,
    },

    #[error(
        "`{type_name}.{field_name}` is missing parameter \
        `{missing_parameter_name}` required by \
        `{interface_name}.{field_name}`"
    )]
    MissingInterfaceSpecifiedFieldParameter {
        field_name: String,
        interface_name: String,
        missing_parameter_name: String,
        type_name: String,
    },

    #[error(
        "`{type_name}` implements {}, therefore \
        `{type_name}` must also implement \
        `{missing_recursive_interface_name}`",
        inheritance_path.iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(" which implements "),
    )]
    MissingRecursiveInterfaceImplementation {
        inheritance_path: Vec<String>,
        missing_recursive_interface_name: String,
        type_name: String,
    },

    #[error(
        "type `{undefined_type_name}` is referenced but not \
        defined"
    )]
    UndefinedTypeName {
        undefined_type_name: String,
    },
}
