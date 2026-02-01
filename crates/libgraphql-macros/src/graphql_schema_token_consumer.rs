use crate::emittable_schema::EmittableSchema;
use crate::parse_error_converter::convert_parse_errors_to_tokenstream;
use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use crate::span_map::SpanMap;
use libgraphql_parser::GraphQLParser;
use proc_macro2::Span;
use quote::quote;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct GraphQLSchemaTokenConsumer(proc_macro::TokenStream);
impl GraphQLSchemaTokenConsumer {
    pub fn new(input: proc_macro::TokenStream) -> Self {
        Self(input)
    }
}

impl std::convert::From<GraphQLSchemaTokenConsumer> for proc_macro::TokenStream {
    fn from(val: GraphQLSchemaTokenConsumer) -> Self {
        let input = proc_macro2::TokenStream::from(val.0);

        // Shared storage for (line, col) → Span mappings.
        // RustMacroGraphQLTokenSource populates this as the
        // parser pulls tokens; we read it afterward to map
        // error positions back to proc_macro2::Span.
        let span_map_storage: Rc<
            RefCell<HashMap<(usize, usize), Span>>,
        > = Rc::new(RefCell::new(HashMap::new()));

        let token_source = RustMacroGraphQLTokenSource::new(
            input,
            span_map_storage.clone(),
        );

        // Parse tokens into GraphQL AST at compile time
        let parser = GraphQLParser::new(token_source);
        let parse_result = parser.parse_schema_document();

        // Build the span map from the now-populated storage.
        //
        // Safety of `try_unwrap`: The only clone of this Rc was
        // moved into the RustMacroGraphQLTokenSource, which was
        // consumed by GraphQLParser::new(). Because
        // parse_schema_document() takes `self` (not `&mut self`),
        // the parser — and with it the token source's Rc clone —
        // is guaranteed to be dropped before we reach this point.
        // Thus exactly one strong reference remains.
        let span_map = SpanMap::new(
            Rc::try_unwrap(span_map_storage)
                .expect(
                    "span_map_storage Rc should have \
                     exactly one strong reference remaining",
                )
                .into_inner(),
        );

        // If there were parse errors, convert them to
        // compile_error! invocations with accurate spans
        if parse_result.has_errors() {
            return convert_parse_errors_to_tokenstream(
                &parse_result.errors,
                &span_map,
            )
            .into();
        }

        let ast_doc = match parse_result.into_valid_ast() {
            Some(doc) => doc,
            None => {
                // Should be unreachable: has_errors() was
                // false, so into_valid_ast() should succeed.
                return quote! {
                    compile_error!(
                        "Internal error: GraphQL parse \
                         produced no AST despite reporting \
                         no errors. Please report this at \
                         https://github.com/jeffmo/\
                         libgraphql/issues"
                    );
                }
                .into();
            },
        };

        // Build the schema at compile time
        let schema =
            match libgraphql_core::schema::SchemaBuilder::build_from_ast(
                None, ast_doc,
            ) {
                Ok(schema) => schema,
                Err(err) => {
                    let error_msg = format!(
                        "Failed to build GraphQL schema: \
                         {err}"
                    );
                    return quote! {
                        compile_error!(#error_msg);
                    }
                    .into();
                },
            };

        EmittableSchema::new(schema).into()
    }
}
