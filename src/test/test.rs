use clap::Args;
use cli_engine::CliCoreError;
use reqwest::Client;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

#[derive(Args, Debug)]
pub struct TestArgs {
    #[arg(long, default_value = "100", help = "Milliseconds delay")]
    pub ms: String,
}

pub async fn handle_test_command(args: &TestArgs) -> Result<(), CliCoreError> {
    require_login()?;

    let client = Client::new();
    let url = format!(
        "{}/api/vpn/test/sleep?ms={}",
        API_BASE,
        urlencoding::encode(&args.ms)
    );

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
            "Test request failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
