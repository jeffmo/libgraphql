#![no_main]

use libfuzzer_sys::fuzz_target;
use libgraphql_parser::GraphQLParser;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let parser = GraphQLParser::new(s);
    let _ = parser.parse_mixed_document();
});
