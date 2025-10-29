use crate::macros::graphql_schema_from_str;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLTypeKind;

#[test]
pub fn basic_functionality() -> Result<(), SchemaBuildError> {
    use crate as libgraphql;
    let schema = graphql_schema_from_str!(r#"
        type Query {
          me: User,
        }

        type User {
          firstName: String,
          lastName: String,
        }
    "#);

    let query_type = schema.query_type();
    assert_eq!(query_type.name(), "Query");

    let user_type = schema.defined_types().get("User").unwrap();
    assert_eq!(user_type.name(), "User");
    assert_eq!(user_type.type_kind(), GraphQLTypeKind::Object);

    Ok(())
}
