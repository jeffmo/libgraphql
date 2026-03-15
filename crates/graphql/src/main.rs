mod cli;
mod command;
mod command_result;
mod commands;
mod output_utils;

use clap::Parser;
pub(crate) use cli::Cli;
pub(crate) use command::RunnableCommand;
pub(crate) use command_result::CommandResult;

const DEFAULT_LOG_LEVEL: tracing::Level = tracing::Level::INFO;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> std::process::ExitCode {
    let mut cli = Cli::parse();
    setup_logger(&cli);

    if let Some(command) = cli.cmd.take() {
        let result = command.run(cli).await;
        if let Some(stdout) = result.stdout {
            println!("{stdout}");
        }
        if let Some(stderr) = result.stderr {
            eprintln!("{stderr}")
        }
        result.exit_code
    } else {
        cli.run_default().await.unwrap();
        std::process::ExitCode::SUCCESS
    }
}

fn setup_logger(cli: &Cli) {
    let mut log_level_warnings: Vec<String> = vec![];
    let log_level =
        if cli.verbose {
            tracing::Level::DEBUG
        } else {
            let env_val =
                std::env::var("LOG_LEVEL")
                    .map(|s| s.trim().to_string());

            match env_val.as_deref() {
                Ok("DEBUG" | "debug") => tracing::Level::DEBUG,
                Ok("INFO" | "info") => tracing::Level::INFO,
                Ok("TRACE" | "trace") => tracing::Level::TRACE,
                Ok("VERBOSE" | "verbose") => tracing::Level::DEBUG,
                Ok(other) => {
                    log_level_warnings.push(format!(
                        "Invalid `LOG_LEVEL` environment variable value: \
                        `{other}`"
                    ));
                    DEFAULT_LOG_LEVEL
                },
                Err(_) => DEFAULT_LOG_LEVEL,
            }
        };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    log::trace!("Initial logging level set to `{log_level}`.");

    for warning in log_level_warnings.drain(..) {
        log::warn!("{warning}");
    }
}
