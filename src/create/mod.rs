use clap::Args;
use cli_engine::CliCoreError;
use reqwest::Client;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long, help = "Country code")]
    pub country: String,
}

pub async fn handle_create_command(args: &CreateArgs) -> Result<(), CliCoreError> {
    require_login()?;

    let client = Client::new();
    let url = format!(
        "{}/api/vpn/create?countryCode={}",
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
            "Create VPN request failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
