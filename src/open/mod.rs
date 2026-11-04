use clap::Args;
use cli_engine::CliCoreError;
use std::process::Command;

#[derive(Args, Debug)]
pub struct OpenArgs {
    #[arg(long, help = "URL to open in the browser")]
    pub url: String,
}

pub async fn handle_open_command(args: &OpenArgs) -> Result<(), CliCoreError> {
    if args.url.is_empty() {
        return Err(CliCoreError::Config("URL cannot be empty. Provide a valid URL with --url".into()));
    }

    // Determine the appropriate opener binary based on the OS
    #[cfg(target_os = "macos")]
    let opener_cmd = "open";
    #[cfg(target_os = "windows")]
    let opener_cmd = "cmd";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let opener_cmd = "xdg-open";

    let mut cmd = Command::new(opener_cmd);
    
    #[cfg(target_os = "windows")]
    {
        cmd.args(["/C", "start", &args.url]);
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.arg(&args.url);
    }

    let status = cmd
        .status()
        .map_err(|e| CliCoreError::Network(format!("Failed to execute '{}': {}", opener_cmd, e)))?;

    if !status.success() {
        return Err(CliCoreError::Api(format!(
            "Failed to open URL '{}' (exit code: {})",
            args.url, status
        )));
    }

    println!("Opened URL successfully: {}", args.url);
    Ok(())
}
