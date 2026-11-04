use clap::{Args, Subcommand};
use cli_engine::{Cli, CliConfig, CliCoreError};
use reqwest::Client;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

#[derive(Args, Debug)]
pub struct ConnectArgs {
    #[arg(long, help = "Country code")]
    pub country: String,
}

pub async fn handle_connect_command(args: &ConnectArgs) -> Result<(), CliCoreError> {
    require_login()?;

    let client = Client::new();
    let url = format!(
        "{}/api/vpn/connect?countryCode={}",
        API_BASE,
        urlencoding::encode(&args.country)
    );

    let response = client
        .post(&url)
        .send()
        .await
        .map_err(|e| CliCoreError::Network(format!("Failed to connect to API: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| CliCoreError::Network(format!("Failed to read response body: {}", e)))?;

    if status.is_success() {
        println!("{}", body);
    } else {
        return Err(CliCoreError::Api(format!(
            "Connect request failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
