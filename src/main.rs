use clap::{Parser, Subcommand, CommandFactory};
use cli_engine::{Cli, CliConfig, CliCoreError};

mod auth;
mod vpn;
mod healthz;
mod countries;
mod completion;
mod connect;
mod create;
mod disconnect;
mod status;
mod test;
mod ip;
mod api;
mod open;
mod exec;

#[derive(Parser, Debug)]
#[command(
    name = "vpn-tool",
    version = "1.0.0",
    about = "CLI tool for managing VPN gateways and API interactions",
    long_about = None
)]
struct CliApp {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Authentication commands (register, login, logout, status, reset, remove)")]
    Auth(AuthArgs),

    #[command(about = "Check API health")]
    Healthz,

    #[command(about = "List VPN countries")]
    Countries,

    #[command(about = "VPN management commands")]
    Vpn(VpnArgs),

    #[command(about = "Connect to VPN")]
    Connect(connect::ConnectArgs),

    #[command(about = "Create a new VPN connection")]
    Create(create::CreateArgs),

    #[command(about = "Disconnect VPN")]
    Disconnect(disconnect::DisconnectArgs),

    #[command(about = "Get VPN status")]
    Status(status::StatusArgs),

    #[command(about = "Test API delay")]
    Test(test::test::TestArgs),

    #[command(about = "Get VPN IP addresses")]
    Ip(ip::IpArgs),

    #[command(about = "Execute custom HTTP/API requests")]
    Api(ApiCommandArgs),

    #[command(about = "Open a URL in the default browser")]
    Open(open::OpenArgs),

    #[command(about = "Execute shell commands via bash")]
    Exec(exec::ExecArgs),

    #[command(about = "Generate shell completion script")]
    Completion(CompletionArgs),
}

#[derive(Parser, Debug)]
struct AuthArgs {
    #[command(subcommand)]
    command: Option<String>,
}

#[derive(Parser, Debug)]
struct VpnArgs {
    #[command(subcommand)]
    command: Option<String>,
}

#[derive(Parser, Debug)]
struct ApiCommandArgs {
    #[arg(short = 'X', long, default_value = "GET", help = "HTTP method")]
    method: String,
    #[arg(long, help = "Request URL")]
    url: String,
    #[arg(short = 'H', long, help = "Additional HTTP header (Key:Value)")]
    header: Vec<String>,
    #[arg(short = 'F', long, help = "Add parameter of inferred type")]
    field: Vec<String>,
    #[arg(short = 'f', long, help = "Add string parameter")]
    raw_field: Vec<String>,
    #[arg(long, help = "Add multipart form field")]
    form: Vec<String>,
    #[arg(long, help = "Request body string")]
    body: Option<String>,
    #[arg(long, help = "File to use as request body")]
    input: Option<String>,
    #[arg(long, default_value = "json", help = "Output format (json, ndjson)")]
    output: String,
    #[arg(short = 'i', long, help = "Include response headers")]
    include: bool,
    #[arg(long, help = "Do not print response body")]
    silent: bool,
    #[arg(long, help = "Paginate results")]
    paginate: bool,
}

#[derive(Parser, Debug)]
struct CompletionArgs {
    #[arg(help = "Shell name (bash, zsh, fish, powershell)")]
    shell: String,
}

#[tokio::main]
async fn main() {
    let cli = CliApp::parse();
    
    let config = CliConfig::default();
    let _engine = Cli::new(config);

    let raw_matches = CliApp::command().get_matches();

    let result: Result<(), CliCoreError> = match &cli.command {
        Some(Commands::Auth(_)) => {
            if let Some((_, sub_matches)) = raw_matches.subcommand() {
                auth::handle_auth_command(sub_matches).await
            } else {
                Ok(())
            }
        }
        Some(Commands::Healthz) => healthz::handle_healthz_command().await,
        Some(Commands::Countries) => countries::handle_countries_command().await,
        Some(Commands::Vpn(_)) => {
            if let Some((_, sub_matches)) = raw_matches.subcommand() {
                vpn::handle_vpn_command(sub_matches).await
            } else {
                Ok(())
            }
        }
        Some(Commands::Connect(args)) => connect::handle_connect_command(args).await,
        Some(Commands::Create(args)) => create::handle_create_command(args).await,
        Some(Commands::Disconnect(args)) => disconnect::handle_disconnect_command(args).await,
        Some(Commands::Status(args)) => status::handle_status_command(args).await,
        Some(Commands::Test(args)) => test::test::handle_test_command(args).await,
        Some(Commands::Ip(args)) => ip::handle_ip_command(args).await,
        Some(Commands::Api(args)) => {
            let api_opts = api::ApiOptions {
                method: args.method.clone(),
                url: args.url.clone(),
                headers: args.header.clone(),
                fields: args.field.clone(),
                raw_fields: args.raw_field.clone(),
                forms: args.form.clone(),
                body: args.body.clone(),
                input_file: args.input.clone(),
                output_format: args.output.clone(),
                include_headers: args.include,
                silent: args.silent,
                paginate: args.paginate,
            };
            api::handle_api_command(api_opts).await
        }
        Some(Commands::Open(args)) => open::handle_open_command(args).await,
        Some(Commands::Exec(args)) => exec::handle_exec_command(args).await,
        Some(Commands::Completion(_)) => {
            let cmd_def = CliApp::command();
            completion::handle_completion_command(&clap::ArgMatches::default(), cmd_def)
        }
        None => {
            println!("vpn-tool v1.0.0 - Use --help for usage information.");
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
