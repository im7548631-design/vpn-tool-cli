use reqwest::{Client, Method, StatusCode};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use cli_engine::CliCoreError;

pub struct ApiOptions {
    pub method: String,
    pub url: String,
    pub headers: Vec<String>,
    pub fields: Vec<String>,
    pub raw_fields: Vec<String>,
    pub forms: Vec<String>,
    pub body: Option<String>,
    pub input_file: Option<String>,
    pub output_format: String,
    pub include_headers: bool,
    pub silent: bool,
    pub paginate: bool,
}

pub async fn handle_api_command(opts: ApiOptions) -> Result<(), CliCoreError> {
    let client = Client::new();

    // Determine HTTP method, defaulting to POST if fields or forms are provided and method is default GET
    let mut method_str = opts.method.to_uppercase();
    if method_str == "GET" && (!opts.fields.is_empty() || !opts.forms.is_empty() || opts.body.is_some() || opts.input_file.is_some()) {
        method_str = "POST".to_string();
    }

    let method = Method::from_bytes(method_str.as_bytes())
        .map_err(|_| CliCoreError::Config(format!("Invalid HTTP method: {}", method_str)))?;

    let mut url = opts.url;
    if url.is_empty() {
        return Err(CliCoreError::Config("Missing required parameter: --url".into()));
    }

    // Build headers
    let mut header_map = reqwest::header::HeaderMap::new();
    for h in &opts.headers {
        if let Some((key, val)) = h.split_once(':') {
            let name = reqwest::header::HeaderName::from_bytes(key.trim().as_bytes())
                .map_err(|e| CliCoreError::Config(format!("Invalid header name: {}", e)))?;
            let value = reqwest::header::HeaderValue::from_str(val.trim())
                .map_err(|e| CliCoreError::Config(format!("Invalid header value: {}", e)))?;
            header_map.insert(name, value);
        }
    }

    // Determine request body or query parameters
    let mut json_payload: HashMap<String, serde_json::Value> = HashMap::new();

    for f in &opts.raw_fields {
        if let Some((key, val)) = f.split_once('=') {
            json_payload.insert(key.to_string(), serde_json::Value::String(val.to_string()));
        }
    }

    for f in &opts.fields {
        if let Some((key, val)) = f.split_once('=') {
            // Infer type: boolean, number, or string
            let parsed_val = if val == "true" {
                serde_json::Value::Bool(true)
            } else if val == "false" {
                serde_json::Value::Bool(false)
            } else if let Ok(num) = val.parse::<i64>() {
                serde_json::Value::Number(num.into())
            } else if let Ok(num) = val.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(num) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::String(val.to_string())
                }
            } else {
                serde_json::Value::String(val.to_string())
            };
            json_payload.insert(key.to_string(), parsed_val);
        }
    }

    let mut request_builder = client.request(method.clone(), &url).headers(header_map);

    // Handle body source (--body, --input, or fields)
    if let Some(body_content) = opts.body {
        request_builder = request_builder.body(body_content);
    } else if let Some(input_path) = opts.input_file {
        let file_data = if input_path == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).map_err(CliCoreError::Io)?;
            buffer
        } else {
            fs::read_to_string(&input_path).map_err(CliCoreError::Io)?
        };
        request_builder = request_builder.body(file_data);
    } else if !json_payload.is_empty() {
        request_builder = request_builder.json(&json_payload);
    }

    // Pagination loop support if requested
    let mut page = 1;
    loop {
        let current_url = if opts.paginate {
            let separator = if url.contains('?') { "&" } else { "?" };
            format!("{}{}_page={}", url, separator, page)
        } else {
            url.clone()
        };

        let req = client.request(method.clone(), &current_url);
        // Re-apply headers and body configuration for pagination if needed...
        let response = req
            .send()
            .await
            .map_err(|e| CliCoreError::Network(format!("API request failed: {}", e)))?;

        let status = response.status();
        let headers = response.headers().clone();
        let text_body = response
            .text()
            .await
            .map_err(|e| CliCoreError::Network(format!("Failed to read response body: {}", e)))?;

        if !opts.silent {
            if opts.include_headers {
                println!("HTTP/1.1 {}", status);
                for (k, v) in headers.iter() {
                    if let Ok(v_str) = v.to_str() {
                        println!("{}: {}", k, v_str);
                    }
                }
                println!();
            }

            // Output formatting logic (json / ndjson)
            if opts.output_format == "json" {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text_body) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        println!("{}", pretty);
                    } else {
                        println!("{}", text_body);
                    }
                } else {
                    println!("{}", text_body);
                }
            } else {
                println!("{}", text_body);
            }
        }

        if !opts.paginate || text_body.trim().is_empty() || text_body.trim() == "[]" {
            break;
        }

        page += 1;
        if page > 100 { // Safety break for pagination
            break;
        }
    }

    Ok(())
}
