use std::process::ExitCode;

#[derive(Debug)]
pub(crate) struct CommandResult {
    pub exit_code: ExitCode,
    pub stderr: Option<String>,
    pub stdout: Option<String>,
}

impl CommandResult {
    pub fn stderr(fmt_args: std::fmt::Arguments<'_>) -> Self {
        Self {
            exit_code: ExitCode::FAILURE,
            stderr: Some(format!("{fmt_args}")),
            stdout: None,
        }
    }

    pub fn stdout(fmt_args: std::fmt::Arguments<'_>) -> Self {
        Self {
            exit_code: ExitCode::FAILURE,
            stderr: None,
            stdout: Some(format!("{fmt_args}")),
        }
    }
}
