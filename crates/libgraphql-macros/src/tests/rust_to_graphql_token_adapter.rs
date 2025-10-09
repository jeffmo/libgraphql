use crate::rust_to_graphql_token_adapter::GraphQLToken;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use quote::quote;

#[test]
fn test_simple_type_definition() {
    let input = quote! {
        type Query {
            hello: String
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let tokens: Vec<_> = adapter.map(|(tok, _)| tok).collect();

    assert_eq!(tokens[0], GraphQLToken::Name("type".to_string()));
    assert_eq!(tokens[1], GraphQLToken::Name("Query".to_string()));
    assert_eq!(tokens[2], GraphQLToken::Punctuator("{".to_string()));
    assert_eq!(tokens[3], GraphQLToken::Name("hello".to_string()));
    assert_eq!(tokens[4], GraphQLToken::Punctuator(":".to_string()));
    assert_eq!(tokens[5], GraphQLToken::Name("String".to_string()));
    assert_eq!(tokens[6], GraphQLToken::Punctuator("}".to_string()));
}

#[test]
fn test_punctuators() {
    let input = quote! {
        field(arg: Int!): [String]!
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let tokens: Vec<_> = adapter.map(|(tok, _)| tok).collect();

    // Check for key punctuators
    assert!(tokens.contains(&GraphQLToken::Punctuator("(".to_string())));
    assert!(tokens.contains(&GraphQLToken::Punctuator(")".to_string())));
    assert!(tokens.contains(&GraphQLToken::Punctuator(":".to_string())));
    assert!(tokens.contains(&GraphQLToken::Punctuator("!".to_string())));
    assert!(tokens.contains(&GraphQLToken::Punctuator("[".to_string())));
    assert!(tokens.contains(&GraphQLToken::Punctuator("]".to_string())));
}

#[test]
fn test_string_literals() {
    let input = quote! {
        description: "A test string"
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let tokens: Vec<_> = adapter.map(|(tok, _)| tok).collect();

    assert!(tokens.contains(&GraphQLToken::StringValue("A test string".to_string())));
}

#[test]
fn test_numeric_literals() {
    let input = quote! {
        int: 42 float: 2.5
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let tokens: Vec<_> = adapter.map(|(tok, _)| tok).collect();

    assert!(tokens.contains(&GraphQLToken::IntValue(42)));
    assert!(tokens.contains(&GraphQLToken::FloatValue(2.5)));
}

#[test]
fn test_lazy_iteration() {
    let input = quote! {
        type Query {
            %% invalid_token_that_would_cause_error
            field: String
        }
    };

    let mut adapter = RustToGraphQLTokenAdapter::new(input);

    // Verify we can consume lazily - only consume first 3 tokens
    assert_eq!(adapter.next().unwrap().0, GraphQLToken::Name("type".to_string()));
    assert_eq!(adapter.next().unwrap().0, GraphQLToken::Name("Query".to_string()));
    assert_eq!(adapter.next().unwrap().0, GraphQLToken::Punctuator("{".to_string()));

    // Don't consume the rest - the invalid "%%" tokens are never processed
    // This demonstrates that we're truly lazy and don't process tokens until needed
}
