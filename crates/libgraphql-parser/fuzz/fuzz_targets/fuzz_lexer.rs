#![no_main]

use libfuzzer_sys::fuzz_target;
use libgraphql_parser::token_source::StrGraphQLTokenSource;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let token_source = StrGraphQLTokenSource::new(s);
    for _ in token_source {}
});
