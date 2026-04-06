use crate::error_note::ErrorNote;
use crate::schema::type_validation_error::TypeValidationError;
use crate::span::Span;

/// An error encountered during schema construction.
///
/// Every error carries:
/// - A primary [`span`](Self::span) pointing to the source
///   location of the error
/// - A [`kind`](Self::kind) for programmatic matching
/// - Optional [`notes`](Self::notes) with secondary locations,
///   spec references, and hints
///
/// The primary span and notes (including secondary spans like
/// "first defined here") live on this struct, NOT on each kind
/// variant. Variants carry only identity data (names, kind
/// discriminants). Validators construct notes at error-creation
/// time when they have access to all relevant spans.
///
/// The `kind` enum is `#[non_exhaustive]` — new error variants
/// can be added in future versions without breaking downstream
/// `match` expressions.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaBuildError {
    kind: SchemaBuildErrorKind,
    notes: Vec<ErrorNote>,
    span: Span,
}

impl SchemaBuildError {
    pub(crate) fn new(
        kind: SchemaBuildErrorKind,
        span: Span,
        notes: Vec<ErrorNote>,
    ) -> Self {
        Self { kind, notes, span }
    }

    pub fn kind(&self) -> &SchemaBuildErrorKind { &self.kind }
    pub fn notes(&self) -> &[ErrorNote] { &self.notes }
    pub fn span(&self) -> Span { self.span }
}

impl std::fmt::Display for SchemaBuildError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for SchemaBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

/// Categorized error kind for programmatic matching.
///
/// `#[non_exhaustive]` — new variants may be added in minor
/// releases. Always include a wildcard arm in `match`
/// expressions.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaBuildErrorKind {
    #[error("duplicate directive definition `@{name}`")]
    DuplicateDirectiveDefinition {
        name: String,
    },

    #[error("duplicate enum value `{value_name}` on `{type_name}`")]
    DuplicateEnumValueDefinition {
        type_name: String,
        value_name: String,
    },

    #[error("duplicate field `{field_name}` on `{type_name}`")]
    DuplicateFieldNameDefinition {
        field_name: String,
        type_name: String,
    },

    #[error(
        "`{type_name}` declares that it implements \
        `{interface_name}` more than once"
    )]
    DuplicateInterfaceImplementsDeclaration {
        interface_name: String,
        type_name: String,
    },

    #[error(
        "duplicate {operation} root operation type definition \
        (already bound to `{type_name}`)"
    )]
    DuplicateOperationDefinition {
        operation: String,
        type_name: String,
    },

    #[error(
        "duplicate parameter `{param_name}` on \
        `{type_name}.{field_name}`"
    )]
    DuplicateParameterDefinition {
        field_name: String,
        param_name: String,
        type_name: String,
    },

    #[error("duplicate type definition `{type_name}`")]
    DuplicateTypeDefinition {
        type_name: String,
    },

    #[error("duplicate union member `{member_name}` on `{type_name}`")]
    DuplicateUnionMember {
        member_name: String,
        type_name: String,
    },

    #[error("{type_kind} type `{type_name}` has no fields")]
    EmptyObjectOrInterfaceType {
        type_kind: crate::types::GraphQLTypeKind,
        type_name: String,
    },

    #[error("union `{type_name}` has no members")]
    EmptyUnionType {
        type_name: String,
    },

    #[error("enum `{type_name}` defines no values")]
    EnumWithNoValues {
        type_name: String,
    },

    #[error("type extension for undefined type `{type_name}`")]
    ExtensionOfUndefinedType {
        type_name: String,
    },

    #[error("directive name `@{name}` must not start with `__`")]
    InvalidDunderPrefixedDirectiveName {
        name: String,
    },

    #[error(
        "field name `{field_name}` on `{type_name}` must not \
        start with `__`"
    )]
    InvalidDunderPrefixedFieldName {
        field_name: String,
        type_name: String,
    },

    #[error(
        "parameter name `{param_name}` on \
        `{type_name}.{field_name}` must not start with `__`"
    )]
    InvalidDunderPrefixedParamName {
        field_name: String,
        param_name: String,
        type_name: String,
    },

    #[error("type name `{type_name}` must not start with `__`")]
    InvalidDunderPrefixedTypeName {
        type_name: String,
    },

    #[error(
        "enum value `{value_name}` on `{type_name}` must not \
        be `true`, `false`, or `null`"
    )]
    InvalidEnumValueName {
        type_name: String,
        value_name: String,
    },

    #[error("type extension kind mismatch for `{type_name}`")]
    InvalidExtensionTypeKind {
        type_name: String,
    },

    #[error("`{interface_name}` must not implement itself")]
    InvalidSelfImplementingInterface {
        interface_name: String,
    },

    #[error("schema has no Query root operation type defined")]
    NoQueryOperationTypeDefined,

    #[error("error parsing schema string: {message}")]
    ParseError {
        message: String,
    },

    #[error("cannot redefine built-in directive `@{name}`")]
    RedefinitionOfBuiltinDirective {
        name: String,
    },

    #[error("root {operation} type `{type_name}` is not defined")]
    RootOperationTypeNotDefined {
        operation: String,
        type_name: String,
    },

    #[error(
        "root {operation} type `{type_name}` must be an \
        object type, found {actual_kind}"
    )]
    RootOperationTypeNotObjectType {
        actual_kind: crate::types::GraphQLTypeKind,
        operation: String,
        type_name: String,
    },

    #[error("{0}")]
    TypeValidation(TypeValidationError),
}
