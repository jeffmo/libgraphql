use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;

#[test]
fn test_multiple_definitions() {
    let schema = r#"
        scalar DateTime

        type User {
            id: ID!
            createdAt: DateTime
        }

        type Query {
            user(id: ID!): User
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 3);
    assert_eq!(ref_ast.definitions.len(), 3);
}

#[test]
fn test_complex_schema() {
    let schema = r#"
        interface Node {
            id: ID!
        }

        type User implements Node {
            id: ID!
            name: String!
            email: String!
            posts: [Post!]!
        }

        type Post implements Node {
            id: ID!
            title: String!
            content: String
            author: User!
        }

        union SearchResult = User | Post

        enum Role {
            ADMIN
            USER
            GUEST
        }

        input CreatePostInput {
            title: String!
            content: String
        }

        type Query {
            node(id: ID!): Node
            search(query: String!): [SearchResult!]!
        }

        type Mutation {
            createPost(input: CreatePostInput!): Post
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 8);
    assert_eq!(ref_ast.definitions.len(), 8);
}
