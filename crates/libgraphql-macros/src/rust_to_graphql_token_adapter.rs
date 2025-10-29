use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use std::iter::Peekable;

/// Represents a GraphQL lexical token based on the GraphQL specification.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLToken {
    /// GraphQL punctuators: ! $ & ( ) ... : = @ [ ] { | }
    Punctuator(String),
    /// GraphQL Name (identifier)
    Name(String),
    /// GraphQL IntValue
    IntValue(i64),
    /// GraphQL FloatValue
    FloatValue(f64),
    /// GraphQL StringValue
    StringValue(String),
}

/// Adapts Rust `TokenStream` into GraphQL tokens with span information.
///
/// This adapter consumes a Rust token stream (from a procedural macro invocation)
/// and produces an iterator of GraphQL tokens, preserving span information for
/// accurate error reporting.
pub struct RustToGraphQLTokenAdapter {
    tokens: Peekable<proc_macro2::token_stream::IntoIter>,
    pending: Vec<(GraphQLToken, Span)>,
}

impl RustToGraphQLTokenAdapter {
    pub fn new(input: TokenStream) -> Self {
        Self {
            tokens: input.into_iter().peekable(),
            pending: Vec::new(),
        }
    }

    fn process_token_tree(&mut self, tree: TokenTree) {
        match tree {
            TokenTree::Group(group) => {
                let span = group.span();
                match group.delimiter() {
                    Delimiter::Brace => {
                        self.pending.push((GraphQLToken::Punctuator("{".to_string()), span));
                        // Process contents
                        for inner in group.stream() {
                            self.process_token_tree(inner);
                        }
                        self.pending.push((GraphQLToken::Punctuator("}".to_string()), span));
                    }
                    Delimiter::Bracket => {
                        self.pending.push((GraphQLToken::Punctuator("[".to_string()), span));
                        for inner in group.stream() {
                            self.process_token_tree(inner);
                        }
                        self.pending.push((GraphQLToken::Punctuator("]".to_string()), span));
                    }
                    Delimiter::Parenthesis => {
                        self.pending.push((GraphQLToken::Punctuator("(".to_string()), span));
                        for inner in group.stream() {
                            self.process_token_tree(inner);
                        }
                        self.pending.push((GraphQLToken::Punctuator(")".to_string()), span));
                    }
                    Delimiter::None => {
                        // Process contents without delimiters
                        for inner in group.stream() {
                            self.process_token_tree(inner);
                        }
                    }
                }
            }

            TokenTree::Ident(ident) => {
                let span = ident.span();
                let name = ident.to_string();
                self.pending.push((GraphQLToken::Name(name), span));
            }

            TokenTree::Punct(punct) => {
                let span = punct.span();
                let ch = punct.as_char();

                // Handle multi-character punctuators
                match ch {
                    '.' => {
                        // Check if this is the start of "..."
                        if self.pending.len() >= 2 {
                            let len = self.pending.len();
                            if let Some((GraphQLToken::Punctuator(p1), _)) = self.pending.get(len - 1)
                                && let Some((GraphQLToken::Punctuator(p2), _)) = self.pending.get(len - 2)
                                && p1 == "." && p2 == "."
                            {
                                // Replace the last two dots with "..."
                                self.pending.pop();
                                self.pending.pop();
                                self.pending.push((GraphQLToken::Punctuator("...".to_string()), span));
                                return;
                            }
                        }
                        self.pending.push((GraphQLToken::Punctuator(".".to_string()), span));
                    }
                    '!' | '$' | '&' | ':' | '=' | '@' | '|' => {
                        self.pending.push((GraphQLToken::Punctuator(ch.to_string()), span));
                    }
                    _ => {
                        // Other punctuation - just emit as-is
                        self.pending.push((GraphQLToken::Punctuator(ch.to_string()), span));
                    }
                }
            }

            TokenTree::Literal(lit) => {
                let span = lit.span();
                let lit_str = lit.to_string();

                // Try to parse as integer
                if let Ok(int_val) = lit_str.parse::<i64>() {
                    self.pending.push((GraphQLToken::IntValue(int_val), span));
                    return;
                }

                // Try to parse as float
                if let Ok(float_val) = lit_str.parse::<f64>() {
                    self.pending.push((GraphQLToken::FloatValue(float_val), span));
                    return;
                }

                // Try to parse as string literal
                if lit_str.starts_with('"') && lit_str.ends_with('"') {
                    // Remove quotes and unescape
                    let string_content = &lit_str[1..lit_str.len() - 1];
                    self.pending.push((GraphQLToken::StringValue(string_content.to_string()), span));
                    return;
                }

                // Fallback: treat as name
                self.pending.push((GraphQLToken::Name(lit_str), span));
            }
        }
    }
}

impl Iterator for RustToGraphQLTokenAdapter {
    type Item = (GraphQLToken, Span);

    fn next(&mut self) -> Option<Self::Item> {
        // Process tokens until we have at least 3 pending to check for triple-quoted strings
        while self.pending.len() < 3 && self.tokens.peek().is_some() {
            if let Some(tree) = self.tokens.next() {
                self.process_token_tree(tree);
            }
        }

        // Check for triple-quoted string pattern: "" + "content" + ""
        if self.pending.len() >= 3
            && let (
                Some((GraphQLToken::StringValue(s1), _)),
                Some((GraphQLToken::StringValue(s2), span2)),
                Some((GraphQLToken::StringValue(s3), _)),
            ) = (
                self.pending.first(),
                self.pending.get(1),
                self.pending.get(2),
            )
            && s1.is_empty() && s3.is_empty() && !s2.is_empty()
        {
            // This is a triple-quoted string (description)
            // Remove all three tokens and return the middle one
            let description = s2.clone();
            let span = *span2;
            self.pending.drain(0..3);
            return Some((GraphQLToken::StringValue(description), span));
        }

        // Return the next pending token
        if !self.pending.is_empty() {
            return Some(self.pending.remove(0));
        }

        None
    }
}
