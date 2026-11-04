use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use cli_engine::CliCoreError;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthData {
    pub version: u32,
    pub pin_hash: String,
}

fn config_dir() -> Result<PathBuf, CliCoreError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| CliCoreError::Config("Could not find home directory".into()))?;
    Ok(PathBuf::from(home).join(".vpn-tool"))
}

fn auth_file() -> Result<PathBuf, CliCoreError> {
    Ok(config_dir()?.join("auth.json"))
}

fn session_file() -> Result<PathBuf, CliCoreError> {
    Ok(config_dir()?.join("session"))
}

fn ensure_config() -> Result<(), CliCoreError> {
    let dir = config_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(CliCoreError::Io)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dir).map_err(CliCoreError::Io)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&dir, perms).map_err(CliCoreError::Io)?;
        }
    }
    Ok(())
}

fn hash_pin(pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn registered() -> bool {
    auth_file().map(|p| p.exists()).unwrap_or(false)
}

pub fn logged_in() -> bool {
    let Ok(path) = session_file() else { return false; };
    if !path.exists() {
        return false;
    }
    match fs::read_to_string(path) {
        Ok(content) => content.trim() == "logged_in",
        Err(_) => false,
    }
}

pub fn load_auth() -> Option<AuthData> {
    let path = auth_file().ok()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn ask(question: &str) -> Result<String, CliCoreError> {
    print!("{}", question);
    io::stdout().flush().map_err(CliCoreError::Io)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(CliCoreError::Io)?;
    Ok(input.trim().to_string())
}

pub fn require_login() -> Result<(), CliCoreError> {
    if !registered() {
        return Err(CliCoreError::Auth("Not registered. Run: vpn-tool auth register".into()));
    }
    if !loggedIn() {
        return Err(CliCoreError::Auth("Not logged in. Run: vpn-tool auth login".into()));
    }
    Ok(())
}

pub async fn handle_auth_command(matches: &clap::ArgMatches) -> Result<(), CliCoreError> {
    match matches.subcommand() {
        Some(("register", _)) => {
            if registered() {
                return Err(CliCoreError::Auth("PIN already registered.".into()));
            }

            let pin = ask("Enter new PIN: ")?;
            if pin.is_empty() {
                return Err(CliCoreError::Auth("PIN cannot be empty.".into()));
            }

            let confirm = ask("Confirm new PIN: ")?;
            if pin != confirm {
                return Err(CliCoreError::Auth("PINs do not match.".into()));
            }

            ensure_config()?;

            let auth_data = AuthData {
                version: 1,
                pin_hash: hash_pin(&pin),
            };

            let json = serde_json::to_string_pretty(&auth_data)
                .map_err(|e| CliCoreError::Serialization(e.to_string()))?;

            let a_file = auth_file()?;
            fs::write(&a_file, json).map_err(CliCoreError::Io)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&a_file).map_err(CliCoreError::Io)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&a_file, perms).map_err(CliCoreError::Io)?;
            }

            fs::write(session_file()?, "logged_out").map_err(CliCoreError::Io)?;

            println!("PIN registered successfully.");
            println!("Run: vpn-tool auth login");
        }
        Some(("login", _)) => {
            if !registered() {
                return Err(CliCoreError::Auth("No PIN registered. Run: vpn-tool auth register".into()));
            }

            let auth_data = load_auth()
                .ok_or_else(|| CliCoreError::Auth("Failed to load auth data.".into()))?;
            let pin = ask("Enter PIN: ")?;

            if auth_data.pin_hash != hash_pin(&pin) {
                return Err(CliCoreError::Auth("Invalid PIN.".into()));
            }

            ensure_config()?;
            fs::write(session_file()?, "logged_in").map_err(CliCoreError::Io)?;
            println!("Login successful.");
        }
        Some(("logout", _)) => {
            if !registered() {
                return Err(CliCoreError::Auth("No PIN registered.".into()));
            }

            let auth_data = load_auth()
                .ok_or_else(|| CliCoreError::Auth("Failed to load auth data.".into()))?;
            let pin = ask("Enter PIN: ")?;

            if auth_data.pin_hash != hash_pin(&pin) {
                return Err(CliCoreError::Auth("Invalid PIN.".into()));
            }

            fs::write(session_file()?, "logged_out").map_err(CliCoreError::Io)?;
            println!("Logout successful.");
        }
        Some(("status", _)) => {
            println!("Registered: {}", if registered() { "yes" } else { "no" });
            println!("Logged in:  {}", if logged_in() { "yes" } else { "no" });
        }
        Some(("reset", _)) => {
            if !registered() {
                return Err(CliCoreError::Auth("No PIN registered.".into()));
            }

            let auth_data = load_auth()
                .ok_or_else(|| CliCoreError::Auth("Failed to load auth data.".into()))?;
            let current = ask("Enter current PIN: ")?;

            if auth_data.pin_hash != hash_pin(&current) {
                return Err(CliCoreError::Auth("Invalid current PIN.".into()));
            }

            let new_pin = ask("Enter new PIN: ")?;
            if new_pin.is_empty() {
                return Err(CliCoreError::Auth("New PIN cannot be empty.".into()));
            }

            let confirm = ask("Confirm new PIN: ")?;
            if new_pin != confirm {
                return Err(CliCoreError::Auth("PINs do not match.".into()));
            }

            let new_auth = AuthData {
                version: 1,
                pin_hash: hash_pin(&new_pin),
            };

            let json = serde_json::to_string_pretty(&new_auth)
                .map_err(|e| CliCoreError::Serialization(e.to_string()))?;

            fs::write(auth_file()?, json).map_err(CliCoreError::Io)?;
            fs::write(session_file()?, "logged_out").map_err(CliCoreError::Io)?;
            println!("PIN reset successfully.");
        }
        Some(("remove", _)) => {
            if !registered() {
                return Err(CliCoreError::Auth("No PIN registered.".into()));
            }

            let auth_data = load_auth()
                .ok_or_else(|| CliCoreError::Auth("Failed to load auth data.".into()))?;
            let pin = ask("Enter PIN: ")?;

            if auth_data.pin_hash != hash_pin(&pin) {
                return Err(CliCoreError::Auth("Invalid PIN.".into()));
            }

            let _ = fs::remove_file(auth_file()?);
            let _ = fs::remove_file(session_file()?);
            println!("Authentication removed.");
        }
        _ => {
            println!("Invalid auth subcommand. Use --help for usage.");
        }
    }
    Ok(())
}
