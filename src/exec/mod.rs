use clap::Args;
use cli_engine::CliCoreError;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

#[derive(Args, Debug)]
pub struct ExecArgs {
    #[arg(long, help = "Command string to execute")]
    pub command: Option<String>,

    #[arg(long, help = "Read command or input from stdin")]
    pub stdin: bool,
}

pub async fn handle_exec_command(args: &ExecArgs) -> Result<(), CliCoreError> {
    let cmd_str = if let Some(c) = &args.command {
        c.clone()
    } else if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(CliCoreError::Io)?;
        buffer.trim().to_string()
    } else {
        // Interactive prompt when no command or stdin flag is provided
        print!("Enter command to execute: ");
        io::stdout().flush().map_err(CliCoreError::Io)?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(CliCoreError::Io)?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err(CliCoreError::Config("Command cannot be empty".into()));
        }
        trimmed
    };

    if cmd_str.is_empty() {
        return Err(CliCoreError::Config("No command provided to execute".into()));
    }

    let status = Command::new("bash")
        .arg("-c")
        .arg(&cmd_str)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| CliCoreError::Network(format!("Failed to execute command via bash: {}", e)))?;

    if !status.success() {
        return Err(CliCoreError::Api(format!(
            "Command execution failed with exit code: {}",
            status
        )));
    }

    Ok(())
}
