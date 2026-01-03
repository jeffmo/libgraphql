use clap::CommandFactory;
use crate::commands;

#[derive(clap::Parser, Debug)]
#[command(name = "graphql", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Option<commands::CommandEnum>,

    #[arg(
        help="Enable verbose output.",
        long,
        short='v',
    )]
    pub verbose: bool,
}
impl Cli {
    pub(crate) async fn run_default(self) -> anyhow::Result<()> {
        Self::command().print_help().unwrap();
        Ok(())
    }
}
