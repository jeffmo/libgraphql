use crate::output_utils;
use crate::Cli;
use crate::CommandResult;
use crate::RunnableCommand;
use libgraphql::schema::SchemaBuilder;
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, clap::Args)]
pub(crate) struct ValidateCmd {
    #[arg(
        default_values_t=[
            "graphql".to_string(),
            "graphqls".to_string(),
        ],
        help="Set of file extensions to filter to when searching for files \
             within a directory.",
        long,
        value_delimiter = ',',
    )]
    graphql_file_exts: Vec<String>,

    #[arg(
        help="Paths to one or more GraphQL files or directories containing \
             GraphQL files which need to be validated.",
        name="FILE_OR_DIR_PATHS",
        required=true,
    )]
    file_or_dir_paths: Vec<PathBuf>,
}

#[inherent::inherent]
impl RunnableCommand for ValidateCmd {
    pub async fn run(self, _cli: Cli) -> CommandResult {
        let mut errors: Vec<Box<dyn Error>> = vec![];

        // Normalize the set of file extensions to filter with
        let graphql_file_exts: HashSet<String> =
            self.graphql_file_exts.iter()
                .map(|ext| {
                    if !ext.starts_with('.') {
                        format!(".{ext}")
                    } else {
                        ext.to_owned()
                    }
                })
                .collect();

        // Find all GraphQL files recursively located at or under each path
        // passed as an arg.
        log::debug!(
            "Scanning {} input paths...",
            self.file_or_dir_paths.len(),
        );
        let mut num_non_graphql_files = 0;
        let mut file_paths = vec![];
        for path in &self.file_or_dir_paths {
            for entry in WalkDir::new(path.as_path()).follow_links(true) {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let file_type = entry.file_type();
                        if file_type.is_file() {
                            log::trace!("Found file at {path:#?}.");
                            if let Some(ext) = path.extension().map(|s| s.to_string_lossy())
                                && graphql_file_exts.contains::<String>(&ext.into()) {
                                file_paths.push(std::fs::canonicalize(path).unwrap().to_owned());
                            }
                        } else {
                            log::trace!("Skipping non-file: {path:#?}.");
                            num_non_graphql_files += 1;
                        }
                    },

                    Err(e) => {
                        log::trace!(
                            "Encountered an error while iterating recursive \
                            filesystem entities at/under {path:#?}."
                        );
                        errors.push(Box::new(e));
                        continue
                    },
                }
            }
        }

        // If the user specifies a single file path as an argument, presume the
        // user explicitly wants that file loaded and validated as a GraphQL
        // file -- even if its file extension doesn't match one of the file
        // extensions specified in `graphql_file_exts`.
        if file_paths.is_empty()
            && self.file_or_dir_paths.len() == 1
            && let Some(first_arg_path) = self.file_or_dir_paths.first()
            && first_arg_path.is_file() {
            let canonicalized_first_arg_path =
                std::fs::canonicalize(first_arg_path)
                    .unwrap()
                    .to_owned();
            log::warn!(
                "Proceeding to validate {canonicalized_first_arg_path:#?} even \
                though it doesn't match any of the --graphql-file-exts \
                ({}).",
                graphql_file_exts.iter()
                    .map(|ext| format!("`{ext}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            file_paths.push(canonicalized_first_arg_path);
        }

        log::debug!(
            "Found {} GraphQL files to be validated.",
            file_paths.len(),
        );

        let schema_builder = match SchemaBuilder::from_files(&file_paths) {
            Ok(builder) => builder,
            Err(e) => {
                errors.push(Box:: new(e));
                return CommandResult::stderr(format_args!(
                    "{} GraphQL validation errors: {errors:#?}",
                    output_utils::RED_X,
                ));
            }
        };

        if !errors.is_empty() {
            return CommandResult::stderr(format_args!(
                "{} GraphQL validation errors: {errors:#?}",
                output_utils::RED_X,
            ));
        }

        match schema_builder.build() {
            Ok(schema) => CommandResult::stdout(format_args!(
                concat!(
                    "{} All GraphQL validated successfully:\n",
                    "  * Analyzed {} files.\n",
                    "  * Skipped {} non-graphql files.\n",
                    "  * Validated {} type definitions.\n",
                    "  * Validated {} directive definitions.\n",
                    "  * Validated {} operations.",
                ),
                output_utils::GREEN_CHECK,
                file_paths.len(),
                num_non_graphql_files,
                schema.defined_types().len(),
                schema.defined_directives().len(),
                todo!(), // TODO(!!!): Need to identify and parse out operations too..
            )),

            Err(e) => CommandResult::stderr(format_args!(
                "{} Errors validating schema: {e:#?}",
                output_utils::RED_X,
            )),
        }
    }
}
