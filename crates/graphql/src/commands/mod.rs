mod validate;

use crate::Cli;
use crate::CommandResult;
use validate::ValidateCmd;

#[derive(Debug, clap::Parser)]
#[command(name = "graphql")]
pub(crate) enum CommandEnum {
    Validate(Box<ValidateCmd>),
}
impl CommandEnum {
    pub(crate) async fn run(self, cli: Cli) -> CommandResult {
        match self {
            Self::Validate(cmd) => cmd.run(cli).await
        }
    }
}
