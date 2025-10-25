use crate::graphql_token_stream::GraphQLTokenStream;
use crate::rust_to_graphql_token_adapter::{GraphQLToken, RustToGraphQLTokenAdapter};
use quote::quote;

#[test]
fn test_peek_without_consuming() {
    let input = quote! { type Query };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Peek multiple times should return same token
    let first_peek = stream.peek().map(|(t, _)| t.clone());
    let second_peek = stream.peek().map(|(t, _)| t.clone());

    assert_eq!(first_peek, second_peek);
    assert!(matches!(first_peek, Some(GraphQLToken::Name(ref name)) if name == "type"));

    // Now consume it
    let consumed = stream.next().map(|(t, _)| t);
    assert_eq!(first_peek, consumed);
}

#[test]
fn test_next_consumes_token() {
    let input = quote! { type Query };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Consume first token
    let first = stream.next().map(|(t, _)| t);
    assert!(matches!(first, Some(GraphQLToken::Name(name)) if name == "type"));

    // Next peek should be different token
    let second = stream.peek().map(|(t, _)| t.clone());
    assert!(matches!(second, Some(GraphQLToken::Name(name)) if name == "Query"));
}

#[test]
fn test_peek_nth_lookahead() {
    let input = quote! { type Query { field: String } };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Peek at different positions
    let token_0 = stream.peek_nth(0).map(|(t, _)| t.clone());
    let token_1 = stream.peek_nth(1).map(|(t, _)| t.clone());
    let token_2 = stream.peek_nth(2).map(|(t, _)| t.clone());

    assert!(matches!(token_0, Some(GraphQLToken::Name(ref name)) if name == "type"));
    assert!(matches!(token_1, Some(GraphQLToken::Name(ref name)) if name == "Query"));
    assert!(matches!(token_2, Some(GraphQLToken::Punctuator(ref p)) if p == "{"));

    // Consuming shouldn't affect what peek_nth saw
    stream.next();
    let new_token_0 = stream.peek_nth(0).map(|(t, _)| t.clone());
    assert_eq!(token_1, new_token_0);
}

#[test]
fn test_peek_nth_beyond_end() {
    let input = quote! { type };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Peek way beyond the stream
    let result = stream.peek_nth(100);
    assert!(result.is_none());
}

#[test]
fn test_check_name() {
    let input = quote! { type Query };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    assert!(stream.check_name("type"));
    assert!(!stream.check_name("Query"));
    assert!(!stream.check_name("interface"));

    // Consume and check next
    stream.next();
    assert!(stream.check_name("Query"));
    assert!(!stream.check_name("type"));
}

#[test]
fn test_check_punctuator() {
    let input = quote! { { } };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    assert!(stream.check_punctuator("{"));
    assert!(!stream.check_punctuator("}"));
    assert!(!stream.check_punctuator(":"));

    stream.next();
    assert!(stream.check_punctuator("}"));
    assert!(!stream.check_punctuator("{"));
}

#[test]
fn test_is_at_end() {
    let input = quote! { type };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    assert!(!stream.is_at_end());

    stream.next();
    assert!(stream.is_at_end());
}

#[test]
fn test_is_at_end_empty_stream() {
    let input = quote! {};
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    assert!(stream.is_at_end());
}

#[test]
fn test_current_span() {
    let input = quote! { type Query };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Before consuming any tokens, should return call_site
    let _initial_span = stream.current_span();

    // After consuming, should return the token's span
    let (_, expected_span) = stream.next().unwrap();
    let current = stream.current_span();

    // Spans should be equal (comparing debug representation since Span doesn't implement PartialEq)
    assert_eq!(format!("{expected_span:?}"), format!("{current:?}"));
}

#[test]
fn test_check() {
    let input = quote! { type };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    let type_token = GraphQLToken::Name("type".to_string());
    let query_token = GraphQLToken::Name("Query".to_string());

    assert!(stream.check(&type_token));
    assert!(!stream.check(&query_token));

    // Should not consume
    assert!(stream.check(&type_token));
}

#[test]
fn test_buffer_management() {
    let input = quote! { type Query { field: String } };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Force buffer to fill by peeking ahead
    stream.peek_nth(3);

    // Now consume tokens and verify they come in correct order
    let tokens: Vec<GraphQLToken> = std::iter::from_fn(|| stream.next().map(|(t, _)| t))
        .take(4)
        .collect();

    assert!(matches!(tokens[0], GraphQLToken::Name(ref n) if n == "type"));
    assert!(matches!(tokens[1], GraphQLToken::Name(ref n) if n == "Query"));
    assert!(matches!(tokens[2], GraphQLToken::Punctuator(ref p) if p == "{"));
    assert!(matches!(tokens[3], GraphQLToken::Name(ref n) if n == "field"));
}

#[test]
fn test_mixed_peek_and_consume() {
    let input = quote! { type Query };
    let adapter = RustToGraphQLTokenAdapter::new(input);
    let mut stream = GraphQLTokenStream::new(adapter);

    // Peek ahead
    let peeked = stream.peek_nth(1).map(|(t, _)| t.clone());

    // Consume first
    stream.next();

    // What we peeked should now be at position 0
    let now_first = stream.peek().map(|(t, _)| t.clone());
    assert_eq!(peeked, now_first);
}
