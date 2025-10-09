use crate::graphql_token_parser::GraphQLTokenParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use libgraphql_core::ast;
use quote::quote;

#[test]
fn test_parse_simple_type() {
    let input = quote! {
        type Query {
            hello: String
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(obj)) => {
            assert_eq!(obj.name, "Query");
            assert_eq!(obj.fields.len(), 1);
            assert_eq!(obj.fields[0].name, "hello");
            match &obj.fields[0].field_type {
                ast::schema::Type::NamedType(name) => {
                    assert_eq!(name, "String");
                }
                unexpected => panic!(
                    "Expected NamedType for field type but found `{unexpected:#?}`",
                ),
            }
        }
        unexpected => panic!("Expected ObjectType definition but found `{unexpected:#?}`"),
    }
}

#[test]
fn test_parse_scalar_type() {
    let input = quote! {
        scalar DateTime
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Scalar(scalar)) => {
            assert_eq!(scalar.name, "DateTime");
        }
        unexpected => panic!("Expected ScalarType definition but found `{unexpected:#?}`"),
    }
}

#[test]
fn test_parse_enum_type() {
    let input = quote! {
        enum Status {
            ACTIVE
            INACTIVE
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Enum(enum_type)) => {
            assert_eq!(enum_type.name, "Status");
            assert_eq!(enum_type.values.len(), 2);
            assert_eq!(enum_type.values[0].name, "ACTIVE");
            assert_eq!(enum_type.values[1].name, "INACTIVE");
        }
        unexpected => panic!("Expected EnumType definition but found `{unexpected:#?}`"),
    }
}

#[test]
fn test_parse_interface_type() {
    let input = quote! {
        interface Node {
            id: ID!
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Interface(iface)) => {
            assert_eq!(iface.name, "Node");
            assert_eq!(iface.fields.len(), 1);
            assert_eq!(iface.fields[0].name, "id");
            match &iface.fields[0].field_type {
                ast::schema::Type::NonNullType(inner) => {
                    match &**inner {
                        ast::schema::Type::NamedType(name) => {
                            assert_eq!(name, "ID");
                        }
                        unexpected => panic!(
                            "Expected NamedType inside NonNullType but found \
                            `{unexpected:#?}`"
                        ),
                    }
                }
                unexpected => panic!(
                    "Expected NonNullType for field type but found \
                    `{unexpected:#?}`"
                ),
            }
        }
        unexpected => panic!(
            "Expected InterfaceType definition but found `{unexpected:#?}`"
        ),
    }
}

#[test]
fn test_parse_union_type() {
    let input = quote! {
        union SearchResult = User | Post
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Union(union_type)) => {
            assert_eq!(union_type.name, "SearchResult");
            assert_eq!(union_type.types.len(), 2);
            assert_eq!(union_type.types[0], "User");
            assert_eq!(union_type.types[1], "Post");
        }
        unexpected => panic!(
            "Expected UnionType definition but found `{unexpected:#?}`"
        ),
    }
}

#[test]
fn test_parse_input_type() {
    let input = quote! {
        input CreateUserInput {
            name: String!
            email: String!
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 1);

    match &doc.definitions[0] {
        ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::InputObject(input_obj)) => {
            assert_eq!(input_obj.name, "CreateUserInput");
            assert_eq!(input_obj.fields.len(), 2);
            assert_eq!(input_obj.fields[0].name, "name");
            assert_eq!(input_obj.fields[1].name, "email");
        }
        unexpected => panic!(
            "Expected InputObjectType definition but found `{unexpected:#?}`"
        ),
    }
}

#[test]
fn test_parse_multiple_types() {
    let input = quote! {
        type Query {
            user(id: ID!): User
        }

        type User {
            id: ID!
            name: String!
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let doc = parser.parse_document().unwrap();

    assert_eq!(doc.definitions.len(), 2);
}

#[test]
fn test_parse_error_invalid_syntax() {
    let input = quote! {
        type Query {
            field: Invalid % Invalid
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLTokenParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_err());
}
