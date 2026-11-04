use clap::Args;
use cli_engine::CliCoreError;
use reqwest::Client;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

#[derive(Args, Debug)]
pub struct IpArgs {
    #[arg(long, help = "Country code")]
    pub country: String,
    #[arg(long, default_value = "10", help = "Number of IPs")]
    pub count: String,
}

pub async fn handle_ip_command(args: &IpArgs) -> Result<(), CliCoreError> {
    require_login()?;

    let client = Client::new();
    let url = format!(
        "{}/api/vpn/ip?countryCode={}&count={}",
        API_BASE,
        urlencoding::encode(&args.country),
        urlencoding::encode(&args.count)
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
            "IP request failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
