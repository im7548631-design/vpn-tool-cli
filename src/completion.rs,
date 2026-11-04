use std::io;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use cli_engine::CliCoreError;

pub fn handle_completion_command(matches: &clap::ArgMatches, mut cmd: clap::Command) -> Result<(), CliCoreError> {
    let shell_str = matches.get_one::<String>("shell")
        .ok_or_else(|| CliCoreError::Config("Missing shell argument for completion. Use: bash, zsh, fish, or powershell".into()))?;

    let shell = match shell_str.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "ps" => Shell::PowerShell,
        _ => {
            return Err(CliCoreError::Config(format!(
                "Unsupported shell: '{}'. Supported shells are: bash, zsh, fish, powershell",
                shell_str
            )));
        }
    };

    generate(shell, &mut cmd, "vpn-tool", &mut io::stdout());

    Ok(())
}
