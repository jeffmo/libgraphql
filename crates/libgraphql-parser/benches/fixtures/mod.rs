use std::path::Path;
use std::path::PathBuf;

pub const SMALL_SCHEMA: &str =
    include_str!("small_schema.graphql");
pub const MEDIUM_SCHEMA: &str =
    include_str!("medium_schema.graphql");
pub const LARGE_SCHEMA: &str =
    include_str!("large_schema.graphql");
pub const SIMPLE_QUERY: &str =
    include_str!("simple_query.graphql");
pub const COMPLEX_QUERY: &str =
    include_str!("complex_query.graphql");
pub const GITHUB_SCHEMA: &str =
    include_str!("third-party/github-schema/schema.graphql");
pub const STARWARS_SCHEMA: &str =
    include_str!("third-party/starwars-schema/schema.graphql");

/// Loads the Shopify Admin GraphQL API schema from disk at
/// runtime. This schema is not checked in to the repository
/// (it is gitignored) and must be fetched locally before
/// running benchmarks.
///
/// Panics with a descriptive error message and fetch
/// instructions if the file is not found.
pub fn load_shopify_admin_schema() -> String {
    let path = shopify_admin_schema_path();
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(
                "scripts\
                 /fetch-shopify-admin-graphql\
                 -schema-fixture.sh",
            );
        panic!(
            "\n\n\
             Shopify Admin schema fixture not found \
             at:\n\n  \
             {}\n\n\
             This file is not checked into the \
             repository and must be fetched \
             locally.\n\
             Run the following command to download \
             it:\n\n  \
             {}\n",
            path.display(),
            script.display(),
        );
    })
}

fn shopify_admin_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "benches/fixtures/third-party\
         /shopify-admin-schema/schema.graphql",
    )
}

pub mod operations;
