use crate::types::GraphQLTypeKind;

#[test]
pub fn basic_functionality() {
    use crate as libgraphql;
    let schema = libgraphql::macros::graphql_schema! {
        type Query {
          me: User
        }

        type User {
          firstName: String
          lastName: String,
        }
    };

    let query_type = schema.query_type();
    assert_eq!(query_type.name(), "Query");

    let user_type = schema.defined_types().get("User").unwrap();
    assert_eq!(user_type.name(), "User");
    assert_eq!(user_type.type_kind(), GraphQLTypeKind::Object);
}
