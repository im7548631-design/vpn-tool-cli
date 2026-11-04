use reqwest::Client;
use cli_engine::CliCoreError;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

pub async fn handle_countries_command() -> Result<(), CliCoreError> {
    // Ensure the user is registered and logged in before making the API request
    require_login()?;

    let client = Client::new();
    let url = format!("{}/api/vpn/countries", API_BASE);

    let response = client
        .get(&url)
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
            "Failed to fetch VPN countries with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
