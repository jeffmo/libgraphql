use crate::Cli;
use crate::CommandResult;

pub(crate) trait RunnableCommand: std::fmt::Debug {
    async fn run(self, cli: Cli) -> CommandResult;
}
