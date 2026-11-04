use reqwest::Client;
use cli_engine::CliCoreError;
use crate::auth::require_login;

const API_BASE: &str = "https://vpn-gateway-manager--jjajajjajahsh.replit.app";

pub async fn handle_vpn_command(matches: &clap::ArgMatches) -> Result<(), CliCoreError> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            handle_vpn_action("create", "POST", sub_matches).await?
        }
        Some(("connect", sub_matches)) => {
            handle_vpn_action("connect", "POST", sub_matches).await?
        }
        Some(("disconnect", sub_matches)) => {
            handle_vpn_action("disconnect", "POST", sub_matches).await?
        }
        Some(("status", sub_matches)) => {
            let country = sub_matches.get_one::<String>("country")
                .ok_or_else(|| CliCoreError::Config("Missing required option: --country".into()))?;
            
            require_login()?;
            let client = Client::new();
            let url = format!("{}/api/vpn/status?countryCode={}", API_BASE, urlencoding::encode(country));

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| CliCoreError::Network(format!("Failed to connect to API: {}", e)))?;

            handle_response(response).await?;
        }
        Some(("test", sub_matches)) => {
            let ms = sub_matches.get_one::<String>("ms").map(|s| s.as_str()).unwrap_or("100");
            
            require_login()?;
            let client = Client::new();
            let url = format!("{}/api/vpn/test/sleep?ms={}", API_BASE, urlencoding::encode(ms));

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| CliCoreError::Network(format!("Failed to connect to API: {}", e)))?;

            handle_response(response).await?;
        }
        Some(("ip", sub_matches)) => {
            let country = sub_matches.get_one::<String>("country")
                .ok_or_else(|| CliCoreError::Config("Missing required option: --country".into()))?;
            let count = sub_matches.get_one::<String>("count").map(|s| s.as_str()).unwrap_or("10");

            require_login()?;
            let client = Client::new();
            let url = format!(
                "{}/api/vpn/ip?countryCode={}&count={}",
                API_BASE,
                urlencoding::encode(country),
                urlencoding::encode(count)
            );

            let response = client
                .post(&url)
                .send()
                .await
                .map_err(|e| CliCoreError::Network(format!("Failed to connect to API: {}", e)))?;

            handle_response(response).await?;
        }
        _ => {
            println!("Invalid vpn subcommand. Use --help for usage.");
        }
    }

    Ok(())
}

async fn handle_vpn_action(action: &str, method: &str, matches: &clap::ArgMatches) -> Result<(), CliCoreError> {
    let country = matches.get_one::<String>("country")
        .ok_or_else(|| CliCoreError::Config("Missing required option: --country".into()))?;

    require_login()?;

    let client = Client::new();
    let url = format!("{}/api/vpn/{}?countryCode={}", API_BASE, action, urlencoding::encode(country));

    let request_builder = match method {
        "POST" => client.post(&url),
        _ => client.get(&url),
    };

    let response = request_builder
        .send()
        .await
        .map_err(|e| CliCoreError::Network(format!("Failed to connect to API: {}", e)))?;

    handle_response(response).await
}

async fn handle_response(response: reqwest::Response) -> Result<(), CliCoreError> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| CliCoreError::Network(format!("Failed to read response body: {}", e)))?;

    if status.is_success() {
        println!("{}", body);
    } else {
        return Err(CliCoreError::Api(format!(
            "VPN request failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
