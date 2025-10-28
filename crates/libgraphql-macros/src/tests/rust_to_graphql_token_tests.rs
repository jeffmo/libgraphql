use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use std::str::FromStr;

#[test]
fn test_triple_quoted_string_tokenization() {
    let schema = r#""""A description"""
scalar DateTime"#;

    // First check raw TokenStream
    let token_stream = proc_macro2::TokenStream::from_str(schema).unwrap();
    eprintln!("\n=== Raw proc_macro2::TokenStream ===");
    for (i, token) in token_stream.clone().into_iter().enumerate() {
        eprintln!("{}: {:?}", i, token);
    }

    // Now check our adapter
    eprintln!("\n=== GraphQL Tokens from Adapter ===");
    let adapter = RustToGraphQLTokenAdapter::new(token_stream);
    for (i, (token, _span)) in adapter.enumerate() {
        eprintln!("{}: {:?}", i, token);
    }
}
