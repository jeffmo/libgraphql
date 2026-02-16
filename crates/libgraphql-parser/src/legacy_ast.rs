#[allow(dead_code)]
pub mod operation {
    pub type Definition = graphql_parser::query::Definition<'static, String>;
    pub type Directive = graphql_parser::query::Directive<'static, String>;
    pub type Document = graphql_parser::query::Document<'static, String>;
    pub type Field = graphql_parser::query::Field<'static, String>;
    pub type FragmentDefinition = graphql_parser::query::FragmentDefinition<'static, String>;
    pub type FragmentSpread = graphql_parser::query::FragmentSpread<'static, String>;
    pub type InlineFragment = graphql_parser::query::InlineFragment<'static, String>;
    pub type Mutation = graphql_parser::query::Mutation<'static, String>;
    pub type OperationDefinition = graphql_parser::query::OperationDefinition<'static, String>;
    pub type Query = graphql_parser::query::Query<'static, String>;
    pub type Selection = graphql_parser::query::Selection<'static, String>;
    pub type SelectionSet = graphql_parser::query::SelectionSet<'static, String>;
    pub type Subscription = graphql_parser::query::Subscription<'static, String>;
    pub type Type = graphql_parser::query::Type<'static, String>;
    pub type TypeCondition = graphql_parser::query::TypeCondition<'static, String>;
    pub type VariableDefinition = graphql_parser::query::VariableDefinition<'static, String>;

    pub type ParseError = graphql_parser::query::ParseError;
    pub fn parse(query_src: &str) -> Result<Document, ParseError> {
        Ok(graphql_parser::query::parse_query::<String>(query_src)?.into_static())
    }
}

#[allow(dead_code)]
pub mod schema {
    pub type Definition = graphql_parser::schema::Definition<'static, String>;
    pub type Directive = graphql_parser::query::Directive<'static, String>;
    pub type DirectiveDefinition = graphql_parser::schema::DirectiveDefinition<'static, String>;
    pub type DirectiveLocation = graphql_parser::schema::DirectiveLocation;
    pub type Document = graphql_parser::schema::Document<'static, String>;
    pub type EnumType = graphql_parser::schema::EnumType<'static, String>;
    pub type EnumTypeExtension = graphql_parser::schema::EnumTypeExtension<'static, String>;
    pub type EnumValue = graphql_parser::schema::EnumValue<'static, String>;
    pub type Field = graphql_parser::schema::Field<'static, String>;
    pub type InputObjectType = graphql_parser::schema::InputObjectType<'static, String>;
    pub type InputObjectTypeExtension =
        graphql_parser::schema::InputObjectTypeExtension<'static, String>;
    pub type InputValue = graphql_parser::schema::InputValue<'static, String>;
    pub type InterfaceType = graphql_parser::schema::InterfaceType<'static, String>;
    pub type InterfaceTypeExtension =
        graphql_parser::schema::InterfaceTypeExtension<'static, String>;
    pub type ObjectType = graphql_parser::schema::ObjectType<'static, String>;
    pub type ObjectTypeExtension = graphql_parser::schema::ObjectTypeExtension<'static, String>;
    pub type ScalarType = graphql_parser::schema::ScalarType<'static, String>;
    pub type ScalarTypeExtension = graphql_parser::schema::ScalarTypeExtension<'static, String>;
    pub type SchemaDefinition = graphql_parser::schema::SchemaDefinition<'static, String>;
    pub type Type = graphql_parser::schema::Type<'static, String>;
    pub type TypeDefinition = graphql_parser::schema::TypeDefinition<'static, String>;
    pub type TypeExtension = graphql_parser::schema::TypeExtension<'static, String>;
    pub type UnionType = graphql_parser::schema::UnionType<'static, String>;
    pub type UnionTypeExtension = graphql_parser::schema::UnionTypeExtension<'static, String>;

    pub type ParseError = graphql_parser::schema::ParseError;

    pub fn parse(schema_src: &str) -> Result<Document, ParseError> {
        Ok(graphql_parser::schema::parse_schema::<String>(schema_src)?.into_static())
    }
}

pub type AstPos = graphql_parser::Pos;
pub type Number = graphql_parser::query::Number;
pub type Value = graphql_parser::query::Value<'static, String>;

// =============================================================================
// Mixed document types (for documents containing both schema and executable
// definitions)
// =============================================================================

/// A definition that can be either a schema definition or an executable
/// definition.
///
/// This enum allows representing GraphQL documents that contain both type
/// system definitions (types, directives, etc.) and executable definitions
/// (operations, fragments) while preserving their relative ordering.
#[derive(Clone, Debug, PartialEq)]
pub enum MixedDefinition {
    /// A schema definition (type, directive, schema, extension).
    Schema(schema::Definition),

    /// An executable definition (operation or fragment).
    Executable(operation::Definition),
}

/// A document that contains both schema and executable definitions,
/// preserving their original order.
///
/// This is useful for tools that process complete GraphQL codebases where
/// schema definitions and operations may be interleaved. Unlike separate
/// schema/executable document types, `MixedDocument` preserves the exact
/// ordering of definitions as they appeared in the source.
///
/// # Example
///
/// A file containing interleaved type definitions and queries:
/// ```graphql
/// type User {
///     id: ID!
///     name: String
/// }
///
/// query GetUser($id: ID!) {
///     user(id: $id) { name }
/// }
///
/// type Post {
///     id: ID!
///     author: User!
/// }
/// ```
///
/// The `definitions` vector will contain `[User type, GetUser query, Post
/// type]` in that exact order.
#[derive(Clone, Debug, PartialEq)]
pub struct MixedDocument {
    /// All definitions in the document, in their original source order.
    pub definitions: Vec<MixedDefinition>,
}

pub mod serde_adapters {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(remote = "crate::legacy_ast::Number")]
    pub struct SerdeNumber(#[serde(getter = "SerdeNumber::as_i32")] pub(crate) i32);

    impl SerdeNumber {
        fn as_i32(num: &crate::legacy_ast::Number) -> i32 {
            num.as_i64().unwrap() as i32
        }
    }

    impl std::convert::From<SerdeNumber> for crate::legacy_ast::Number {
        fn from(value: SerdeNumber) -> Self {
            value.0.into()
        }
    }
}
